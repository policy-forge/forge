#!/usr/bin/env python3
"""Offline draft CycloneDX SBOM from the committed dependency store.

Deterministic CycloneDX 1.6 JSON derived only from Cargo.lock and the committed
supply-chain/dependency-graph.json (per-target normal/build/dev edges), with
input digests pinned by the graph's own `inputs` block. No Cargo invocation,
network access, timestamps or randomness: identical inputs give identical
bytes. Output is a DRAFT dependency inventory for review, never the release
SBOM: `.github/workflows/release.yml` generates the published SBOM with
cargo-cyclonedx at the release tag. Checksums are the registry crate-archive
SHA-256 digests recorded by Cargo.lock, not built-binary hashes.
"""
from __future__ import annotations

import argparse
from datetime import date
import json
from pathlib import Path
import re
import sys
import uuid

# Importing helpers must not create cache files in a consuming command.
sys.dont_write_bytecode = True

from dependency_common import InputError, canonical_bytes, load_json, load_toml, read_bytes, sha256

REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"
PURL_NAMESPACE = uuid.UUID("1d8840a0-84a4-5def-9a6f-4c99a58f3f70")
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
SEMVER = re.compile(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"
                    r"(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
                    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?\Z")
DAY = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}\Z")
MAX_PACKAGES = 5000
MAX_EDGES = 100000
SCHEMA = "http://cyclonedx.org/schema/bom-1.6.schema.json"
DISCLAIMER = ("DRAFT pre-release dependency SBOM generated offline from Cargo.lock and "
              "supply-chain/dependency-graph.json by scripts/generate_sbom.py. Not the "
              "published release SBOM: .github/workflows/release.yml regenerates the "
              "release SBOM with cargo-cyclonedx at the tag. Hashes are registry "
              "crate-archive SHA-256 digests from Cargo.lock, not built-binary hashes; "
              "this document is not an audit, safety or release-acceptance claim.")


def _fail(message: str) -> None:
    raise InputError(message)


def _identity(raw: object) -> tuple[str, str, str]:
    if not isinstance(raw, str):
        _fail("graph edge endpoint is not a string identity")
    value = load_json(raw.encode("utf-8"))
    if (not isinstance(value, list) or len(value) != 3
            or not all(isinstance(part, str) for part in value)):
        _fail("graph identity is not a [name, version, source] triple")
    return tuple(value)


def _purl(name: str, version: str) -> str:
    return f"pkg:cargo/{name}@{version}"


def _ref(name: str, version: str) -> str:
    return _purl(name, version)


def build_sbom(root: Path, as_of: str) -> dict:
    if not DAY.fullmatch(as_of):
        _fail("--as-of must be an explicit YYYY-MM-DD date")
    date.fromisoformat(as_of)  # rejects impossible calendar dates

    lock_bytes = read_bytes(root, "Cargo.lock")
    manifest_bytes = read_bytes(root, "Cargo.toml")
    workflow_bytes = read_bytes(root, ".github/workflows/release.yml")
    graph = load_json(read_bytes(root, "supply-chain/dependency-graph.json"))
    if not isinstance(graph, dict) or graph.get("format") != "forge.dependency-graph/1":
        _fail("unsupported dependency-graph format")

    inputs = graph.get("inputs")
    if not isinstance(inputs, dict):
        _fail("dependency graph is missing its inputs block")
    manifests = inputs.get("manifests")
    if not isinstance(manifests, dict) or set(manifests) != {"Cargo.toml"}:
        _fail("dependency graph must pin exactly Cargo.toml")
    for label, expected, actual in (
        ("Cargo.lock", inputs.get("lock_sha256"), sha256(lock_bytes)),
        ("Cargo.toml", manifests["Cargo.toml"], sha256(manifest_bytes)),
        (".github/workflows/release.yml", inputs.get("release_workflow_sha256"),
         sha256(workflow_bytes)),
    ):
        if not isinstance(expected, str) or expected != actual:
            _fail(f"{label} does not match the graph pin; run "
                  "python3 -B scripts/dependency_graph.py refresh after review")

    manifest = load_toml(manifest_bytes)
    package = manifest.get("package")
    if not isinstance(package, dict) or package.get("name") != "forge":
        _fail("root manifest is not the forge package")
    root_version = package.get("version")
    license_id = package.get("license")
    if not isinstance(root_version, str) or not isinstance(license_id, str):
        _fail("root manifest lacks version or license")

    packages = graph.get("packages")
    graphs = graph.get("graphs")
    if (not isinstance(packages, list) or not 0 < len(packages) <= MAX_PACKAGES
            or not isinstance(graphs, list) or not graphs):
        _fail("dependency graph packages/graphs are missing or unbounded")
    targets = graph.get("targets")
    if (not isinstance(targets, list) or len(targets) != len(graphs)
            or sorted(targets) != sorted(str(g.get("target")) for g in graphs)):
        _fail("graph targets do not match the per-target graphs")

    locked = load_toml(lock_bytes).get("package")
    if not isinstance(locked, list):
        _fail("Cargo.lock has no package table")
    lock_by_name_version: dict[tuple[str, str], dict] = {}
    for row in locked:
        if not isinstance(row, dict):
            _fail("Cargo.lock package row is not a table")
        key = (row.get("name"), row.get("version"))
        if key in lock_by_name_version:
            _fail(f"Cargo.lock has duplicate identity {key}")
        lock_by_name_version[key] = row

    by_id: dict[tuple[str, str, str], dict] = {}
    roots: dict[str, str] = {}
    for pkg in packages:
        if not isinstance(pkg, dict):
            _fail("graph package is not an object")
        name, version, source = pkg.get("name"), pkg.get("version"), pkg.get("source")
        if (not isinstance(name, str) or not name
                or not isinstance(version, str) or not SEMVER.fullmatch(version)):
            _fail("graph package has an invalid name or version")
        if pkg.get("workspace_member"):
            if name != "forge" or source != "workspace" or version != root_version:
                _fail("workspace member does not match the root manifest")
            if name in roots:
                _fail("duplicate workspace root")
            roots[name] = version
            continue
        if source != REGISTRY:
            _fail(f"unsupported package source for {name}")
        checksum = pkg.get("checksum")
        if not isinstance(checksum, str) or not HEX64.fullmatch(checksum):
            _fail(f"graph package {name} lacks a SHA-256 checksum")
        row = lock_by_name_version.get((name, version))
        if row is None:
            _fail(f"graph package {name}@{version} is absent from Cargo.lock")
        if row.get("checksum") != checksum:
            _fail(f"graph checksum for {name}@{version} differs from Cargo.lock")
        if row.get("source") != REGISTRY:
            _fail(f"Cargo.lock source for {name}@{version} is not the crates.io registry")
        ident = (name, version, source)
        if ident in by_id:
            _fail(f"duplicate graph identity {name}@{version}")
        by_id[ident] = pkg
    if len(roots) != 1:
        _fail("graph must declare exactly one workspace root")
    if len(lock_by_name_version) != len(by_id) + 1:
        _fail("Cargo.lock and graph package populations differ")

    edge_total = 0
    all_edges: list[tuple[tuple[str, str, str], tuple[str, str, str], str]] = []
    root_deps: set[str] = set()
    for entry in graphs:
        if not isinstance(entry, dict):
            _fail("graph entry is not an object")
        edges = entry.get("edges")
        entry_roots = entry.get("roots")
        if not isinstance(edges, list) or not isinstance(entry_roots, list):
            _fail("graph entry lacks edges or roots")
        edge_total += len(edges)
        if edge_total > MAX_EDGES:
            _fail("dependency graph exceeds the edge bound")
        expected_root = json.dumps(["forge", roots["forge"], "workspace"],
                                   separators=(",", ":"), ensure_ascii=True)
        if entry_roots != [expected_root]:
            _fail("graph entry root is not the forge workspace package")
        for edge in edges:
            if not isinstance(edge, dict):
                _fail("graph edge is not an object")
            kind = edge.get("kind")
            if kind not in ("normal", "build", "dev"):
                _fail(f"unsupported edge kind {kind!r}")
            source = _identity(edge.get("from", ""))
            target = _identity(edge.get("to", ""))
            for ident in (source, target):
                if ident[0] == "forge":
                    if ident != ("forge", roots["forge"], "workspace"):
                        _fail("edge references an unknown forge identity")
                elif ident not in by_id:
                    _fail(f"edge references unknown package {ident[0]}@{ident[1]}")
            all_edges.append((source, target, kind))
            if source[0] == "forge":
                root_deps.add(_ref(*target[:2]))  # dev edges included: full lock inventory

    # A crate ships only if reachable from the root through normal/build edges;
    # dev-only closures stay in the inventory marked optional.
    required: set[tuple[str, str, str]] = set()
    adjacency: dict[tuple[str, str, str], list[tuple[str, str, str]]] = {}
    for source, target, kind in all_edges:
        if kind in ("normal", "build"):
            adjacency.setdefault(source, []).append(target)
    root_ident = ("forge", roots["forge"], "workspace")
    frontier = [root_ident]
    while frontier:
        current = frontier.pop()
        for target in adjacency.get(current, ()):
            if target not in required:
                required.add(target)
                frontier.append(target)

    components = []
    for (name, version, _source), pkg in sorted(by_id.items()):
        component = {
            "type": "library",
            "bom-ref": _ref(name, version),
            "name": name,
            "version": version,
            "purl": _purl(name, version),
            "hashes": [{"alg": "SHA-256", "content": pkg["checksum"]}],
            "externalReferences": [{
                "type": "distribution",
                "url": f"https://static.crates.io/crates/{name}/{name}-{version}.crate",
                "hashes": [{"alg": "SHA-256", "content": pkg["checksum"]}],
            }],
            "properties": [
                {"name": "forge:source", "value": REGISTRY},
                {"name": "forge:proc-macro", "value": str(bool(pkg.get("proc_macro"))).lower()},
            ],
            "scope": ("required" if (name, version, REGISTRY) in required else "optional"),
        }
        components.append(component)

    root_ref = _ref("forge", roots["forge"])
    dependencies = [{"ref": root_ref, "dependsOn": sorted(root_deps)}]
    for source, target, _kind in all_edges:
        if source[0] == "forge":
            continue
        dependencies.append({"ref": _ref(*source[:2]), "dependsOn": [_ref(*target[:2])]})
    merged: dict[str, set[str]] = {}
    for dep in dependencies:
        merged.setdefault(dep["ref"], set()).update(dep["dependsOn"])
    dependencies = [{"ref": ref, "dependsOn": sorted(values)}
                    for ref, values in sorted(merged.items())]

    serial = uuid.uuid5(PURL_NAMESPACE,
                        f"forge-draft-sbom:{as_of}:{sha256(lock_bytes)}")
    return {
        "bomFormat": "CycloneDX",
        "specVersion": "1.6",
        "$schema": SCHEMA,
        "serialNumber": f"urn:uuid:{serial}",
        "version": 1,
        "metadata": {
            "licenses": [{"license": {"id": license_id}}],
            "component": {
                "type": "application",
                "bom-ref": root_ref,
                "name": "forge",
                "version": roots["forge"],
                "purl": _purl("forge", roots["forge"]),
                "description": "DRAFT pre-release; workspace source, not a published artifact.",
            },
            "properties": [
                {"name": "forge:draft", "value": "true"},
                {"name": "forge:as-of", "value": as_of},
                {"name": "forge:disclaimer", "value": DISCLAIMER},
                {"name": "forge:cargo-lock-sha256", "value": sha256(lock_bytes)},
                {"name": "forge:dependency-graph-sha256",
                 "value": sha256(read_bytes(root, "supply-chain/dependency-graph.json"))},
                {"name": "forge:targets", "value": ",".join(sorted(targets))},
            ],
        },
        "components": components,
        "dependencies": dependencies,
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--as-of", required=True, help="explicit YYYY-MM-DD evaluation date")
    parser.add_argument("--output", type=Path, required=True, help="destination SBOM JSON file")
    args = parser.parse_args(argv)
    try:
        document = build_sbom(args.root, args.as_of)
        payload = canonical_bytes(document)
        args.output.write_bytes(payload)
    except (InputError, ValueError, OSError) as error:
        sys.stderr.write(f"generate_sbom: {error}\n")
        return 2
    sys.stdout.write(f"generate_sbom: wrote {len(document['components'])} components "
                     f"to {args.output} (DRAFT, sha256 {sha256(payload)})\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())