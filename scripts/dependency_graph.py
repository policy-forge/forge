#!/usr/bin/env python3
"""Prepare or qualify the pinned, metadata-derived dependency graph.

Both commands run Cargo metadata offline and locked, without building crates.
Check never publishes files: exit 0 means exact metadata parity, 1 means graph
drift, and 2 means invalid input or failed metadata preparation. Qualification is
separate from inventory policy and audit acceptance. Inventory and gate commands
never invoke Cargo. Refresh requires an explicit output; its optional legacy
baseline is an initialization candidate, not an audit or owner decision.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import stat
import subprocess
import sys
import threading
import uuid

sys.dont_write_bytecode = True

from dependency_common import (
    InputError,
    _plain,
    _relative,
    canonical_bytes,
    identity,
    load_json,
    load_toml,
    package_id,
    read_bytes,
    read_release_targets,
    sha256,
)

METADATA_LIMIT = 16 * 1024 * 1024
STDERR_LIMIT = 64 * 1024
METADATA_TIMEOUT = 60
CRITERIA = frozenset(("safe-to-deploy", "safe-to-run"))


def _error(message: str) -> InputError:
    return InputError(message)


def _string(value: object, label: str) -> str:
    if not isinstance(value, str) or not value or len(value) > 4096:
        raise _error(f"{label} must be a nonempty bounded string")
    if any(ord(char) < 32 or ord(char) == 127 for char in value):
        raise _error(f"{label} contains a control character")
    return value


def _cargo_metadata(root: Path, target: str | None) -> dict:
    command = [
        "cargo", "metadata", "--offline", "--locked", "--format-version", "1",
        "--manifest-path", str(root / "Cargo.toml"),
    ]
    if target is not None:
        command.extend(("--filter-platform", target))
    try:
        process = subprocess.Popen(
            command, cwd=root, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            stdin=subprocess.DEVNULL,
        )
    except OSError as exc:
        raise _error(f"could not run offline locked Cargo metadata: {exc}") from exc

    outputs: dict[str, bytes] = {}
    failures: list[str] = []

    def collect(label: str, stream: object, limit: int) -> None:
        pieces: list[bytes] = []
        size = 0
        try:
            while True:
                chunk = stream.read(min(65536, limit - size + 1))
                if not chunk:
                    break
                size += len(chunk)
                if size > limit:
                    failures.append(f"Cargo metadata {label} exceeds {limit} bytes")
                    process.kill()
                    break
                pieces.append(chunk)
            outputs[label] = b"".join(pieces)
        except OSError as exc:
            failures.append(f"Cargo metadata {label} read failed: {exc}")
            process.kill()
        finally:
            stream.close()

    readers = [
        threading.Thread(target=collect, args=("stdout", process.stdout, METADATA_LIMIT), daemon=True),
        threading.Thread(target=collect, args=("stderr", process.stderr, STDERR_LIMIT), daemon=True),
    ]
    for reader in readers:
        reader.start()
    try:
        returncode = process.wait(timeout=METADATA_TIMEOUT)
    except subprocess.TimeoutExpired as exc:
        process.kill()
        process.wait()
        raise _error("offline locked Cargo metadata timed out") from exc
    finally:
        for reader in readers:
            reader.join(timeout=10)
    if any(reader.is_alive() for reader in readers):
        raise _error("Cargo metadata output reader did not finish")
    if failures:
        raise _error("; ".join(failures))
    if returncode:
        detail = outputs.get("stderr", b"").decode("utf-8", errors="backslashreplace")
        # Escape control bytes before diagnostics reach a terminal.
        detail = detail.encode("unicode_escape").decode("ascii")
        raise _error(f"offline locked Cargo metadata failed ({returncode}): {detail}")
    metadata = load_json(outputs.get("stdout", b""))
    if not isinstance(metadata, dict):
        raise _error("Cargo metadata must be an object")
    return metadata


def _lock_packages(lock: dict) -> dict[tuple[str, str, str], dict]:
    rows = lock.get("package")
    if not isinstance(rows, list) or not rows:
        raise _error("Cargo.lock must contain a nonempty package list")
    packages: dict[tuple[str, str, str], dict] = {}
    for row in rows:
        if not isinstance(row, dict):
            raise _error("Cargo.lock package must be an object")
        for key in ("name", "version"):
            _string(row.get(key), f"Cargo.lock package {key}")
        if "source" in row:
            _string(row["source"], "Cargo.lock source")
        checksum = row.get("checksum")
        if checksum is not None and (
            not isinstance(checksum, str) or len(checksum) != 64
            or any(char not in "0123456789abcdef" for char in checksum)
        ):
            raise _error("Cargo.lock checksum must be lowercase SHA-256")
        if row.get("source", "").startswith("registry+") and checksum is None:
            raise _error("registry package is missing a locked checksum")
        key = identity(row)
        if key in packages:
            raise _error(f"duplicate normalized locked identity: {package_id(row)}")
        packages[key] = row
    return packages


def _metadata_packages(metadata: dict) -> tuple[dict[str, dict], set[str], list[str]]:
    rows = metadata.get("packages")
    members = metadata.get("workspace_members")
    defaults = metadata.get("workspace_default_members", members)
    if not isinstance(rows, list) or not isinstance(members, list) or not members:
        raise _error("Cargo metadata packages/workspace_members are invalid")
    if not isinstance(defaults, list) or not defaults:
        raise _error("Cargo metadata workspace_default_members are invalid")
    packages: dict[str, dict] = {}
    identities: set[tuple[str, str, str]] = set()
    for row in rows:
        if not isinstance(row, dict):
            raise _error("Cargo metadata package must be an object")
        for key in ("id", "name", "version", "manifest_path"):
            _string(row.get(key), f"Cargo metadata package {key}")
        if row.get("source") is not None:
            _string(row["source"], "Cargo metadata package source")
        if row["id"] in packages or identity(row) in identities:
            raise _error("duplicate Cargo metadata package identity")
        packages[row["id"]] = row
        identities.add(identity(row))
    if len(set(members)) != len(members) or any(member not in packages for member in members):
        raise _error("Cargo metadata workspace members do not resolve uniquely")
    if len(set(defaults)) != len(defaults) or any(member not in members for member in defaults):
        raise _error("Cargo metadata default members are not unique workspace members")
    return packages, set(members), defaults


def _proc_macro(package: dict) -> bool:
    targets = package.get("targets")
    if not isinstance(targets, list) or not targets:
        raise _error("Cargo metadata package targets are invalid")
    kinds = []
    for target in targets:
        if not isinstance(target, dict) or not isinstance(target.get("kind"), list):
            raise _error("Cargo metadata target kind is invalid")
        kinds.extend(_string(kind, "Cargo metadata target kind") for kind in target["kind"])
    return "proc-macro" in kinds


def _relative_manifest(root: Path, value: str) -> str:
    path = Path(value)
    if not path.is_absolute():
        raise _error("Cargo metadata manifest path must be absolute")
    try:
        relative = path.relative_to(root)
    except ValueError as exc:
        raise _error("workspace manifest is outside the repository root") from exc
    return relative.as_posix()


def _target_graph(metadata: dict, target: str, full: dict[str, dict], members: set[str], defaults: list[str]) -> dict:
    packages, current_members, current_defaults = _metadata_packages(metadata)
    if current_members != members or current_defaults != defaults:
        raise _error("workspace membership changed between metadata passes")
    for key, package in packages.items():
        if key not in full or identity(package) != identity(full[key]):
            raise _error("filtered metadata changed a normalized package identity")
        if _proc_macro(package) != _proc_macro(full[key]):
            raise _error("filtered metadata changed a proc-macro flag")
    resolve = metadata.get("resolve")
    if not isinstance(resolve, dict) or not isinstance(resolve.get("nodes"), list):
        raise _error("Cargo metadata resolve nodes are missing")
    edges: set[tuple[str, str, str]] = set()
    seen: set[str] = set()
    for node in resolve["nodes"]:
        if not isinstance(node, dict) or node.get("id") not in packages or node["id"] in seen:
            raise _error("Cargo metadata resolve node is invalid or duplicated")
        seen.add(node["id"])
        dependencies = node.get("deps")
        if not isinstance(dependencies, list):
            raise _error("Cargo metadata node dependencies are invalid")
        for dependency in dependencies:
            if not isinstance(dependency, dict) or dependency.get("pkg") not in packages:
                raise _error("Cargo metadata dependency does not resolve")
            kinds = dependency.get("dep_kinds")
            if not isinstance(kinds, list) or not kinds:
                raise _error("Cargo metadata dependency kinds are missing")
            for entry in kinds:
                if not isinstance(entry, dict) or entry.get("kind") not in (None, "build", "dev"):
                    raise _error("Cargo metadata dependency kind is invalid")
                kind = entry.get("kind") or "normal"
                edges.add((package_id(packages[node["id"]]), package_id(packages[dependency["pkg"]]), kind))
    if seen != set(packages):
        raise _error("Cargo metadata resolve closure is incomplete")
    return {
        "target": target,
        "roots": sorted(package_id(full[member]) for member in defaults),
        "edges": [{"from": start, "to": end, "kind": kind} for start, end, kind in sorted(edges)],
    }


def legacy_baseline(lock: dict, config: dict, lock_bytes: bytes, config_bytes: bytes) -> dict:
    """Freeze matching native exemptions without inventing approval metadata."""
    locked = _lock_packages(lock)
    by_version: dict[tuple[str, str], list[dict]] = {}
    for row in locked.values():
        by_version.setdefault((row["name"], row["version"]), []).append(row)
    exemptions = config.get("exemptions", {})
    if not isinstance(exemptions, dict):
        raise _error("native exemptions must be a table")
    rows: list[dict] = []
    seen: set[tuple[str, str]] = set()
    for name, entries in exemptions.items():
        _string(name, "native exemption name")
        if not isinstance(entries, list):
            raise _error("native exemption entries must be arrays of tables")
        for entry in entries:
            if not isinstance(entry, dict):
                raise _error("native exemption must be an object")
            version = _string(entry.get("version"), "native exemption version")
            key = (name, version)
            if key in seen:
                raise _error(f"duplicate native exemption: {name} {version}")
            seen.add(key)
            criteria = entry.get("criteria")
            if isinstance(criteria, str):
                criteria = [criteria]
            if not isinstance(criteria, list) or not criteria or any(item not in CRITERIA for item in criteria):
                raise _error("native exemption criteria must be built-in criteria")
            if len(set(criteria)) != len(criteria):
                raise _error("native exemption repeats a criterion")
            candidates = by_version.get(key, [])
            if len(candidates) > 1:
                raise _error(f"native exemption source is ambiguous: {name} {version}")
            if not candidates:
                continue  # Retired native exemptions remain in config, never grandfathered.
            package = candidates[0]
            rows.append({
                "name": package["name"], "version": package["version"],
                "source": package.get("source") or "workspace", "checksum": package.get("checksum"),
                "criteria": sorted(criteria),
            })
    return {
        "format": "forge.dependency-legacy/1",
        "inputs": {"lock_sha256": sha256(lock_bytes), "config_sha256": sha256(config_bytes)},
        "packages": sorted(rows, key=identity),
    }


def prepare_graph(root: Path) -> tuple[dict, dict[str, bytes]]:
    """Read inputs, gather all package flags, then resolve every release target."""
    inputs = {
        "Cargo.lock": read_bytes(root, "Cargo.lock"),
        "Cargo.toml": read_bytes(root, "Cargo.toml"),
        ".github/workflows/release.yml": read_bytes(root, ".github/workflows/release.yml"),
    }
    lock = load_toml(inputs["Cargo.lock"])
    locked = _lock_packages(lock)
    targets = read_release_targets(inputs[".github/workflows/release.yml"])
    metadata = _cargo_metadata(root, None)
    if metadata.get("workspace_root") != str(root):
        raise _error("Cargo metadata workspace root differs from --root")
    full, members, defaults = _metadata_packages(metadata)
    if {identity(package) for package in full.values()} != set(locked):
        raise _error("unfiltered Cargo metadata does not contain every locked identity exactly once")
    for package in full.values():
        if package.get("source") is not None:
            continue
        relative = _relative_manifest(root, package["manifest_path"])
        current = read_bytes(root, relative)
        if relative in inputs and inputs[relative] != current:
            raise _error(f"manifest changed during metadata preparation: {relative}")
        inputs[relative] = current
    packages = []
    for cargo_id, package in full.items():
        row = locked[identity(package)]
        packages.append({
            "name": row["name"], "version": row["version"],
            "source": row.get("source") or "workspace", "checksum": row.get("checksum"),
            "workspace_member": cargo_id in members, "proc_macro": _proc_macro(package),
        })
    graphs = [
        _target_graph(_cargo_metadata(root, target), target, full, members, defaults)
        for target in targets
    ]
    graph = {
        "format": "forge.dependency-graph/1",
        "inputs": {
            "lock_sha256": sha256(inputs["Cargo.lock"]),
            "manifests": {path: sha256(value) for path, value in inputs.items() if path.endswith("Cargo.toml")},
            "release_workflow_sha256": sha256(inputs[".github/workflows/release.yml"]),
        },
        "targets": targets, "features": ["default"],
        "packages": sorted(packages, key=identity), "graphs": graphs,
    }
    return graph, inputs


def _output_path(root: Path, relative: str, replace: bool) -> Path:
    path = Path(*_relative(relative))
    if path.suffix != ".json":
        raise _error("dependency sidecar output must have a .json suffix")
    candidate = root
    for part in path.parts[:-1]:
        candidate = candidate / part
        try:
            info = candidate.lstat()
        except OSError as exc:
            raise _error(f"output parent must already exist: {path}") from exc
        _plain(info)
        if not stat.S_ISDIR(info.st_mode):
            raise _error("output parent must be a directory without symlinks")
    output = root / path
    try:
        info = output.lstat()
    except FileNotFoundError:
        return output
    _plain(info)
    if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
        raise _error("output must be a single regular file without symlinks or aliases")
    if not replace:
        raise _error("legacy baseline already exists; ordinary refresh cannot replace it")
    return output


def _graph_output_path(root: Path, relative: str) -> Path:
    # --output must never offer a second route around baseline immutability or
    # overwrite another authored sidecar. Only a graph may replace a graph.
    if relative == "supply-chain/legacy-baseline.json":
        raise _error("graph output cannot replace the reserved legacy baseline")
    output = _output_path(root, relative, replace=True)
    if output.exists():
        existing = load_json(read_bytes(root, relative))
        if not isinstance(existing, dict) or existing.get("format") != "forge.dependency-graph/1":
            raise _error("graph output can replace only an existing dependency graph")
    return output


def _publish(root: Path, relative: str, content: bytes, replace: bool) -> None:
    """Publish through pinned directory descriptors, never following links."""
    path = Path(relative)
    temporary = f".{path.name}.{uuid.uuid4().hex}.tmp"
    descriptor = None
    parent = None
    try:
        if not hasattr(os, "O_NOFOLLOW") or not hasattr(os, "O_DIRECTORY"):
            raise _error("safe sidecar publication requires no-follow directory descriptors")
        parent = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        for part in path.parts[:-1]:
            following = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=parent)
            os.close(parent)
            parent = following
        try:
            info = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        except FileNotFoundError:
            info = None
        if info is not None:
            if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
                raise _error("output must be a single regular file without symlinks or aliases")
            if not replace:
                raise _error("legacy baseline already exists; ordinary refresh cannot replace it")
        descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o644, dir_fd=parent)
        with os.fdopen(descriptor, "wb") as stream:
            descriptor = None
            stream.write(content)
            stream.flush()
            os.fsync(stream.fileno())
        if replace:
            # A replacement swaps the directory entry and cannot write through a
            # destination symlink introduced concurrently.
            os.replace(temporary, path.name, src_dir_fd=parent, dst_dir_fd=parent)
        else:
            # link is atomic and refuses an existing destination, including a
            # symlink introduced after the earlier validation. No rebaselining.
            os.link(temporary, path.name, src_dir_fd=parent, dst_dir_fd=parent, follow_symlinks=False)
            os.unlink(temporary, dir_fd=parent)
        os.fsync(parent)
    except OSError as exc:
        raise _error(f"could not publish dependency sidecar: {exc}") from exc
    finally:
        if descriptor is not None:
            os.close(descriptor)
        if parent is not None:
            try:
                os.unlink(temporary, dir_fd=parent)
            except FileNotFoundError:
                # Successful publication already removed the staging name.
                pass
            os.close(parent)


def check_graph(root: Path, relative: str = "supply-chain/dependency-graph.json") -> dict:
    """Compare the committed graph with freshly resolved metadata; write nothing.

    Hashes in the sidecar alone cannot qualify authored edge or proc-macro flags.
    This explicit preparation check therefore rebuilds metadata independently and
    requires the complete canonical sidecar bytes to match.
    """
    committed = read_bytes(root, relative)
    parsed = load_json(committed)
    if not isinstance(parsed, dict) or parsed.get("format") != "forge.dependency-graph/1":
        raise _error("graph qualification input must be a dependency graph")
    graph, inputs = prepare_graph(root)
    for path, original in inputs.items():
        if read_bytes(root, path) != original:
            raise _error(f"input changed during graph qualification: {path}")
    if read_bytes(root, relative) != committed:
        raise _error("committed graph changed during qualification")
    resolved = canonical_bytes(graph)
    return {
        "result": "qualified" if committed == resolved else "drift",
        "graph_input": relative,
        "committed_sha256": sha256(committed),
        "metadata_sha256": sha256(resolved),
        "packages": len(graph["packages"]),
        "targets": graph["targets"],
        "evidence": "locked offline Cargo metadata comparison; separate from inventory gate and audit acceptance",
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    refresh = commands.add_parser("refresh", help="explicitly publish refreshed graph")
    refresh.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    refresh.add_argument("--output", required=True, help="repository-relative graph output")
    refresh.add_argument("--baseline-output", help="initialize a repository-relative legacy baseline; refuses replacement")
    check = commands.add_parser("check", help="qualify metadata parity without publishing files")
    check.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    check.add_argument("--graph", default="supply-chain/dependency-graph.json", help="repository-relative committed graph")
    arguments = parser.parse_args(argv)
    try:
        root = arguments.root.absolute()
        if root.resolve() != root or not root.is_dir():
            raise _error("--root must be an existing directory without symlink aliases")
        if arguments.command == "check":
            result = check_graph(root, arguments.graph)
            print(canonical_bytes(result).decode("ascii"), end="")
            return 0 if result["result"] == "qualified" else 1
        graph_path = _graph_output_path(root, arguments.output)
        baseline_path = None
        if arguments.baseline_output:
            baseline_path = _output_path(root, arguments.baseline_output, replace=False)
            if baseline_path == graph_path:
                raise _error("graph and baseline outputs must differ")
        graph, inputs = prepare_graph(root)
        baseline = None
        if baseline_path:
            config_bytes = read_bytes(root, "supply-chain/config.toml")
            inputs["supply-chain/config.toml"] = config_bytes
            baseline = legacy_baseline(load_toml(inputs["Cargo.lock"]), load_toml(config_bytes), inputs["Cargo.lock"], config_bytes)
        for relative, original in inputs.items():
            if read_bytes(root, relative) != original:
                raise _error(f"input changed during metadata preparation: {relative}")
        _graph_output_path(root, arguments.output)
        if baseline_path:
            _publish(root, arguments.baseline_output, canonical_bytes(baseline), replace=False)
        _publish(root, arguments.output, canonical_bytes(graph), replace=True)
        print(canonical_bytes({
            "result": "prepared", "graph_output": arguments.output,
            "packages": len(graph["packages"]), "targets": graph["targets"],
            "edges_by_target": {item["target"]: len(item["edges"]) for item in graph["graphs"]},
            "baseline_output": arguments.baseline_output,
            "legacy_candidates": len(baseline["packages"]) if baseline else None,
            "evidence": "offline locked Cargo metadata; no compilation or audit approval",
        }).decode("ascii"), end="")
        return 0
    except (InputError, OSError, ValueError, TypeError) as exc:
        message = str(exc).encode("unicode_escape").decode("ascii")
        print(f"dependency graph invalid: {message}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
