#!/usr/bin/env python3
"""Bind the standalone Windows console prerequisite to exact source, tools and release bytes.

This separate receipt does not extend the API runner's closed three-suite /2
contract. Native claims require the real producer; synthetic controls are not
Windows, browser, network-denial, accessibility or release acceptance evidence.
"""
import argparse
import json
import os
from pathlib import Path
import re
import stat
import sys
import tempfile

import verify_workspace as shared

sys.dont_write_bytecode = True
SCHEMA = "forge.workspace-windows-console-verification/1"
NATIVE_SCHEMA = "forge.windows-console-smoke/1"
NATIVE_FILENAME = "windows-console-smoke.json"
OUTPUT_FILENAME = "workspace-windows-console-verification.json"
MAX_NATIVE_RECEIPT = 65536
MAX_NATIVE_BINARY = 268435456
EXTRA_INPUTS = (
    "scripts/test_workspace_windows_console.py",
    "scripts/test_workspace_windows_console_controls.py",
    "scripts/verify_workspace_windows_console.py",
    "scripts/test_verify_workspace_windows_console.py",
)
RESPONSES = (
    ("GET", "/api/v1/project/summary", 401),
    ("GET", "/api/v1/session", 401),
    ("POST", "/api/v1/session/unlock", 200),
    ("GET", "/api/v1/session", 200),
    ("GET", "/api/v1/project/summary", 200),
    ("POST", "/api/v1/session/shutdown", 200),
)
BOOL_CHECKS = (
    "worker_job", "console_job", "terminal_bounded", "no_echo_observed",
    "browser_mode", "read_only", "forge_exit_zero", "conpty_closed",
    "terminal_eof", "worker_exit_zero", "job_empty", "forced_cleanup", "binary_unchanged",
)
COUNT_LIMITS = {"prompts": 2, "no_echo_modes": 2, "input_writes": 2, "terminal_bytes": 73728}
FAILURE_STAGES = {"platform", "binary", "admission", "terminal", "unlock", "query", "shutdown", "cleanup", "receipt"}
FAILURE_CODES = {
    "unsupported-platform", "unsupported-architecture", "missing-ctypes", "missing-native-api",
    "invalid-binary", "binary-too-large", "native-api-failed", "job-membership-unverified",
    "owned-process-bound", "worker-timeout", "worker-result-invalid", "prompt-timeout",
    "terminal-malformed", "terminal-output-limit", "terminal-echo", "console-mode-unverified",
    "request-bound", "response-invalid", "response-bound", "request-failed", "unexpected-response",
    "forge-exit-failed", "console-close-timeout", "cleanup-unverified", "fixture-changed",
    "binary-changed", "internal-control-error",
}
STATUS_EXIT = {"passed": 0, "failed": 1, "incomplete": 2}


def exact_keys(value, keys):
    """Reject open objects, including arbitrary private text fields, at every native receipt boundary."""
    if not isinstance(value, dict) or set(value) != set(keys):
        raise ValueError("Native receipt object is not closed")


def file_pin(value, maximum, nullable=False, positive=False):
    """Validate exact integral byte counts and lowercase SHA256 without accepting booleans as integers."""
    if value is None and nullable:
        return
    exact_keys(value, ("bytes", "sha256"))
    if type(value["bytes"]) is not int or not int(positive) <= value["bytes"] <= maximum:
        raise ValueError("Native file byte count is invalid")
    if not isinstance(value["sha256"], str) or not re.fullmatch(r"[0-9a-f]{64}", value["sha256"]):
        raise ValueError("Native file hash is invalid")


def unique_object(pairs):
    """Reject duplicate decoded keys at any JSON nesting depth instead of collapsing their evidence."""
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("Native receipt has a duplicate field")
        result[key] = value
    return result


def reject_constant(_value):
    """Reject JSON nonfinite literals without copying the producer's value into an error."""
    raise ValueError("Native receipt has a nonfinite value")


def native_bytes(path):
    """Read one bounded unchanged regular receipt descriptor; refuse links and retain no raw output."""
    path = Path(path)
    initial = path.lstat()
    if not stat.S_ISREG(initial.st_mode) or path.is_symlink():
        raise ValueError("Native receipt is not regular")
    descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0))
    try:
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode) or before.st_size > MAX_NATIVE_RECEIPT:
            raise ValueError("Native receipt exceeds its bound")
        if (initial.st_dev, initial.st_ino, initial.st_size, initial.st_mtime_ns, initial.st_ctime_ns) != (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns):
            raise ValueError("Native receipt changed before capture")
        with os.fdopen(descriptor, "rb", closefd=False) as stream:
            raw = stream.read(MAX_NATIVE_RECEIPT + 1)
        after = os.fstat(descriptor)
        current = os.stat(path, follow_symlinks=False)
        before_tuple = (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns)
        for observed in (after, current):
            if before_tuple != (observed.st_dev, observed.st_ino, observed.st_size, observed.st_mtime_ns, observed.st_ctime_ns):
                raise ValueError("Native receipt changed during capture")
        if not raw or len(raw) > MAX_NATIVE_RECEIPT:
            raise ValueError("Native receipt exceeds its bound")
        return raw
    finally:
        os.close(descriptor)


def validate_platform(value):
    """Keep only the closed Windows/platform identifiers supplied by the native producer."""
    exact_keys(value, ("system", "build", "architecture", "python"))
    if value["system"] not in {"Windows", "unsupported"} or value["architecture"] not in {"x64", "unsupported"}:
        raise ValueError("Native platform is invalid")
    build = value["build"]
    if build is not None and (type(build) is not int or not 1 <= build <= 99999999):
        raise ValueError("Native build is invalid")
    if not isinstance(value["python"], str) or not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", value["python"]):
        raise ValueError("Native Python identity is invalid")
    if value["system"] == "unsupported" and build is not None:
        raise ValueError("Unsupported platform cannot assert a Windows build")
    if value["system"] == "Windows" and build is None:
        raise ValueError("Windows build must be observed")


def validate_checks(value):
    """Validate bounded counts and a method-aware response prefix, keeping failures separate from pass rules."""
    exact_keys(value, (*BOOL_CHECKS, *COUNT_LIMITS, "responses"))
    for name in BOOL_CHECKS:
        if type(value[name]) is not bool:
            raise ValueError("Native flag is invalid")
    for name, maximum in COUNT_LIMITS.items():
        if type(value[name]) is not int or not 0 <= value[name] <= maximum:
            raise ValueError("Native counter is invalid")
    responses = value["responses"]
    if not isinstance(responses, list) or len(responses) > len(RESPONSES):
        raise ValueError("Native responses exceed their bound")
    for actual, expected in zip(responses, RESPONSES):
        exact_keys(actual, ("method", "path", "status"))
        if (actual["method"], actual["path"]) != expected[:2]:
            raise ValueError("Native response route is invalid")
        if type(actual["status"]) is not int or not 100 <= actual["status"] <= 599:
            raise ValueError("Native response status is invalid")


def validate_pass(value, identity, tools):
    """Require every native observation and exact source/release binding; partial or synthetic skips cannot pass."""
    platform = value["platform"]
    checks = value["checks"]
    if platform["system"] != "Windows" or platform["architecture"] != "x64" or platform["build"] is None or platform["build"] < 17763:
        raise ValueError("Native Windows support is unverified")
    if tools["os"] != "Windows" or "windows" not in tools["rust_host"] or platform["python"] != tools["python"]:
        raise ValueError("Native tool/platform binding differs")
    if tuple(int(part) for part in platform["python"].split(".")) < (3, 11, 0):
        raise ValueError("Native Python support is unverified")
    if any(checks[name] is not (name != "forced_cleanup") for name in BOOL_CHECKS):
        raise ValueError("Native checks are incomplete")
    if any(checks[name] != 2 for name in ("prompts", "no_echo_modes", "input_writes")) or checks["terminal_bytes"] > 65536:
        raise ValueError("Native terminal denominator is incomplete")
    if checks["responses"] != [{"method": method, "path": path, "status": status} for method, path, status in RESPONSES]:
        raise ValueError("Native response denominator is incomplete")
    binaries = value["identity"]["provided_release_binary"]
    if binaries["before"] != identity["provided_release_binary"] or binaries["after"] != binaries["before"]:
        raise ValueError("Native binary binding differs")
    if value["fixture"]["before"] is None or value["fixture"]["after"] != value["fixture"]["before"] or value["fixture"]["unchanged"] is not True:
        raise ValueError("Native fixture preservation is unverified")


def read_native_receipt(path, producer_exit, identity, tools):
    """Replay a closed redacted native receipt and its producer exit, never trusting a passed-looking file alone."""
    raw = native_bytes(path)
    value = json.loads(raw.decode("utf8"), object_pairs_hook=unique_object, parse_constant=reject_constant)
    exact_keys(value, ("schema_version", "scope", "truth_state", "acceptance_eligible", "status", "scoped_complete", "platform", "identity", "fixture", "checks", "failure"))
    if value["schema_version"] != NATIVE_SCHEMA or value["scope"] != "browser-session-console-smoke" or value["truth_state"] != "synthetic-development" or value["acceptance_eligible"] is not False:
        raise ValueError("Native receipt scope is invalid")
    if value["status"] not in STATUS_EXIT or type(producer_exit) is not int or producer_exit != STATUS_EXIT[value["status"]] or type(value["scoped_complete"]) is not bool:
        raise ValueError("Native producer outcome differs")
    validate_platform(value["platform"])
    exact_keys(value["identity"], ("harness", "provided_release_binary"))
    file_pin(value["identity"]["harness"], 131072, positive=True)
    if value["identity"]["harness"] != identity["inputs"][EXTRA_INPUTS[0]]:
        raise ValueError("Native harness binding differs")
    binaries = value["identity"]["provided_release_binary"]
    exact_keys(binaries, ("before", "after"))
    for item in binaries.values():
        file_pin(item, MAX_NATIVE_BINARY, nullable=True)
    exact_keys(value["fixture"], ("before", "after", "unchanged"))
    file_pin(value["fixture"]["before"], MAX_NATIVE_BINARY, nullable=True)
    file_pin(value["fixture"]["after"], MAX_NATIVE_BINARY, nullable=True)
    if type(value["fixture"]["unchanged"]) is not bool:
        raise ValueError("Native fixture flag is invalid")
    validate_checks(value["checks"])
    exact_keys(value["failure"], ("stage", "code"))
    if value["status"] == "passed":
        if value["scoped_complete"] is not True or value["failure"] != {"stage": None, "code": None}:
            raise ValueError("Native pass is contradictory")
        validate_pass(value, identity, tools)
    else:
        if value["scoped_complete"] is not False or value["failure"]["stage"] not in FAILURE_STAGES or value["failure"]["code"] not in FAILURE_CODES:
            raise ValueError("Native failure is contradictory")
        if value["status"] == "incomplete" and (value["failure"]["stage"] != "platform" or value["failure"]["code"] not in {"unsupported-platform", "unsupported-architecture", "missing-ctypes", "missing-native-api"}):
            raise ValueError("Native runtime failure cannot become an unsupported skip")
        if value["status"] == "incomplete" and (any(value["checks"][name] for name in BOOL_CHECKS) or any(value["checks"][name] for name in COUNT_LIMITS) or value["checks"]["responses"]):
            raise ValueError("Unsupported native execution cannot assert observed checks")
    return value, {"sha256": shared.hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def capture_identity(root, forge, timeout):
    """Extend the existing exact commit capture with every new helper/control/wrapper input without changing /2."""
    identity = shared.capture_identity(root, forge, timeout)
    identity["inputs"].update({name: shared.hash_file(root / name) for name in EXTRA_INPUTS})
    return identity


def native_run(root, forge, timeout, identity, tools):
    """Invoke only the predefined native helper in fresh temporary storage and discard all raw child output."""
    with tempfile.TemporaryDirectory(prefix="forge-windows-console-receipt-") as private:
        command = [sys.executable, "-B", str(root / EXTRA_INPUTS[0]), "--forge", str(forge), "--output-dir", private]
        completed = shared.run_command(command, root, timeout)
        row = {"status": "failed", "producer_exit_code": completed["exit_code"], "failure": "native-producer-failed", "receipt": None, "receipt_pin": None}
        if completed["failure"] is not None:
            row["failure"] = "native-execution-unverified"
            return row
        try:
            value, receipt_pin = read_native_receipt(Path(private) / NATIVE_FILENAME, completed["exit_code"], identity, tools)
            row.update(status=value["status"], failure=None if value["status"] == "passed" else "native-prerequisite-not-passed", receipt=value, receipt_pin=receipt_pin)
        except (ValueError, OSError, UnicodeError, TypeError, KeyError, RecursionError):
            row["failure"] = "invalid-native-receipt"
        return row


def tools_complete(tools):
    """Require validated Rust/Cargo/Python/platform fields before starting or crediting native execution."""
    return all(tools.get(key) is not None for key in ("cargo", "rustc", "rust_host")) and all(isinstance(tools.get(key), str) and tools[key] for key in ("rust_host", "python", "os", "os_release", "machine"))


def verify(root, forge, output_dir, timeout=120, expected_commit=None, build_outcome="unrecorded", *, event="local", checkout_kind="local", requested_head=None, requested_base=None):
    """Publish a scoped prerequisite receipt only after source/tool/release stability and native outcome reconciliation."""
    if build_outcome not in shared.BUILD_OUTCOMES or type(timeout) is not int or not 1 <= timeout <= 300:
        raise ValueError("Invalid orchestrator bounds")
    output_dir = Path(output_dir)
    if output_dir.exists() and (not output_dir.is_dir() or any(output_dir.iterdir())):
        raise ValueError("Use fresh receipt storage")
    output_dir.mkdir(parents=True, exist_ok=True)
    receipt = {"schema_version": SCHEMA, "scope": "f04-windows-controlling-terminal-prerequisite", "truth_state": "synthetic-development", "acceptance_eligible": False,
               "status": "failed", "failure": None, "identity": None, "checkout": None, "tools": None,
               "input_stability": "unverified", "tool_stability": "unverified", "native": {"status": "not-run", "producer_exit_code": None, "failure": "not-run", "receipt": None, "receipt_pin": None},
               "build": {"outcome": build_outcome, "profile": "release", "features": "default", "locked": True, "offline": True, "binding": "workflow-step-assertion"}, "pending_gates": list(shared.PENDING)}
    try:
        before = capture_identity(root, forge, min(timeout, 30))
        receipt["identity"] = before
        tools = shared.tool_versions(root, min(timeout, 30))
        receipt["tools"] = tools
        receipt["checkout"] = shared.checkout_binding(before, expected_commit, event=event, checkout_kind=checkout_kind, requested_head=requested_head, requested_base=requested_base)
        if not before["tracked_source_clean"]:
            receipt["failure"] = "tracked-source-unclean"
        elif not tools_complete(tools):
            receipt["failure"] = "tool-identity-unavailable"
        elif build_outcome != "success":
            receipt["status"] = "incomplete" if build_outcome in {"unrecorded", "skipped"} else "failed"
            receipt["failure"] = "release-build-not-qualified"
        else:
            receipt["native"] = native_run(root, forge, timeout, before, tools)
            receipt["status"] = {"passed": "passed", "failed": "failed", "incomplete": "incomplete"}[receipt["native"]["status"]]
            receipt["failure"] = receipt["native"]["failure"]
        after = capture_identity(root, forge, min(timeout, 30))
        tools_after = shared.tool_versions(root, min(timeout, 30))
        receipt["input_stability"] = "unchanged" if before == after else "changed"
        receipt["tool_stability"] = "unchanged" if tools == tools_after else "changed"
        if before != after or tools != tools_after or not tools_complete(tools_after):
            receipt.update(status="failed", failure="verification-input-changed")
    except Exception:
        # No arbitrary exception text enters the receipt or public process output.
        receipt.update(status="failed", failure="verification-input-invalid")
    shared.atomic_receipt(output_dir / OUTPUT_FILENAME, receipt)
    return receipt


def main():
    """Expose only bounded inputs and fixed diagnostics; publication failure is nonzero even if a link remains."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forge", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--timeout", type=int, default=120)
    parser.add_argument("--expected-commit")
    parser.add_argument("--event", choices=("local", "pull_request", "push", "workflow_dispatch"), default="local")
    parser.add_argument("--checkout-kind", choices=("local", "pull-request-merge", "head"), default="local")
    parser.add_argument("--requested-head")
    parser.add_argument("--requested-base")
    parser.add_argument("--build-outcome", choices=shared.BUILD_OUTCOMES, default="unrecorded")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    try:
        value = verify(root, args.forge.resolve(), args.output_dir, args.timeout, args.expected_commit, args.build_outcome,
                       event=args.event, checkout_kind=args.checkout_kind, requested_head=args.requested_head, requested_base=args.requested_base)
    except Exception:
        print("Windows console verification could not publish a fresh receipt.", file=sys.stderr)
        return 2
    print("Windows console prerequisite: " + value["status"] + "; acceptance gates remain open.")
    return {"passed": 0, "incomplete": 1, "failed": 2}[value["status"]]


if __name__ == "__main__":
    raise SystemExit(main())
