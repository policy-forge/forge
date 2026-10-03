#!/usr/bin/env python3
"""Stdlib source-client controls; authored wire fixtures establish no native restore authority."""
import copy
import hashlib
import json
import stat
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
from workspace_client import MAX_SOURCE_BUNDLE, SourceRestoreUncertain, Workspace, WorkspaceError

OP = "op_" + "2" * 64
PREVIEW = "prev_" + "1" * 64
KEY = "source-client-private-key"
A = "a" * 64
B = "b" * 64


def index_bytes(value):
    """Build ordered synthetic index bytes with explicit newline for fixture hash facts only."""
    ordered = {"schema_version":value["schema_version"], "label":value["label"], "resources":value["resources"]}
    return (json.dumps(ordered, ensure_ascii=False, indent=2) + "\n").encode()


def prepared_response():
    """Create complete synthetic old/new membership with one overwrite and one dropped registration."""
    alpha = {"key":"alpha", "role":"policy-source", "path":"alpha.md"}
    beta = {"key":"beta", "role":"policy-source", "path":"beta.md"}
    old = {"schema_version":"forge.workspace/1", "label":"Synthetic private label", "resources":[alpha, beta]}
    new = {"schema_version":"forge.workspace/2", "label":"Synthetic private label", "resources":[alpha]}
    old_raw, new_raw = index_bytes(old), index_bytes(new)
    old_hash, new_hash = hashlib.sha256(old_raw).hexdigest(), hashlib.sha256(new_raw).hexdigest()
    validation = {"state":"valid", "error_count":0, "warning_count":0, "diagnostics":[]}
    targets = [
        {"path":"alpha.md", "kind":"resource", "key":"alpha", "role":"policy-source", "status":"overwrite",
         "base_sha256":A, "base_size":1, "target_version":A, "exact_bytes_sha256":B, "size":2,
         "diff_text":"- old\n+ new\n", "diff_truncated":False, "binary":False},
        {"path":"forge.workspace.json", "kind":"index", "key":None, "role":None, "status":"overwrite",
         "base_sha256":old_hash, "base_size":len(old_raw), "target_version":A, "exact_bytes_sha256":new_hash,
         "size":len(new_raw), "diff_text":"", "diff_truncated":False, "binary":False},
    ]
    bindings = [
        {"path":"forge.workspace.json", "kind":"index", "current":{"key":None, "role":None, "resource_id":None, "sha256":old_hash, "size":len(old_raw)},
         "proposed":{"key":None, "role":None, "sha256":new_hash, "size":len(new_raw)}},
        {"path":"alpha.md", "kind":"resource", "current":{"key":"alpha", "role":"policy-source", "resource_id":"res_" + "3" * 64, "sha256":A, "size":1},
         "proposed":{"key":"alpha", "role":"policy-source", "sha256":B, "size":2}},
        {"path":"beta.md", "kind":"resource", "current":{"key":"beta", "role":"policy-source", "resource_id":"res_" + "4" * 64, "sha256":A, "size":1}, "proposed":None},
    ]
    preview = {"preview_id":PREVIEW, "operation_id":OP, "operation_type":"project-source-restore", "snapshot_version":A,
               "observed_batch_version":B, "exact_manifest_sha256":A, "targets":targets, "input_bindings":bindings,
               "directories":[], "validation":validation, "semantic_summary":"Source bytes and metadata may be sensitive.",
               "receipt":{"token":"synthetic-private-receipt-token", "expires_at":"2026-10-03T00:00:00Z"}}
    return {"validation":copy.deepcopy(validation), "preview":preview,
            "replacement":{"previous_index":old, "proposed_index":new, "supplied_index_sha256":new_hash,
                           "proposed_index_sha256":new_hash, "removed_resource_keys":["beta"], "consumed_file_count":3}}


def response_for_index_version(version, *, previous_version=1):
    """Adapt every synthetic index target/base hash and size consistently to a selected version."""
    value = prepared_response()
    replacement, preview = value["replacement"], value["preview"]
    replacement["proposed_index"]["schema_version"] = "forge.workspace/" + str(version)
    replacement["previous_index"]["schema_version"] = "forge.workspace/" + str(previous_version)
    proposed, previous = index_bytes(replacement["proposed_index"]), index_bytes(replacement["previous_index"])
    new_hash, old_hash = hashlib.sha256(proposed).hexdigest(), hashlib.sha256(previous).hexdigest()
    replacement.update(supplied_index_sha256=new_hash, proposed_index_sha256=new_hash)
    preview["targets"][-1].update(exact_bytes_sha256=new_hash, size=len(proposed), base_sha256=old_hash, base_size=len(previous))
    preview["input_bindings"][0]["proposed"].update(sha256=new_hash, size=len(proposed))
    preview["input_bindings"][0]["current"].update(sha256=old_hash, size=len(previous))
    return value


def operation(state="running", *, cancel=False, cleanup="unmeasured", outcome="unmeasured"):
    """Build one separate batch DTO; only chosen synthetic observations are represented."""
    return {"operation_id":OP, "kind":"bundle-restore", "state":state, "created_at":"2026-10-03T00:00:00Z",
            "updated_at":"2026-10-03T00:00:01Z", "cancel_requested":cancel, "write_outcome":outcome,
            "progress":None, "result":None, "error":None, "cleanup_state":cleanup}


def succeeded(cleanup="verified"):
    """Return complete committed target records with independent cleanup state."""
    value = operation("succeeded", cleanup=cleanup, outcome="committed")
    preview = prepared_response()["preview"]
    value["result"] = {"write_committed":True, "exact_manifest_sha256":A, "cleanup_state":cleanup,
                       "committed_targets":[{"path":row["path"], "sha256":row["exact_bytes_sha256"], "size":row["size"]} for row in preview["targets"]]}
    return value


def rejected(state="failed", *, outcome="none", cleanup="verified"):
    """Represent safe rollback/recovery error facts without a receipt or private exception text."""
    value = operation(state, cleanup=cleanup, outcome=outcome, cancel=state == "cancelled")
    value["error"] = {"code":"bundle-restore-recovery-required", "message":"Query the retained outcome before continuing.", "retryable":False}
    return value


class SourceClientTests(unittest.TestCase):
    """Exercise the maintained transport and complete source wrappers without spawning a product."""
    def setUp(self):
        """Create an isolated explicit2.3 client and a bounded fake loopback HTTP seam."""
        self.client = Workspace.__new__(Workspace)
        self.client._api_major, self.client._contract_version, self.client._api_prefix = 2, "2.3.0", "/api/v2"
        self.client._port, self.client._capability, self.client._next_request_at = 1, "synthetic-private-capability", 0
        self.wire, self.closed = [], []
        self.payload, self.status, self.media = json.dumps(prepared_response()).encode(), 200, "application/json; charset=utf-8"
        self.error = None
        owner = self

        class Response:
            """Expose only authored bytes and metadata through the real response decoder."""
            @property
            def status(self):
                """Return the current control status without inventing native work."""
                return owner.status

            def read(self, bound):
                """Keep the caller read cap observable, including one excess byte."""
                return owner.payload[:bound]

            def getheader(self, name, default=""):
                """Supply only the chosen Content-Type observation."""
                return owner.media if name.lower() == "content-type" else default

        class Connection:
            """Record the actual maintained connection call and always account for close."""
            def __init__(self, host, port, timeout):
                """Reject any transport change away from fixed loopback and30-second timeout."""
                owner.assertEqual((host, port, timeout), ("127.0.0.1", 1, 30))

            def request(self, method, path, body, headers):
                """Retain exact private wire bytes and header names for assertions only."""
                owner.wire.append((method, path, body, headers))

            def getresponse(self):
                """Fail or return only this control observation after request accounting."""
                if owner.error is not None:
                    raise owner.error
                return Response()

            def close(self):
                """Count owned cleanup on success, rejection and uncertain response."""
                owner.closed.append(True)

        transport = patch("workspace_client.http.client.HTTPConnection", Connection)
        transport.start()
        self.addCleanup(transport.stop)
        pacing = patch("workspace_client.time.sleep")
        pacing.start()
        self.addCleanup(pacing.stop)

    def import_bytes(self, raw=b"{}", **changed):
        """Call the complete raw import wrapper with explicit replace/source acknowledgments."""
        fields = {"target_index_schema_version":2, "acknowledge_index_replacement":True,
                  "acknowledge_source_content":True, "acknowledge_replace_files":True, "idempotency_key":KEY}
        fields.update(changed)
        return self.client.prepare_source_bundle_import(raw, **fields)

    def set_response(self, value):
        """Serialize a synthetic finite observation without changing client production behavior."""
        self.payload = json.dumps(value, ensure_ascii=False, allow_nan=False).encode()

    def test_raw_duplicate_unicode_and_bom_spelling_is_not_collapsed(self):
        """Preserve selected raw bytes even when server admission will reject the envelope."""
        raw = '\ufeff{"key":"π","key":"duplicate"}'.encode()
        self.import_bytes(raw)
        method, path, body, headers = self.wire[0]
        suffix = b',"target_index_schema_version":2,"acknowledge_index_replacement":true,"acknowledge_source_content":true,"acknowledge_replace_files":true}'
        self.assertEqual((method, path, body), ("POST", "/api/v2/project/source-bundle-imports", b'{"bundle":' + raw + suffix))
        self.assertEqual(len(body) - len(raw), 147)
        self.assertEqual(headers["Idempotency-Key"], KEY)
        self.assertEqual(len(self.closed), 1)

    def test_exact_artifact_and_wrapper_boundary_then_one_excess(self):
        """Admit exactly1MiB raw envelope and refuse one extra artifact byte before HTTP."""
        self.import_bytes(b" " * MAX_SOURCE_BUNDLE)
        self.assertEqual(len(self.wire[0][2]), 1048576)
        with self.assertRaises(ValueError):
            self.import_bytes(b" " * (MAX_SOURCE_BUNDLE + 1))
        self.assertEqual(len(self.wire), 1)

    def test_every_ack_requires_literal_true_and_selector_requires_exact_int(self):
        """Refuse partial or truthy confirmation substitutes without an effect request."""
        for name in ("acknowledge_index_replacement", "acknowledge_source_content", "acknowledge_replace_files"):
            for value in (False, 1, "true", None):
                with self.subTest(name=name, value=value), self.assertRaises(ValueError):
                    self.import_bytes(**{name:value})
        for value in (True, "2", 0, 3):
            with self.subTest(selector=value), self.assertRaises(ValueError):
                self.import_bytes(target_index_schema_version=value)
        self.assertFalse(self.wire)

    def test_new_idempotency_headers_are_bounded_and_exact(self):
        """Refuse missing, nonstring, short, oversized and newline keys before dispatch."""
        for value in (None, 42, "short", "x" * 256, "x" * 16 + "\n"):
            with self.subTest(key_type=type(value).__name__), self.assertRaises(ValueError):
                self.import_bytes(idempotency_key=value)
        self.assertFalse(self.wire)

    def test_source_gate_is_exact23_and_other_feature_gates_remain_supported(self):
        """New source methods require2.3 while currentS3 and metadata remain available there."""
        for version in ("1.2.0", "2.0.0", "2.1.0", "2.2.0", "2.12.3"):
            self.client._contract_version = version
            with self.subTest(version=version), self.assertRaises(ValueError):
                self.import_bytes()
        self.client._contract_version = "2.3.0"
        self.client.lifecycle_records()
        self.client.prepare_bundle_import(b"{}", target_index_schema_version=1, acknowledge_index_replacement=True, idempotency_key=KEY)
        self.assertEqual([row[1] for row in self.wire], ["/api/v2/lifecycle/records?page_size=50", "/api/v2/project/bundle-imports"])

    def test_api1_namespace_does_not_gain_source_authority(self):
        """Preserve explicit major selection instead of translating legacy capabilities."""
        self.client._api_major, self.client._api_prefix, self.client._contract_version = 1, "/api/v1", "1.2.0"
        with self.assertRaises(ValueError):
            self.import_bytes()
        self.assertFalse(self.wire)

    def test_complete_preview_and_replacement_are_returned_without_commit(self):
        """Expose all binding and removal rows plus preknown lookup but prepare no accepted intent."""
        response = self.import_bytes()
        self.assertEqual(response, prepared_response())
        self.assertEqual(response["preview"]["operation_id"], OP)
        self.assertEqual(len(response["preview"]["targets"]), 2)
        self.assertEqual(len(response["preview"]["input_bindings"]), 3)
        self.assertEqual(response["replacement"]["removed_resource_keys"], ["beta"])
        self.assertEqual(len(self.wire), 1)
        self.assertNotIn("commit", self.wire[0][1])

    def test_missing_preknown_id_and_unknown_preview_fields_refuse_confirmation(self):
        """Malformed preview DTOs cannot reach the commit transport."""
        for changed in ("missing", "extra"):
            value = prepared_response()["preview"]
            if changed == "missing":
                del value["operation_id"]
            else:
                value["private_debug"] = "PRIVATE_SENTINEL"
            with self.subTest(change=changed), self.assertRaises(RuntimeError):
                self.client.commit_source_restore(value, confirmed=True, idempotency_key=KEY)
        self.assertFalse(self.wire)

    def test_full_target_binding_hash_size_key_role_and_base_relation(self):
        """Every target byte fact and current base must reconcile before a write request."""
        mutations = (("sha256", A), ("size", 9), ("key", "other"), ("role", "lifecycle-source"))
        for field, value in mutations:
            preview = prepared_response()["preview"]
            preview["input_bindings"][1]["proposed"][field] = value
            with self.subTest(field=field), self.assertRaises(RuntimeError):
                self.client.commit_source_restore(preview, confirmed=True, idempotency_key=KEY)
        preview = prepared_response()["preview"]
        preview["input_bindings"][1]["current"]["sha256"] = B
        with self.assertRaises(RuntimeError):
            self.client.commit_source_restore(preview, confirmed=True, idempotency_key=KEY)
        self.assertFalse(self.wire)

    def test_index_last_and_complete_binding_membership_are_required(self):
        """Partial target/binding lists or reordered index cannot be treated as full preview."""
        for change in ("order", "binding", "duplicate"):
            value = prepared_response()["preview"]
            if change == "order":
                value["targets"].reverse()
            elif change == "binding":
                value["input_bindings"] = value["input_bindings"][:1]
            else:
                value["targets"].append(copy.deepcopy(value["targets"][0]))
            with self.subTest(change=change), self.assertRaises(RuntimeError):
                self.client.commit_source_restore(value, confirmed=True, idempotency_key=KEY)
        self.assertFalse(self.wire)

    def test_shared_diff_budget_counts_utf8_bytes_across_every_target(self):
        """Exactly200k UTF8 diff bytes pass while one additional byte refuses before confirmation."""
        preview = prepared_response()["preview"]
        preview["targets"][0]["diff_text"] = "π" * 100000
        Workspace._source_preview(preview)
        preview["targets"][1]["diff_text"] = "x"
        with self.assertRaises(RuntimeError):
            Workspace._source_preview(preview)

    def test_complete_replacement_indices_removals_hash_and_count_are_reconciled(self):
        """Reject malformed full indexes and a plausible but incomplete replacement projection."""
        for change in ("index", "removed", "hash", "count", "member", "label"):
            value = prepared_response()
            replacement = value["replacement"]
            if change == "index":
                replacement["proposed_index"]["private"] = "PRIVATE_SENTINEL"
            elif change == "removed":
                replacement["removed_resource_keys"] = []
            elif change == "hash":
                replacement["proposed_index_sha256"] = A
            elif change == "count":
                replacement["consumed_file_count"] = 2
            elif change == "member":
                replacement["proposed_index"]["resources"] = []
            else:
                replacement["proposed_index"]["label"] = "Different complete index"
            self.set_response(value)
            with self.subTest(change=change), self.assertRaises(RuntimeError):
                self.import_bytes()

    def test_selected_version_matches_full_projection_and_never_downgrades(self):
        """Consistent hashes cannot replace selector2 with1 or downgrade a captured index2."""
        for requested, proposed, previous in ((2, 1, 1), (1, 2, 1), (1, 1, 2)):
            self.set_response(response_for_index_version(proposed, previous_version=previous))
            with self.subTest(requested=requested, proposed=proposed, previous=previous), self.assertRaises(RuntimeError):
                self.import_bytes(target_index_schema_version=requested)
        self.set_response(response_for_index_version(1))
        response = self.import_bytes(target_index_schema_version=1)
        self.assertEqual(response["replacement"]["proposed_index"]["schema_version"], "forge.workspace/1")

    def test_unregistered_existing_base_has_explicit_null_registration(self):
        """Real base facts without prior registration remain displayable, without fabricating a resource."""
        value = prepared_response()
        value["replacement"]["previous_index"]["resources"] = [value["replacement"]["previous_index"]["resources"][1]]
        value["replacement"]["removed_resource_keys"] = ["beta"]
        prior = value["preview"]["input_bindings"][1]["current"]
        prior.update(key=None, role=None, resource_id=None)
        self.set_response(value)
        response = self.import_bytes()
        self.assertIsNone(response["preview"]["input_bindings"][1]["current"]["resource_id"])
        self.assertIsNone(response["preview"]["input_bindings"][1]["current"]["key"])

    def test_portable_alias_device_and_whole_count_bounds_are_refused(self):
        """Response lexical checks cannot silently confer portable path or over-cap meaning."""
        for path in ("../private", "/private", "a/CON.txt", "dir/x."):
            with self.subTest(path=path), self.assertRaises(RuntimeError):
                Workspace._source_path(path)
        value = prepared_response()["preview"]
        value["targets"] = value["targets"] * 51
        with self.assertRaises(RuntimeError):
            Workspace._source_preview(value)

    def test_directory_intents_are_closed_unique_and_parent_first(self):
        """Complete mkdir plan facts are visible; reverse parent order refuses before commit."""
        preview = prepared_response()["preview"]
        preview["directories"] = [{"path":"parent", "status":"create", "nearest_existing_parent_version":A},
                                  {"path":"parent/child", "status":"create", "nearest_existing_parent_version":B}]
        Workspace._source_preview(preview)
        preview["directories"].reverse()
        with self.assertRaises(RuntimeError):
            Workspace._source_preview(preview)

    def test_complete_confirmation_sends_one_bound_commit(self):
        """One POST binds receipt and observed batch to the preview retained operation ID."""
        preview = prepared_response()["preview"]
        self.set_response(operation())
        result = self.client.commit_source_restore(preview, confirmed=True, idempotency_key=KEY)
        self.assertEqual(result["operation_id"], OP)
        self.assertEqual(len(self.wire), 1)
        self.assertEqual(self.wire[0][:2], ("POST", "/api/v2/project/bundle-restores/" + PREVIEW + "/commit"))
        self.assertEqual(json.loads(self.wire[0][2]), {"receipt":preview["receipt"]["token"], "observed_batch_version":B, "acknowledge_exact_restore":True})

    def test_false_confirmation_and_invalid_validation_never_dispatch(self):
        """Only literal confirmation of a valid whole plan can send the commit request."""
        for confirmed in (False, 1, "true"):
            with self.subTest(confirmed=confirmed), self.assertRaises(ValueError):
                self.client.commit_source_restore(prepared_response()["preview"], confirmed=confirmed, idempotency_key=KEY)
        value = prepared_response()["preview"]
        value["validation"]["state"], value["validation"]["error_count"] = "invalid", 1
        with self.assertRaises(ValueError):
            self.client.commit_source_restore(value, confirmed=True, idempotency_key=KEY)
        self.assertFalse(self.wire)

    def test_lost_reply_retains_lookup_and_does_not_resend_or_auto_query(self):
        """An actual transport exception after one POST keeps only nonauthorizing ID for later recovery."""
        self.error = OSError("PRIVATE_SENTINEL token path")
        with self.assertRaises(SourceRestoreUncertain) as caught:
            self.client.commit_source_restore(prepared_response()["preview"], confirmed=True, idempotency_key=KEY)
        self.assertEqual(caught.exception.operation_id, OP)
        self.assertNotIn("PRIVATE_SENTINEL", str(caught.exception))
        self.assertEqual(len(self.wire), 1)
        self.assertEqual(len(self.closed), 1)
        self.assertNotIn("receipt", caught.exception.__dict__)

    def test_typed_commit_rejection_preserves_error_and_never_resends(self):
        """Exact409/422 failures remain typed; neither becomes invented acceptance or an automatic retry."""
        for status, code in ((409, "version-conflict"), (422, "validation-failed")):
            self.status = status
            self.set_response({"code":code, "message":"Review the retained plan.", "retryable":False})
            with self.subTest(status=status), self.assertRaises(WorkspaceError) as caught:
                self.client.commit_source_restore(prepared_response()["preview"], confirmed=True, idempotency_key=KEY)
            self.assertEqual(caught.exception.payload["code"], code)
        self.assertEqual(len(self.wire), 2)
        self.assertEqual(len(self.closed), 2)

    def test_wrong_reply_id_is_uncertain_without_repeating_the_write(self):
        """A successful-looking reply for another operation cannot replace the preview lookup."""
        value = operation()
        value["operation_id"] = "op_" + "9" * 64
        self.set_response(value)
        with self.assertRaises(SourceRestoreUncertain) as caught:
            self.client.commit_source_restore(prepared_response()["preview"], confirmed=True, idempotency_key=KEY)
        self.assertEqual(caught.exception.operation_id, OP)
        self.assertEqual(len(self.wire), 1)

    def test_fresh_session_recovers_only_by_known_id_read(self):
        """A newly negotiated synthetic session can query known outcome without old receipt authority."""
        fresh = Workspace.__new__(Workspace)
        fresh.__dict__.update(self.client.__dict__)
        fresh._capability = "different-private-capability"
        self.set_response(succeeded())
        result = fresh.source_restore_status(OP)
        self.assertEqual(result["write_outcome"], "committed")
        self.assertEqual(self.wire[0][:3], ("GET", "/api/v2/project/bundle-restores/" + OP, None))
        self.assertNotIn("Idempotency-Key", self.wire[0][3])
        self.assertNotIn("receipt", self.wire[0][3])

    def test_not_found_never_means_no_write_or_causes_post(self):
        """Typed404 propagates; status lookup neither fabricates rollback nor resends confirmation."""
        self.status = 404
        self.set_response({"code":"not-found", "message":"No retained outcome.", "retryable":False})
        with self.assertRaises(WorkspaceError) as caught:
            self.client.source_restore_status(OP)
        self.assertEqual(caught.exception.payload["code"], "not-found")
        self.assertEqual([row[0] for row in self.wire], ["GET"])

    def test_cancel_acknowledgment_is_running_and_uses_existing_empty_post_convention(self):
        """Cancel request uses no key and no semantic payload; acknowledgment is not terminal rollback."""
        self.set_response(operation(cancel=True))
        result = self.client.cancel_source_restore(OP)
        self.assertEqual(result["state"], "running")
        self.assertTrue(result["cancel_requested"])
        self.assertEqual(self.wire[0][:3], ("POST", "/api/v2/project/bundle-restores/" + OP + "/cancel", b"{}"))
        self.assertNotIn("Idempotency-Key", self.wire[0][3])

    def test_late_cancel_typed_conflict_is_not_disguised_as_cancelled(self):
        """A terminal cancellation conflict retains the server typed failure and no invented success."""
        self.status = 409
        self.set_response({"code":"operation-not-cancellable", "message":"The operation is terminal.", "retryable":False})
        with self.assertRaises(WorkspaceError):
            self.client.cancel_source_restore(OP)
        self.assertEqual(len(self.wire), 1)

    def test_verified_rollback_unknown_recovery_and_known_commit_cleanup_stay_distinct(self):
        """Closed batch outcomes keep write evidence separate from pending/unverified cleanup."""
        for value in (rejected("cancelled"), rejected("recovery-required", outcome="unknown", cleanup="unverified"), succeeded("pending")):
            self.set_response(value)
            self.assertEqual(self.client.source_restore_status(OP), value)
        recovery = rejected("recovery-required", outcome="committed", cleanup="unverified")
        recovery["result"] = succeeded("unverified")["result"]
        self.set_response(recovery)
        self.assertEqual(self.client.source_restore_status(OP)["write_outcome"], "committed")

    def test_failed_or_cancelled_cannot_claim_unverified_rollback(self):
        """Unsafe none outcome with unverified cleanup and a terminal progress counter refuse."""
        for value in (rejected("failed", cleanup="unverified"), rejected("cancelled", outcome="unknown")):
            self.set_response(value)
            with self.assertRaises(RuntimeError):
                self.client.source_restore_status(OP)
        value = succeeded()
        value["progress"] = {"completed_files":2, "total_files":2}
        self.set_response(value)
        with self.assertRaises(RuntimeError):
            self.client.source_restore_status(OP)

    def test_committed_result_cannot_use_unmeasured_cleanup_even_if_top_level_agrees(self):
        """A matching invalid cleanup pair is still outside the closed committed-result enum."""
        self.set_response(succeeded("unmeasured"))
        with self.assertRaises(RuntimeError):
            self.client.source_restore_status(OP)

    def test_polling_status_only_has_one_budget_and_no_restart(self):
        """Expiry preserves lookup; only GET-like status method is called, never commit/cancel."""
        with patch("workspace_client.time.monotonic", side_effect=[0, 0, 1, 2]), patch.object(self.client, "source_restore_status", return_value=operation()) as read:
            with self.assertRaises(SourceRestoreUncertain) as caught:
                self.client.wait_source_restore(OP, timeout=1)
        self.assertEqual(caught.exception.operation_id, OP)
        self.assertEqual(read.call_args.args, (OP,))
        self.assertFalse(self.wire)
        for value in (0, True, float("nan"), float("inf"), 121):
            with self.subTest(timeout=str(value)), self.assertRaises(ValueError):
                self.client.wait_source_restore(OP, timeout=value)

    def test_opt_in_export_returns_only_a_single_file_operation(self):
        """Source output acknowledges sensitivity/content explicitly without invoking any confirmation."""
        self.set_response({"operation_id":OP, "kind":"report-export", "state":"pending"})
        result = self.client.prepare_source_bundle_export("source.json", acknowledge_sensitive_metadata=True,
                    acknowledge_source_content=True, idempotency_key=KEY)
        self.assertEqual(result["kind"], "report-export")
        self.assertEqual(json.loads(self.wire[0][2]), {"target_path":"source.json", "acknowledge_sensitive_metadata":True, "acknowledge_source_content":True})
        with self.assertRaises(ValueError):
            self.client.prepare_source_bundle_export("source.json", acknowledge_sensitive_metadata=True,
                        acknowledge_source_content=False, idempotency_key=KEY)
        self.assertEqual(len(self.wire), 1)

    def test_source_download_requires_exact_family_media_hash_and_artifact_bound(self):
        """Return only committed exact bytes; media, digest and finite reimport size all remain required."""
        raw = b'{"schema_version":"forge.workspace-index-bundle/3"}'
        self.payload = raw
        self.assertEqual(self.client.download_source_bundle_export(OP, expected_sha256=hashlib.sha256(raw).hexdigest()), raw)
        with self.assertRaises(RuntimeError):
            self.client.download_source_bundle_export(OP, expected_sha256=A)
        self.media = "text/html"
        with self.assertRaises(RuntimeError):
            self.client.download_source_bundle_export(OP, expected_sha256=hashlib.sha256(raw).hexdigest())
        self.media, self.payload = "application/json", b"x" * (MAX_SOURCE_BUNDLE + 1)
        with self.assertRaises(RuntimeError):
            self.client.download_source_bundle_export(OP, expected_sha256=hashlib.sha256(self.payload).hexdigest())

    def test_narrow_raw_and_download_transport_do_not_gain_other_routes(self):
        """New raw modes never authorize unrelated methods/paths or source effects under2.2."""
        for path in ("/api/v2/project/source-bundle-exports", "/api/v2/session", "/api/v1/project/source-bundle-imports"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                self.client.request("POST", path, raw_json_body=b"{}", idempotency_key=KEY)
        self.client._contract_version = "2.2.0"
        with self.assertRaises(ValueError):
            self.client.request("POST", "/api/v2/project/source-bundle-imports", raw_json_body=b"{}", idempotency_key=KEY)
        self.assertFalse(self.wire)

    def test_selected_regular_local_file_retains_exact_bytes_and_observed_stability(self):
        """Bounded selected-file input reaches the same raw envelope without normalizing its contents."""
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "bundle.json"
            path.write_bytes(b" {\"key\":1,\"key\":2} ")
            self.client.prepare_source_bundle_import_file(path, target_index_schema_version=2,
                acknowledge_index_replacement=True, acknowledge_source_content=True,
                acknowledge_replace_files=True, idempotency_key=KEY)
        self.assertTrue(self.wire[0][2].startswith(b'{"bundle": {"key":1,"key":2} '))

    def test_verified_native_cancelled_outcome_needs_no_synthetic_error(self):
        """The actual native cancellation shape keeps verified-none with nullable error, not a fabricated failure."""
        value = operation("cancelled", cancel=True, outcome="none", cleanup="verified")
        self.set_response(value)
        self.assertEqual(self.client.source_restore_status(OP), value)
        value["cleanup_state"] = "unverified"
        self.set_response(value)
        with self.assertRaises(RuntimeError):
            self.client.source_restore_status(OP)
        for state in ("failed", "recovery-required"):
            value = operation(state, outcome="none", cleanup="verified")
            self.set_response(value)
            with self.subTest(state=state), self.assertRaises(RuntimeError):
                self.client.source_restore_status(OP)

    def test_local_file_changed_nonregular_oversize_and_io_error_refuse_before_http(self):
        """Observed file faults fail locally without private path/error text or source HTTP dispatch."""
        regular = SimpleNamespace(st_mode=stat.S_IFREG, st_dev=1, st_ino=2, st_size=2, st_mtime_ns=3)
        after = SimpleNamespace(**regular.__dict__)
        after.st_mtime_ns = 4
        observations = ((regular, after), (SimpleNamespace(**{**regular.__dict__, "st_mode":stat.S_IFIFO}), regular),
                        (SimpleNamespace(**{**regular.__dict__, "st_size":MAX_SOURCE_BUNDLE + 1}), regular))
        for before, later in observations:
            stream = unittest.mock.MagicMock()
            stream.__enter__.return_value = stream
            stream.read.return_value = b"{}"
            stream.fileno.return_value = 123
            with patch("workspace_client.os.open", return_value=123), patch("workspace_client.os.fdopen", return_value=stream), patch("workspace_client.os.close"), patch("workspace_client.os.fstat", side_effect=[before, later]), self.assertRaises(ValueError):
                self.client.prepare_source_bundle_import_file("PRIVATE_PATH", target_index_schema_version=2,
                    acknowledge_index_replacement=True, acknowledge_source_content=True,
                    acknowledge_replace_files=True, idempotency_key=KEY)
        with patch("workspace_client.os.open", side_effect=OSError("PRIVATE_PATH PRIVATE_SENTINEL")), self.assertRaises(ValueError) as caught:
            self.client.prepare_source_bundle_import_file("PRIVATE_PATH")
        self.assertNotIn("PRIVATE", str(caught.exception))
        self.assertFalse(self.wire)


if __name__ == "__main__":
    unittest.main()
