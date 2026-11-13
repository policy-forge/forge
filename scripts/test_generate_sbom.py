#!/usr/bin/env python3
"""Black-box tests for the offline draft SBOM generator (synthetic fixtures)."""
from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True

import dependency_common as common
import generate_sbom

SCRIPT = Path(__file__).resolve().with_name("generate_sbom.py")
REGISTRY = generate_sbom.REGISTRY
TARGET = "x86_64-unknown-linux-gnu"


def ident(name: str, version: str, source: str = REGISTRY) -> str:
    return json.dumps([name, version, source], separators=(",", ":"))


class SbomFixture:
    """Minimal locked workspace: forge -> alpha (normal) -> beta; gamma is dev-only."""

    def __init__(self, root: Path):
        self.root = root
        self.packages = [
            {"name": "forge", "version": "9.9.9", "source": "workspace",
             "checksum": None, "proc_macro": False, "workspace_member": True},
            {"name": "alpha", "version": "1.0.0", "source": REGISTRY,
             "checksum": "a" * 64, "proc_macro": False, "workspace_member": False},
            {"name": "beta", "version": "2.0.0+spec-1", "source": REGISTRY,
             "checksum": "b" * 64, "proc_macro": True, "workspace_member": False},
            {"name": "gamma", "version": "3.0.0", "source": REGISTRY,
             "checksum": "c" * 64, "proc_macro": False, "workspace_member": False},
        ]
        self.edges = [
            {"from": ident("forge", "9.9.9", "workspace"), "kind": "normal",
             "to": ident("alpha", "1.0.0")},
            {"from": ident("forge", "9.9.9", "workspace"), "kind": "dev",
             "to": ident("gamma", "3.0.0")},
            {"from": ident("alpha", "1.0.0"), "kind": "normal",
             "to": ident("beta", "2.0.0+spec-1")},
        ]
        self.write_manifest()
        self.write_workflow()
        self.sync()

    def write(self, relative: str, text: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def hash(self, relative: str) -> str:
        return common.sha256((self.root / relative).read_bytes())

    def write_manifest(self) -> None:
        self.write("Cargo.toml", '[package]\nname = "forge"\nversion = "9.9.9"\n'
                                 'edition = "2024"\nlicense = "MIT"\n')

    def write_workflow(self) -> None:
        self.write(".github/workflows/release.yml",
                   "jobs:\n  build:\n    strategy:\n      matrix:\n        include:\n"
                   f"          - target: {TARGET}\n            os: ubuntu-latest\n"
                   "            archive: tar.gz\n")

    def sync(self) -> None:
        locked = ["version = 4\n"]
        for pkg in self.packages:
            locked.append(f'[[package]]\nname="{pkg["name"]}"\nversion="{pkg["version"]}"\n')
            deps = sorted({json.loads(edge["to"])[0] for edge in self.edges
                           if json.loads(edge["from"])[0] == pkg["name"]})
            if deps:
                locked.append("dependencies=" + json.dumps(deps) + "\n")
            if pkg["source"] != "workspace":
                locked.append(f'source="{pkg["source"]}"\nchecksum="{pkg["checksum"]}"\n')
        self.write("Cargo.lock", "".join(locked))
        self.graph = {
            "format": "forge.dependency-graph/1",
            "features": ["default"],
            "targets": [TARGET],
            "packages": self.packages,
            "graphs": [{"target": TARGET, "roots": [ident("forge", "9.9.9", "workspace")],
                        "edges": self.edges}],
            "inputs": {"lock_sha256": self.hash("Cargo.lock"),
                       "manifests": {"Cargo.toml": self.hash("Cargo.toml")},
                       "release_workflow_sha256": self.hash(".github/workflows/release.yml")},
        }
        self.write("supply-chain/dependency-graph.json", json.dumps(self.graph))

    def build(self, as_of: str = "2026-11-12") -> dict:
        return generate_sbom.build_sbom(self.root, as_of)

    def run_cli(self, *extra: str) -> subprocess.CompletedProcess:
        return subprocess.run([sys.executable, "-B", str(SCRIPT), "--root", str(self.root),
                               "--as-of", "2026-11-12", *extra],
                              capture_output=True, text=True, check=False)


class GenerateSbomTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.fixture = SbomFixture(Path(self.temp.name))

    def component(self, document: dict, name: str) -> dict:
        return next(c for c in document["components"] if c["name"] == name)

    def test_deterministic_canonical_output(self):
        first = common.canonical_bytes(self.fixture.build())
        second = common.canonical_bytes(self.fixture.build())
        self.assertEqual(first, second)
        self.assertTrue(first.endswith(b"\n"))
        self.assertEqual(first, json.dumps(json.loads(first), sort_keys=True,
                                           separators=(",", ":"), ensure_ascii=True).encode() + b"\n")

    def test_spec_shape_versions_checksums_and_purls(self):
        document = self.fixture.build()
        self.assertEqual(document["bomFormat"], "CycloneDX")
        self.assertEqual(document["specVersion"], "1.6")
        self.assertEqual(document["version"], 1)
        self.assertTrue(document["serialNumber"].startswith("urn:uuid:"))
        self.assertEqual(document["metadata"]["component"]["name"], "forge")
        self.assertEqual(document["metadata"]["component"]["version"], "9.9.9")
        self.assertEqual(document["metadata"]["licenses"], [{"license": {"id": "MIT"}}])
        self.assertEqual(len(document["components"]), 3)
        alpha = self.component(document, "alpha")
        self.assertEqual(alpha["purl"], "pkg:cargo/alpha@1.0.0")
        self.assertEqual(alpha["hashes"], [{"alg": "SHA-256", "content": "a" * 64}])
        self.assertEqual(alpha["externalReferences"][0]["url"],
                         "https://static.crates.io/crates/alpha/alpha-1.0.0.crate")
        beta = self.component(document, "beta")
        self.assertEqual(beta["purl"], "pkg:cargo/beta@2.0.0+spec-1")
        self.assertEqual(beta["version"], "2.0.0+spec-1")

    def test_scope_separates_runtime_from_dev_only(self):
        document = self.fixture.build()
        self.assertEqual(self.component(document, "alpha")["scope"], "required")
        self.assertEqual(self.component(document, "beta")["scope"], "required")
        self.assertEqual(self.component(document, "gamma")["scope"], "optional")
        root = next(d for d in document["dependencies"] if d["ref"] == "pkg:cargo/forge@9.9.9")
        self.assertEqual(root["dependsOn"], ["pkg:cargo/alpha@1.0.0", "pkg:cargo/gamma@3.0.0"])
        refs = {c["bom-ref"] for c in document["components"]}
        refs.add(document["metadata"]["component"]["bom-ref"])
        for entry in document["dependencies"]:
            self.assertIn(entry["ref"], refs)
            self.assertTrue(set(entry["dependsOn"]) <= refs)

    def test_as_of_changes_serial_only(self):
        first = self.fixture.build("2026-11-12")
        second = self.fixture.build("2026-11-13")
        self.assertNotEqual(first["serialNumber"], second["serialNumber"])
        first.pop("serialNumber"), second.pop("serialNumber")
        first["metadata"]["properties"] = [p for p in first["metadata"]["properties"]
                                           if p["name"] != "forge:as-of"]
        second["metadata"]["properties"] = [p for p in second["metadata"]["properties"]
                                            if p["name"] != "forge:as-of"]
        self.assertEqual(first, second)

    def test_lock_drift_refused(self):
        (self.fixture.root / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
        with self.assertRaises(common.InputError):
            self.fixture.build()

    def test_manifest_drift_refused(self):
        self.fixture.write("Cargo.toml", '[package]\nname = "forge"\nversion = "9.9.9"\n'
                                         'license = "Apache-2.0"\n')
        with self.assertRaises(common.InputError):
            self.fixture.build()

    def test_checksum_mismatch_refused(self):
        # Keep the lock pin valid but let the graph disagree with the lock checksum.
        self.fixture.graph["packages"][1]["checksum"] = "d" * 64
        self.fixture.write("supply-chain/dependency-graph.json", json.dumps(self.fixture.graph))
        with self.assertRaises(common.InputError):
            self.fixture.build()

    def test_unknown_edge_target_refused(self):
        self.fixture.edges.append({"from": ident("alpha", "1.0.0"), "kind": "normal",
                                   "to": ident("ghost", "0.0.1")})
        self.fixture.sync()
        with self.assertRaises(common.InputError):
            self.fixture.build()

    def _build_with_edge(self, edge: dict) -> None:
        # The fixture's sync() only accepts well-formed edges, so write the malformed
        # edge into the graph file directly, as the other graph-corruption tests do.
        self.fixture.graph["graphs"][0]["edges"].append(edge)
        self.fixture.write("supply-chain/dependency-graph.json", json.dumps(self.fixture.graph))
        self.fixture.build()

    def test_non_string_edge_endpoint_refused(self):
        # A non-string `from`/`to` must fail as InputError, not an uncaught AttributeError.
        for field in ("from", "to"):
            for bad in (None, 7, ["alpha", "1.0.0", "x"], {"name": "alpha"}):
                with self.subTest(field=field, bad=bad):
                    edge = {"from": ident("alpha", "1.0.0"), "kind": "normal",
                            "to": ident("alpha", "1.0.0")}
                    edge[field] = bad
                    with self.assertRaises(common.InputError):
                        self._build_with_edge(edge)

    def test_missing_edge_endpoint_refused(self):
        with self.assertRaises(common.InputError):
            self._build_with_edge({"kind": "normal", "to": ident("alpha", "1.0.0")})

    def test_malformed_graph_target_blocks_refused(self):
        # Wrong-typed graphs/targets must fail as InputError, not a raw
        # AttributeError/TypeError from the target-population comparison.
        good = self.fixture.graph["graphs"][0]
        for graphs, targets in (
            ([["not", "an", "object"]], [TARGET]),   # non-dict graph entry
            ([good, good], [1, TARGET]),             # mixed-type targets
            ([good, good], [None, TARGET]),
        ):
            with self.subTest(graphs=repr(graphs)[:40], targets=targets):
                self.fixture.graph["graphs"] = graphs
                self.fixture.graph["targets"] = targets
                self.fixture.write("supply-chain/dependency-graph.json",
                                   json.dumps(self.fixture.graph))
                with self.assertRaises(common.InputError):
                    self.fixture.build()

    def test_missing_root_refused(self):
        self.fixture.graph["graphs"][0]["roots"] = []
        self.fixture.write("supply-chain/dependency-graph.json", json.dumps(self.fixture.graph))
        with self.assertRaises(common.InputError):
            self.fixture.build()

    def test_bad_date_refused(self):
        for bad in ("2026-13-01", "2026-11-12T00:00:00Z", "yesterday"):
            with self.subTest(bad=bad), self.assertRaises((common.InputError, ValueError)):
                self.fixture.build(bad)

    def test_cli_writes_file_and_refuses_drift(self):
        out = self.fixture.root / "sbom.json"
        done = self.fixture.run_cli("--output", str(out))
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(json.loads(out.read_bytes()), self.fixture.build())
        (self.fixture.root / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
        failed = self.fixture.run_cli("--output", str(out))
        self.assertEqual(failed.returncode, 2)
        self.assertIn("does not match the graph pin", failed.stderr)


class CommittedStoreTests(unittest.TestCase):
    """The real committed store must produce a self-consistent draft SBOM."""

    @classmethod
    def setUpClass(cls):
        cls.root = Path(__file__).resolve().parents[1]
        cls.document = generate_sbom.build_sbom(cls.root, "2026-11-12")

    def test_population_and_referential_integrity(self):
        document = self.document
        locked = common.load_toml((self.root / "Cargo.lock").read_bytes())["package"]
        self.assertEqual(len(document["components"]), len(locked) - 1)
        refs = {c["bom-ref"] for c in document["components"]}
        refs.add(document["metadata"]["component"]["bom-ref"])
        self.assertEqual(len(refs), len(locked))
        for entry in document["dependencies"]:
            self.assertIn(entry["ref"], refs)
            self.assertTrue(set(entry["dependsOn"]) <= refs)
        for component in document["components"]:
            self.assertRegex(component["hashes"][0]["content"], r"^[0-9a-f]{64}$")
            self.assertEqual(component["scope"] in ("required", "optional"), True)

    def test_runtime_stack_required_and_dev_tools_optional(self):
        by_name = {c["name"]: c for c in self.document["components"]}
        for name in ("hyper", "argon2", "tokio", "quick-xml", "serde_json"):
            self.assertEqual(by_name[name]["scope"], "required", name)
        for name in ("insta", "criterion"):
            self.assertEqual(by_name[name]["scope"], "optional", name)

    def test_committed_draft_artifact_matches_generator(self):
        artifact = self.root / "docs/release/sbom-2026-11-12.json"
        self.assertEqual(artifact.read_bytes(), common.canonical_bytes(self.document))


if __name__ == "__main__":
    unittest.main()