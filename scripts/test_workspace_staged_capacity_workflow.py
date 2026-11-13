#!/usr/bin/env python3
"""Qualify larger staged source capacity on owned Unix fixtures, beyond the inline workflow.

The maintained staged workflow exercises a ~1.2 MiB artifact. This separate
control qualifies the explicit 10 MiB logical transport cap, the shared 20 MiB
retention pool including unresolved reservations, conservative shared admission
below the transport cap, one larger exact restore, and durable outcome survival
across abnormal termination and restart. It reuses the maintained client and the
existing staged workflow helpers; it adds no client or production behavior.

Native interrupted-publication rollback is qualified by the deterministic Rust
control restart_after_interrupted_publication_rolls_back_owned_targets; there is
no production phase-fault hook for racy native mid-restore injection, so this
script terminates only after a settled commit and asserts durable recovery.
Receipts contain closed observations only, never tokens, bodies, paths or raw
errors. No browser/Windows/power-loss/AT/human/release acceptance is implied.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import sys
import tempfile
import time
import traceback

import workspace_client
from workspace_client import Workspace
from test_workspace_staged_source_workflow import (
    Budget, client_refusal, inventory, key, pin, publish,
    server_refusal, session,
)
from test_workspace_source_bundle_client import index_bytes

ARTIFACT_CAP = 10 * 1024 * 1024
PART_BYTES = 32_768
# A 3 MiB opaque source hex-encodes to ~6 MiB; its private plan (2x content + 4 MiB
# journal reserve) plus held stage raw stays inside the shared 20 MiB pool.
RESTORE_CONTENT_BYTES = 3 * 1024 * 1024
# A 4 MiB opaque source hex-encodes to an 8 MiB artifact (below the 10 MiB
# transport cap), but its private plan (2x content + 4 MiB journal reserve = 12 MiB)
# plus the held 8 MiB stage raw exceeds the shared 20 MiB pool, so preview is refused.
REFUSED_CONTENT_BYTES = 4 * 1024 * 1024
CHECKS = (
    "full-capacity-stage-create-and-ready",
    "logical-cap-boundary-exact-and-one-over",
    "unresolved-reservation-blocks-shared-pool-overflow",
    "discard-releases-unresolved-reservation",
    "conservative-shared-admission-refuses-below-cap-preview",
    "larger-restorable-preview-and-exact-commit",
    "abnormal-termination-durable-outcome-survives-restart",
    "fresh-read-only-restart-outcome-and-write-refusal",
)

def checked(receipt, name):
    """Record one completed fixed capacity check against this control's identities."""
    if name not in CHECKS or name in receipt["checks"]:
        raise AssertionError("The native capacity check identity is unsupported")
    receipt["checks"].append(name)


def sha(raw):
    """Hash only exact bytes; the normalized index hash remains a different fact."""
    return hashlib.sha256(raw).hexdigest()


def bundle(payload, *, label="Synthetic staged capacity"):
    """Author one strict Bundle4 artifact with a single opaque lifecycle source.

    The index hash uses the same ordered pretty bytes the server normalizes; the
    artifact is compact so a large payload fits the explicit logical cap. No
    native role admission, freshness or publication is inferred from these bytes.
    """
    resources = [{"key": "fingerprint", "role": "lifecycle-source", "path": "evidence/large.bin"}]
    index = {"schema_version": "forge.workspace/2", "label": label, "resources": resources}
    value = {
        "schema_version": "forge.workspace-index-bundle/4",
        "profile": "index-and-source-hex-staged",
        "source_content_included": True,
        "index": index,
        "index_sha256": sha(index_bytes(index)),
        "pins": [{"key": "fingerprint", "sha256": sha(payload), "size": len(payload)}],
        "contents": [{"key": "fingerprint", "encoding": "hex",
                      "chunks": [payload[n:n + PART_BYTES].hex() for n in range(0, len(payload), PART_BYTES)]}],
    }
    return (json.dumps(value, ensure_ascii=False, separators=(",", ":"), allow_nan=False)).encode()


def declaration(raw):
    """Derive the closed create declaration from exact whole-artifact bytes."""
    return {"schema_version": "forge.workspace-index-bundle/4", "profile": "index-and-source-hex-staged",
            "artifact_sha256": sha(raw), "artifact_size_bytes": len(raw), "chunk_size_bytes": PART_BYTES,
            "chunk_count": (len(raw) + PART_BYTES - 1) // PART_BYTES,
            "acknowledge_sensitive_metadata": True, "acknowledge_source_content": True}


def create_stage(client, raw, budget):
    """Create one acknowledged stage from exact bytes and return its closed status."""
    return budget.call(client.create_source_transfer_stage, raw, acknowledge_sensitive_metadata=True,
                       acknowledge_source_content=True, idempotency_key=key())


def upload(client, stage, raw, budget):
    """Send every exact ordinal once and require the ready full counters."""
    ready = budget.call(client.upload_source_transfer_stage, stage, raw, timeout=budget.remaining(120))
    assert ready["state"] == "ready"
    assert ready["received_chunk_count"] == stage["chunk_count"]
    assert ready["received_bytes"] == len(raw)
    return ready


def seed(project):
    """Seed one small registered policy plus a retained unregistered foreign file."""
    (project / "evidence").mkdir()
    (project / "local.md").write_bytes(b"# Local\n\n## Rule\nRetained old source.\n")
    (project / "leave.bin").write_bytes(b"removed registration file retained\x00")
    index = {"schema_version": "forge.workspace/2", "label": "Synthetic capacity recipient",
             "resources": [{"key": "local", "role": "policy-source", "path": "local.md"}]}
    (project / "forge.workspace.json").write_text(json.dumps(index, separators=(",", ":")) + "\n")


def full_capacity_transport(client, project, budget, receipt):
    """Create and fill a stage at the exact 10 MiB logical cap; readiness grants no write."""
    before = inventory(project)
    payload = bytes(range(256)) * (4096)  # 1 MiB opaque block, repeated to approach the cap
    raw = bundle(payload)
    # Pad with trailing whitespace to the exact logical cap, as the Rust codec control does.
    assert len(raw) < ARTIFACT_CAP
    raw = raw + b" " * (ARTIFACT_CAP - len(raw))
    assert len(raw) == ARTIFACT_CAP
    stage = create_stage(client, raw, budget)
    assert stage["artifact_size_bytes"] == ARTIFACT_CAP and stage["chunk_count"] == 320
    ready = upload(client, stage, raw, budget)
    assert ready["artifact_sha256"] == sha(raw)
    assert inventory(project) == before
    checked(receipt, "full-capacity-stage-create-and-ready")
    # Release the held reservation so later controls can admit their own stages.
    budget.call(client.discard_source_transfer_stage, stage["stage_id"])


def logical_cap_boundary(client, project, budget, receipt):
    """One byte over the logical cap is refused at parse, before any pool admission.

    The exact-cap admission itself is proven by full_capacity_transport; this
    control isolates the logical declaration boundary, which is enforced before
    shared-pool admission and therefore holds even while a full-cap stage is live.
    """
    before = inventory(project)
    raw = bundle(b"\x00\xff\r\n")
    over = raw + b" " * (ARTIFACT_CAP + 1 - len(raw))
    assert len(over) == ARTIFACT_CAP + 1
    server_refusal(client, budget, "POST", client.api_path("/project/source-transfer-stages"),
                   declaration(over), "invalid-request", idempotency=key())
    assert inventory(project) == before
    checked(receipt, "logical-cap-boundary-exact-and-one-over")


def unresolved_reservation_retention(client, project, budget, receipt):
    """A held unresolved 10 MiB reservation blocks a second beyond the shared pool."""
    before = inventory(project)
    raw = bundle(b"\x00\xff\r\n")
    raw = raw + b" " * (ARTIFACT_CAP - len(raw))
    first = create_stage(client, raw, budget)
    assert first["state"] == "receiving" and first["received_bytes"] == 0
    # The first stage reserves its complete declared size before any part arrives;
    # a second full-cap stage would exceed the one shared 20 MiB retention pool.
    server_refusal(client, budget, "POST", client.api_path("/project/source-transfer-stages"),
                   declaration(raw), "payload-too-large", idempotency=key())
    assert budget.call(client.source_transfer_status, first["stage_id"])["state"] == "receiving"
    assert inventory(project) == before
    checked(receipt, "unresolved-reservation-blocks-shared-pool-overflow")
    budget.call(client.discard_source_transfer_stage, first["stage_id"])
    second = create_stage(client, raw, budget)
    assert second["artifact_size_bytes"] == ARTIFACT_CAP
    budget.call(client.discard_source_transfer_stage, second["stage_id"])
    assert inventory(project) == before
    checked(receipt, "discard-releases-unresolved-reservation")


def conservative_preview_refusal(client, project, budget, receipt):
    """A below-cap ready artifact is refused by conservative shared admission, not transport.

    The 8 MiB artifact transports completely (below the 10 MiB logical cap), but
    its private restore plan plus the held stage raw exceeds the one shared 20 MiB
    retention pool, so the preview is refused with the session-retention error and
    no project byte changes. This qualifies the documented conservative boundary.
    """
    before = inventory(project)
    payload = bytes(range(256)) * (REFUSED_CONTENT_BYTES // 256)
    assert len(payload) == REFUSED_CONTENT_BYTES
    raw = bundle(payload)
    assert len(raw) < ARTIFACT_CAP
    stage = create_stage(client, raw, budget)
    ready = upload(client, stage, raw, budget)
    assert ready["state"] == "ready" and ready["artifact_size_bytes"] == len(raw)
    body = {"target_index_schema_version": 2, "acknowledge_index_replacement": True,
            "acknowledge_source_content": True, "acknowledge_replace_files": True}
    server_refusal(client, budget, "POST",
                   client.api_path("/project/source-transfer-stages/" + stage["stage_id"] + "/preview"),
                   body, "invalid-request", idempotency=key())
    assert inventory(project) == before
    # The refused preview leaves the ready transport intact and confers no authority.
    assert budget.call(client.source_transfer_status, stage["stage_id"])["state"] == "ready"
    budget.call(client.discard_source_transfer_stage, stage["stage_id"])
    checked(receipt, "conservative-shared-admission-refuses-below-cap-preview")


def larger_restore(client, project, budget, receipt):
    """Preview and commit one larger exact restore inside the shared restorable ceiling."""
    payload = bytes(range(256)) * (RESTORE_CONTENT_BYTES // 256)
    assert len(payload) == RESTORE_CONTENT_BYTES
    raw = bundle(payload)
    stage = create_stage(client, raw, budget)
    upload(client, stage, raw, budget)
    preview = budget.call(client.prepare_staged_source_restore, stage["stage_id"],
                          target_index_schema_version=2, acknowledge_index_replacement=True,
                          acknowledge_source_content=True, acknowledge_replace_files=True, idempotency_key=key())
    assert preview["preview"]["validation"]["state"] == "valid"
    assert preview["preview"]["validation"]["error_count"] == 0
    assert preview["preview"]["targets"][-1]["kind"] == "index"
    operation_id = preview["preview"]["operation_id"]
    before = inventory(project)
    committed = budget.call(client.commit_source_restore, preview["preview"], confirmed=True, idempotency_key=key())
    assert committed["operation_id"] == operation_id
    settled = budget.call(client.wait_source_restore, operation_id, timeout=budget.remaining(60))
    # "succeeded/committed" is public at commit-decided, before the terminal journal
    # persist; a SIGKILL inside that replace leaves transactions.next, which makes
    # every later journal open fail closed. Settle cleanup first, as the maintained
    # staged workflow does, so abnormal termination lands after a settled commit.
    while settled["state"] == "succeeded" and settled["cleanup_state"] == "pending":
        budget.remaining()
        time.sleep(0.01)
        settled = budget.call(client.source_restore_status, operation_id)
    assert settled["state"] == "succeeded" and settled["write_outcome"] == "committed"
    assert settled["cleanup_state"] == "verified"
    assert (project / "evidence/large.bin").read_bytes() == payload
    assert (project / "leave.bin").read_bytes() == b"removed registration file retained\x00"
    restored_index = json.loads((project / "forge.workspace.json").read_bytes())
    assert any(row["path"] == "evidence/large.bin" for row in restored_index["resources"])
    assert inventory(project) != before
    checked(receipt, "larger-restorable-preview-and-exact-commit")
    return operation_id, payload


def abnormal_restart(binary, project, operation_id, payload, budget, receipt):
    """A settled commit survives SIGKILL and a fresh writable restart with exact bytes."""
    with session(binary, project, False, budget, receipt) as client:
        observed = budget.call(client.source_restore_status, operation_id)
        assert observed["state"] == "succeeded" and observed["write_outcome"] == "committed"
        assert (project / "evidence/large.bin").read_bytes() == payload
    checked(receipt, "abnormal-termination-durable-outcome-survives-restart")


def readonly_restart(binary, project, operation_id, budget, receipt):
    """A fresh read-only restart reads the known outcome and refuses every write."""
    with session(binary, project, True, budget, receipt) as client:
        observed = budget.call(client.source_restore_status, operation_id)
        assert observed["state"] == "succeeded" and observed["write_outcome"] == "committed"
        client_refusal(budget, lambda: client.create_source_transfer_stage(
            bundle(b"\x00"), acknowledge_sensitive_metadata=True, acknowledge_source_content=True,
            idempotency_key=key()), "read-only-session")
    checked(receipt, "fresh-read-only-restart-outcome-and-write-refusal")


def run(args):
    """Run an explicitly supplied binary, owned sessions, and the exact fixed capacity checks."""
    if os.name != "posix" or not __debug__:
        raise RuntimeError("Native Unix staged capacity controls require POSIX and active assertions")
    out = Path(args.out).resolve()
    out.mkdir(mode=0o700)
    binary = Path(args.forge).resolve(strict=True)
    script = Path(__file__).resolve()
    library = Path(workspace_client.__file__).resolve()
    if library.parent != script.parent:
        raise RuntimeError("Use the maintained same-directory client without alternate module fallback")
    sources = {"binary": pin(binary), "source_client": pin(library), "source_capacity_workflow": pin(script)}
    receipt = {"format": "forge.s6-real-staged-capacity-workflow/1", "status": "failed", "platform": sys.platform,
               "sources": sources, "checks": [], "expected_check_count": len(CHECKS), "normal_session_exits": 0,
               "failure": None,
               "scope": "actual native Unix authenticated API2.4 larger-capacity only; no browser/Windows/power-loss/AT/release/human acceptance"}
    budget = Budget(seconds=900)
    try:
        with tempfile.TemporaryDirectory(prefix="forge-staged-capacity-") as name:
            project = Path(name)
            seed(project)
            client = budget.call(Workspace, binary, project, read_only=False, api_major=2)
            assert client._contract_version == "2.4.0"
            operation_id = None
            try:
                full_capacity_transport(client, project, budget, receipt)
                logical_cap_boundary(client, project, budget, receipt)
                unresolved_reservation_retention(client, project, budget, receipt)
                conservative_preview_refusal(client, project, budget, receipt)
                operation_id, payload = larger_restore(client, project, budget, receipt)
            finally:
                # Abnormal termination after a settled commit; never a graceful shutdown credit.
                if client.process.poll() is None:
                    os.kill(client.process.pid, signal.SIGKILL)
                client.process.wait(timeout=budget.remaining(5))
                client.close()
            assert operation_id is not None
            abnormal_restart(binary, project, operation_id, payload, budget, receipt)
            readonly_restart(binary, project, operation_id, budget, receipt)
            assert tuple(receipt["checks"]) == CHECKS and receipt["normal_session_exits"] == 2
            receipt["status"] = "passed"
    except Exception as error:
        frame = traceback.extract_tb(error.__traceback__)[-1]
        receipt["failure"] = {"type": type(error).__name__,
                              "line": frame.lineno,
                              "next_check": CHECKS[len(receipt["checks"])] if len(receipt["checks"]) < len(CHECKS) else None}
    finally:
        receipt["check_count"] = len(receipt["checks"])
        receipt["unobserved_checks"] = [name for name in CHECKS if name not in receipt["checks"]]
        publish(out, receipt)
        print(json.dumps({"status": receipt["status"], "check_count": receipt["check_count"],
                          "expected_check_count": len(CHECKS)}))
    return 0 if receipt["status"] == "passed" else 1


def main():
    """Require a supplied native build and fresh private output without installing tools."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--forge", required=True)
    parser.add_argument("--out", required=True)
    return run(parser.parse_args())


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception:
        print('{"status":"failed","failure":"capacity-workflow-or-receipt-unverified"}')
        raise SystemExit(1)