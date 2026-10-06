#!/usr/bin/env python3
"""Retain bounded, redacted F04 verification receipts from real predefined suites.

This runner does not install tools, build Forge, approve dependencies, or certify
browser/accessibility/network-denial gates. Raw subprocess output is ephemeral.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import signal
import stat
import subprocess
import sys
import tempfile
import threading

sys.dont_write_bytecode = True
MAX_CAPTURE = 4 * 1024 * 1024
MAX_INPUT = 1024 * 1024 * 1024
# One shared attempt bound for both maintained-client sessions and receipt replay.
MAX_CLIENT_REQUESTS = 10000
SUITES = ("api-contract", "api-workflow", "maintained-client")
INPUTS = (
    "Cargo.toml", "Cargo.lock", "docs/api/forge-workspace-v1.openapi.yaml",
    "docs/api/capability-matrix.json", "schemas/forge.workspace-1.schema.json",
    "ui/workspace.js", "ui/workspace.css", "src/workspace/assets.rs",
    "scripts/workspace_client.py", "scripts/test_workspace_client.py",
    "scripts/verify_workspace.py", "scripts/test_verify_workspace.py",
    ".github/workflows/ci.yml", ".github/workflows/workspace-verification.yml",
    ".gitattributes", "tests/api_contract_validation.rs",
    "tests/workspace_cli_test.rs",
)
BUILD_OUTCOMES = ("success", "failure", "skipped", "cancelled", "unrecorded")
PENDING = (
    "supported-browser-products-and-versions", "packaged-runtime-os-network-denial",
    "complete-api-cli-parity", "automated-accessibility-rule-engine",
    "manual-keyboard-screen-reader-and-zoom", "independent-security-acceptance",
    "dependency-audit-and-owner-disposition", "five-user-product-study",
    "release-package-and-provenance", "final-roadmap-documentation-review",
)


def canonical_bytes(value):
    """Serialize JSON with sorted keys, ASCII escaping and one LF; reject nonfinite numbers."""
    return (json.dumps(value, sort_keys=True, ensure_ascii=True, allow_nan=False,
                       separators=(",", ":")) + "\n").encode("ascii")


def atomic_receipt(path, value):
    """Publish one closed receipt; never replace an existing receipt."""
    path = Path(path)
    data = canonical_bytes(value)
    if len(data) > MAX_CAPTURE:
        raise ValueError("Receipt exceeds the supported bound")
    path.parent.mkdir(parents=True, exist_ok=True)
    # A private staging file has no project content, credential, or raw output.
    descriptor, temporary = tempfile.mkstemp(prefix=".receipt-", dir=path.parent)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        # Hard-link publication is atomic and refuses an existing destination on
        # supported local CI filesystems. Unsupported filesystems fail closed.
        os.link(temporary, path)
    finally:
        os.unlink(temporary)


def hash_file(path):
    """Return a SHA-256 and byte count only for an unchanged, bounded regular input."""
    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
    if Path(path).is_symlink():
        raise ValueError("A verification input is not a regular file")
    descriptor = os.open(path, flags)
    try:
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode) or before.st_size > MAX_INPUT:
            raise ValueError("A verification input exceeds its file bound")
        digest = hashlib.sha256()
        length = 0
        with os.fdopen(descriptor, "rb", closefd=False) as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b""):
                length += len(block)
                if length > MAX_INPUT:
                    raise ValueError("A verification input exceeds its file bound")
                digest.update(block)
        after = os.fstat(descriptor)
        identity = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
        current = os.stat(path, follow_symlinks=False)
        if identity(before) != identity(after) or identity(before) != identity(current):
            raise ValueError("A verification input changed during capture")
        return {"sha256": digest.hexdigest(), "bytes": length}
    finally:
        os.close(descriptor)


def contract_routes(root):
    """Read a bounded API inventory; reject unsupported YAML layouts rather than dropping operations."""
    with (root / "docs/api/forge-workspace-v1.openapi.yaml").open("rb") as stream:
        raw = stream.read(MAX_CAPTURE + 1)
    if len(raw) > MAX_CAPTURE:
        raise ValueError("API contract exceeds its bound")
    text = raw.decode("utf-8")
    routes = []
    paths = set()
    in_paths = seen_paths = False
    route = method = None
    operation_seen = False
    methods = {"get", "put", "post", "delete", "options", "head", "patch", "trace"}
    fields = {"tags", "summary", "description", "externalDocs", "parameters", "requestBody",
              "responses", "callbacks", "deprecated", "security", "servers"}
    for line in text.splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indentation = len(line) - len(line.lstrip(" "))
        if "\t" in line[:indentation + 1]:
            raise ValueError("Unsupported API contract indentation")
        if indentation == 0:
            if in_paths and method is not None and not operation_seen:
                raise ValueError("API method has no operation identifier")
            in_paths = False
            if re.match(r"[\"']?paths[\"']?\s*:", line):
                if line != "paths:" or seen_paths:
                    raise ValueError("Unsupported or duplicate API paths layout")
                in_paths = seen_paths = True
                route = method = None
            continue
        if not in_paths:
            continue
        if indentation == 2:
            if method is not None and not operation_seen:
                raise ValueError("API method has no operation identifier")
            path = re.fullmatch(r"  (/api/v1/[A-Za-z0-9_{}/.-]+):", line)
            if not path or path[1] in paths:
                raise ValueError("Unsupported or duplicate API path declaration")
            route, method, operation_seen = path[1], None, False
            paths.add(route)
        elif indentation == 4:
            if method is not None and not operation_seen:
                raise ValueError("API method has no operation identifier")
            verb = re.fullmatch(r"    ([a-z]+):", line)
            if route is None or not verb or verb[1] not in methods:
                raise ValueError("Unsupported API method or path-item layout")
            method, operation_seen = verb[1].upper(), False
        elif indentation == 6:
            if route is None or method is None:
                raise ValueError("API operation field has no path and method")
            key = re.match(r"      ([A-Za-z][A-Za-z0-9_-]*):", line)
            if not key:
                raise ValueError("Unsupported API operation field layout")
            if key[1] == "operationId":
                operation = re.fullmatch(r"      operationId: ([A-Za-z][A-Za-z0-9_-]{0,99})", line)
                if not operation or operation_seen:
                    raise ValueError("Unsupported or duplicate API operation identifier")
                escaped = re.escape(route)
                pattern = re.sub(r"\\\{[A-Za-z0-9_]+\\\}", "[^/]+", escaped)
                routes.append((method, re.compile(pattern), operation[1]))
                operation_seen = True
            elif key[1] not in fields and not key[1].startswith("x-"):
                raise ValueError("Unsupported API operation field")
        elif indentation < 8:
            raise ValueError("Unsupported API contract indentation")
    if in_paths and method is not None and not operation_seen:
        raise ValueError("API method has no operation identifier")
    if not seen_paths or not routes or len(routes) > 1000:
        raise ValueError("API operation denominator is invalid")
    if len({name for _, _, name in routes}) != len(routes):
        raise ValueError("Duplicate API operation identifier")
    if len({(method, pattern.pattern) for method, pattern, _ in routes}) != len(routes):
        raise ValueError("Duplicate API operation route")
    return routes


def run_command(command, root, timeout):
    """Drain output concurrently into bounded memory; retain no raw log file."""
    try:
        process = subprocess.Popen(command, cwd=root, stdin=subprocess.DEVNULL,
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                   start_new_session=os.name == "posix")
    except OSError:
        return {"exit_code": None, "failure": "tool-unavailable", "output": b""}
    captured = bytearray()
    exceeded = threading.Event()

    def stop():
        """Kill the POSIX process group or the direct Windows child; suppress already-ended errors."""
        if os.name == "posix":
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                # The owned process group may already have exited.
                pass
        else:
            try:
                process.kill()
            except OSError:
                # Best-effort shutdown; the subsequent wait still verifies exit.
                pass

    def drain():
        """Drain the child pipe, retain only bounded bytes and stop the child on overflow."""
        try:
            while True:
                block = process.stdout.read(65536)
                if not block:
                    break
                remaining = MAX_CAPTURE - len(captured)
                captured.extend(block[:remaining])
                if len(block) > remaining:
                    exceeded.set()
                    stop()
        except (OSError, ValueError):
            exceeded.set()
        finally:
            process.stdout.close()

    thread = threading.Thread(target=drain, daemon=True)
    thread.start()
    failure = None
    try:
        process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        failure = "suite-timeout"
        stop()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            failure = "subprocess-cleanup-unverified"
    thread.join(timeout=5)
    if thread.is_alive():
        stop()
        failure = failure or "subprocess-output-not-closed"
    if exceeded.is_set():
        failure = failure or "output-bound-exceeded"
    return {"exit_code": process.returncode, "failure": failure,
            "output": bytes(captured) if failure is None else b""}


def rust_summary(output):
    """Parse one nonempty libtest denominator and reject success credit for failed, ignored, measured or filtered tests."""
    matches = re.findall(rb"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", output)
    if len(matches) != 1:
        raise ValueError("Missing or ambiguous Rust test summary")
    state, passed, failed, ignored, measured, filtered = matches[0]
    counts = dict(zip(("passed", "failed", "ignored", "measured", "filtered_out"),
                      map(int, (passed, failed, ignored, measured, filtered))))
    if sum(counts.values()) > 100000 or counts["passed"] + counts["failed"] == 0:
        raise ValueError("Rust test denominator is invalid")
    return counts, state == b"ok" and counts["failed"] == 0 and counts["ignored"] == 0 and counts["measured"] == 0 and counts["filtered_out"] == 0


def read_client_receipt(path, declared):
    """Validate the closed client receipt, required checks and complete operation denominator."""
    with path.open("rb") as stream:
        raw = stream.read(MAX_CAPTURE + 1)
    if len(raw) > MAX_CAPTURE:
        raise ValueError("Client receipt exceeds its bound")
    def object_pairs(pairs):
        """Reject duplicate members at every JSON object level instead of accepting the last value."""
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("Duplicate receipt key")
            result[key] = value
        return result
    try:
        value = json.loads(raw, object_pairs_hook=object_pairs,
                           parse_constant=lambda _: (_ for _ in ()).throw(ValueError("Invalid numeric constant")))
    except RecursionError as error:
        raise ValueError("Receipt nesting exceeds its bound") from error
    if not isinstance(value, dict):
        raise ValueError("Client receipt must be an object")
    allowed = {"schema_version", "status", "checks", "check_count", "declared_operation_count",
               "observed_operations", "unobserved_operations", "operation_outcomes"}
    if set(value) != allowed or value["schema_version"] != "forge.workspace-client-verification/2" or value["status"] != "passed":
        raise ValueError("Client receipt did not pass its closed contract")
    expected_checks = {
        "upload_requires_confirmation", "idempotent_commit_replays_exact_bytes", "conversion_succeeds",
        "explicit_no_relationship_is_preserved", "omitted_scope_decisions_remain_under_review",
        "explicit_scope_decision_validates", "committed_scope_analysis_succeeds",
        "export_matches_committed_redacted_bytes", "read_only_summary_reconciles_resources",
        "read_only_mutation_is_rejected", "clean_shutdown_writable", "clean_shutdown_read_only",
        "metadata_bundle_preview_preserves_complete_denominator",
        "metadata_bundle_fingerprints_match_exact_bytes",
        "registered_bundle_comparison_reconciles_expected_current",
        "metadata_bundle_queries_preserve_workspace_files",
    }
    checks = value["checks"]
    if not isinstance(checks, list) or any(not isinstance(item, str) for item in checks) or set(checks) != expected_checks or len(checks) != len(expected_checks) or type(value["check_count"]) is not int or value["check_count"] != len(checks):
        raise ValueError("Client checks are incomplete")
    observed, missing = value["observed_operations"], value["unobserved_operations"]
    if not isinstance(observed, list) or not isinstance(missing, list) or observed != sorted(set(observed)) or missing != sorted(set(missing)):
        raise ValueError("Client operation inventory is invalid")
    if type(value["declared_operation_count"]) is not int or value["declared_operation_count"] != len(declared) or set(observed) & set(missing) or set(observed) | set(missing) != set(declared):
        raise ValueError("Client operation denominator is inconsistent")
    outcomes = value["operation_outcomes"]
    if not isinstance(outcomes, dict) or set(outcomes) != set(observed):
        raise ValueError("Client outcomes do not match observed operations")
    total_attempts = 0
    for counts in outcomes.values():
        if not isinstance(counts, dict) or set(counts) != {"succeeded", "rejected", "transport_failed"}:
            raise ValueError("Client outcomes are not closed")
        if any(type(count) is not int or not 0 <= count <= MAX_CLIENT_REQUESTS for count in counts.values()) or sum(counts.values()) == 0:
            raise ValueError("Client outcome count is invalid")
        total_attempts += sum(counts.values())
    if not 0 < total_attempts <= MAX_CLIENT_REQUESTS:
        raise ValueError("Client total request count is invalid")
    return value


def commit_parents(raw, commit):
    """Verify one bounded raw Git commit object and return only its ordered parent identifiers."""
    if not isinstance(raw, bytes) or not raw or len(raw) > MAX_CAPTURE:
        raise ValueError("Commit object exceeds its supported bound")
    if not isinstance(commit, str) or not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", commit):
        raise ValueError("Commit identifier is invalid")
    algorithm = "sha1" if len(commit) == 40 else "sha256"
    digest = hashlib.new(algorithm, b"commit " + str(len(raw)).encode("ascii") + b"\0" + raw, usedforsecurity=False).hexdigest()
    if digest != commit:
        raise ValueError("Commit object does not match captured HEAD")
    headers, separator, _ = raw.partition(b"\n\n")
    if not separator or len(headers) > 65536 or b"\0" in headers:
        raise ValueError("Commit object header is invalid")
    lines = headers.split(b"\n")
    identifier = rb"[0-9a-f]{" + str(len(commit)).encode("ascii") + rb"}"
    if not lines or not re.fullmatch(rb"tree " + identifier, lines[0]):
        raise ValueError("Commit tree identifier is invalid")
    parents = []
    parent_section = True
    for line in lines[1:]:
        if line.startswith(b"parent "):
            if not parent_section or not re.fullmatch(rb"parent " + identifier, line):
                raise ValueError("Commit parent identifier is invalid")
            parents.append(line[7:].decode("ascii"))
            if len(parents) > 256 or len(parents) != len(set(parents)):
                raise ValueError("Commit parent inventory is invalid")
        else:
            parent_section = False
            if not line or line.startswith(b"parent"):
                raise ValueError("Commit object header is invalid")
    return parents


def checkout_binding(identity, expected_commit=None, *, event="local", checkout_kind="local",
                     requested_head=None, requested_base=None):
    """Check the workflow context against the captured object; local runs receive no hosted-context credit."""
    commit = identity["source_commit"]
    parents = identity["ordered_parents"]
    if not isinstance(commit, str) or not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", commit):
        raise ValueError("Captured commit identifier is invalid")
    identifier = re.compile(r"[0-9a-f]{" + str(len(commit)) + r"}")
    if not isinstance(parents, list) or len(parents) > 256 or any(not isinstance(value, str) or not identifier.fullmatch(value) for value in parents) or len(parents) != len(set(parents)):
        raise ValueError("Captured parent inventory is invalid")
    if expected_commit is not None and (not isinstance(expected_commit, str) or not identifier.fullmatch(expected_commit)):
        raise ValueError("Expected commit identifier is invalid")
    if event not in {"local", "pull_request", "push", "workflow_dispatch"} or checkout_kind not in {"local", "pull-request-merge", "head"}:
        raise ValueError("Checkout context is unsupported")
    if event == "local":
        if checkout_kind != "local" or requested_head is not None or requested_base is not None:
            raise ValueError("Local verification has no requested hosted context")
    else:
        if expected_commit is None or (not isinstance(requested_head, str) or not identifier.fullmatch(requested_head)) or checkout_kind == "local":
            raise ValueError("Hosted verification requires exact workflow identifiers")
        if event == "pull_request":
            if (not isinstance(requested_base, str) or not identifier.fullmatch(requested_base)):
                raise ValueError("Pull request verification requires its requested base")
        elif checkout_kind != "head" or requested_base is not None:
            raise ValueError("Head verification has unsupported merge context")
        if checkout_kind == "pull-request-merge":
            if event != "pull_request" or parents != [requested_base, requested_head]:
                raise ValueError("Pull request merge parents differ from requested base and head")
        elif commit != requested_head:
            raise ValueError("Head checkout differs from its requested head")
    if expected_commit is not None and commit != expected_commit:
        raise ValueError("Source commit differs from requested tested commit")
    return {"event": event, "kind": checkout_kind, "requested_head": requested_head,
            "requested_base": requested_base, "tested_commit": commit,
            "ordered_parents": parents.copy(), "hosted_context_asserted": event != "local",
            "binding": "local-commit-object-only" if event == "local" else "workflow-context-and-commit-object"}


def capture_identity(root, forge, timeout):
    """Capture bounded inputs, exact raw commit parents and source cleanliness; discard authors and commit messages."""
    inputs = {name: hash_file(root / name) for name in INPUTS}
    identity = {"inputs": inputs, "provided_release_binary": hash_file(forge)}
    command = run_command(["git", "rev-parse", "HEAD"], root, timeout)
    commit = command["output"].strip().decode("ascii")
    if command["exit_code"] != 0 or command["failure"] is not None or not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", commit):
        raise ValueError("Source commit is unavailable")
    captured = run_command(["git", "cat-file", "commit", "HEAD"], root, timeout)
    if captured["exit_code"] != 0 or captured["failure"] is not None:
        raise ValueError("HEAD commit object is unavailable")
    parents = commit_parents(captured["output"], commit)
    confirmed = run_command(["git", "rev-parse", "HEAD"], root, timeout)
    if confirmed["exit_code"] != 0 or confirmed["failure"] is not None or confirmed["output"].strip() != commit.encode("ascii"):
        raise ValueError("HEAD changed during commit capture")
    dirty = run_command(["git", "status", "--porcelain", "--untracked-files=no"], root, timeout)
    if dirty["exit_code"] != 0 or dirty["failure"] is not None:
        raise ValueError("Tracked source state is unavailable")
    identity.update(source_commit=commit, ordered_parents=parents, tracked_source_clean=not bool(dirty["output"]))
    return identity


def tool_versions(root, timeout):
    """Retain platform versions and validated tool identity fields without copying raw banners."""
    result = {"python": platform.python_version(), "os": platform.system(),
              "os_release": platform.release(), "machine": platform.machine()}
    # Untrusted banners are never copied wholesale into receipts.
    for tool in ("cargo", "rustc"):
        command = run_command([tool, "--version"], root, timeout)
        match = re.fullmatch(rb"(?:cargo|rustc) ([0-9]+\.[0-9]+\.[0-9]+(?:-[A-Za-z0-9.-]+)?) \(([0-9a-f]+) ([0-9-]+)\)\n?", command["output"])
        result[tool] = None if command["exit_code"] != 0 or command["failure"] is not None or not match else {
            "version": match[1].decode("ascii"), "commit": match[2].decode("ascii"),
            "commit_date": match[3].decode("ascii"),
        }
    verbose = run_command(["rustc", "--version", "--verbose"], root, timeout)
    match = re.search(rb"^host: ([A-Za-z0-9_-]+)$", verbose["output"], re.MULTILINE)
    result["rust_host"] = None if verbose["exit_code"] != 0 or verbose["failure"] is not None or not match else match[1].decode("ascii")
    return result


def suite_binding(suite, identity, tools):
    """Distinguish the supplied release bytes from Cargo test-profile executables with unmeasured hashes."""
    inputs = identity["inputs"]
    binding = {"source_commit": identity["source_commit"], "cargo_lock": inputs["Cargo.lock"],
               "cargo_manifest": inputs["Cargo.toml"]}
    if suite == "maintained-client":
        binding.update(cargo_profile="release", executable_binding="provided-release-bytes",
                       provided_release_binary=identity["provided_release_binary"],
                       client_sources={name: inputs[name] for name in (
                           "scripts/test_workspace_client.py", "scripts/workspace_client.py",
                           "scripts/verify_workspace.py")}, tools={"python": tools["python"]})
    else:
        name = "tests/api_contract_validation.rs" if suite == "api-contract" else "tests/workspace_cli_test.rs"
        binding.update(cargo_profile="test", executable_binding="source-lock-tool-test-pins",
                       binary_hash_measured=False, test_source=inputs[name],
                       executable_kind="integration-test" if suite == "api-contract" else "integration-test-and-CARGO_BIN_EXE_forge",
                       tools={key: tools[key] for key in ("cargo", "rustc", "rust_host")})
    return binding


def verify(root, forge, output_dir, selected, timeout=900, expected_commit=None, build_outcome="unrecorded",
           *, event="local", checkout_kind="local", requested_head=None, requested_base=None):
    """Run selected suites despite individual failures and publish a receipt for this slice; keep acceptance gates open."""
    if build_outcome not in BUILD_OUTCOMES:
        raise ValueError("Invalid build outcome")
    if not selected or len(selected) != len(set(selected)) or any(suite not in SUITES for suite in selected):
        raise ValueError("Select unique predefined suites")
    output_dir = Path(output_dir)
    if output_dir.exists() and any(output_dir.iterdir()):
        raise ValueError("Use an empty receipt directory")
    output_dir.mkdir(parents=True, exist_ok=True)
    receipt = {"schema_version": "forge.workspace-verification/2", "scope": "f04-api-foundation",
               "status": "incomplete", "identity": None, "checkout": None, "tools": None,
               "input_stability": "unverified", "suites": {}, "pending_gates": list(PENDING),
               "build": {"outcome": build_outcome, "profile": "release", "features": "default",
                         "locked": True, "offline": True, "binding": "workflow-step-assertion"}}
    for suite in SUITES:
        receipt["suites"][suite] = {"status": "incomplete", "exit_code": None,
                                   "failure": "not-run" if suite in selected else "not-selected", "counts": None,
                                   "execution": None}
    try:
        before = capture_identity(root, forge, min(timeout, 30))
        receipt["identity"] = before
        receipt["tools"] = tool_versions(root, min(timeout, 30))
        declared = [name for _, _, name in contract_routes(root)]
        receipt["checkout"] = checkout_binding(before, expected_commit, event=event,
            checkout_kind=checkout_kind, requested_head=requested_head, requested_base=requested_base)
        with tempfile.TemporaryDirectory(prefix="forge-workspace-receipts-") as directory:
            client_path = Path(directory) / "client.json"
            for suite in selected:
                if suite == "maintained-client":
                    command = [sys.executable, "-B", str(root / "scripts/test_workspace_client.py"),
                               "--forge", str(forge), "--receipt", str(client_path)]
                else:
                    test = "api_contract_validation" if suite == "api-contract" else "workspace_cli_test"
                    command = ["cargo", "test", "--locked", "--offline", "--test", test, "--", "--test-threads=1"]
                completed = run_command(command, root, timeout)
                row = {"status": "failed", "exit_code": completed["exit_code"],
                       "failure": completed["failure"] or "suite-failed", "counts": None,
                       "execution": suite_binding(suite, before, receipt["tools"])}
                if completed["exit_code"] == 0 and completed["failure"] is None:
                    try:
                        if suite == "maintained-client":
                            client = read_client_receipt(client_path, declared)
                            row["counts"] = {"explicit_checks": client["check_count"],
                                             "declared_operations": client["declared_operation_count"],
                                             "observed_operations": len(client["observed_operations"])}
                            row["coverage"] = {key: client[key] for key in (
                                "observed_operations", "unobserved_operations", "operation_outcomes")}
                            passed = True
                        else:
                            row["counts"], passed = rust_summary(completed["output"])
                        row.update(status="passed" if passed else "incomplete",
                                   failure=None if passed else "incomplete-test-denominator")
                    except (ValueError, OSError, TypeError):
                        row["failure"] = "invalid-suite-receipt"
                receipt["suites"][suite] = row
        after = capture_identity(root, forge, min(timeout, 30))
        receipt["input_stability"] = "unchanged" if before == after else "changed"
        tools_complete = all(receipt["tools"][key] is not None for key in ("cargo", "rustc", "rust_host"))
        if before != after:
            receipt["status"] = "failed"
        elif any(row["status"] == "failed" for row in receipt["suites"].values()):
            receipt["status"] = "failed"
        elif build_outcome == "success" and before["tracked_source_clean"] and tools_complete and all(row["status"] == "passed" for row in receipt["suites"].values()):
            receipt["status"] = "passed"
        # Passed means only this declared three-suite slice. All F04 acceptance
        # gates in pending_gates remain open regardless of this status.
    except (ValueError, OSError, UnicodeError, TypeError):
        receipt["status"] = "failed"
        receipt["failure"] = "verification-input-invalid"
    atomic_receipt(output_dir / "workspace-verification.json", receipt)
    return receipt


def main():
    """Validate runner arguments, publish a fresh receipt and return passed/incomplete/failed exit codes."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forge", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--suite", action="append", choices=SUITES, required=True)
    parser.add_argument("--timeout", type=int, default=900)
    parser.add_argument("--expected-commit")
    parser.add_argument("--event", choices=("local", "pull_request", "push", "workflow_dispatch"), default="local")
    parser.add_argument("--checkout-kind", choices=("local", "pull-request-merge", "head"), default="local")
    parser.add_argument("--requested-head")
    parser.add_argument("--requested-base")
    parser.add_argument("--build-outcome", choices=BUILD_OUTCOMES, default="unrecorded")
    args = parser.parse_args()
    if not 1 <= args.timeout <= 1800:
        parser.error("timeout must be between 1 and 1800 seconds")
    root = Path(__file__).resolve().parents[1]
    try:
        receipt = verify(root, args.forge.resolve(), args.output_dir, args.suite,
                         args.timeout, args.expected_commit, args.build_outcome, event=args.event,
                         checkout_kind=args.checkout_kind, requested_head=args.requested_head,
                         requested_base=args.requested_base)
    except (OSError, ValueError):
        print("Workspace verification could not publish a fresh receipt.", file=sys.stderr)
        return 2
    print("Workspace API foundation: " + receipt["status"] + "; acceptance gates remain open.")
    return {"passed": 0, "incomplete": 1, "failed": 2}[receipt["status"]]


if __name__ == "__main__":
    raise SystemExit(main())
