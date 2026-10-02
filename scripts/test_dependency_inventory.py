#!/usr/bin/env python3
"""Black-box dependency gates using synthetic evidence, never production signoff."""
from __future__ import annotations

import copy
import contextlib
import io
from datetime import date
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import dependency_common as common

SCRIPT = Path(__file__).resolve().with_name("dependency_inventory.py")
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"
TARGET = "x86_64-unknown-linux-gnu"
AS_OF = "2026-10-02"


class Fixture:
    def __init__(self, root):
        self.root = Path(root)
        self.workspace_names = {"forge"}
        (self.root / "supply-chain").mkdir(parents=True)
        (self.root / ".github/workflows").mkdir(parents=True)
        self.write("Cargo.toml", '[package]\nname="forge"\nversion="2.0.0"\nedition="2024"\n')
        self.write(".github/workflows/release.yml", "jobs:\n  build:\n    strategy:\n      matrix:\n        include:\n          - target: " + TARGET + "\n            os: ubuntu-latest\n            archive: tar.gz\n")
        self.packages = [{"name": "forge", "version": "2.0.0", "source": "workspace", "checksum": None}]
        for name in ("alpha", "beta", "gamma", "delta", "epsilon", "zeta"):
            self.packages.append({"name": name, "version": "1.0.0", "source": REGISTRY,
                                  "checksum": hashlib.sha256(name.encode()).hexdigest()})
        self.packages.sort(key=common.identity)
        self.config_prefix = '[cargo-vet]\nversion="0.10"\n[policy.forge]\ncriteria="safe-to-deploy"\ndev-criteria="safe-to-run"\n'
        self.write_config()
        self.write("supply-chain/audits.toml", "# Synthetic fixture only\n")
        self.write("supply-chain/imports.lock", "# Synthetic fixture only\n")
        self.graph = {"format": "forge.dependency-graph/1", "inputs": {}, "targets": [TARGET],
                      "features": ["default"], "packages": [], "graphs": []}
        by_name = {p["name"]: common.package_id(p) for p in self.packages}
        edges = [("forge", "alpha", "normal"), ("forge", "beta", "build"),
                 ("forge", "gamma", "normal"), ("forge", "delta", "dev"),
                 ("gamma", "epsilon", "normal"), ("alpha", "delta", "normal")]
        self.graph["graphs"] = [{"target": TARGET, "roots": [by_name["forge"]],
                                  "edges": sorted([{"from": by_name[a], "to": by_name[b], "kind": k}
                                                   for a, b, k in edges], key=lambda e: (e["from"], e["to"], e["kind"]))}]
        self.sync_graph()
        self.legacy = {"format": "forge.dependency-legacy/1",
                       "inputs": {"lock_sha256": self.hash("Cargo.lock"),
                                  "config_sha256": self.hash("supply-chain/config.toml")},
                       "packages": [{**p, "criteria": ["safe-to-deploy"]}
                                    for p in self.packages if p["name"] != "forge"]}
        self.json("supply-chain/legacy-baseline.json", self.legacy)
        self.policy = {"format": "forge.dependency-policy/1", "status": "approved",
                       "audit_authority": "human-signoff", "max_exception_days": 90,
                       "imports": [], "legacy_baseline_sha256": self.hash("supply-chain/legacy-baseline.json"),
                       "decision": {"owner": "Fixture Owner", "recorded_on": AS_OF,
                                    "reference": "synthetic fixture, not production approval"}}
        self.json("supply-chain/dependency-policy.json", self.policy)
        self.reviews = {"format": "forge.dependency-reviews/1", "records": []}
        self.exceptions = {"format": "forge.dependency-exceptions/1", "entries": []}
        self.json("supply-chain/dependency-reviews.json", self.reviews)
        self.json("supply-chain/dependency-exceptions.json", self.exceptions)
        self.audits = {}

    def write(self, path, text):
        (self.root / path).write_text(text, encoding="utf-8")

    def json(self, path, value):
        (self.root / path).write_bytes(common.canonical_bytes(value))

    def hash(self, path):
        return common.sha256((self.root / path).read_bytes())

    def package(self, name):
        return next(p for p in self.packages if p["name"] == name)

    def write_config(self):
        self.write("supply-chain/config.toml", self.config_prefix + "".join(
            f'[[exemptions.{p["name"]}]]\nversion="{p["version"]}"\ncriteria="safe-to-deploy"\n'
            for p in self.packages if p["name"] != "forge"))

    def sync_graph(self):
        locked = ["version = 4\n"]
        for p in self.packages:
            locked.append(f'[[package]]\nname="{p["name"]}"\nversion="{p["version"]}"\n')
            dependencies = sorted({json.loads(edge["to"])[0]
                for graph in self.graph["graphs"] for edge in graph["edges"]
                if json.loads(edge["from"])[0] == p["name"]})
            if dependencies:
                locked.append("dependencies=" + json.dumps(dependencies) + "\n")
            if p["source"] != "workspace":
                locked.append('source=' + json.dumps(p["source"]) + '\n')
            if p["checksum"] is not None:
                locked.append('checksum=' + json.dumps(p["checksum"]) + '\n')
        self.write("Cargo.lock", "".join(locked))
        self.graph["packages"] = [{**p, "workspace_member": p["name"] in self.workspace_names,
                                    "proc_macro": p["name"] == "gamma"} for p in self.packages]
        self.graph["inputs"] = {"lock_sha256": self.hash("Cargo.lock"),
                                  "manifests": {"Cargo.toml": self.hash("Cargo.toml")},
                                  "release_workflow_sha256": self.hash(".github/workflows/release.yml")}
        self.json("supply-chain/dependency-graph.json", self.graph)

    def exception(self, name="alpha", **changes):
        item = {**self.package(name), "criteria": ["safe-to-deploy"], "owner": "Fixture Owner",
                "rationale": "Synthetic test, never real approval", "created_on": "2026-09-01",
                "review_by": "2026-10-02", "expires_on": "2026-11-01",
                "approval_reference": "fixture:exception"}
        item.update(changes)
        self.exceptions["entries"].append(item)
        self.json("supply-chain/dependency-exceptions.json", self.exceptions)
        return item

    def audit(self, crate="alpha", *, ver=None, delta=None, checksums=None,
              criterion="safe-to-deploy", method="human", signed=True, **signoff_changes):
        record = {"who": "Fixture Reviewer", "criteria": criterion}
        record["delta" if delta else "version"] = delta or ver or self.package(crate)["version"]
        self.audits.setdefault(crate, []).append(record)
        text = ""
        for name, records in sorted(self.audits.items()):
            for raw in records:
                text += f"[[audits.{name}]]\n" + "".join(f"{k}={json.dumps(v)}\n" for k, v in raw.items())
        self.write("supply-chain/audits.toml", text)
        if signed:
            signoff = {"owner": "Fixture Reviewer", "recorded_on": AS_OF,
                       "reference": "fixture:review, no production evidence"}
            signoff.update(signoff_changes)
            self.reviews["records"].append({"crate": crate, "source": REGISTRY,
                "record_sha256": common.sha256(common.canonical_bytes(record)), "method": method,
                "checksums": checksums or {record["version"]: self.package(crate)["checksum"]},
                "signoff": signoff})
            self.json("supply-chain/dependency-reviews.json", self.reviews)
        return record


def consume(fixture, command="gate", *, strict=False, as_of=AS_OF, guarded=False):
    args = [str(SCRIPT), command, "--root", str(fixture.root), "--as-of", as_of]
    if strict:
        args.append("--strict")
    with tempfile.TemporaryDirectory() as cache:
        environment = {**os.environ, "CARGO_HOME": cache, "RUSTUP_HOME": cache, "PYTHONDONTWRITEBYTECODE": ""}
        if guarded:
            wrapper = """import runpy,sys,pathlib
sys.argv=sys.argv[1:]
sys.path.insert(0,str(pathlib.Path(sys.argv[0]).parent))
def guard(event,args):
    if event.startswith(('subprocess.', 'socket.', 'os.system', 'os.posix_spawn')):
        raise RuntimeError('consumer tried external operation '+event)
sys.addaudithook(guard)
runpy.run_path(sys.argv[0],run_name='__main__')
"""
            result = subprocess.run([sys.executable, "-c", wrapper, *args], env=environment,
                                    capture_output=True, timeout=10)
        else:
            result = subprocess.run([sys.executable, *args], env=environment, capture_output=True, timeout=10)
    try:
        report = json.loads(result.stdout)
    except Exception as error:
        raise AssertionError(f"no JSON report, rc={result.returncode}: {result.stderr!r}") from error
    if report["exit_code"] != result.returncode:
        raise AssertionError("reported and actual exit differ")
    return result, report


class DependencyGateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.fixture = Fixture(self.temp.name)

    def check_exit(self, code, **args):
        result, report = consume(self.fixture, **args)
        self.assertEqual(result.returncode, code, report)
        return report

    def test_offline_complete_classification_and_no_writes(self):
        before = {str(p.relative_to(self.fixture.root)): p.read_bytes()
                  for p in self.fixture.root.rglob("*") if p.is_file()}
        report = self.check_exit(0, guarded=True)
        after = {str(p.relative_to(self.fixture.root)): p.read_bytes()
                 for p in self.fixture.root.rglob("*") if p.is_file()}
        self.assertEqual(before, after)
        rows = {r["name"]: r for r in report["packages"]}
        self.assertEqual(len(rows), 7)
        self.assertEqual(rows["alpha"]["class"], "safe-to-deploy")
        self.assertEqual(rows["delta"]["class"], "safe-to-deploy")
        self.assertEqual(rows["beta"]["class"], "safe-to-run")
        self.assertEqual(rows["gamma"]["class"], "safe-to-run")
        self.assertEqual(rows["epsilon"]["class"], "safe-to-run")
        self.assertEqual(rows["zeta"]["usage"], ["unsupported-target-only"])
        self.assertEqual(report["runtime_readiness"]["denominator"], 2)
        self.assertFalse(report["runtime_readiness"]["ready"])

    def test_relocated_identical_bytes(self):
        first, _ = consume(self.fixture)
        with tempfile.TemporaryDirectory() as other:
            shutil.copytree(self.fixture.root, Path(other) / "repo")
            relocated = copy.copy(self.fixture)
            relocated.root = Path(other) / "repo"
            second, _ = consume(relocated)
        self.assertEqual(first.stdout, second.stdout)

    def test_legacy_never_strict_credit_and_store_requires_owner(self):
        self.check_exit(1, strict=True)
        report = self.check_exit(2, command="validate-store")
        self.assertTrue(any("legacy-unowned" in e for e in report["errors"]))

    def test_proposed_policy_never_clean(self):
        self.fixture.policy.pop("decision")
        self.fixture.policy["status"] = "proposed"
        self.fixture.json("supply-chain/dependency-policy.json", self.fixture.policy)
        report = self.check_exit(1)
        self.assertFalse(report["runtime_readiness"]["ready"])

    def test_stale_input_hashes(self):
        for path in ("Cargo.lock", "Cargo.toml", ".github/workflows/release.yml"):
            with self.subTest(path=path):
                original = (self.fixture.root / path).read_bytes()
                (self.fixture.root / path).write_bytes(original + b"\n# changed\n")
                self.check_exit(2)
                (self.fixture.root / path).write_bytes(original)

    def test_changed_checksum_does_not_grandfather(self):
        self.fixture.package("alpha")["checksum"] = "b" * 64
        self.fixture.sync_graph()
        report = self.check_exit(1)
        self.assertTrue(any('"alpha"' in p for p in report["new_unvetted_runtime"]))

    def test_changed_version_does_not_grandfather(self):
        old = common.package_id(self.fixture.package("alpha"))
        self.fixture.package("alpha")["version"] = "1.0.1"
        new = common.package_id(self.fixture.package("alpha"))
        for graph in self.fixture.graph["graphs"]:
            for edge in graph["edges"]:
                edge["from"] = new if edge["from"] == old else edge["from"]
                edge["to"] = new if edge["to"] == old else edge["to"]
        self.fixture.sync_graph()
        report = self.check_exit(1)
        self.assertIn(new, report["new_unvetted_runtime"])

    def test_changed_source_does_not_grandfather(self):
        old = common.package_id(self.fixture.package("alpha"))
        self.fixture.package("alpha")["source"] = "registry+https://example.invalid/index"
        new = common.package_id(self.fixture.package("alpha"))
        for edge in self.fixture.graph["graphs"][0]["edges"]:
            edge["from"] = new if edge["from"] == old else edge["from"]
            edge["to"] = new if edge["to"] == old else edge["to"]
        self.fixture.sync_graph()
        self.assertIn(new, self.check_exit(1)["new_unvetted_runtime"])

    def test_baseline_digest_cannot_change_silently(self):
        self.fixture.legacy["packages"].pop()
        self.fixture.json("supply-chain/legacy-baseline.json", self.fixture.legacy)
        self.check_exit(2)

    def test_exception_boundaries_and_ages(self):
        self.fixture.exception()
        report = self.check_exit(0)
        self.assertEqual(next(r for r in report["packages"] if r["name"] == "alpha")["status"], "owned-exception")
        self.assertEqual(report["trends"]["exception_age_days_median"], 31)
        self.check_exit(1, as_of="2026-10-03")
        self.check_exit(1, as_of="2026-11-02")

    def test_invalid_exception_metadata(self):
        for changes in ({"owner": ""}, {"created_on": "2026-10-03"},
                        {"review_by": "2026-08-31"}, {"expires_on": "2027-01-01"},
                        {"checksum": "f" * 64}, {"created_on": "2026-9-1"}):
            with self.subTest(changes=changes):
                self.fixture.exceptions["entries"] = []
                self.fixture.exception(**changes)
                self.check_exit(2)

    def test_full_local_review_and_run_only_not_deploy(self):
        self.fixture.audit()
        report = self.check_exit(0)
        row = next(r for r in report["packages"] if r["name"] == "alpha")
        self.assertEqual(row["status"], "local-full")
        self.assertEqual(report["runtime_readiness"]["accepted_reviews"], 1)
        self.fixture.audit(crate="delta", criterion="safe-to-run")
        self.assertEqual(self.check_exit(1, strict=True)["runtime_readiness"]["accepted_reviews"], 1)

    def test_delta_needs_full_base_and_matching_checksums(self):
        base = "a" * 64
        self.fixture.audit(ver="0.9.0", checksums={"0.9.0": base})
        self.fixture.audit(delta="0.9.0 -> 1.0.0", checksums={"0.9.0": base, "1.0.0": self.fixture.package("alpha")["checksum"]})
        row = next(r for r in self.check_exit(0)["packages"] if r["name"] == "alpha")
        self.assertEqual(row["status"], "local-delta")
        self.fixture.reviews["records"][1]["checksums"]["0.9.0"] = "b" * 64
        self.fixture.json("supply-chain/dependency-reviews.json", self.fixture.reviews)
        self.check_exit(2)

    def test_signed_orphan_delta_cannot_use_exemption_base(self):
        self.fixture.audit(delta="0.9.0 -> 1.0.0", checksums={"0.9.0": "a" * 64, "1.0.0": self.fixture.package("alpha")["checksum"]})
        self.check_exit(2)

    def test_unsigned_delta_remains_pending(self):
        self.fixture.audit(delta="0.9.0 -> 1.0.0", signed=False)
        report = self.check_exit(0)
        self.assertEqual(report["runtime_readiness"]["accepted_reviews"], 0)

    def test_future_evidence_invalid(self):
        self.fixture.audit(recorded_on="2026-10-03")
        self.check_exit(2)

    def test_human_only_rejects_agent_assisted_credit(self):
        self.fixture.policy["audit_authority"] = "human-only"
        self.fixture.json("supply-chain/dependency-policy.json", self.fixture.policy)
        self.fixture.audit(method="agent-assisted")
        report = self.check_exit(2)
        self.assertEqual(report["runtime_readiness"]["accepted_reviews"], 0)

    def test_unknown_native_semantics_invalid(self):
        self.fixture.write("supply-chain/audits.toml", '[wildcard-audits]\n')
        self.check_exit(2)

    def test_forged_workspace_membership_invalid(self):
        self.fixture.graph["packages"][0]["workspace_member"] = True
        self.fixture.json("supply-chain/dependency-graph.json", self.fixture.graph)
        self.check_exit(2)

    def test_duplicate_unknown_and_deep_inputs_invalid(self):
        path = "supply-chain/dependency-exceptions.json"
        for raw in ('{"format":"forge.dependency-exceptions/1","entries":[],"entries":[]}',
                    '{"format":"forge.dependency-exceptions/1","entries":[],"unknown":true}',
                    '[' * 65 + '0' + ']' * 65):
            with self.subTest(raw=raw[:35]):
                self.fixture.write(path, raw)
                self.check_exit(2)

    def test_malformed_schema_types_always_report_json(self):
        original = copy.deepcopy(self.fixture.graph)
        mutations = [("targets", [{}]), ("packages", [None]), ("graphs", [None]), ("features", {})]
        for field, value in mutations:
            with self.subTest(field=field):
                changed = copy.deepcopy(original)
                changed[field] = value
                self.fixture.json("supply-chain/dependency-graph.json", changed)
                self.check_exit(2)

    def test_invalid_date_and_missing_asof_report_json(self):
        self.check_exit(2, as_of="2026-02-30")
        result = subprocess.run([sys.executable, str(SCRIPT), "gate"], capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(json.loads(result.stdout)["exit_code"], 2)

    def imports(self):
        self.fixture.config_prefix += '[imports.fixture-base]\nurl="https://example.invalid/base"\n[imports.fixture-delta]\nurl="https://example.invalid/delta"\n'
        self.fixture.write_config()
        self.fixture.write("supply-chain/imports.lock", '[[audits.fixture-base.audits.alpha]]\nwho="Fixture Reviewer"\ncriteria="safe-to-deploy"\nversion="0.9.0"\n[[audits.fixture-delta.audits.alpha]]\nwho="Fixture Reviewer"\ncriteria="safe-to-deploy"\ndelta="0.9.0 -> 1.0.0"\n')
        snapshot = self.fixture.hash("supply-chain/imports.lock")
        self.fixture.policy["imports"] = [{"name": "fixture-base", "url": "https://example.invalid/base", "snapshot_sha256": snapshot},
                                          {"name": "fixture-delta", "url": "https://example.invalid/delta", "snapshot_sha256": snapshot}]
        self.fixture.json("supply-chain/dependency-policy.json", self.fixture.policy)

    def test_pinned_import_cross_source_full_delta_chain(self):
        self.imports()
        row = next(r for r in self.check_exit(0)["packages"] if r["name"] == "alpha")
        self.assertEqual(row["status"], "imported-delta")
        self.assertEqual(len(row["proof_chain"]), 2)
        self.assertEqual(self.check_exit(0)["runtime_readiness"]["accepted_reviews"], 1)

    def test_import_pin_and_url_change_invalid(self):
        self.imports()
        self.fixture.write("supply-chain/imports.lock", (self.fixture.root / "supply-chain/imports.lock").read_text() + "\n# changed\n")
        self.check_exit(2)
        self.imports()
        self.fixture.policy["imports"][0]["url"] = "https://example.invalid/other"
        self.fixture.json("supply-chain/dependency-policy.json", self.fixture.policy)
        self.check_exit(2)

    def test_import_orphan_and_cycle_have_no_credit(self):
        self.imports()
        raw = '[[audits.fixture-base.audits.alpha]]\nwho="Fixture Reviewer"\ncriteria="safe-to-deploy"\ndelta="1.0.0 -> 0.9.0"\n[[audits.fixture-delta.audits.alpha]]\nwho="Fixture Reviewer"\ncriteria="safe-to-deploy"\ndelta="0.9.0 -> 1.0.0"\n'
        self.fixture.write("supply-chain/imports.lock", raw)
        for item in self.fixture.policy["imports"]:
            item["snapshot_sha256"] = self.fixture.hash("supply-chain/imports.lock")
        self.fixture.json("supply-chain/dependency-policy.json", self.fixture.policy)
        self.assertEqual(self.check_exit(0)["runtime_readiness"]["accepted_reviews"], 0)

    def test_registry_source_switch_cannot_use_imported_review(self):
        self.imports()
        old = common.package_id(self.fixture.package("alpha"))
        self.fixture.package("alpha")["source"] = "registry+https://example.invalid/index"
        new = common.package_id(self.fixture.package("alpha"))
        for edge in self.fixture.graph["graphs"][0]["edges"]:
            edge["from"] = new if edge["from"] == old else edge["from"]
            edge["to"] = new if edge["to"] == old else edge["to"]
        self.fixture.sync_graph()
        report = self.check_exit(1)
        self.assertEqual(report["runtime_readiness"]["accepted_reviews"], 0)

    def test_source_less_nonmember_is_not_first_party(self):
        old = common.package_id(self.fixture.package("alpha"))
        self.fixture.package("alpha")["source"] = "workspace"
        self.fixture.package("alpha")["checksum"] = None
        new = common.package_id(self.fixture.package("alpha"))
        for edge in self.fixture.graph["graphs"][0]["edges"]:
            edge["from"] = new if edge["from"] == old else edge["from"]
            edge["to"] = new if edge["to"] == old else edge["to"]
        self.fixture.sync_graph()
        report = self.check_exit(1)
        row = next(r for r in report["packages"] if r["name"] == "alpha")
        self.assertNotEqual(row["status"], "first-party")

    def test_duplicate_review_exception_and_future_decision_invalid(self):
        self.fixture.exception()
        self.fixture.exceptions["entries"].append(copy.deepcopy(self.fixture.exceptions["entries"][0]))
        self.fixture.json("supply-chain/dependency-exceptions.json", self.fixture.exceptions)
        self.check_exit(2)
        self.fixture.exceptions["entries"] = []
        self.fixture.json("supply-chain/dependency-exceptions.json", self.fixture.exceptions)
        self.fixture.policy["decision"]["recorded_on"] = "2026-10-03"
        self.fixture.json("supply-chain/dependency-policy.json", self.fixture.policy)
        self.check_exit(2)

    def test_new_name_named_by_gate(self):
        pkg = {"name": "new-runtime", "version": "1.0.0", "source": REGISTRY, "checksum": "d" * 64}
        self.fixture.packages.append(pkg)
        self.fixture.packages.sort(key=common.identity)
        self.fixture.graph["graphs"][0]["edges"].append({"from": common.package_id(self.fixture.package("forge")),
                                                         "to": common.package_id(pkg), "kind": "normal"})
        self.fixture.graph["graphs"][0]["edges"].sort(key=lambda e: (e["from"], e["to"], e["kind"]))
        self.fixture.sync_graph()
        self.assertIn(common.package_id(pkg), self.check_exit(1)["new_unvetted_runtime"])

    def test_owned_sidecar_integrates_native_exemption(self):
        self.fixture.exception()
        self.fixture.write("supply-chain/config.toml", self.fixture.config_prefix + '[[exemptions.alpha]]\nversion="1.0.0"\ncriteria="safe-to-deploy"\n')
        report = self.check_exit(0, command="validate-store")
        self.assertEqual(report["current_exemptions"][0]["status"], "owned-exception")

    def test_strict_accepts_owned_exact_runtime_set(self):
        self.fixture.exception("alpha")
        self.fixture.exception("delta")
        report = self.check_exit(0, strict=True)
        self.assertTrue(report["runtime_readiness"]["ready"])
        self.assertEqual(report["runtime_readiness"]["fresh_owned_exceptions"], 2)

    def test_stale_exception_action_survives_new_audit(self):
        self.fixture.audit()
        self.fixture.exception(review_by="2026-10-01")
        report = self.check_exit(1)
        self.assertEqual(next(r for r in report["packages"] if r["name"] == "alpha")["status"], "local-full")
        self.assertEqual(report["trends"]["stale_exception_count"], 1)

    def test_checksumless_path_cannot_be_excepted(self):
        old = common.package_id(self.fixture.package("alpha"))
        self.fixture.package("alpha")["source"] = "workspace"
        self.fixture.package("alpha")["checksum"] = None
        new = common.package_id(self.fixture.package("alpha"))
        for edge in self.fixture.graph["graphs"][0]["edges"]:
            edge["from"] = new if edge["from"] == old else edge["from"]
            edge["to"] = new if edge["to"] == old else edge["to"]
        self.fixture.sync_graph()
        self.fixture.exception()
        self.check_exit(2)

    def test_valid_delta_chain_retains_independent_alternate_base(self):
        final = self.fixture.package("alpha")["checksum"]
        for ver, checksum in (("0.7.0", "a" * 64), ("1.0.0", final)):
            self.fixture.audit(ver=ver, checksums={ver: checksum})
        for old, new, old_hash, new_hash in (("0.7.0", "0.8.0", "a" * 64, "b" * 64),
                                            ("0.8.0", "0.9.0", "b" * 64, "c" * 64),
                                            ("1.0.0", "0.9.0", final, "c" * 64),
                                            ("0.9.0", "1.0.0", "c" * 64, final)):
            self.fixture.audit(delta=old + " -> " + new, checksums={old: old_hash, new: new_hash})
        self.check_exit(0)

    def test_oversized_and_special_cli_inputs_complete_report(self):
        p = self.fixture.root / "supply-chain/dependency-exceptions.json"
        with p.open("wb") as stream:
            stream.truncate(common.MAX_INPUT_BYTES + 1)
        self.check_exit(2)
        if os.name == "posix":
            p.unlink()
            os.mkfifo(p)
            self.check_exit(2)

    def test_graph_qualification_detects_deleted_edges_and_proc_macro_flip(self):
        import dependency_graph as graph_tool
        root = self.fixture.root.resolve()
        metadata = {"workspace_root": str(root), "workspace_members": [], "workspace_default_members": [],
                    "packages": [], "resolve": {"nodes": []}}
        for p in self.fixture.packages:
            pid = common.package_id(p)
            member = p["name"] == "forge"
            if member:
                metadata["workspace_members"].append(pid)
                metadata["workspace_default_members"].append(pid)
            metadata["packages"].append({**p, "source": None if member else p["source"], "id": pid,
                "manifest_path": str(root / "Cargo.toml") if member else "/external-cache/" + p["name"] + "/Cargo.toml",
                "targets": [{"kind": ["proc-macro" if p["name"] == "gamma" else "lib"]}]})
            deps = [{"pkg": edge["to"], "dep_kinds": [{"kind": None if edge["kind"] == "normal" else edge["kind"], "target": None}]}
                    for edge in self.fixture.graph["graphs"][0]["edges"] if edge["from"] == pid]
            metadata["resolve"]["nodes"].append({"id": pid, "deps": deps})
        with patch.object(graph_tool, "_cargo_metadata", return_value=metadata), patch.object(graph_tool, "_publish", side_effect=AssertionError("check wrote output")):
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(graph_tool.main(["check", "--root", str(root)]), 0)
            for kind in ("deleted-edges", "proc-macro"):
                with self.subTest(kind=kind):
                    changed = copy.deepcopy(self.fixture.graph)
                    if kind == "deleted-edges":
                        changed["graphs"][0]["edges"] = []
                    else:
                        next(p for p in changed["packages"] if p["name"] == "alpha")["proc_macro"] = True
                    self.fixture.json("supply-chain/dependency-graph.json", changed)
                    before = (root / "supply-chain/dependency-graph.json").read_bytes()
                    with contextlib.redirect_stdout(io.StringIO()):
                        self.assertEqual(graph_tool.main(["check", "--root", str(root)]), 1)
                    self.assertEqual((root / "supply-chain/dependency-graph.json").read_bytes(), before)

    def test_refresh_cannot_overwrite_legacy_baseline_or_alias(self):
        import dependency_graph as graph_tool
        with patch.object(graph_tool, "_cargo_metadata", side_effect=AssertionError("invalid output ran Cargo")):
            for args in (["--output", "supply-chain/dependency-graph.json", "--baseline-output", "supply-chain/legacy-baseline.json"],
                         ["--output", "supply-chain/legacy-baseline.json"], ["--output", "supply-chain/CON.json"],
                         ["--output", "supply-chain/x:y.json"]):
                with self.subTest(args=args), contextlib.redirect_stderr(io.StringIO()):
                    self.assertEqual(graph_tool.main(["refresh", "--root", str(self.fixture.root.resolve()), *args]), 2)

    def test_workspace_roots_follow_cargo_default_members(self):
        fixture = self.fixture
        fixture.workspace_names.add("member_two")
        (fixture.root / "members/member_two").mkdir(parents=True)
        fixture.write("members/member_two/Cargo.toml", '[package]\nname="member_two"\nversion="1.0.0"\n')
        fixture.packages.append({"name": "member_two", "version": "1.0.0", "source": "workspace", "checksum": None})
        fixture.packages.sort(key=common.identity)
        root_manifest = '[package]\nname="forge"\nversion="2.0.0"\n[workspace]\nmembers=["members/member_two"]\n'
        fixture.write("Cargo.toml", root_manifest)
        fixture.sync_graph()
        fixture.graph["inputs"]["manifests"]["members/member_two/Cargo.toml"] = fixture.hash("members/member_two/Cargo.toml")
        fixture.json("supply-chain/dependency-graph.json", fixture.graph)
        report = self.check_exit(0)
        self.assertEqual(report["runtime_readiness"]["denominator"], 2)
        self.assertEqual(next(r for r in report["packages"] if r["name"] == "member_two")["status"], "first-party")
        fixture.write("Cargo.toml", root_manifest + 'default-members=["members/member_two"]\n')
        fixture.sync_graph()
        fixture.graph["inputs"]["manifests"]["members/member_two/Cargo.toml"] = fixture.hash("members/member_two/Cargo.toml")
        fixture.graph["graphs"][0]["roots"] = [common.package_id(fixture.package("member_two"))]
        fixture.json("supply-chain/dependency-graph.json", fixture.graph)
        self.assertEqual(self.check_exit(0)["runtime_readiness"]["denominator"], 0)

    def test_overbound_locked_collection_emits_invalid_report(self):
        self.fixture.write("Cargo.lock", 'version=4\n' + '[[package]]\nname="alpha"\nversion="1.0.0"\n' * 5001)
        report = self.check_exit(2)
        self.assertTrue(any("5000" in error for error in report["errors"]))

    def test_overbound_proof_graph_invalid(self):
        import dependency_inventory as inventory_tool
        record = {"target": "1.0.0", "base": None, "criteria": ["safe-to-deploy"], "origin": "local",
                  "record_sha256": "a" * 64, "checksums": {"1.0.0": "b" * 64}}
        with self.assertRaisesRegex(common.InputError, "1000"):
            inventory_tool.proof_paths([record] * 1001, "safe-to-deploy")


class CommonSafetyTests(unittest.TestCase):
    def test_json_and_toml_decoder_bounds(self):
        for raw in (b'{"n":NaN}', b'{"n":1e400}', b'{"n":-1e400}', b'{"n":0,"n":1}', b"1" * 5000,
                    b"[" * 65 + b"0" + b"]" * 65, b"\xff"):
            with self.subTest(raw=raw[:24]), self.assertRaises(common.InputError):
                common.load_json(raw)
        with self.assertRaises(common.InputError):
            common.load_toml(b"x=" + b"1" * 5000)
        self.assertEqual(common.load_json(b'{"n":1.5}'), {"n": 1.5})

    def test_raw_crlf_bytes_preserve_locked_digest(self):
        with tempfile.TemporaryDirectory() as root:
            raw = b"first\r\nsecond\r\n"
            Path(root, "Cargo.lock").write_bytes(raw)
            self.assertEqual(common.read_bytes(root, "Cargo.lock"), raw)

    def test_static_path_aliases(self):
        with tempfile.TemporaryDirectory() as root:
            for path in ("../Cargo.lock", "/Cargo.lock", "C:/outside/file", "Cargo.lock:stream", "aux", "a.", "a ", "a//b", "a\\b"):
                with self.subTest(path=path), self.assertRaises(common.InputError):
                    common.read_bytes(root, path)

    def test_symlink_hardlink_fifo_and_byte_bound(self):
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            (root / "file").write_bytes(b"hello")
            self.assertEqual(common.read_bytes(root, "file", 5), b"hello")
            with self.assertRaises(common.InputError):
                common.read_bytes(root, "file", 4)
            if os.name == "posix":
                (root / "link").symlink_to("file")
                os.mkfifo(root / "fifo")
                for name in ("link", "fifo"):
                    with self.subTest(name=name), self.assertRaises(common.InputError):
                        common.read_bytes(root, name)
                os.link(root / "file", root / "alias")
                with self.assertRaises(common.InputError):
                    common.read_bytes(root, "file")

    def test_release_matrix_structural_contract(self):
        good = b"jobs:\n  build:\n    strategy:\n      matrix:\n        include:\n          - target: x86_64-unknown-linux-gnu\n            os: ubuntu-latest\n            archive: tar.gz\n"
        self.assertEqual(common.read_release_targets(good), [TARGET])
        for raw in (good.replace(b"strategy:", b"env:"), good.replace(b"            os: ubuntu-latest\n", b""),
                    good + b"            os: macos-latest\n", good.replace(b"archive: tar.gz", b"archive: ${{ expression }}")):
            with self.subTest(raw=raw), self.assertRaises(common.InputError):
                common.read_release_targets(raw)


if __name__ == "__main__":
    unittest.main()
