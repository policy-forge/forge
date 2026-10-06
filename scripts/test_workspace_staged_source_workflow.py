#!/usr/bin/env python3
"""Exercise real API2.4 staged source transfers on owned Unix fixtures.

This maintained workflow uses the selected Python client and authenticated
loopback requests. Its author packet is an unexecuted proposal. A successful
native run establishes only these concrete checks, not Windows, power-loss,
browser, accessibility, release, or human acceptance.
"""

import argparse
from contextlib import contextmanager
import datetime
import hashlib
import http.client
import json
import math
import os
from pathlib import Path
import re
import stat
import sys
import tempfile
import time
import uuid

import workspace_client
from workspace_client import Workspace, WorkspaceError

INLINE_CAP = 1_048_429
ARTIFACT_CAP = 10 * 1024 * 1024
PART_BYTES = 32_768
OPAQUE_SIZE = 600 * 1024
RESPONSE_CAP = 1024 * 1024
CHECKS = (
    "stream-original-202-replay",
    "stream-preview-no-project-write",
    "stream-manifest-before-commit-refused",
    "stream-one-explicit-generic-commit",
    "stream-full-part-and-download-hashes",
    "bundle4-exceeds-unchanged-inline-cap",
    "stream-private-family-isolation",
    "stream-invalid-query-and-get-body-refusals",
    "donor-original-registered-inputs-unchanged",
    "stage-original-201-replay",
    "stage-first-part-original-acknowledgment",
    "stage-ready-exact-full-counters",
    "stage-old-replays-retain-original-counters-and-expiry",
    "stage-malformed-unknown-and-query-refusals",
    "stage-get-empty-object-only-and-legacy-get-unchanged",
    "separate-stage-discard-does-not-touch-project",
    "genuine-preview-reply-discard-and-explicit-same-request-retry",
    "prepared-stage-original-preview-replay-without-raw-renewal",
    "complete-native-preview-and-index-last-no-write",
    "known-id-before-acceptance-is-not-found",
    "one-exact-restore-confirmation",
    "restore-normal-committed-cleanup-settlement",
    "restored-exact-source-and-complete-index",
    "removed-registration-files-retained",
    "terminal-cancel-preserves-committed-facts",
    "fresh-read-only-known-outcome-lookup",
    "fresh-read-only-stage-session-isolation",
    "fresh-read-only-staged-and-confirmation-writes-refused",
    "fresh-writable-old-receipt-does-not-revive-authority",
)


class Budget:
    """Keep one caller deadline across launches, paced calls, and settlement."""

    def __init__(self, seconds=600):
        """Fix the original finite workflow deadline before any native launch."""
        self.deadline = time.monotonic() + seconds

    def remaining(self, ceiling=30):
        """Refuse an exhausted budget without renewing it or starting a call."""
        remaining = self.deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("The native staged workflow budget expired")
        return min(ceiling, remaining)

    def call(self, function, *args, **kwargs):
        """Fence actual client work cooperatively before and after its bounded call."""
        self.remaining()
        value = function(*args, **kwargs)
        self.remaining()
        return value


def sha(raw):
    """Hash only exact bytes; normalized index hashes remain a different fact."""
    return hashlib.sha256(raw).hexdigest()


def pin(path):
    """Read one bounded stable regular file without retaining its content in receipts."""
    with Path(path).open("rb") as stream:
        before = os.fstat(stream.fileno())
        if not stat.S_ISREG(before.st_mode) or before.st_size > 256 * 1024 * 1024:
            raise ValueError("The workflow input is not a bounded regular file")
        hasher = hashlib.sha256()
        count = 0
        while True:
            raw = stream.read(64 * 1024)
            if not raw:
                break
            count += len(raw)
            if count > before.st_size:
                raise ValueError("The workflow input changed during hashing")
            hasher.update(raw)
        after = os.fstat(stream.fileno())
    identity = ("st_dev", "st_ino", "st_size", "st_mtime_ns")
    if count != before.st_size or any(getattr(before, name) != getattr(after, name) for name in identity):
        raise ValueError("The workflow input changed during hashing")
    return {"sha256": hasher.hexdigest(), "bytes": count}


def inventory(project):
    """Compare the complete small owned fixture, including prospective directory creation."""
    files, directories, pending, count = {}, [], [Path(project)], 0
    while pending:
        parent = pending.pop()
        with os.scandir(parent) as entries:
            for entry in entries:
                count += 1
                if count > 64:
                    raise ValueError("The owned fixture inventory exceeds its bound")
                relative = Path(entry.path).relative_to(project).as_posix()
                if entry.is_symlink():
                    raise ValueError("The owned fixture contains an unexpected link")
                if entry.is_dir(follow_symlinks=False):
                    directories.append(relative)
                    pending.append(Path(entry.path))
                elif entry.is_file(follow_symlinks=False):
                    files[relative] = pin(entry.path)
                else:
                    raise ValueError("The owned fixture contains an unsupported entry")
    return {"files": files, "directories": sorted(directories)}


def key():
    """Generate a server-admitted identity without recording capability or receipt tokens."""
    return str(uuid.uuid4())


def checked(receipt, name):
    """Record one completed fixed check, never partial credit or duplicate uniqueness."""
    if name not in CHECKS or name in receipt["checks"]:
        raise AssertionError("The native check identity is unsupported")
    receipt["checks"].append(name)


def exchange(client, budget, method, path, body=None, *, idempotency=None, discard_reply=False):
    """Send one real authenticated bounded HTTP request, with no automatic retry.

    GET bodies are sent only by explicit negative/empty-object checks. An
    intentionally discarded successful reply is drained and hashed privately;
    the caller must explicitly retry the original method/path/key/body.
    """
    if not path.startswith(client._api_prefix + "/") or "#" in path:
        raise ValueError("Use the selected public namespace")
    delay = client._next_request_at - time.monotonic()
    if delay > 0:
        time.sleep(min(delay, budget.remaining(delay)))
    client._next_request_at = time.monotonic() + 0.06
    headers = {"Authorization": "Bearer " + client._capability, "Accept": "application/json"}
    encoded = None if body is None else json.dumps(body, ensure_ascii=False, allow_nan=False).encode()
    if method != "GET" or encoded is not None:
        headers["Content-Type"] = "application/json"
    if idempotency is not None:
        headers["Idempotency-Key"] = idempotency
    if encoded is not None and len(encoded) > RESPONSE_CAP:
        raise ValueError("The explicit request exceeds the unchanged body bound")
    connection = http.client.HTTPConnection("127.0.0.1", client._port, timeout=budget.remaining())
    try:
        connection.request(method, path, body=encoded, headers=headers)
        response = connection.getresponse()
        raw = response.read(RESPONSE_CAP + 1)
        budget.remaining()
        if len(raw) > RESPONSE_CAP:
            raise RuntimeError("The explicit response exceeds its bound")
        if response.getheader("Content-Type", "").split(";", 1)[0].strip().lower() != "application/json":
            raise RuntimeError("The explicit response has an unsupported media type")
        facts = {"status": response.status, "bytes": len(raw), "sha256": sha(raw)}
        value = None if discard_reply else json.loads(raw)
        return response.status, value, facts
    finally:
        connection.close()


def server_refusal(client, budget, method, path, body, code, *, idempotency=None):
    """Require a genuine non-success public response with the exact flat typed error."""
    status, value, _ = exchange(client, budget, method, path, body, idempotency=idempotency)
    assert 400 <= status < 600 and isinstance(value, dict) and value.get("code") == code
    assert type(value.get("retryable")) is bool and isinstance(value.get("message"), str)
    return {"status": status, "code": code}


def client_refusal(budget, action, code):
    """Check a real maintained-client server refusal without logging its private envelope."""
    try:
        budget.call(action)
    except WorkspaceError as error:
        assert error.payload["code"] == code
        return code
    raise AssertionError("The expected server refusal was not observed")


@contextmanager
def session(binary, project, read_only, budget, receipt):
    """Own one actual process and count only authenticated shutdown plus native zero/reap."""
    client = budget.call(Workspace, binary, project, read_only=read_only, api_major=2)
    try:
        assert client._contract_version == "2.4.0"
        yield client
        assert budget.call(client.request, "POST", client.api_path("/session/shutdown"), {}) == {"state": "shutting-down"}
        assert client.process.wait(timeout=budget.remaining(5)) == 0
        budget.remaining()
        receipt["normal_session_exits"] += 1
    finally:
        # A failed session may require the library's bounded owned cleanup; it
        # cannot earn a successful normal-exit or nonforced-cleanup observation.
        client.close()


def write_index(project, label, resources):
    """Write one explicit authorial index2 in the isolated fixture, without discovery."""
    value = {"schema_version": "forge.workspace/2", "label": label, "resources": resources}
    (project / "forge.workspace.json").write_text(json.dumps(value, separators=(",", ":")) + "\n")


def fixtures(donor, recipient):
    """Seed a real 600KiB registered opaque source plus retained old recipient files."""
    source = b"# Policy\n\n## Rule\nThe service must restrict access.\n"
    opaque = bytes(range(256)) * (OPAQUE_SIZE // 256)
    assert len(opaque) == OPAQUE_SIZE
    (donor / "policies").mkdir()
    (donor / "evidence").mkdir()
    (donor / "policies/access.md").write_bytes(source)
    (donor / "evidence/source.bin").write_bytes(opaque)
    resources = [{"key": "policy", "role": "policy-source", "path": "policies/access.md"},
                 {"key": "fingerprint", "role": "lifecycle-source", "path": "evidence/source.bin"}]
    write_index(donor, "Synthetic staged donor", resources)
    (recipient / "local.md").write_bytes(b"# Local\n\n## Rule\nRetained old source.\n")
    (recipient / "leave.bin").write_bytes(b"removed registration file retained\x00")
    write_index(recipient, "Synthetic staged recipient", [
        {"key": "local", "role": "policy-source", "path": "local.md"},
        {"key": "removed", "role": "lifecycle-source", "path": "leave.bin"}])
    return source, opaque, resources


def stream_export(client, donor, before, budget, receipt):
    """Observe a real202/no-write preview, one commit, and every actual exact transport part."""
    request = {"target_path": "staged-source.json", "acknowledge_sensitive_metadata": True, "acknowledge_source_content": True}
    export_key = key()
    status, operation, original_reply = exchange(client, budget, "POST", client.api_path("/project/source-stream-exports"), request, idempotency=export_key)
    assert status == 202
    assert budget.call(client.prepare_source_stream_export, "staged-source.json", acknowledge_sensitive_metadata=True,
                       acknowledge_source_content=True, idempotency_key=export_key) == operation
    status, replay, replay_reply = exchange(client, budget, "POST", client.api_path("/project/source-stream-exports"), request, idempotency=export_key)
    assert status == 202 and replay == operation and replay_reply == original_reply
    checked(receipt, "stream-original-202-replay")
    completed = budget.call(client.wait, operation)
    assert completed["state"] == "succeeded" and completed.get("error") is None
    preview = completed["result"]["preview"]
    assert inventory(donor) == before and not (donor / "staged-source.json").exists()
    checked(receipt, "stream-preview-no-project-write")
    client_refusal(budget, lambda: client.source_stream_manifest(operation["operation_id"], expected_sha256=preview["exact_bytes_sha256"]), "not-found")
    checked(receipt, "stream-manifest-before-commit-refused")
    committed = budget.call(client.commit, preview, confirmed=True, idempotency_key=key())
    assert committed["state"] == "succeeded"
    checked(receipt, "stream-one-explicit-generic-commit")
    manifest = budget.call(client.source_stream_manifest, operation["operation_id"], expected_sha256=preview["exact_bytes_sha256"])
    original = (donor / "staged-source.json").read_bytes()
    assert len(original) == manifest["artifact_size_bytes"] <= ARTIFACT_CAP
    assert sha(original) == manifest["artifact_sha256"] == preview["exact_bytes_sha256"]
    assert manifest["chunk_count"] == math.ceil(len(original) / PART_BYTES)
    joined = bytearray()
    for ordinal in range(manifest["chunk_count"]):
        part = budget.call(client.source_stream_chunk, manifest, ordinal)
        raw = bytes.fromhex(part["hex"])
        assert raw == original[ordinal * PART_BYTES:(ordinal + 1) * PART_BYTES]
        assert part["sha256"] == sha(raw) and part["size_bytes"] == len(raw)
        joined.extend(raw)
    assert bytes(joined) == original
    downloaded = budget.call(client.download_source_stream_export, operation["operation_id"], expected_sha256=manifest["artifact_sha256"])
    assert downloaded == original == Workspace._staged_bundle(original)
    receipt["artifact"] = {"sha256": sha(original), "bytes": len(original), "chunk_count": manifest["chunk_count"], "chunk_size_bytes": PART_BYTES}
    checked(receipt, "stream-full-part-and-download-hashes")
    assert len(original) > INLINE_CAP
    checked(receipt, "bundle4-exceeds-unchanged-inline-cap")
    return original, operation["operation_id"]


def stream_refusals(client, operation_id, budget, receipt):
    """Refuse foreign private families, inadmissible queries/bodies, and out-of-range ordinals."""
    for family in ("bundle-exports", "source-bundle-exports"):
        server_refusal(client, budget, "GET", client.api_path("/project/" + family + "/" + operation_id + "/download"), None, "not-found")
    checked(receipt, "stream-private-family-isolation")
    path = client.api_path("/project/source-stream-exports/" + operation_id + "/manifest")
    server_refusal(client, budget, "GET", path + "?unexpected=1", None, "invalid-request")
    server_refusal(client, budget, "GET", path, {"unexpected": True}, "invalid-request")
    server_refusal(client, budget, "GET", client.api_path("/project/source-stream-exports/" + operation_id + "/chunks/320"), None, "invalid-request")
    checked(receipt, "stream-invalid-query-and-get-body-refusals")


def upload(client, bundle, budget, receipt):
    """Create one actual201 stage and verify old PUT replay counters after all parts arrive."""
    stage_key = key()
    declaration = {"schema_version": "forge.workspace-index-bundle/4", "profile": "index-and-source-hex-staged",
                   "artifact_sha256": sha(bundle), "artifact_size_bytes": len(bundle), "chunk_size_bytes": PART_BYTES,
                   "chunk_count": math.ceil(len(bundle) / PART_BYTES)}
    request = {**declaration, "acknowledge_sensitive_metadata": True, "acknowledge_source_content": True}
    status, stage, original_reply = exchange(client, budget, "POST", client.api_path("/project/source-transfer-stages"), request, idempotency=stage_key)
    assert status == 201
    assert budget.call(client.create_source_transfer_stage, bundle, acknowledge_sensitive_metadata=True,
                       acknowledge_source_content=True, idempotency_key=stage_key) == stage
    status, replay, replay_reply = exchange(client, budget, "POST", client.api_path("/project/source-transfer-stages"), request, idempotency=stage_key)
    assert status == 201 and replay == stage and replay_reply == original_reply
    assert stage["received_chunk_count"] == 0 and stage["received_bytes"] == 0
    checked(receipt, "stage-original-201-replay")
    first_request = {"sha256": sha(bundle[:PART_BYTES]), "size_bytes": PART_BYTES, "hex": bundle[:PART_BYTES].hex()}
    first_path = client.api_path("/project/source-transfer-stages/" + stage["stage_id"] + "/chunks/0")
    status, first, first_reply = exchange(client, budget, "PUT", first_path, first_request)
    assert status == 200
    assert first["received_chunk_count"] == 1 and first["received_bytes"] == PART_BYTES
    assert budget.call(client.put_source_transfer_chunk, stage, 0, bundle[:PART_BYTES]) == first
    status, replay, replay_reply = exchange(client, budget, "PUT", first_path, first_request)
    assert status == 200 and replay == first and replay_reply == first_reply
    checked(receipt, "stage-first-part-original-acknowledgment")
    for ordinal in range(1, stage["chunk_count"]):
        budget.call(client.put_source_transfer_chunk, stage, ordinal, bundle[ordinal * PART_BYTES:(ordinal + 1) * PART_BYTES])
    ready = budget.call(client.source_transfer_status, stage["stage_id"])
    assert ready["state"] == "ready" and ready["received_chunk_count"] == stage["chunk_count"]
    assert ready["received_bytes"] == len(bundle) and ready["expires_at"] == stage["expires_at"]
    checked(receipt, "stage-ready-exact-full-counters")
    assert budget.call(client.put_source_transfer_chunk, stage, 0, bundle[:PART_BYTES]) == first
    status, replay, replay_reply = exchange(client, budget, "PUT", first_path, first_request)
    assert status == 200 and replay == first and replay_reply == first_reply
    assert budget.call(client.create_source_transfer_stage, bundle, acknowledge_sensitive_metadata=True,
                       acknowledge_source_content=True, idempotency_key=stage_key) == stage
    status, replay, replay_reply = exchange(client, budget, "POST", client.api_path("/project/source-transfer-stages"), request, idempotency=stage_key)
    assert status == 201 and replay == stage and replay_reply == original_reply
    assert budget.call(client.source_transfer_status, stage["stage_id"]) == ready
    checked(receipt, "stage-old-replays-retain-original-counters-and-expiry")
    receipt["stage"] = {"artifact_bytes": len(bundle), "chunk_count": stage["chunk_count"], "received_chunk_count": ready["received_chunk_count"], "received_bytes": ready["received_bytes"]}
    return stage, request


def stage_refusals(client, stage, declaration, bundle, recipient, before, budget, receipt):
    """Exercise public admission and a separate transport discard without native writes."""
    base = client.api_path("/project/source-transfer-stages")
    path = base + "/" + stage["stage_id"]
    server_refusal(client, budget, "GET", base + "/bst_" + "0" * 16, None, "not-found")
    server_refusal(client, budget, "GET", base + "/bad", None, "invalid-request")
    server_refusal(client, budget, "GET", path + "?unexpected=1", None, "invalid-request")
    server_refusal(client, budget, "PUT", path + "/chunks/320", {"sha256": sha(b"x"), "size_bytes": 1, "hex": "78"}, "invalid-request")
    server_refusal(client, budget, "PUT", path + "/chunks/0", {"sha256": sha(bundle[:PART_BYTES]), "size_bytes": PART_BYTES, "hex": bundle[:PART_BYTES].hex(), "unexpected": True}, "invalid-request")
    server_refusal(client, budget, "POST", base, {**declaration, "unexpected": True}, "invalid-request", idempotency=key())
    checked(receipt, "stage-malformed-unknown-and-query-refusals")
    status, empty_body, _ = exchange(client, budget, "GET", path, {})
    assert status == 200 and empty_body == budget.call(client.source_transfer_status, stage["stage_id"])
    server_refusal(client, budget, "GET", path, {"unexpected": True}, "invalid-request")
    server_refusal(client, budget, "GET", client.api_path("/project/summary"), {}, "invalid-request")
    checked(receipt, "stage-get-empty-object-only-and-legacy-get-unchanged")
    extra = budget.call(client.create_source_transfer_stage, bundle, acknowledge_sensitive_metadata=True,
                        acknowledge_source_content=True, idempotency_key=key())
    retired = budget.call(client.discard_source_transfer_stage, extra["stage_id"])
    assert retired == {"stage_id": extra["stage_id"], "discarded": True}
    discarded = budget.call(client.source_transfer_status, extra["stage_id"])
    assert discarded["state"] == "discarded" and discarded["received_chunk_count"] == 0 and discarded["received_bytes"] == 0
    client_refusal(budget, lambda: client.put_source_transfer_chunk(extra, 0, bundle[:PART_BYTES]), "version-conflict")
    assert inventory(recipient) == before
    checked(receipt, "separate-stage-discard-does-not-touch-project")


def restore_preview(client, stage, bundle, recipient, before, resources, budget, receipt):
    """Discard one genuine200 preview reply, then explicitly recover its original same-key wrapper."""
    preview_key = key()
    body = {"target_index_schema_version": 2, "acknowledge_index_replacement": True,
            "acknowledge_source_content": True, "acknowledge_replace_files": True}
    path = client.api_path("/project/source-transfer-stages/" + stage["stage_id"] + "/preview")
    status, dropped, original_facts = exchange(client, budget, "POST", path, body, idempotency=preview_key, discard_reply=True)
    assert status == 200 and dropped is None and inventory(recipient) == before
    # This is an explicit manual retry at the call site, not an exception retry
    # inside exchange, Workspace, or the upload/preparation helpers.
    status, recovered, replay_facts = exchange(client, budget, "POST", path, body, idempotency=preview_key)
    assert status == 200 and replay_facts == original_facts
    checked(receipt, "genuine-preview-reply-discard-and-explicit-same-request-retry")
    arguments = {**body, "idempotency_key": preview_key}
    result = budget.call(client.prepare_staged_source_restore, stage["stage_id"], **arguments)
    assert result == recovered
    assert budget.call(client.source_transfer_status, stage["stage_id"])["state"] == "prepared"
    client_refusal(budget, lambda: client.prepare_staged_source_restore(stage["stage_id"], **{**arguments, "idempotency_key": key()}), "version-conflict")
    checked(receipt, "prepared-stage-original-preview-replay-without-raw-renewal")
    preview, replacement = result["preview"], result["replacement"]
    artifact = json.loads(bundle)
    pins_by_key = {row["key"]: row for row in artifact["pins"]}
    assert len(preview["targets"]) == 3 and preview["targets"][-1]["kind"] == "index"
    assert [row["path"] for row in preview["targets"]] == [row["path"] for row in resources] + ["forge.workspace.json"]
    assert {row["path"] for row in preview["directories"]} == {"policies", "evidence"}
    assert len(preview["input_bindings"]) == replacement["consumed_file_count"] == 5
    assert replacement["removed_resource_keys"] == ["local", "removed"]
    assert replacement["proposed_index"]["resources"] == resources
    assert replacement["supplied_index_sha256"] == artifact["index_sha256"]
    assert replacement["previous_index"] == json.loads((recipient / "forge.workspace.json").read_bytes())
    for target in preview["targets"][:-1]:
        supplied = pins_by_key[target["key"]]
        assert target["exact_bytes_sha256"] == supplied["sha256"] and target["size"] == supplied["size"]
    for binding in preview["input_bindings"]:
        if binding["current"] is not None:
            original = before["files"][binding["path"]]
            assert binding["current"]["sha256"] == original["sha256"] and binding["current"]["size"] == original["bytes"]
    assert inventory(recipient) == before
    checked(receipt, "complete-native-preview-and-index-last-no-write")
    client_refusal(budget, lambda: client.source_restore_status(preview["operation_id"]), "not-found")
    assert inventory(recipient) == before
    checked(receipt, "known-id-before-acceptance-is-not-found")
    return preview


def restore_confirm(client, preview, recipient, before, source, opaque, resources, budget, receipt):
    """Confirm once, observe truthful pending cleanup, and require normal committed settlement."""
    accepted = budget.call(client.commit_source_restore, preview, confirmed=True, idempotency_key=key())
    assert accepted["operation_id"] == preview["operation_id"] and accepted["state"] == "pending"
    assert accepted["write_outcome"] == "unmeasured"
    checked(receipt, "one-exact-restore-confirmation")
    settlement_deadline = min(budget.deadline, time.monotonic() + 65)
    outcome = budget.call(client.wait_source_restore, preview["operation_id"], timeout=min(65, budget.remaining(65)))
    pending = 0
    while outcome["state"] == "succeeded" and outcome["cleanup_state"] == "pending":
        pending += 1
        remaining = settlement_deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("The native staged cleanup did not settle within its original budget")
        time.sleep(min(0.01, remaining))
        outcome = budget.call(client.source_restore_status, preview["operation_id"])
    assert outcome["state"] == "succeeded" and outcome["write_outcome"] == "committed" and outcome["cleanup_state"] == "verified"
    receipt["cleanup_pending_observations"] = pending
    receipt["outcome"] = {name: outcome[name] for name in ("operation_id", "state", "write_outcome", "cleanup_state")}
    checked(receipt, "restore-normal-committed-cleanup-settlement")
    assert (recipient / "policies/access.md").read_bytes() == source and (recipient / "evidence/source.bin").read_bytes() == opaque
    assert json.loads((recipient / "forge.workspace.json").read_bytes())["resources"] == resources
    checked(receipt, "restored-exact-source-and-complete-index")
    assert before["files"]["local.md"] == pin(recipient / "local.md") and before["files"]["leave.bin"] == pin(recipient / "leave.bin")
    checked(receipt, "removed-registration-files-retained")
    cancelled = budget.call(client.cancel_source_restore, preview["operation_id"])
    assert cancelled == outcome
    checked(receipt, "terminal-cancel-preserves-committed-facts")
    return outcome


def readonly_checks(client, stage, declaration, preview, outcome, budget, receipt):
    """Read the durable same-root result while refusing old-session stage and fresh write authority."""
    assert budget.call(client.source_restore_status, preview["operation_id"]) == outcome
    checked(receipt, "fresh-read-only-known-outcome-lookup")
    client_refusal(budget, lambda: client.source_transfer_status(stage["stage_id"]), "not-found")
    checked(receipt, "fresh-read-only-stage-session-isolation")
    base = client.api_path("/project/source-transfer-stages")
    server_refusal(client, budget, "POST", base, declaration, "read-only-session", idempotency=key())
    server_refusal(client, budget, "PUT", base + "/" + stage["stage_id"] + "/chunks/0", {"sha256": sha(b"x"), "size_bytes": 1, "hex": "78"}, "read-only-session")
    server_refusal(client, budget, "DELETE", base + "/" + stage["stage_id"], {}, "read-only-session")
    server_refusal(client, budget, "POST", base + "/" + stage["stage_id"] + "/preview", {"target_index_schema_version": 2,
        "acknowledge_index_replacement": True, "acknowledge_source_content": True, "acknowledge_replace_files": True}, "read-only-session", idempotency=key())
    server_refusal(client, budget, "POST", client.api_path("/project/source-stream-exports"), {"target_path": "forbidden.json",
        "acknowledge_sensitive_metadata": True, "acknowledge_source_content": True}, "read-only-session", idempotency=key())
    client_refusal(budget, lambda: client.commit_source_restore(preview, confirmed=True, idempotency_key=key()), "read-only-session")
    client_refusal(budget, lambda: client.cancel_source_restore(preview["operation_id"]), "read-only-session")
    checked(receipt, "fresh-read-only-staged-and-confirmation-writes-refused")


def publish(out, receipt):
    """Publish one fresh private canonical receipt with no token, body, diff, or raw error text."""
    raw = (json.dumps(receipt, sort_keys=True, ensure_ascii=True, separators=(",", ":"), allow_nan=False) + "\n").encode("ascii")
    descriptor = os.open(out / "receipt.json", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "wb") as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())


def run(args):
    """Run only an explicitly supplied binary, four sequential owned sessions, and exact fixed checks."""
    if os.name != "posix" or not __debug__:
        raise RuntimeError("Native Unix staged controls require POSIX and active assertions")
    out = Path(args.out).resolve()
    out.mkdir(mode=0o700)
    binary = Path(args.forge).resolve(strict=True)
    script = Path(__file__).resolve()
    library = Path(workspace_client.__file__).resolve()
    if library.parent != script.parent:
        raise RuntimeError("Use the maintained same-directory client without alternate module fallback")
    sources = {"binary": pin(binary), "source_client": pin(library), "source_workflow": pin(script)}
    receipt = {"format": "forge.s6-real-staged-source-workflow/1", "status": "failed", "platform": sys.platform,
               "started_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "sources": sources,
               "checks": [], "expected_check_count": len(CHECKS), "normal_session_exits": 0,
               "source_stability": "unverified", "failure": None,
               "scope": "actual native Unix authenticated API2.4 only; no browser/Windows/power-loss/AT/release/human acceptance"}
    budget = Budget()
    try:
        with tempfile.TemporaryDirectory(prefix="forge-staged-donor-") as donor_name, tempfile.TemporaryDirectory(prefix="forge-staged-recipient-") as recipient_name:
            donor, recipient = Path(donor_name), Path(recipient_name)
            source, opaque, resources = fixtures(donor, recipient)
            donor_before, recipient_before = inventory(donor), inventory(recipient)
            with session(binary, donor, False, budget, receipt) as client:
                bundle, export_id = stream_export(client, donor, donor_before, budget, receipt)
                stream_refusals(client, export_id, budget, receipt)
                assert donor_before["files"] == {name: pin(donor / name) for name in donor_before["files"]}
                checked(receipt, "donor-original-registered-inputs-unchanged")
            with session(binary, recipient, False, budget, receipt) as client:
                stage, declaration = upload(client, bundle, budget, receipt)
                stage_refusals(client, stage, declaration, bundle, recipient, recipient_before, budget, receipt)
                preview = restore_preview(client, stage, bundle, recipient, recipient_before, resources, budget, receipt)
                outcome = restore_confirm(client, preview, recipient, recipient_before, source, opaque, resources, budget, receipt)
                recipient_after = inventory(recipient)
            with session(binary, recipient, True, budget, receipt) as client:
                readonly_checks(client, stage, declaration, preview, outcome, budget, receipt)
                assert inventory(recipient) == recipient_after
            with session(binary, recipient, False, budget, receipt) as client:
                assert budget.call(client.source_restore_status, preview["operation_id"]) == outcome
                client_refusal(budget, lambda: client.commit_source_restore(preview, confirmed=True, idempotency_key=key()), "not-found")
                assert inventory(recipient) == recipient_after
                checked(receipt, "fresh-writable-old-receipt-does-not-revive-authority")
            assert tuple(receipt["checks"]) == CHECKS and receipt["normal_session_exits"] == 4
        receipt["status"] = "passed"
    except BaseException as error:
        safe_type = type(error).__name__ if isinstance(error, (AssertionError, TimeoutError, WorkspaceError, ValueError, OSError, KeyError, TypeError, RuntimeError)) else "unexpected-failure"
        code = error.payload.get("code") if isinstance(error, WorkspaceError) else None
        receipt["failure"] = {"type": safe_type, "server_code": code if isinstance(code, str) and re.fullmatch(r"[a-z][a-z0-9-]{0,63}", code) else None}
        frames = []
        frame = error.__traceback__
        while frame is not None:
            frames.append({"file": Path(frame.tb_frame.f_code.co_filename).name,
                           "function": frame.tb_frame.f_code.co_name, "line": frame.tb_lineno})
            frame = frame.tb_next
        receipt["failure"]["locations"] = frames
        receipt["failure"]["next_check"] = CHECKS[len(receipt["checks"])] if len(receipt["checks"]) < len(CHECKS) else None
    finally:
        try:
            current = {"binary": pin(binary), "source_client": pin(library), "source_workflow": pin(script)}
            receipt["source_stability"] = "unchanged" if current == sources else "changed"
        except Exception:
            receipt["source_stability"] = "unverified"
        if receipt["source_stability"] != "unchanged":
            receipt["status"] = "failed"
        receipt["finished_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        receipt["check_count"] = len(receipt["checks"])
        receipt["unobserved_checks"] = [name for name in CHECKS if name not in receipt["checks"]]
        publish(out, receipt)
        print(json.dumps({"status": receipt["status"], "check_count": receipt["check_count"], "expected_check_count": len(CHECKS)}))
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
        print('{"status":"failed","failure":"workflow-or-receipt-unverified"}')
        raise SystemExit(1)
