#!/usr/bin/env python3
"""Offline, attributable inventory of the committed Cargo dependency store.

No consuming command runs Cargo, reads its cache, downloads data, or changes files.
An accepted record is an asserted signoff, not authenticated identity or a claim
that a crate is safe. Metadata closure is not proof of final artifact contents.
"""

from __future__ import annotations

import argparse
from collections import Counter, defaultdict, deque
from datetime import date
import fnmatch
import json
from pathlib import Path, PurePosixPath
import re
import statistics
import sys

# Importing helpers must not create cache files in a consuming command.
sys.dont_write_bytecode = True

from dependency_common import (
    InputError, canonical_bytes, identity, package_id, read_bytes, load_json,
    load_toml, sha256, read_release_targets,
)


STORE = "supply-chain/"
PATHS = {
    "graph": STORE + "dependency-graph.json",
    "legacy": STORE + "legacy-baseline.json",
    "policy": STORE + "dependency-policy.json",
    "exceptions": STORE + "dependency-exceptions.json",
    "reviews": STORE + "dependency-reviews.json",
}
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"
BUILTINS = {"safe-to-run", "safe-to-deploy"}
HEX = re.compile(r"[0-9a-f]{64}\Z")
SEMVER = re.compile(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?\Z")
NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]*\Z")
MAX_PACKAGES = 5000
MAX_RECORDS = 5000
MAX_EDGES = 100000
MAX_PROOF_RECORDS = 1000
MAX_PROOF_CHAIN = 256


def obj(value, keys, where, optional=()):
    if not isinstance(value, dict):
        raise InputError(f"{where}: expected object")
    missing = set(keys) - value.keys()
    unknown = value.keys() - set(keys) - set(optional)
    if missing or unknown:
        raise InputError(f"{where}: missing {sorted(missing)}, unknown {sorted(unknown)}")
    return value


def array(value, where, limit=MAX_EDGES):
    if not isinstance(value, list):
        raise InputError(f"{where}: expected array")
    if len(value) > limit:
        raise InputError(f"{where}: array exceeds {limit} entries")
    return value


def string(value, where, limit=4096, multiline=False):
    if not isinstance(value, str) or not value.strip() or len(value) > limit:
        raise InputError(f"{where}: expected nonempty bounded string")
    if any((ord(c) < 32 and not (multiline and c in "\n\r\t")) or ord(c) == 127 for c in value):
        raise InputError(f"{where}: control characters are forbidden")
    return value


def digest(value, where, nullable=False):
    if nullable and value is None:
        return None
    if not isinstance(value, str) or not HEX.fullmatch(value):
        raise InputError(f"{where}: expected lowercase SHA-256")
    return value


def version(value, where):
    string(value, where, 256)
    if not SEMVER.fullmatch(value):
        raise InputError(f"{where}: expected exact semantic version")
    pre_release = value.split("+", 1)[0].partition("-")[2]
    if any(part.isdigit() and len(part) > 1 and part.startswith("0") for part in pre_release.split(".")):
        raise InputError(f"{where}: numeric prerelease identifiers cannot have leading zeroes")
    return value


def day(value, where):
    if not isinstance(value, str) or not re.fullmatch(r"\d{4}-\d{2}-\d{2}", value):
        raise InputError(f"{where}: expected YYYY-MM-DD")
    try:
        return date.fromisoformat(value)
    except ValueError as error:
        raise InputError(f"{where}: invalid calendar date") from error


def criteria(value, where):
    values = [value] if isinstance(value, str) else array(value, where)
    if not values or any(not isinstance(v, str) or v not in BUILTINS for v in values):
        raise InputError(f"{where}: only built-in cargo-vet criteria are supported")
    if len(values) != len(set(values)):
        raise InputError(f"{where}: duplicate criterion")
    return sorted(values)


def satisfies(values, required):
    return required in values or (required == "safe-to-run" and "safe-to-deploy" in values)


def row_identity(value, where, extra=()):
    obj(value, ("name", "version", "source", "checksum", *extra), where)
    name = string(value["name"], where + ".name", 256)
    if not NAME.fullmatch(name):
        raise InputError(f"{where}.name: invalid crate name")
    version(value["version"], where + ".version")
    source = string(value["source"], where + ".source")
    checksum = digest(value["checksum"], where + ".checksum", nullable=True)
    if source.startswith("registry+") and checksum is None:
        raise InputError(f"{where}: registry identity needs checksum")
    return identity(value)


def sidecar(value, name, keys):
    obj(value, ("format", *keys), name)
    if value["format"] != f"forge.dependency-{name}/1":
        raise InputError(f"{name}: unsupported format")
    return value


def native_records(value, where):
    """Validate native audit objects without interpreting arbitrary new formats."""
    result = []
    seen = set()
    if not isinstance(value, dict):
        raise InputError(f"{where}: expected crate audit table")
    for crate in sorted(value):
        if not NAME.fullmatch(crate):
            raise InputError(f"{where}: invalid crate name")
        for record in array(value[crate], where + "." + crate, MAX_RECORDS):
            if len(result) >= MAX_RECORDS:
                raise InputError(f"{where}: audits exceed {MAX_RECORDS} total records")
            obj(record, ("who", "criteria"), where + "." + crate,
                ("version", "delta", "notes", "importable"))
            authors = [record["who"]] if isinstance(record["who"], str) else array(record["who"], where + ".who")
            if not authors:
                raise InputError(f"{where}.{crate}: missing auditor")
            for author in authors:
                string(author, where + ".who")
            values = criteria(record["criteria"], where + ".criteria")
            if ("version" in record) == ("delta" in record):
                raise InputError(f"{where}.{crate}: exactly one of version or delta required")
            if "version" in record:
                base, target = None, version(record["version"], where + ".version")
            else:
                raw = string(record["delta"], where + ".delta", 1024)
                pair = raw.split(" -> ")
                if len(pair) != 2:
                    raise InputError(f"{where}.{crate}: delta requires exact 'old -> new' pair")
                base, target = (version(v, where + ".delta") for v in pair)
                if base == target:
                    raise InputError(f"{where}.{crate}: delta cannot be self-referential")
            if "notes" in record:
                string(record["notes"], where + ".notes", 65536, multiline=True)
            if "importable" in record and not isinstance(record["importable"], bool):
                raise InputError(f"{where}.{crate}: importable must be boolean")
            record_hash = sha256(canonical_bytes(record))
            if (crate, record_hash) in seen:
                raise InputError(f"{where}.{crate}: duplicate audit record")
            seen.add((crate, record_hash))
            result.append({"crate": crate, "base": base, "target": target,
                           "criteria": values, "auditors": sorted(authors),
                           "record_sha256": record_hash,
                           "native_record": record})
    return sorted(result, key=lambda r: (r["crate"], r["target"], r["base"] or "", r["record_sha256"]))


def validate_lock(lock):
    obj(lock, ("version", "package"), "Cargo.lock", ("metadata",))
    if type(lock["version"]) is not int or lock["version"] not in (3, 4):
        raise InputError("Cargo.lock: unsupported version")
    packages = {}
    for raw in array(lock["package"], "Cargo.lock.package", MAX_PACKAGES):
        obj(raw, ("name", "version"), "Cargo.lock.package", ("source", "checksum", "dependencies", "replace"))
        package = {"name": raw["name"], "version": raw["version"],
                   "source": raw.get("source") or "workspace", "checksum": raw.get("checksum")}
        key = row_identity(package, "Cargo.lock.package")
        if key in packages:
            raise InputError(f"Cargo.lock: duplicate identity {package_id(package)}")
        if "dependencies" in raw:
            for dep in array(raw["dependencies"], "Cargo.lock.dependencies"):
                string(dep, "Cargo.lock.dependencies")
        packages[key] = package
    if not packages:
        raise InputError("Cargo.lock: empty package set")
    return packages


def validate_graph(graph, packages, root, lock_hash):
    sidecar(graph, "graph", ("inputs", "targets", "features", "packages", "graphs"))
    inputs = obj(graph["inputs"], ("lock_sha256", "manifests", "release_workflow_sha256"), "graph.inputs")
    if digest(inputs["lock_sha256"], "graph.lock_sha256") != lock_hash:
        raise InputError("graph: stale Cargo.lock digest")
    lock_bytes = read_bytes(root, "Cargo.lock")
    if sha256(lock_bytes) != lock_hash:
        raise InputError("graph: Cargo.lock changed during evaluation")
    lock_entries = load_toml(lock_bytes)["package"]
    allowed_edges = set()
    for entry in lock_entries:
        parent = package_id(entry)
        for dep in entry.get("dependencies", []):
            match = re.fullmatch(r"([^ ]+)(?: ([^ ]+))?(?: \((.+)\))?", dep)
            if not match:
                raise InputError("graph: malformed lock dependency reference")
            name, ver, source = match.groups()
            candidates = [p for p in packages.values() if p["name"] == name
                          and (ver is None or p["version"] == ver)
                          and (source is None or p["source"] == source)]
            if len(candidates) != 1:
                raise InputError(f"graph: missing/ambiguous lock dependency {dep}")
            allowed_edges.add((parent, package_id(candidates[0])))
    manifests = inputs["manifests"]
    if not isinstance(manifests, dict) or "Cargo.toml" not in manifests:
        raise InputError("graph: manifests must include Cargo.toml")
    for path, expected in sorted(manifests.items()):
        if Path(path).name != "Cargo.toml":
            raise InputError("graph: invalid manifest path")
        if digest(expected, "graph.manifest digest") != sha256(read_bytes(root, path)):
            raise InputError(f"graph: stale manifest digest {path}")
    parsed_manifests = {path: load_toml(read_bytes(root, path)) for path in sorted(manifests)}
    root_manifest = parsed_manifests["Cargo.toml"]
    declared_members = {"Cargo.toml"} if "package" in root_manifest else set()
    workspace_table = root_manifest.get("workspace", {})
    if not isinstance(workspace_table, dict):
        raise InputError("graph: invalid workspace manifest")
    workspace_package = workspace_table.get("package", {})
    if not isinstance(workspace_package, dict):
        raise InputError("graph: malformed workspace package defaults")
    member_patterns = workspace_table.get("members", [])
    excluded_patterns = workspace_table.get("exclude", [])
    for pattern in [*array(member_patterns, "manifest.workspace.members"),
                    *array(excluded_patterns, "manifest.workspace.exclude")]:
        string(pattern, "manifest.workspace.path")
        if pattern.startswith("/") or ".." in pattern.split("/") or "\\" in pattern:
            raise InputError("graph: workspace member path traverses repository")
    for pattern in member_patterns:
        candidate = pattern.rstrip("/") + "/Cargo.toml"
        matches = {p for p in parsed_manifests if fnmatch.fnmatchcase(p, candidate)}
        if not matches:
            raise InputError(f"graph: workspace member manifest missing {candidate}")
        declared_members.update(matches)
    # Cargo automatically includes in-repository path dependencies as workspace
    # members when a workspace is declared, unless explicitly excluded. Pin all
    # path manifests even for a standalone package's nonmember dependencies.
    local_paths = set()
    for path, manifest in parsed_manifests.items():
        manifest_workspace = manifest.get("workspace", {})
        if not isinstance(manifest_workspace, dict):
            raise InputError(f"graph: malformed workspace table {path}")
        tables = [manifest.get(key, {}) for key in ("dependencies", "build-dependencies", "dev-dependencies")]
        tables.append(manifest_workspace.get("dependencies", {}))
        target_tables = manifest.get("target", {})
        if not isinstance(target_tables, dict):
            raise InputError(f"graph: malformed target dependency table {path}")
        for target_table in target_tables.values():
            if not isinstance(target_table, dict):
                raise InputError(f"graph: malformed target dependency table {path}")
            tables.extend(target_table.get(key, {}) for key in ("dependencies", "build-dependencies", "dev-dependencies"))
        for table in tables:
            if not isinstance(table, dict):
                raise InputError(f"graph: malformed dependency table {path}")
            for dep in table.values():
                if not isinstance(dep, dict) or "path" not in dep:
                    continue
                local = string(dep["path"], "manifest.dependency.path")
                if local.startswith("/") or "\\" in local:
                    raise InputError("graph: local dependency must stay within repository")
                parts = list(PurePosixPath(path).parent.parts)
                for part in local.split("/"):
                    if part in ("", "."):
                        continue
                    if part == "..":
                        if not parts:
                            raise InputError("graph: local dependency escapes repository")
                        parts.pop()
                    else:
                        parts.append(part)
                candidate = "/".join([*parts, "Cargo.toml"])
                if candidate not in parsed_manifests:
                    raise InputError(f"graph: unpinned local dependency manifest {candidate}")
                local_paths.add(candidate)
    if "workspace" in root_manifest:
        declared_members.update(local_paths)
    declared_members = {p for p in declared_members
                        if not any(fnmatch.fnmatchcase(p, pattern.rstrip("/") + "/Cargo.toml") for pattern in excluded_patterns)}
    actual_members = set()
    member_ids = {}
    for path in sorted(declared_members):
        manifest = parsed_manifests[path]
        package = manifest.get("package")
        if not isinstance(package, dict):
            raise InputError(f"graph: member manifest lacks package {path}")
        ver = package.get("version")
        if isinstance(ver, dict) and ver == {"workspace": True}:
            ver = workspace_package.get("version")
        member = {"name": package.get("name"), "version": ver, "source": "workspace", "checksum": None}
        member_key = row_identity(member, "manifest.package")
        if member_key in actual_members:
            raise InputError("graph: duplicate workspace member identity")
        actual_members.add(member_key)
        member_ids[path] = package_id(member)
    default_members = workspace_table.get("default-members")
    if default_members is None:
        default_paths = {"Cargo.toml"} if "package" in root_manifest else declared_members
    else:
        default_paths = set()
        for pattern in array(default_members, "manifest.workspace.default-members", MAX_PACKAGES):
            string(pattern, "manifest.workspace.default-member")
            if pattern.startswith("/") or ".." in pattern.split("/") or "\\" in pattern:
                raise InputError("graph: default member path traverses repository")
            candidate = pattern.rstrip("/") + "/Cargo.toml"
            matches = {path for path in declared_members if fnmatch.fnmatchcase(path, candidate)}
            if not matches:
                raise InputError(f"graph: default member is not a pinned workspace member {candidate}")
            default_paths.update(matches)
    release_roots = {member_ids[path] for path in default_paths}
    if not release_roots:
        raise InputError("graph: no default release workspace roots")
    workflow = read_bytes(root, ".github/workflows/release.yml")
    if digest(inputs["release_workflow_sha256"], "graph.workflow digest") != sha256(workflow):
        raise InputError("graph: stale release workflow digest")
    workflow_targets = read_release_targets(workflow)
    targets = array(graph["targets"], "graph.targets")
    if not targets or targets != sorted(set(targets)) or targets != workflow_targets:
        raise InputError("graph: target set differs from committed release matrix")
    if graph["features"] != ["default"]:
        raise InputError("graph: expected release default features")
    by_id = {}
    identities = []
    for raw in array(graph["packages"], "graph.packages", MAX_PACKAGES):
        key = row_identity(raw, "graph.package", ("workspace_member", "proc_macro"))
        if type(raw["workspace_member"]) is not bool or type(raw["proc_macro"]) is not bool:
            raise InputError("graph: membership and proc_macro must be booleans")
        if key not in packages or raw["checksum"] != packages[key]["checksum"]:
            raise InputError(f"graph: identity or checksum differs from lock {package_id(raw)}")
        if raw["workspace_member"] and raw["source"] != "workspace":
            raise InputError("graph: external package cannot be workspace member")
        if raw["workspace_member"] != (key in actual_members):
            raise InputError(f"graph: workspace membership differs from manifests {package_id(raw)}")
        pid = package_id(raw)
        if pid in by_id:
            raise InputError(f"graph: duplicate package {pid}")
        by_id[pid] = raw
        identities.append(key)
    if identities != sorted(identities) or set(identities) != packages.keys():
        raise InputError("graph: package set must be sorted and equal to Cargo.lock")
    usage = defaultdict(set)
    runtime_targets = defaultdict(set)
    seen_targets = []
    workspace = {pid for pid, raw in by_id.items() if raw["workspace_member"]}
    if not workspace:
        raise InputError("graph: no workspace members")
    for item in array(graph["graphs"], "graph.graphs"):
        obj(item, ("target", "roots", "edges"), "graph.graph")
        target = item["target"]
        if target not in targets or target in seen_targets:
            raise InputError("graph: unknown or duplicate target graph")
        seen_targets.append(target)
        roots = array(item["roots"], "graph.roots")
        if roots != sorted(set(roots)) or set(roots) != release_roots:
            raise InputError("graph: roots must equal sorted default release workspace members")
        adjacency = defaultdict(list)
        seen_edges = set()
        edges = array(item["edges"], "graph.edges", MAX_EDGES)
        for edge in edges:
            obj(edge, ("from", "to", "kind"), "graph.edge")
            key = (edge["from"], edge["to"], edge["kind"])
            if key[0] not in by_id or key[1] not in by_id or key[2] not in ("normal", "build", "dev") or key in seen_edges:
                raise InputError("graph: unknown, duplicate or invalid edge")
            if (key[0], key[1]) not in allowed_edges:
                raise InputError("graph: edge absent from Cargo.lock dependencies")
            seen_edges.add(key)
            adjacency[key[0]].append((key[1], key[2]))
        if list(seen_edges) and edges != sorted(edges, key=lambda e: (e["from"], e["to"], e["kind"])):
            raise InputError("graph: edges must be sorted")
        for path in declared_members:
            manifest = parsed_manifests[path]
            package = manifest["package"]
            ver = package["version"]
            if isinstance(ver, dict):
                ver = workspace_package.get("version")
            parent = package_id({"name": package["name"], "version": ver, "source": "workspace"})
            for alias, specification in manifest.get("dependencies", {}).items():
                if isinstance(specification, dict) and specification.get("workspace") is True:
                    specification = workspace_table.get("dependencies", {}).get(alias, {})
                if isinstance(specification, dict) and specification.get("optional") is True:
                    continue
                name = specification.get("package", alias) if isinstance(specification, dict) else alias
                candidates = {child for src, child in allowed_edges if src == parent and by_id[child]["name"] == name}
                if not candidates or not any((parent, child, "normal") in seen_edges for child in candidates):
                    raise InputError(f"graph: missing unconditional normal dependency {package['name']} -> {name} ({target})")
        # State follows why a node is reached: normal dependencies of build/dev
        # dependencies retain that nonruntime use. Runtime always wins later.
        todo = [(pid, "runtime") for pid in roots]
        visited = set()
        while todo:
            pid, context = todo.pop()
            if (pid, context) in visited:
                continue
            visited.add((pid, context))
            if by_id[pid]["proc_macro"]:
                context = "proc-macro"
            usage[pid].add(context)
            if context == "runtime":
                runtime_targets[pid].add(target)
            for child, kind in adjacency[pid]:
                next_context = context if kind == "normal" else kind
                todo.append((child, next_context))
    if seen_targets != targets:
        raise InputError("graph: graphs must be sorted and cover every release target")
    classified = {}
    for pid, raw in by_id.items():
        uses = sorted(usage.get(pid, {"unsupported-target-only"}))
        classified[identity(raw)] = {
            "workspace_member": raw["workspace_member"], "proc_macro": raw["proc_macro"],
            "class": "safe-to-deploy" if "runtime" in uses else "safe-to-run",
            "usage": uses, "runtime_targets": sorted(runtime_targets.get(pid, set())),
        }
    return classified


def validate_policy(policy, imports_config, imports_hash, legacy_hash, as_of):
    obj(policy, ("format", "status", "audit_authority", "max_exception_days", "imports", "legacy_baseline_sha256"), "policy", ("decision",))
    if policy["format"] != "forge.dependency-policy/1":
        raise InputError("policy: unsupported format")
    if policy["status"] not in ("proposed", "approved") or policy["audit_authority"] not in ("human-signoff", "human-only"):
        raise InputError("policy: invalid status or audit authority")
    if type(policy["max_exception_days"]) is not int or policy["max_exception_days"] <= 0:
        raise InputError("policy: max_exception_days must be positive integer")
    digest(policy["legacy_baseline_sha256"], "policy.legacy_baseline_sha256")
    if policy["legacy_baseline_sha256"] != legacy_hash:
        raise InputError("policy: immutable legacy baseline digest changed")
    if policy["status"] == "proposed":
        if "decision" in policy:
            raise InputError("policy: proposed policy cannot assert decision")
    else:
        decision = obj(policy.get("decision"), ("owner", "recorded_on", "reference"), "policy.decision")
        string(decision["owner"], "policy.owner")
        string(decision["reference"], "policy.reference")
        if day(decision["recorded_on"], "policy.recorded_on") > as_of:
            raise InputError("policy: decision is future dated")
    approved = set()
    seen = set()
    for item in array(policy["imports"], "policy.imports"):
        obj(item, ("name", "url", "snapshot_sha256"), "policy.import")
        name = string(item["name"], "policy.import.name", 256)
        url = string(item["url"], "policy.import.url")
        digest(item["snapshot_sha256"], "policy.import.snapshot_sha256")
        if name in seen:
            raise InputError("policy: duplicate import source")
        seen.add(name)
        if name not in imports_config or imports_config[name]["url"] != url:
            raise InputError(f"policy: configured source/name/URL differs for {name}")
        if item["snapshot_sha256"] != imports_hash:
            raise InputError(f"policy: snapshot pin mismatch for {name}")
        if policy["status"] == "approved":
            approved.add(name)
    return approved


def validate_legacy(value):
    sidecar(value, "legacy", ("inputs", "packages"))
    inputs = obj(value["inputs"], ("lock_sha256", "config_sha256"), "legacy.inputs")
    for key, val in inputs.items():
        digest(val, "legacy.inputs." + key)
    rows = {}
    order = []
    for raw in array(value["packages"], "legacy.packages", MAX_PACKAGES):
        key = row_identity(raw, "legacy.package", ("criteria",))
        criteria(raw["criteria"], "legacy.criteria")
        if key in rows:
            raise InputError("legacy: duplicate package identity")
        rows[key] = raw
        order.append(key)
    if order != sorted(order):
        raise InputError("legacy: package rows must be sorted")
    return rows


def native_config(config, packages):
    obj(config, ("cargo-vet",), "config", ("imports", "policy", "exemptions", "default-criteria"))
    cv = obj(config["cargo-vet"], ("version",), "config.cargo-vet")
    string(cv["version"], "config.cargo-vet.version", 64)
    if "default-criteria" in config:
        criteria(config["default-criteria"], "config.default-criteria")
    imports = config.get("imports", {})
    if not isinstance(imports, dict):
        raise InputError("config.imports: expected table")
    for name, val in imports.items():
        string(name, "config.import.name", 256)
        obj(val, ("url",), "config.import", ("exclude", "criteria-map"))
        string(val["url"], "config.import.url")
        # An unimplemented criteria map must never silently widen trust.
        if "exclude" in val or "criteria-map" in val:
            raise InputError("config.import: exclude/criteria-map needs versioned support")
    policies = config.get("policy", {})
    if not isinstance(policies, dict):
        raise InputError("config.policy: expected table")
    for name, val in policies.items():
        string(name, "config.policy.name")
        obj(val, (), "config.policy", ("criteria", "dev-criteria", "notes"))
        for key in ("criteria", "dev-criteria"):
            if key in val:
                criteria(val[key], "config.policy." + key)
        if "notes" in val:
            string(val["notes"], "config.policy.notes", 65536, multiline=True)
    exemptions = config.get("exemptions", {})
    if not isinstance(exemptions, dict):
        raise InputError("config.exemptions: expected table")
    current, retired = [], []
    seen = set()
    for name in sorted(exemptions):
        if not NAME.fullmatch(name):
            raise InputError("config.exemptions: invalid crate name")
        for raw in array(exemptions[name], "config.exemption"):
            if len(current) + len(retired) >= MAX_RECORDS:
                raise InputError(f"config: exemptions exceed {MAX_RECORDS} total records")
            obj(raw, ("version", "criteria"), "config.exemption", ("notes", "suggest"))
            ver = version(raw["version"], "config.exemption.version")
            crit = criteria(raw["criteria"], "config.exemption.criteria")
            if (name, ver) in seen:
                raise InputError(f"config: duplicate exemption {name} {ver}")
            seen.add((name, ver))
            if "notes" in raw:
                string(raw["notes"], "config.exemption.notes", 65536, multiline=True)
            if "suggest" in raw and type(raw["suggest"]) is not bool:
                raise InputError("config.exemption.suggest: expected boolean")
            matched = [p for p in packages.values() if p["name"] == name and p["version"] == ver]
            row = {"name": name, "version": ver, "criteria": crit,
                   "status": "legacy-unowned", "identities": sorted(package_id(p) for p in matched)}
            (current if matched else retired).append(row)
    return imports, current, retired


def validate_exceptions(value, packages, policy, as_of):
    sidecar(value, "exceptions", ("entries",))
    result = {}
    fields = ("criteria", "owner", "rationale", "created_on", "review_by", "expires_on", "approval_reference")
    for raw in array(value["entries"], "exceptions.entries", MAX_RECORDS):
        key = row_identity(raw, "exception", fields)
        if key in result:
            raise InputError(f"exceptions: duplicate identity {package_id(raw)}")
        if key not in packages or raw["checksum"] != packages[key]["checksum"]:
            raise InputError(f"exceptions: retired/unknown/checksum mismatch {package_id(raw)}")
        if raw["source"] == "workspace" or raw["checksum"] is None:
            raise InputError(f"exceptions: source lacks supported content checksum binding {package_id(raw)}")
        crit = criteria(raw["criteria"], "exception.criteria")
        for field in ("owner", "rationale", "approval_reference"):
            string(raw[field], "exception." + field, multiline=field == "rationale")
        created = day(raw["created_on"], "exception.created_on")
        review = day(raw["review_by"], "exception.review_by")
        expiry = day(raw["expires_on"], "exception.expires_on")
        if not created <= review <= expiry or created > as_of:
            raise InputError(f"exception: date ordering/future creation {package_id(raw)}")
        if (expiry - created).days > policy["max_exception_days"]:
            raise InputError(f"exception: term exceeds policy {package_id(raw)}")
        result[key] = {**raw, "criteria": crit, "age_days": (as_of - created).days,
                       "past_review": as_of > review, "expired": as_of > expiry,
                       "status": "stale-exception" if as_of > review or as_of > expiry else "owned-exception"}
    return result


def validate_reviews(value, native, packages, policy, as_of):
    sidecar(value, "reviews", ("records",))
    native_by_hash = defaultdict(list)
    for item in native:
        native_by_hash[(item["crate"], item["record_sha256"])].append(item)
    accepted, bindings = [], set()
    checksum_bindings = {}
    for raw in array(value["records"], "reviews.records", MAX_RECORDS):
        obj(raw, ("crate", "source", "record_sha256", "method", "checksums", "signoff"), "review")
        crate = string(raw["crate"], "review.crate", 256)
        source = string(raw["source"], "review.source")
        bound = digest(raw["record_sha256"], "review.record_sha256")
        key = (crate, source, bound)
        if key in bindings:
            raise InputError("reviews: duplicate binding")
        bindings.add(key)
        matches = native_by_hash.get((crate, bound), [])
        if len(matches) != 1:
            raise InputError(f"review: missing/ambiguous exact native record {crate} {bound}")
        item = matches[0]
        if raw["method"] not in ("human", "agent-assisted"):
            raise InputError("review: unsupported method")
        if policy["audit_authority"] == "human-only" and raw["method"] != "human":
            raise InputError("review: agent-assisted method excluded by human-only policy")
        signoff = obj(raw["signoff"], ("owner", "recorded_on", "reference"), "review.signoff")
        for field in ("owner", "reference"):
            string(signoff[field], "review.signoff." + field)
        recorded = day(signoff["recorded_on"], "review.signoff.recorded_on")
        if recorded > as_of:
            raise InputError("review: future signoff date")
        checksums = raw["checksums"]
        versions = {item["target"]} | ({item["base"]} if item["base"] else set())
        if not isinstance(checksums, dict) or checksums.keys() != versions:
            raise InputError("review: checksum map must bind exact full version or delta pair")
        for ver, check in checksums.items():
            version(ver, "review.checksums.version")
            digest(check, "review.checksums.digest")
            locked = packages.get((crate, ver, source))
            if locked is not None and check != locked["checksum"]:
                raise InputError(f"review: checksum differs from locked identity {crate} {ver} {source}")
            checksum_key = (crate, ver, source)
            if checksum_key in checksum_bindings and checksum_bindings[checksum_key] != check:
                raise InputError(f"review: conflicting exact-version checksum {crate} {ver} {source}")
            checksum_bindings[checksum_key] = check
        if not any(p["name"] == crate and p["source"] == source for p in packages.values()):
            raise InputError(f"review: unknown source/crate {crate} {source}")
        accepted.append({**item, "source": source, "origin": "local", "checksums": checksums,
                         "signoff": signoff, "method": raw["method"],
                         "age_days": (as_of - recorded).days})
    return accepted, bindings


def imported_records(imports, configured):
    obj(imports, (), "imports.lock", ("audits",))
    sources = imports.get("audits", {})
    if not isinstance(sources, dict):
        raise InputError("imports.lock.audits: expected table")
    result = []
    for name in sorted(sources):
        if name not in configured:
            raise InputError(f"imports.lock: unknown source {name}")
        src = obj(sources[name], (), "imports.lock.source", ("audits", "criteria"))
        if src.get("criteria"):
            raise InputError("imports.lock: custom criteria need versioned support")
        for record in native_records(src.get("audits", {}), "imports.lock." + name):
            if len(result) >= MAX_RECORDS:
                raise InputError(f"imports.lock: audits exceed {MAX_RECORDS} total records")
            result.append({**record, "source": REGISTRY, "origin": "imported", "import_source": name})
    return result


def proof_paths(records, required, forbidden_version=None):
    """Find deterministic shortest full-rooted proof paths, never exemption roots."""
    if len(records) > MAX_PROOF_RECORDS:
        raise InputError(f"proof graph exceeds {MAX_PROOF_RECORDS} records per crate/source")
    relevant = [r for r in records if satisfies(r["criteria"], required)
                and r["target"] != forbidden_version]
    relevant.sort(key=lambda r: (r["target"], r["base"] or "", r["origin"], r.get("import_source", ""), r["record_sha256"]))
    paths = {}
    outgoing = defaultdict(list)
    for record in relevant:
        if record["base"] is None:
            check = record.get("checksums", {}).get(record["target"])
            paths.setdefault((record["target"], check), [record])
        else:
            outgoing[record["base"]].append(record)
    # Breadth-first expansion gives deterministic shortest full-rooted chains.
    # States include checksums, so alternate signed bases cannot mask a mismatch.
    todo = deque(paths)
    while todo:
        state = todo.popleft()
        ver, checksum = state
        path = paths[state]
        for record in outgoing[ver]:
            target_checksum = record.get("checksums", {}).get(record["target"])
            target_key = (record["target"], target_checksum)
            if target_key in paths:
                continue
            if record["origin"] == "local" and checksum is not None and checksum != record["checksums"][ver]:
                continue
            if record["target"] in {step["target"] for step in path}:
                continue
            if len(path) >= MAX_PROOF_CHAIN:
                raise InputError(f"proof chain exceeds {MAX_PROOF_CHAIN} records")
            paths[target_key] = [*path, record]
            todo.append(target_key)
    return paths


def select_proof(paths, ver, checksum, forbidden_version=None):
    candidates = [path for (v, check), path in paths.items()
                  if v == ver and (check is None or check == checksum)
                  and forbidden_version not in {step["target"] for step in path}]
    if not candidates:
        return None
    return min(candidates, key=lambda p: (
        len(p), sum(step["origin"] == "imported" for step in p),
        tuple((s["origin"], s.get("import_source", ""), s["record_sha256"]) for s in p)))


def base_states(records, required, forbidden_version):
    """Bounded reachability for orphan checks, without retaining every path.

    Excluding the delta target avoids circular justification. These searches
    retain only states/depths; full proof paths are built once for reporting.
    """
    if len(records) > MAX_PROOF_RECORDS:
        raise InputError(f"proof graph exceeds {MAX_PROOF_RECORDS} records per crate/source")
    relevant = [record for record in records
                if satisfies(record["criteria"], required) and record["target"] != forbidden_version]
    relevant.sort(key=lambda record: (record["target"], record["base"] or "", record["record_sha256"]))
    seen = set()
    outgoing = defaultdict(list)
    todo = deque()
    for record in relevant:
        if record["base"] is None:
            state = (record["target"], record.get("checksums", {}).get(record["target"]))
            if state not in seen:
                seen.add(state)
                todo.append((state, 1))
        else:
            outgoing[record["base"]].append(record)
    while todo:
        (ver, checksum), depth = todo.popleft()
        for record in outgoing[ver]:
            state = (record["target"], record.get("checksums", {}).get(record["target"]))
            if state in seen:
                continue
            if record["origin"] == "local" and checksum is not None and checksum != record["checksums"][ver]:
                continue
            if depth >= MAX_PROOF_CHAIN:
                raise InputError(f"proof chain exceeds {MAX_PROOF_CHAIN} records")
            seen.add(state)
            todo.append((state, depth + 1))
    return seen


def proof_step(item):
    result = {"kind": "full" if item["base"] is None else "delta", "version": item["target"],
              "criteria": item["criteria"], "auditors": item["auditors"],
              "record_sha256": item["record_sha256"], "origin": item["origin"]}
    if item["base"] is not None:
        result["from_version"] = item["base"]
    for key in ("signoff", "method", "checksums", "import_source", "snapshot_sha256", "age_days"):
        if key in item:
            result[key] = item[key]
    return result


def escaped(value):
    return json.dumps(value, ensure_ascii=True)[1:-1]


def text_report(report):
    lines = [f"dependency {report['command']}: {report['outcome']} (exit {report['exit_code']}); as-of {report['as_of']}"]
    for error in report["errors"]:
        lines.append("INVALID " + escaped(error))
    for reason in report["review_actions"]:
        lines.append("REVIEW " + escaped(reason))
    for item in report["packages"]:
        fields = [str(item.get(k)) for k in ("name", "version", "source", "checksum", "class", "status")]
        fields.extend(f"{key}={item.get(key)}" for key in ("review_criteria", "auditors", "owner", "age_days"))
        lines.append(" | ".join(escaped(field) for field in fields))
        for proof in item.get("proof_chain", []):
            delta = (proof.get("from_version") + " -> " if proof.get("from_version") else "") + proof["version"]
            lines.append("PROOF " + escaped(f"{item['name']} {proof['origin']} {proof['kind']} {delta}; criteria={proof['criteria']}; auditors={proof['auditors']}; record_sha256={proof['record_sha256']}"))
        for pending in item.get("pending_records", []):
            delta = (pending.get("from_version") + " -> " if pending.get("from_version") else "") + pending["version"]
            lines.append("PENDING " + escaped(f"{item['name']} {pending['origin']} {pending['kind']} {delta}; criteria={pending['criteria']}; auditors={pending['auditors']}; record_sha256={pending['record_sha256']}"))
        if "exception" in item:
            ex = item["exception"]
            lines.append("EXCEPTION " + escaped(f"{item['name']} owner={ex['owner']}; age_days={ex['age_days']}; review_by={ex['review_by']}; expires_on={ex['expires_on']}; approval_reference={ex['approval_reference']}"))
    for entry in report.get("retired_exemptions", []):
        lines.append("RETIRED EXEMPTION " + escaped(f"{entry['name']} {entry['version']}; criteria={entry['criteria']}; {entry['status']}"))
    if report.get("pending_imports"):
        lines.append("PENDING IMPORT SOURCES " + escaped(", ".join(report["pending_imports"])))
    lines.append("runtime readiness " + json.dumps(report["runtime_readiness"], sort_keys=True))
    lines.append("trends " + json.dumps(report["trends"], sort_keys=True))
    return "\n".join(lines) + "\n"


def inventory(root, as_of, command="inventory", strict=False):
    report = {"format": "forge.dependency-inventory/1", "command": command,
              "as_of": as_of.isoformat(), "strict": strict, "errors": [],
              "review_actions": [], "packages": [], "current_exemptions": [],
              "retired_exemptions": [], "pending_imports": [], "input_sha256": {},
              "graph_evidence": "metadata-derived; not final artifact proof",
              "graph_qualification": "committed snapshot with input bindings; semantic parity requires separate explicit dependency_graph.py check",
              "release_safety": "metadata-runtime denominator excludes build and proc-macro execution; not total release safety",
              "review_evidence": "asserted signoffs; not identity authentication or safety certification",
              "first_party_evidence": "workspace membership only; no third-party audit proof is asserted",
              "runtime_readiness": {"denominator": 0, "accepted_reviews": 0, "fresh_owned_exceptions": 0,
                                    "legacy_unowned": 0, "missing": 0, "ready": False},
              "new_unvetted_runtime": [], "trends": {}}
    errors = report["errors"]

    def attempt(label, fn, default=None):
        try:
            return fn()
        except (InputError, ValueError, TypeError, KeyError, AttributeError, OSError, UnicodeError, RecursionError) as error:
            errors.append(f"{label}: {error}")
            return default

    def read(path, parser):
        data = read_bytes(root, path)
        report["input_sha256"][path] = sha256(data)
        return parser(data)

    packages = attempt("Cargo.lock", lambda: validate_lock(read("Cargo.lock", load_toml)), {})
    config = attempt("config", lambda: read(STORE + "config.toml", load_toml))
    native_audits = attempt("audits", lambda: read(STORE + "audits.toml", load_toml))
    imports = attempt("imports.lock", lambda: read(STORE + "imports.lock", load_toml))
    authored = {key: attempt(key, lambda key=key: read(path=PATHS[key], parser=load_json)) for key in PATHS}
    classified = attempt("graph", lambda: validate_graph(authored["graph"], packages, root,
                         report["input_sha256"].get("Cargo.lock")), {}) if authored["graph"] is not None else {}
    configuration = attempt("config", lambda: native_config(config, packages)) if config is not None else None
    configured, current, retired = configuration or ({}, [], [])
    report["current_exemptions"], report["retired_exemptions"] = current, retired
    legacy = attempt("legacy", lambda: validate_legacy(authored["legacy"]), {}) if authored["legacy"] is not None else {}
    approved_imports = attempt("policy", lambda: validate_policy(authored["policy"], configured,
        report["input_sha256"].get(STORE + "imports.lock"), report["input_sha256"].get(PATHS["legacy"]), as_of)) if authored["policy"] is not None else None
    valid_policy = authored["policy"] if approved_imports is not None else None
    if valid_policy is not None and valid_policy["status"] == "proposed":
        report["review_actions"].append("D069 policy disposition is proposed; owner decision required")
    local_native = []
    if native_audits is not None:
        def audit_load():
            obj(native_audits, (), "audits.toml", ("audits", "criteria"))
            if native_audits.get("criteria"):
                raise InputError("audits.toml: custom criteria need versioned support")
            return native_records(native_audits.get("audits", {}), "audits.toml")
        local_native = attempt("audits", audit_load, [])
    local, bindings = [], set()
    if authored["reviews"] is not None and valid_policy is not None:
        result = attempt("reviews", lambda: validate_reviews(authored["reviews"], local_native, packages, valid_policy, as_of))
        if result is not None:
            local, bindings = result
    else:
        if authored["reviews"] is not None:
            attempt("reviews", lambda: sidecar(authored["reviews"], "reviews", ("records",)))
    external = attempt("imports", lambda: imported_records(imports, configured), []) if imports is not None else []
    report["pending_imports"] = sorted(name for name in configured if name not in (approved_imports or set()))
    imported = []
    for record in external:
        if record["import_source"] in (approved_imports or set()):
            imported.append({**record, "snapshot_sha256": report["input_sha256"][STORE + "imports.lock"]})
    exception_rows = {}
    if authored["exceptions"] is not None and valid_policy is not None:
        exception_rows = attempt("exceptions", lambda: validate_exceptions(authored["exceptions"], packages, valid_policy, as_of), {})
    elif authored["exceptions"] is not None:
        attempt("exceptions", lambda: sidecar(authored["exceptions"], "exceptions", ("entries",)))
    for key, exception in sorted(exception_rows.items()):
        if exception["status"] == "stale-exception":
            report["review_actions"].append(f"stale exception {package_id(exception)}")
    for entry in current:
        matches = [pkg for pkg in packages.values() if pkg["name"] == entry["name"] and pkg["version"] == entry["version"]]
        owned = [exception_rows.get(identity(pkg)) for pkg in matches]
        if len(matches) == 1 and owned[0] is not None and all(satisfies(owned[0]["criteria"], criterion) for criterion in entry["criteria"]):
            entry["status"] = owned[0]["status"]
            entry["owner"] = owned[0]["owner"]
            entry["age_days"] = owned[0]["age_days"]
            entry["metadata_assertion"] = "owned sidecar; policy disposition required" if valid_policy["status"] == "proposed" else "owned sidecar under approved policy"
        elif len(matches) != 1:
            entry["mapping"] = "ambiguous"
    if command == "validate-store":
        for entry in [*current, *retired]:
            if entry["status"] == "legacy-unowned":
                errors.append(f"config: legacy-unowned exemption {entry['name']} {entry['version']} lacks exact owned metadata or has retired/ambiguous mapping")

    # Validate all signed-off delta records, including retired target versions.
    grouped = defaultdict(list)
    for item in [*local, *imported]:
        grouped[(item["crate"], item["source"])].append(item)
    proofs = {}
    for key, records in sorted(grouped.items()):
        group_failed = False
        for required in sorted(BUILTINS):
            found = attempt(f"proof {key[0]} {key[1]} {required}",
                            lambda required=required: proof_paths(records, required))
            if found is None:
                group_failed = True
            proofs[(key, required)] = found or {}
        if group_failed:
            # A bound error invalidates this group's credit. Do not repeatedly
            # search a graph already known to exceed the declared limits.
            for required in BUILTINS:
                proofs[(key, required)] = {}
            continue
        orphan_queries = defaultdict(list)
        for item in records:
            if item["origin"] == "local" and item["base"] is not None:
                for crit in item["criteria"]:
                    orphan_queries[(crit, item["target"])].append(item)
        for (crit, target), items in sorted(orphan_queries.items()):
            # Only one reachability result is retained at a time. The search
            # excludes the target from the start, allowing longer independent
            # base routes without storing quadratic collections of full paths.
            states = attempt(f"review base {key[0]} {key[1]} {crit} excluding {target}",
                             lambda: base_states(records, crit, target), set())
            for item in items:
                if (item["base"], None) not in states and (item["base"], item["checksums"][item["base"]]) not in states:
                    errors.append(f"reviews: signed-off orphan or inconsistent delta {key[0]} {item['base']} -> {item['target']} ({crit})")

    if valid_policy is None or valid_policy["status"] != "approved":
        proofs = {}

    native_by_crate = defaultdict(list)
    for item in [{**record, "origin": "local"} for record in local_native] + external:
        native_by_crate[item["crate"]].append(item)
    accepted_statuses = {"local-full", "local-delta", "imported-full", "imported-delta"}
    for key, pkg in sorted(packages.items()):
        cls = classified.get(key)
        row = {**pkg, "package_id": package_id(pkg), "class": cls["class"] if cls else "unknown",
               "usage": cls["usage"] if cls else [], "runtime_targets": cls["runtime_targets"] if cls else [],
               "status": "unreviewed", "proof_chain": [], "owner": None, "age_days": None,
               "legacy_eligible": False, "review_criteria": [], "auditors": [], "pending_records": []}
        if cls and cls["workspace_member"]:
            row["status"] = "first-party"
        else:
            required = row["class"]
            path = select_proof(proofs.get(((pkg["name"], pkg["source"]), required), {}), pkg["version"], pkg["checksum"])
            if path:
                last = path[-1]
                row["status"] = last["origin"] + ("-full" if last["base"] is None else "-delta")
                row["proof_chain"] = [proof_step(s) for s in path]
                row["review_criteria"] = sorted(set(last["criteria"]))
                row["auditors"] = sorted({author for step in path for author in step["auditors"]})
                row["owner"] = last.get("signoff", {}).get("owner")
                row["age_days"] = last.get("age_days")
            exception = exception_rows.get(key)
            # A stale exception remains visible and actionable even if a review
            # also exists; record cleanup is part of store validity.
            if exception:
                row["exception"] = exception
                if row["status"] not in accepted_statuses and satisfies(exception["criteria"], required):
                    approved = valid_policy is not None and valid_policy["status"] == "approved"
                    exception_status = exception["status"] if approved or exception["status"] == "stale-exception" else "pending-review"
                    row["status"], row["owner"], row["age_days"] = exception_status, exception["owner"], exception["age_days"]
            baseline = legacy.get(key)
            if baseline and pkg["checksum"] is not None and baseline["checksum"] == pkg["checksum"]:
                row["legacy_eligible"] = "safe-to-deploy" in baseline["criteria"]
            proved_records = {step["record_sha256"] for step in row["proof_chain"]}
            pending_records = [record for record in native_by_crate[pkg["name"]]
                               if record["target"] == pkg["version"]
                               and record["record_sha256"] not in proved_records
                               and (record["origin"] == "local" or record["source"] == pkg["source"])]
            row["pending_records"] = [proof_step(record) for record in pending_records]
            if row["status"] == "unreviewed":
                is_exempt = any(package_id(pkg) in item["identities"] for item in current)
                if is_exempt:
                    row["status"] = "legacy-unowned"
                elif pending_records:
                    row["status"] = "pending-review"
            if required == "safe-to-deploy":
                ready = report["runtime_readiness"]
                ready["denominator"] += 1
                if row["status"] in accepted_statuses:
                    ready["accepted_reviews"] += 1
                elif row["status"] == "owned-exception":
                    ready["fresh_owned_exceptions"] += 1
                elif row["status"] == "legacy-unowned":
                    ready["legacy_unowned"] += 1
                    ready["missing"] += 1
                else:
                    ready["missing"] += 1
                if row["status"] not in accepted_statuses | {"owned-exception"}:
                    if not row["legacy_eligible"]:
                        report["new_unvetted_runtime"].append(package_id(pkg))
                        report["review_actions"].append(f"new unvetted runtime identity {package_id(pkg)}")
                    if strict:
                        report["review_actions"].append(f"strict runtime review missing {package_id(pkg)}")
        report["packages"].append(row)

    # Policy approval and valid closure are necessary even at a zero denominator.
    readiness = report["runtime_readiness"]
    readiness["ready"] = bool(classified) and not errors and valid_policy is not None and valid_policy["status"] == "approved" and readiness["missing"] == 0
    counts = Counter((p["class"], p["status"]) for p in report["packages"])
    class_counts = Counter(p["class"] for p in report["packages"])
    ages = [e["age_days"] for e in exception_rows.values()]
    report["trends"] = {
        "locked_denominator": len(packages), "class_denominators": dict(sorted(class_counts.items())),
        "class_status_counts": [{"class": cls, "status": status, "count": count,
                                 "class_denominator": class_counts[cls]} for (cls, status), count in sorted(counts.items())],
        "exception_denominator": len(exception_rows), "exception_age_days_median": statistics.median(ages) if ages else None,
        "accepted_review_count": sum(p["status"] in accepted_statuses for p in report["packages"]),
        "fresh_owned_exception_count": sum(e["status"] == "owned-exception" for e in exception_rows.values()) if valid_policy is not None and valid_policy["status"] == "approved" else 0,
        "asserted_exception_count": len(exception_rows),
        "stale_exception_count": sum(e["status"] == "stale-exception" for e in exception_rows.values()),
        "legacy_unowned_count": sum(p["status"] == "legacy-unowned" for p in report["packages"]),
    }
    report["errors"] = sorted(set(errors))
    report["review_actions"] = sorted(set(report["review_actions"]))
    report["exit_code"] = 2 if report["errors"] else 1 if report["review_actions"] else 0
    report["outcome"] = ("clean", "review-required", "invalid-input")[report["exit_code"]]
    return report


class ReportParser(argparse.ArgumentParser):
    def error(self, message):
        raise InputError(message)


def main(argv=None):
    parser = ReportParser(description=__doc__)
    parser.add_argument("command", choices=("inventory", "gate", "validate-store"))
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--as-of", required=True, help="explicit YYYY-MM-DD evaluation date")
    parser.add_argument("--format", choices=("json", "text"), default="json")
    parser.add_argument("--strict", action="store_true", help="require full external runtime review or fresh owned exceptions")
    args = None
    try:
        args = parser.parse_args(argv)
        as_of = day(args.as_of, "--as-of")
        report = inventory(args.root, as_of, args.command, args.strict)
    except (InputError, ValueError, OSError) as error:
        report = {"format": "forge.dependency-inventory/1", "command": "unknown", "as_of": None,
                  "strict": False, "exit_code": 2, "outcome": "invalid-input", "errors": [str(error)],
                  "review_actions": [], "packages": [], "current_exemptions": [], "retired_exemptions": [],
                  "pending_imports": [], "input_sha256": {}, "runtime_readiness": {"ready": False},
                  "new_unvetted_runtime": [], "trends": {}}
    if args is not None and args.format == "text":
        sys.stdout.write(text_report(report))
    else:
        sys.stdout.buffer.write(canonical_bytes(report))
    return report["exit_code"]


if __name__ == "__main__":
    raise SystemExit(main())
