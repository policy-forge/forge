#!/usr/bin/env python3
"""Prepared Windows staged transaction/crash/rollback qualification; PREP ONLY.

This script is the agent-prepared harness for the later Windows hardware gate.
It is NOT run on macOS/Linux and earns no acceptance here. On a non-Windows host
it fails closed with a typed unsupported-platform receipt; it never simulates
Windows observations. The Windows source-restore port currently returns typed
unavailable until its native implementation and qualification land, so the
transaction/crash/rollback checks below remain unmeasured until that port exists.

The closed check identities mirror the Unix capacity qualification plus the
Windows-specific native ports (owner/DACL private state, directory durability,
no-replace publication, crash/rollback). Receipts contain closed observations
only, never tokens, bodies, paths or raw errors. No merge/release/human acceptance.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys

SCHEMA = "forge.windows-staged-qualification/1"
OUTPUT = "windows-staged-qualification.json"
MAX_BINARY = 256 * 1024 * 1024
# Prepared closed check identities; every one stays unmeasured until the Windows
# native source-restore port exists and the hardware gate runs them for real.
CHECKS = (
    "windows-qualified-private-state-owner-dacl",
    "windows-directory-durability-no-link-ancestry",
    "windows-staged-full-capacity-transport",
    "windows-shared-pool-unresolved-reservation-retention",
    "windows-conservative-shared-admission-refusal",
    "windows-larger-exact-restore-index-last",
    "windows-no-replace-publication-foreign-preserved",
    "windows-crash-before-commit-conditional-rollback",
    "windows-crash-after-commit-durable-outcome",
    "windows-restart-recovery-truthful-none-committed-unknown",
    "windows-read-only-restart-no-recovery-write",
    "windows-sticky-cancellation-deadline",
)
STAGES = ("platform", "binary", "admission", "private-state", "transport",
          "preview", "commit", "crash", "recovery", "cleanup", "receipt")
CODES = ("unsupported-platform", "unsupported-architecture", "invalid-binary",
         "binary-too-large", "native-port-unavailable", "qualification-unmeasured")


class QualificationFault(Exception):
    """Carry one closed stage/code without leaking private bytes or paths."""

    def __init__(self, stage, code):
        super().__init__(code)
        self.stage, self.code = stage, code


def file_pin(path, cap):
    """Hash one bounded stable regular file without retaining its content."""
    try:
        with open(path, "rb") as source:
            before = os.fstat(source.fileno())
            digest, count = hashlib.sha256(), 0
            while True:
                chunk = source.read(min(65536, cap + 1 - count))
                if not chunk:
                    break
                count += len(chunk)
                if count > cap:
                    raise QualificationFault("binary", "binary-too-large")
                digest.update(chunk)
            after = os.fstat(source.fileno())
            if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (
                    after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns) or count != before.st_size:
                raise QualificationFault("binary", "invalid-binary")
            return {"bytes": count, "sha256": digest.hexdigest()}
    except OSError:
        raise QualificationFault("binary", "invalid-binary") from None


def new_receipt():
    """Create the closed envelope; every check is unmeasured, unsupported is never a pass."""
    windows = sys.platform == "win32"
    return {
        "schema_version": SCHEMA,
        "scope": "windows-staged-transaction-crash-rollback-qualification",
        "truth_state": "prepared-not-run",
        "acceptance_eligible": False,
        "status": "incomplete",
        "scoped_complete": False,
        "platform": {
            "system": "Windows" if windows else "unsupported",
            "build": sys.getwindowsversion().build if windows else None,
            "architecture": (os.environ.get("PROCESSOR_ARCHITECTURE") or "unknown") if windows else "unsupported",
            "python": ".".join(map(str, sys.version_info[:3])),
        },
        "identity": {"harness": file_pin(__file__, 131072), "provided_release_binary": None},
        "checks": {name: "unmeasured" for name in CHECKS},
        "expected_check_count": len(CHECKS),
        "failure": None if windows else {"stage": "platform", "code": "unsupported-platform"},
        "limitations": [
            "Prepared by an agent on a non-Windows host; no Windows observation is recorded.",
            "The Windows source-restore port returns typed unavailable until its native implementation lands.",
            "Crash/rollback and no-replace publication require the real Windows hardware gate.",
            "No merge, tag, release, publication or human acceptance is authorized or recorded.",
        ],
    }


def run(args):
    """Fail closed off Windows; on Windows require the native port before any check."""
    out = Path(args.out).resolve()
    out.mkdir(mode=0o700)  # fresh private output: fail fast if it already exists, like the sibling workflows
    receipt = new_receipt()
    try:
        if sys.platform != "win32":
            raise QualificationFault("platform", "unsupported-platform")
        if not (sys.version_info >= (3, 11) and os.environ.get("PROCESSOR_ARCHITECTURE") == "AMD64"):
            raise QualificationFault("platform", "unsupported-architecture")
        binary = Path(args.forge).resolve(strict=True)
        receipt["identity"]["provided_release_binary"] = file_pin(binary, MAX_BINARY)
        # The native Windows source-restore port is not yet implemented; the
        # transaction/crash/rollback checks stay unmeasured until it lands. This
        # prepared harness deliberately records no synthetic Windows credit.
        raise QualificationFault("admission", "native-port-unavailable")
    except QualificationFault as fault:
        receipt["failure"] = {"stage": fault.stage, "code": fault.code}
    finally:
        receipt["measured_check_count"] = sum(1 for v in receipt["checks"].values() if v != "unmeasured")
        raw = (json.dumps(receipt, sort_keys=True, ensure_ascii=True, separators=(",", ":"), allow_nan=False) + "\n").encode("ascii")
        descriptor = os.open(out / OUTPUT, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        print(json.dumps({"status": receipt["status"], "platform": receipt["platform"]["system"],
                          "measured_check_count": receipt["measured_check_count"],
                          "expected_check_count": len(CHECKS)}))
    # A prepared unsupported/unavailable run is a successful closed receipt, not a pass.
    return 0


def main():
    """Require a supplied binary and fresh private output; never install tools."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--forge", required=True)
    parser.add_argument("--out", required=True)
    return run(parser.parse_args())


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception:
        print('{"status":"failed","failure":"windows-qualification-or-receipt-unverified"}')
        raise SystemExit(1)