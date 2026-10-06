#!/usr/bin/env python3
"""Stdlib staged-client proposals; synthetic transport observations grant no native restore authority."""
import copy
import hashlib
import json
import stat
import tempfile
import unittest
from collections import deque
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
from workspace_client import (MAX_SOURCE_BUNDLE, MAX_STAGED_SOURCE_BUNDLE, MAX_STAGED_PARTS,
                              STAGED_PART_BYTES, Workspace, WorkspaceError)
from test_workspace_source_bundle_client import index_bytes, prepared_response

STAGE = "bst_0123456789abcdef"
OP = "op_" + "2" * 64
KEY = "staged-client-private-key"
EXPIRES = "2026-10-03T00:00:00Z"


def bundle_value(payload=b"\x00\xff\r\n", *, version=2, resources=1):
    """Build opaque source fixtures with exact ordered index, pin and content facts, never domain approval."""
    rows = [{"key":"policy-" + str(i), "role":"policy-source", "path":"policy-" + str(i) + ".md"}
            for i in range(resources)]
    index = {"schema_version":"forge.workspace/" + str(version), "label":"Synthetic staged π", "resources":rows}
    return {"schema_version":"forge.workspace-index-bundle/4", "profile":"index-and-source-hex-staged",
            "source_content_included":True, "index":index, "index_sha256":hashlib.sha256(index_bytes(index)).hexdigest(),
            "pins":[{"key":row["key"], "sha256":hashlib.sha256(payload).hexdigest(), "size":len(payload)} for row in rows],
            "contents":[{"key":row["key"], "encoding":"hex", "chunks":[payload[n:n + STAGED_PART_BYTES].hex()
                         for n in range(0, len(payload), STAGED_PART_BYTES)]} for row in rows]}


def encoded(value):
    """Encode fixture JSON without discarding original authorial whitespace or Unicode on later dispatch."""
    return (json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n").encode()


def declaration(raw):
    """Derive one full artifact identity independently from the production create helper."""
    return {"schema_version":"forge.workspace-index-bundle/4", "profile":"index-and-source-hex-staged",
            "artifact_sha256":hashlib.sha256(raw).hexdigest(), "artifact_size_bytes":len(raw),
            "chunk_size_bytes":STAGED_PART_BYTES, "chunk_count":(len(raw) + STAGED_PART_BYTES - 1) // STAGED_PART_BYTES}


def three_part_raw():
    """Retain valid original JSON plus whitespace at65537 bytes: two full32KiB parts and one byte."""
    raw = encoded(bundle_value())
    return raw + b" " * (STAGED_PART_BYTES * 2 + 1 - len(raw))


def stage_value(raw, *, state="receiving", received=0):
    """Represent actual chosen transport counters; readiness does not stand in for native admission."""
    return {"stage_id":STAGE, **declaration(raw), "received_chunk_count":received,
            "received_bytes":min(len(raw), received * STAGED_PART_BYTES), "state":state, "expires_at":EXPIRES}


def ack_value(raw, ordinal, *, received=None):
    """Bind an original ordinal acknowledgment to sent bytes and unchanged expiry, including old replay counters."""
    part = raw[ordinal * STAGED_PART_BYTES:(ordinal + 1) * STAGED_PART_BYTES]
    count = ordinal + 1 if received is None else received
    return {"stage_id":STAGE, "chunk_ordinal":ordinal, "sha256":hashlib.sha256(part).hexdigest(),
            "size_bytes":len(part), "received_chunk_count":count,
            "received_bytes":min(len(raw), count * STAGED_PART_BYTES), "expires_at":EXPIRES}


def manifest_value(raw):
    """Describe a synthetic committed-family artifact using exact complete bytes, not preparation authority."""
    return {"operation_id":OP, **declaration(raw), "source_content_included":True}


def stream_value(raw, ordinal):
    """Return an independent exact part DTO with both full-artifact and decoded-part identities."""
    part = raw[ordinal * STAGED_PART_BYTES:(ordinal + 1) * STAGED_PART_BYTES]
    return {"operation_id":OP, "artifact_sha256":hashlib.sha256(raw).hexdigest(), "chunk_ordinal":ordinal,
            "sha256":hashlib.sha256(part).hexdigest(), "size_bytes":len(part), "hex":part.hex()}


class Clock:
    """Expose deterministic cooperative-deadline boundaries without advancing native time or doing IO."""
    def __init__(self, value=0):
        """Retain the exact synthetic monotonic value selected by a control."""
        self.value = value

    def __call__(self):
        """Read the selected time; a response callback can move it to an exact deadline."""
        return self.value


class StagedClientTests(unittest.TestCase):
    """Observe actual maintained request routes, raw bodies, cleanup and decoder refusal with fake HTTP only."""
    def setUp(self):
        """Select exact2.4 and replace only the HTTP adapter and pacing sleep, never the request method."""
        self.client = Workspace.__new__(Workspace)
        self.client._api_major, self.client._contract_version, self.client._api_prefix = 2, "2.4.0", "/api/v2"
        self.client._port, self.client._capability, self.client._next_request_at = 1, "synthetic-private-capability", 0
        self.wire, self.closed, self.read_bounds = [], [], []
        self.responses = deque()
        self.on_response = None
        self.raw = encoded(bundle_value(bytes(range(256)) * 67))
        self.assertGreater(len(self.raw), STAGED_PART_BYTES)
        self.assertLess(len(self.raw), STAGED_PART_BYTES * 2)
        owner = self

        class Response:
            """Expose only this authored payload, status and media through the real response decoder."""
            def __init__(self, value):
                """Hold the chosen raw payload and closed fake response metadata."""
                self.payload, self.status, self.media = value

            def read(self, bound):
                """Record the actual response cap and return at most the requested bytes."""
                owner.read_bounds.append(bound)
                return self.payload[:bound]

            def getheader(self, name, default=""):
                """Return only a deliberately chosen Content-Type value."""
                return self.media if name.lower() == "content-type" else default

        class Connection:
            """Record real transport invocation and close on decoded failure as well as success."""
            def __init__(self, host, port, timeout):
                """Fence the unchanged loopback and30-second connection parameters."""
                owner.assertEqual((host, port, timeout), ("127.0.0.1", 1, 30))

            def request(self, method, path, body, headers):
                """Capture the actual method, selected prefix, exact body and private headers for assertions."""
                owner.wire.append((method, path, body, headers))

            def getresponse(self):
                """Return one queued observation or fail once after dispatch without any automatic replay."""
                if not owner.responses:
                    raise AssertionError("Unexpected extra transport request")
                value = owner.responses.popleft()
                if owner.on_response is not None:
                    owner.on_response()
                if isinstance(value, Exception):
                    raise value
                return Response(value)

            def close(self):
                """Account for the original owned connection cleanup independently of business outcome."""
                owner.closed.append(True)

        connection = patch("workspace_client.http.client.HTTPConnection", Connection)
        connection.start()
        self.addCleanup(connection.stop)
        sleep = patch("workspace_client.time.sleep")
        sleep.start()
        self.addCleanup(sleep.stop)

    def queue(self, *values, status=200, media="application/json; charset=utf-8"):
        """Queue exact synthetic outcomes; no queue is a useful assertion against unexpected extra effects."""
        for value in values:
            if isinstance(value, Exception):
                self.responses.append(value)
            else:
                self.responses.append((value if type(value) is bytes else encoded(value), status, media))

    def create(self, raw=None, **changed):
        """Call the public create helper with all explicit sensitive-content acknowledgments."""
        fields = {"acknowledge_sensitive_metadata":True, "acknowledge_source_content":True, "idempotency_key":KEY}
        fields.update(changed)
        return self.client.create_source_transfer_stage(self.raw if raw is None else raw, **fields)

    def prepare(self, **changed):
        """Call staged native preparation with explicit complete replacement intent and no confirmation."""
        fields = {"target_index_schema_version":2, "acknowledge_index_replacement":True,
                  "acknowledge_source_content":True, "acknowledge_replace_files":True, "idempotency_key":KEY}
        fields.update(changed)
        return self.client.prepare_staged_source_restore(STAGE, **fields)

    def assert_refused_bundle(self, value):
        """Require an invalid codec fixture to fail before any transport or local authority is created."""
        before = len(self.wire)
        with self.assertRaises((ValueError, RuntimeError)):
            self.create(encoded(value))
        self.assertEqual(len(self.wire), before)

    def test_exact24_opt_in_and_no_api1_translation(self):
        """Staged creation rejects other version spellings and API1 before paced HTTP."""
        for version in ("1.2.0", "2.0.0", "2.1.0", "2.2.0", "2.3.0", "2.4.1", "2.40.0"):
            self.client._contract_version = version
            with self.subTest(version=version), self.assertRaises(ValueError):
                self.create()
        self.client._api_major, self.client._contract_version, self.client._api_prefix = 1, "2.4.0", "/api/v1"
        with self.assertRaises(ValueError):
            self.create()
        self.assertFalse(self.wire)

    def test_24_preserves_s3_metadata_and_inline_source_routes(self):
        """Version2.4 admits prior features without changing their selected paths or raw-byte envelopes."""
        self.queue({}, {}, prepared_response())
        self.client.lifecycle_records(as_of="2026-10-03")
        self.client.prepare_bundle_import(b"{}", target_index_schema_version=1, acknowledge_index_replacement=True, idempotency_key=KEY)
        self.client.prepare_source_bundle_import(b"{}", target_index_schema_version=2, acknowledge_index_replacement=True,
                                               acknowledge_source_content=True, acknowledge_replace_files=True, idempotency_key=KEY)
        self.assertEqual([row[1] for row in self.wire], ["/api/v2/lifecycle/records?as_of=2026-10-03&page_size=50",
                          "/api/v2/project/bundle-imports", "/api/v2/project/source-bundle-imports"])
        self.assertEqual(len(self.wire[-1][2]) - 2, 147)
        self.assertEqual(len(self.closed), 3)

    def test_inline_bundle3_cap_is_not_promoted_by24(self):
        """The old source envelope still stops at1048429 artifact bytes and one excess never dispatches."""
        self.queue(prepared_response())
        fields = {"target_index_schema_version":2, "acknowledge_index_replacement":True,
                  "acknowledge_source_content":True, "acknowledge_replace_files":True, "idempotency_key":KEY}
        self.client.prepare_source_bundle_import(b" " * MAX_SOURCE_BUNDLE, **fields)
        self.assertEqual(len(self.wire[0][2]), 1048576)
        with self.assertRaises(ValueError):
            self.client.prepare_source_bundle_import(b" " * (MAX_SOURCE_BUNDLE + 1), **fields)
        self.assertEqual(len(self.wire), 1)

    def test_create_hashes_original_bytes_and_declares_only_transport(self):
        """An explicit create201 binds original bytes and does not upload, prepare, commit or publish them."""
        raw = encoded(bundle_value()) + b" \n"
        value = stage_value(raw)
        self.queue(value, status=201)
        self.assertEqual(self.create(raw), value)
        method, path, body, headers = self.wire[0]
        self.assertEqual((method, path), ("POST", "/api/v2/project/source-transfer-stages"))
        self.assertEqual(json.loads(body), {**declaration(raw), "acknowledge_sensitive_metadata":True, "acknowledge_source_content":True})
        self.assertEqual(headers["Idempotency-Key"], KEY)
        self.assertEqual(len(self.wire), 1)
        self.assertEqual(len(self.closed), 1)

    def test_strict_raw_duplicate_bom_utf8_scalar_and_trailing_refusal(self):
        """Raw duplicates including escaped aliases, BOM, malformedUTF8, nonfinite and trailing JSON fail beforeHTTP."""
        base = encoded(bundle_value())
        invalid = [b"\xef\xbb\xbf" + base, b"\xff", b'{"x":1,"x":2}', b'{"x":1,"\\u0078":2}',
                   b'{"x":NaN}', b'{"x":Infinity}', base + b"{}", b"", bytearray(base)]
        for raw in invalid:
            with self.subTest(kind=type(raw).__name__), self.assertRaises((ValueError, RuntimeError)):
                self.create(raw)
        self.assertFalse(self.wire)

    def test_closed_bundle_index_pin_and_content_shapes(self):
        """Unknown or missing nested fields cannot silently widen the Bundle4 schema."""
        for location in ("top", "index", "pin", "content"):
            for mutation in ("extra", "missing"):
                value = bundle_value()
                target = value if location == "top" else value["index"] if location == "index" else value["pins"][0] if location == "pin" else value["contents"][0]
                if mutation == "extra":
                    target["PRIVATE_SENTINEL"] = "not-authority"
                else:
                    del target[next(iter(target))]
                with self.subTest(location=location, mutation=mutation):
                    self.assert_refused_bundle(value)

    def test_profile_and_literal_source_opt_in_are_not_translated(self):
        """Bundle3 tags, wrong staged profile and truthy substitutes for source opt-in refuse early."""
        for key, wrong in [("schema_version", "forge.workspace-index-bundle/3"), ("profile", "index-and-source-hex"),
                           ("source_content_included", 1), ("source_content_included", False)]:
            value = bundle_value(); value[key] = wrong
            with self.subTest(field=key, value=wrong):
                self.assert_refused_bundle(value)

    def test_normalized_index_sha_is_distinct_from_original_artifact_sha(self):
        """Changing index identity without updating its normalized hash is refused before stage creation."""
        value = bundle_value(); value["index"]["label"] += " changed"
        self.assert_refused_bundle(value)
        value = bundle_value(); value["index_sha256"] = "a" * 64
        self.assert_refused_bundle(value)
        self.assertNotEqual(bundle_value()["index_sha256"], hashlib.sha256(encoded(bundle_value())).hexdigest())

    def test_pin_content_bijection_and_authorial_order_are_complete(self):
        """Missing, duplicate and reordered keys cannot replace one resource's content with another."""
        for mutation in ("missing-pin", "extra-content", "duplicate-pin", "reorder-content", "different-key"):
            value = bundle_value(resources=2)
            if mutation == "missing-pin": value["pins"].pop()
            elif mutation == "extra-content": value["contents"].append(copy.deepcopy(value["contents"][0]))
            elif mutation == "duplicate-pin": value["pins"][1] = copy.deepcopy(value["pins"][0])
            elif mutation == "reorder-content": value["contents"].reverse()
            else: value["contents"][0]["key"] = "different"
            with self.subTest(mutation=mutation): self.assert_refused_bundle(value)

    def test_decoded_pin_size_sha_and_exact_types_are_verified(self):
        """A decoded file's size andSHA must match its pin rather than a declared transport hash."""
        for key, wrong in [("size", True), ("size", 3), ("sha256", "a" * 64), ("sha256", "A" * 64)]:
            value = bundle_value(); value["pins"][0][key] = wrong
            with self.subTest(field=key, value=wrong): self.assert_refused_bundle(value)

    def test_hex_parts_are_lowercase_even_nonempty_and_bounded(self):
        """Noncanonical or oversized decoded-file chunks fail independently of whole-artifact transport."""
        for part in ("", "0", "FF", "gg", "00" * (STAGED_PART_BYTES + 1), None):
            value = bundle_value(); value["contents"][0]["chunks"] = [part]
            with self.subTest(kind=type(part).__name__): self.assert_refused_bundle(value)

    def test_nonfinal_content_chunk_must_fill32k_and_empty_file_is_exact(self):
        """A short nonfinal chunk is refused, while zero bytes have no chunks and the empty-fileSHA."""
        value = bundle_value(b"xy"); value["contents"][0]["chunks"] = ["78", "79"]
        self.assert_refused_bundle(value)
        raw = encoded(bundle_value(b"")); self.queue(stage_value(raw), status=201)
        self.assertEqual(self.create(raw)["artifact_sha256"], hashlib.sha256(raw).hexdigest())

    def test_logical10mib_boundary_does_not_claim_native_capacity(self):
        """Exactly10MiB original JSON plus whitespace can be declared; one excess fails beforeHTTP."""
        raw = encoded(bundle_value()); raw += b" " * (MAX_STAGED_SOURCE_BUNDLE - len(raw))
        self.queue(stage_value(raw), status=201)
        self.assertEqual(self.create(raw)["chunk_count"], MAX_STAGED_PARTS)
        self.assertEqual(json.loads(self.wire[0][2])["artifact_size_bytes"], 10485760)
        with self.assertRaises(ValueError): self.create(raw + b" ")
        self.assertEqual(len(self.wire), 1)

    def test_complete_incoming_membership_reserves_raw_index_slot(self):
        """A100-resource incoming index is refused even when every synthetic pin/content row is present."""
        self.assert_refused_bundle(bundle_value(resources=100))

    def test_acknowledgments_and_creation_key_are_checked_before_http(self):
        """Each sensitivity flag needs literalTrue and malformed idempotency keys create no stage."""
        for name in ("acknowledge_sensitive_metadata", "acknowledge_source_content"):
            for wrong in (False, 1, None, "true"):
                with self.subTest(field=name, value=wrong), self.assertRaises(ValueError): self.create(**{name:wrong})
        for key in (None, True, "short", "x" * 256, "x" * 16 + "\n"):
            with self.subTest(kind=type(key).__name__), self.assertRaises((ValueError, RuntimeError)): self.create(idempotency_key=key)
        self.assertFalse(self.wire)

    def test_create_response_closed_identity_is_exact(self):
        """A server declaration cannot change the artifactSHA, count, part size, schema orclosed fields."""
        for field, wrong in [("artifact_sha256", "a" * 64), ("chunk_count", 1), ("chunk_size_bytes", True),
                             ("schema_version", "forge.workspace-index-bundle/3"), ("private", "PRIVATE_SENTINEL")]:
            value = stage_value(self.raw); value[field] = wrong; self.queue(value)
            with self.subTest(field=field), self.assertRaises((ValueError, RuntimeError)): self.create()
        self.assertEqual(len(self.wire), 5)
        self.assertEqual(len(self.closed), 5)

    def test_status_closed_counters_states_and_boolean_types(self):
        """Transport-only DTOs keep count/byte bounds and refuse impossible ready orunknown states."""
        for changes in [{"received_chunk_count":True}, {"received_bytes":1}, {"received_chunk_count":1,"received_bytes":0},
                        {"state":"ready"}, {"state":"unknown"}, {"received_chunk_count":3,"received_bytes":3}, {"extra":0}]:
            value = stage_value(self.raw); value.update(changes); self.queue(value)
            with self.subTest(changes=changes), self.assertRaises(RuntimeError): self.client.source_transfer_status(STAGE)
        self.assertTrue(all(row[0] == "GET" for row in self.wire))

    def test_status_mismatch_and_invalid_identifier_do_not_become_authority(self):
        """Foreign stageDTOs refuse afterGET, invalid route identifiers beforeGET, and404 never implies no writes."""
        value = stage_value(self.raw); value["stage_id"] = "bst_other0123456789"; self.queue(value)
        with self.assertRaises(RuntimeError): self.client.source_transfer_status(STAGE)
        with self.assertRaises(ValueError): self.client.source_transfer_status("PRIVATE_PATH/..");
        self.queue({"code":"not-found","message":"Retained stage unavailable","retryable":False}, status=404)
        with self.assertRaises(WorkspaceError) as caught: self.client.source_transfer_status(STAGE)
        self.assertEqual(caught.exception.payload["code"], "not-found")
        self.assertEqual([row[0] for row in self.wire], ["GET", "GET"])

    def test_expiry_is_validated_as_original_metadata_not_renewed(self):
        """Unqualified timestamp spellings refuse; a past valid timestamp is merely returned, never auto renewed."""
        for expires in ("2026-10-03", "2026-02-30T00:00:00Z", "2026-10-03T00:00:00", "x" * 41, None):
            value = stage_value(self.raw); value["expires_at"] = expires; self.queue(value)
            with self.subTest(expires=expires), self.assertRaises(RuntimeError): self.client.source_transfer_status(STAGE)
        value = stage_value(self.raw); self.queue(value)
        self.assertEqual(self.client.source_transfer_status(STAGE)["expires_at"], EXPIRES)
        self.assertTrue(all(row[0] == "GET" for row in self.wire))

    def test_upload_two_exact_ordinals_then_actual_status_only(self):
        """The public uploader sends original bytes once per ordinal and queries complete observed readiness."""
        stage = stage_value(self.raw); ready = stage_value(self.raw, state="ready", received=2)
        self.queue(ack_value(self.raw, 0), ack_value(self.raw, 1), ready)
        self.assertEqual(self.client.upload_source_transfer_stage(stage, self.raw), ready)
        self.assertEqual([(row[0],row[1]) for row in self.wire], [("PUT","/api/v2/project/source-transfer-stages/"+STAGE+"/chunks/0"),
                          ("PUT","/api/v2/project/source-transfer-stages/"+STAGE+"/chunks/1"),("GET","/api/v2/project/source-transfer-stages/"+STAGE)])
        chunks = [json.loads(row[2]) for row in self.wire[:2]]
        self.assertEqual(b"".join(bytes.fromhex(row["hex"]) for row in chunks), self.raw)
        self.assertEqual(chunks[-1]["size_bytes"], len(self.raw) - STAGED_PART_BYTES)
        self.assertTrue(all("Idempotency-Key" not in row[3] for row in self.wire))
        self.assertEqual(len(self.closed), 3)

    def test_explicit_identical_part_replay_preserves_original_ack(self):
        """A caller-chosen retry can return older counters but must retain the original part and expiry."""
        stage = stage_value(self.raw); original = ack_value(self.raw, 0)
        self.queue(original, original)
        part = self.raw[:STAGED_PART_BYTES]
        self.assertEqual(self.client.put_source_transfer_chunk(stage, 0, part), original)
        self.assertEqual(self.client.put_source_transfer_chunk(stage, 0, part), original)
        self.assertEqual(self.wire[0][2], self.wire[1][2])
        self.assertEqual(stage["received_chunk_count"], 0)
        self.assertEqual(original["expires_at"], EXPIRES)

    def test_chunk_ack_identity_hash_size_count_expiry_and_closure(self):
        """A mismatched acknowledgment cannot advance the upload or renew its stage."""
        for field, wrong in [("chunk_ordinal",1),("sha256","a"*64),("size_bytes",True),("received_chunk_count",0),
                             ("received_bytes",0),("expires_at","2026-10-04T00:00:00Z"),("extra",0)]:
            value = ack_value(self.raw,0); value[field]=wrong; self.queue(value)
            with self.subTest(field=field), self.assertRaises(RuntimeError):
                self.client.put_source_transfer_chunk(stage_value(self.raw),0,self.raw[:STAGED_PART_BYTES])
        self.assertTrue(all(row[0] == "PUT" for row in self.wire))

    def test_part_length_and_ordinal_types_refuse_before_put(self):
        """No boolean/out-of-range ordinal or wrong exact remainder reaches the transport."""
        stage = stage_value(self.raw)
        for ordinal, part in [(True,self.raw[:STAGED_PART_BYTES]),(-1,b"x"),(2,b"x"),(0,b"x"),(1,self.raw[:STAGED_PART_BYTES])]:
            with self.subTest(ordinal=ordinal), self.assertRaises((ValueError,RuntimeError)):
                self.client.put_source_transfer_chunk(stage,ordinal,part)
        self.assertFalse(self.wire)

    def test_lost_create_reply_has_no_automatic_resend_or_lookup(self):
        """A lost creation response closes the original connection and causes exactly one attemptedPOST."""
        self.queue(OSError("synthetic lost response"))
        with self.assertRaises(OSError): self.create()
        self.assertEqual([(row[0],row[1]) for row in self.wire],[("POST","/api/v2/project/source-transfer-stages")])
        self.assertEqual(len(self.closed),1)

    def test_lost_upload_reply_stops_without_retry_or_preview(self):
        """An uncertain ordinal response stops before the nextPUT, status, preview orconfirmation."""
        self.queue(OSError("synthetic lost response"))
        with self.assertRaises(OSError): self.client.upload_source_transfer_stage(stage_value(self.raw),self.raw)
        self.assertEqual(len(self.wire),1)
        self.assertEqual(self.wire[0][0],"PUT")
        self.assertEqual(len(self.closed),1)

    def test_upload_invalid_or_expired_budget_and_changed_raw_do_not_dispatch(self):
        """Budget/type and exact full-artifact fences precede any ordinal transport."""
        stage = stage_value(self.raw)
        for timeout in (0,-1,True,601,float("nan"),float("inf")):
            with self.subTest(timeout=timeout), self.assertRaises(ValueError):
                self.client.upload_source_transfer_stage(stage,self.raw,timeout=timeout)
        with self.assertRaises(ValueError): self.client.upload_source_transfer_stage(stage,self.raw+b" ")
        with patch("workspace_client.time.monotonic",side_effect=[0,1]), self.assertRaises(TimeoutError):
            self.client.upload_source_transfer_stage(stage,self.raw,timeout=1)
        self.assertFalse(self.wire)

    def test_late_ack_at_exact_deadline_stops_before_next_put(self):
        """An actual response can consume the unchanged upload budget; equality prevents a second dispatch."""
        clock=Clock(); self.queue(ack_value(self.raw,0))
        def expire():
            """Move the cooperative clock only after the first request has actually been recorded."""
            clock.value=1
        self.on_response=expire
        with patch("workspace_client.time.monotonic",clock), self.assertRaises(TimeoutError):
            self.client.upload_source_transfer_stage(stage_value(self.raw),self.raw,timeout=1)
        self.assertEqual(len(self.wire),1)
        self.assertEqual(self.wire[0][0],"PUT")

    def test_final_status_cannot_change_original_stage_lifetime(self):
        """Complete uploaded bytes do not permit status to swap artifact identity orrenew expiry."""
        ready=stage_value(self.raw,state="ready",received=2);ready["expires_at"]="2026-10-04T00:00:00Z"
        self.queue(ack_value(self.raw,0),ack_value(self.raw,1),ready)
        with self.assertRaises(RuntimeError):self.client.upload_source_transfer_stage(stage_value(self.raw),self.raw)
        self.assertEqual([row[0]for row in self.wire],["PUT","PUT","GET"])

    def test_discard_is_closed_stage_retirement_only(self):
        """Retirement requires the exactID/literalTrue and does not issue operation cancellation orwrites."""
        value={"stage_id":STAGE,"discarded":True};self.queue(value)
        self.assertEqual(self.client.discard_source_transfer_stage(STAGE),value)
        for changed in [{"discarded":1},{"stage_id":"bst_other0123456789"},{"extra":0}]:
            malformed=copy.deepcopy(value);malformed.update(changed);self.queue(malformed)
            with self.assertRaises(RuntimeError):self.client.discard_source_transfer_stage(STAGE)
        self.assertTrue(all(row[0]=="DELETE"and row[1].endswith(STAGE)for row in self.wire))
        self.assertTrue(all(json.loads(row[2])=={}for row in self.wire))

    def test_complete_staged_preview_reuses_binding_and_no_confirmation(self):
        """The preparation returns all native preview rows and preknownID but performs noaccepted restore."""
        value=prepared_response();self.queue(value)
        self.assertEqual(self.prepare(),value)
        self.assertEqual(self.wire[0][1],"/api/v2/project/source-transfer-stages/"+STAGE+"/preview")
        self.assertEqual(json.loads(self.wire[0][2]),{"target_index_schema_version":2,"acknowledge_index_replacement":True,
                         "acknowledge_source_content":True,"acknowledge_replace_files":True})
        self.assertEqual(value["preview"]["operation_id"],OP)
        self.assertEqual(value["replacement"]["consumed_file_count"],3)
        self.assertEqual(len(self.wire),1)
        self.assertNotIn("commit",self.wire[0][1])

    def test_staged_prepare_ack_selector_and_key_refuse_before_post(self):
        """Native preparation requires everyliteralTrue, exactinteger selector and validatedkey."""
        for name in ("acknowledge_index_replacement","acknowledge_source_content","acknowledge_replace_files"):
            with self.subTest(field=name),self.assertRaises(ValueError):self.prepare(**{name:1})
        for selector in (True,"2",0,3):
            with self.subTest(selector=selector),self.assertRaises(ValueError):self.prepare(target_index_schema_version=selector)
        with self.assertRaises((ValueError,RuntimeError)):self.prepare(idempotency_key="short")
        self.assertFalse(self.wire)

    def test_preview_full_membership_hash_and_unknown_field_refusal(self):
        """Incomplete native reply rows do not gain confirmation merely because transportfinished."""
        for mutation in ("binding","hash","count","unknown","missing-id"):
            value=prepared_response()
            if mutation=="binding":value["preview"]["input_bindings"].pop()
            elif mutation=="hash":value["replacement"]["proposed_index_sha256"]="a"*64
            elif mutation=="count":value["replacement"]["consumed_file_count"]=2
            elif mutation=="unknown":value["preview"]["private"]=True
            else:del value["preview"]["operation_id"]
            self.queue(value)
            with self.subTest(mutation=mutation),self.assertRaises(RuntimeError):self.prepare()
        self.assertTrue(all(row[1].endswith("/preview")for row in self.wire))

    def test_stream_export_is_single_unconfirmed202_effect(self):
        """Stream export selection keeps explicit source acknowledgments and doesnotdownload orcommit."""
        value={"operation_id":OP,"state":"running"};self.queue(value,status=202)
        self.assertEqual(self.client.prepare_source_stream_export("private-output.json",acknowledge_sensitive_metadata=True,
                         acknowledge_source_content=True,idempotency_key=KEY),value)
        self.assertEqual(self.wire[0][1],"/api/v2/project/source-stream-exports")
        self.assertEqual(json.loads(self.wire[0][2]),{"target_path":"private-output.json","acknowledge_sensitive_metadata":True,
                         "acknowledge_source_content":True})
        self.assertEqual(len(self.wire),1)

    def test_committed_manifest_closed_count_hash_and_opt_in(self):
        """A manifest must bind this operation/fullSHA/profile and exactceil count before anychunk."""
        for field,wrong in [("operation_id","op_"+"3"*64),("artifact_sha256","a"*64),("chunk_count",1),
                           ("source_content_included",1),("chunk_size_bytes",True),("profile","index-and-source-hex"),("extra",0)]:
            value=manifest_value(self.raw);value[field]=wrong;self.queue(value)
            with self.subTest(field=field),self.assertRaises(RuntimeError):
                self.client.source_stream_manifest(OP,expected_sha256=hashlib.sha256(self.raw).hexdigest())
        self.assertTrue(all(row[1].endswith("/manifest")for row in self.wire))

    def test_stream_chunk_closes_identity_part_sha_size_hex_and_ordinal(self):
        """Every returned ordinal verifies the fullidentity and exactdecodedbytes before exposing content."""
        for field,wrong in [("operation_id","op_"+"3"*64),("artifact_sha256","a"*64),("chunk_ordinal",1),
                           ("sha256","a"*64),("size_bytes",True),("hex","FF"),("extra",0)]:
            value=stream_value(self.raw,0);value[field]=wrong;self.queue(value)
            with self.subTest(field=field),self.assertRaises(RuntimeError):self.client.source_stream_chunk(manifest_value(self.raw),0)
        self.assertTrue(all(row[1].endswith("/chunks/0")for row in self.wire))

    def test_bad_chunk_ordinal_or_manifest_id_never_reads_transport(self):
        """Invalididentifier and exacttype/range fences run before committedpartGET."""
        for ordinal in (True,-1,2):
            with self.subTest(ordinal=ordinal),self.assertRaises(RuntimeError):self.client.source_stream_chunk(manifest_value(self.raw),ordinal)
        value=manifest_value(self.raw);value["operation_id"]="private/path"
        with self.assertRaises(ValueError):self.client.source_stream_chunk(value,0)
        self.assertFalse(self.wire)

    def test_download_reconstructs_original_bytes_and_no_local_publication(self):
        """Two realwrapperGET parts reconstruct authorial bytes and require completeSHA without importing them."""
        self.queue(manifest_value(self.raw),stream_value(self.raw,0),stream_value(self.raw,1))
        result=self.client.download_source_stream_export(OP,expected_sha256=hashlib.sha256(self.raw).hexdigest())
        self.assertEqual(result,self.raw)
        self.assertEqual([row[0]for row in self.wire],["GET","GET","GET"])
        self.assertEqual([row[1].rsplit("/",1)[-1]for row in self.wire],["manifest","0","1"])
        self.assertEqual(len(self.closed),3)

    def test_per_part_valid_corruption_still_fails_complete_artifact_sha(self):
        """Rehashed corrupt parts cannot defeat the independently declared complete artifactSHA."""
        altered=bytearray(self.raw);altered[-2]=ord(" ")if altered[-2]!=ord(" ")else ord("x")
        value=stream_value(self.raw,1);part=bytes(altered[STAGED_PART_BYTES:]);value.update(hex=part.hex(),sha256=hashlib.sha256(part).hexdigest())
        self.queue(manifest_value(self.raw),stream_value(self.raw,0),value)
        with self.assertRaises(RuntimeError):self.client.download_source_stream_export(OP,expected_sha256=hashlib.sha256(self.raw).hexdigest())
        self.assertEqual(len(self.wire),3)

    def test_correct_transport_hash_does_not_admit_duplicate_or_bom_bundle(self):
        """Full transportidentity is insufficient when reconstructed bytes fail the strictBundle4 decoder."""
        for raw in (b'{"x":1,"x":2}',b"\xef\xbb\xbf"+encoded(bundle_value())):
            self.queue(manifest_value(raw),stream_value(raw,0))
            with self.subTest(raw_size=len(raw)),self.assertRaises(ValueError):
                self.client.download_source_stream_export(OP,expected_sha256=hashlib.sha256(raw).hexdigest())
        self.assertTrue(all(row[0]=="GET"for row in self.wire))

    def test_download_invalid_budget_and_late_manifest_no_parts(self):
        """Invalidbudgets issue noGET; a manifest consuming the exactdeadline prevents anypartread."""
        for timeout in (True,0,601,float("nan"),float("inf")):
            with self.subTest(timeout=timeout),self.assertRaises(ValueError):
                self.client.download_source_stream_export(OP,expected_sha256=hashlib.sha256(self.raw).hexdigest(),timeout=timeout)
        self.assertFalse(self.wire)
        clock=Clock();self.queue(manifest_value(self.raw))
        def expire():
            """Consume the synthetic budget only after actual manifestdispatch."""
            clock.value=1
        self.on_response=expire
        with patch("workspace_client.time.monotonic",clock),self.assertRaises(TimeoutError):
            self.client.download_source_stream_export(OP,expected_sha256=hashlib.sha256(self.raw).hexdigest(),timeout=1)
        self.assertEqual(len(self.wire),1)
        self.assertTrue(self.wire[0][1].endswith("/manifest"))

    def test_stable_selected_file_creation_and_changed_upload_refusal(self):
        """Selectedlocal bytes can create the exactstage, but a changed original file cannot upload into it."""
        with tempfile.TemporaryDirectory()as directory:
            path=Path(directory)/"synthetic.json";path.write_bytes(self.raw);stage=stage_value(self.raw)
            self.queue(stage,status=201)
            self.assertEqual(self.client.create_source_transfer_stage_file(path,acknowledge_sensitive_metadata=True,
                             acknowledge_source_content=True,idempotency_key=KEY),stage)
            path.write_bytes(self.raw+b" ")
            with self.assertRaises(ValueError):self.client.upload_source_transfer_stage_file(stage,path)
        self.assertEqual(len(self.wire),1)
        self.assertEqual(json.loads(self.wire[0][2])["artifact_sha256"],hashlib.sha256(self.raw).hexdigest())

    def test_selected_file_empty_oversized_or_nonregular_no_dispatch(self):
        """Readbounds and regular-file type precede stage creation, with no source publication."""
        with tempfile.TemporaryDirectory()as directory:
            path=Path(directory)/"synthetic.json"
            for size in (0,MAX_STAGED_SOURCE_BUNDLE+1):
                with path.open("wb")as stream:stream.truncate(size)
                with self.subTest(size=size),self.assertRaises(ValueError):
                    self.client.create_source_transfer_stage_file(path,acknowledge_sensitive_metadata=True,
                                                                 acknowledge_source_content=True,idempotency_key=KEY)
            path.write_bytes(self.raw)
            bad=SimpleNamespace(st_mode=stat.S_IFDIR,st_size=len(self.raw))
            with patch("workspace_client.os.fstat",return_value=bad),self.assertRaises(ValueError):
                self.client.create_source_transfer_stage_file(path,acknowledge_sensitive_metadata=True,
                                                             acknowledge_source_content=True,idempotency_key=KEY)
        self.assertFalse(self.wire)

    def test_selected_file_instance_change_is_refused_before_create(self):
        """A changed inode ormetadata generation across boundedread never becomes a declaration."""
        with tempfile.TemporaryDirectory()as directory:
            path=Path(directory)/"synthetic.json";path.write_bytes(self.raw)
            before=SimpleNamespace(st_mode=stat.S_IFREG,st_size=len(self.raw),st_dev=1,st_ino=2,st_mtime_ns=3)
            after=SimpleNamespace(st_mode=stat.S_IFREG,st_size=len(self.raw),st_dev=1,st_ino=4,st_mtime_ns=3)
            with patch("workspace_client.os.fstat",side_effect=[before,after]),self.assertRaises(ValueError):
                self.client.create_source_transfer_stage_file(path,acknowledge_sensitive_metadata=True,
                                                             acknowledge_source_content=True,idempotency_key=KEY)
        self.assertFalse(self.wire)

    def test_file_gate_precedes_any_selected_file_open(self):
        """Unnegotiated stage helpers refuse without touching an authored localpath."""
        self.client._contract_version="2.3.0"
        with patch("workspace_client.os.open",side_effect=AssertionError("Unexpected file open")):
            with self.assertRaises(ValueError):self.client.create_source_transfer_stage_file("PRIVATE_SENTINEL",acknowledge_sensitive_metadata=True,
                                                                                          acknowledge_source_content=True,idempotency_key=KEY)
            with self.assertRaises(ValueError):self.client.upload_source_transfer_stage_file(stage_value(self.raw),"PRIVATE_SENTINEL")
        self.assertFalse(self.wire)


    def test_three_part_status_totals_require_actual_subset_geometry(self):
        """The three-part response admits final-first subsets and discarded-complete, never impossible totals."""
        raw = three_part_raw()
        self.assertEqual(len(raw),65537)
        self.assertEqual(declaration(raw)["chunk_count"],3)
        for count, size, state in [(1,2,"receiving"),(3,65536,"ready"),(3,65537,"receiving")]:
            value=stage_value(raw,state=state,received=count);value["received_bytes"]=size;self.queue(value)
            with self.subTest(count=count,size=size,state=state),self.assertRaises(RuntimeError):
                self.client.source_transfer_status(STAGE)
        for count, size, state in [(1,1,"receiving"),(1,32768,"receiving"),(2,32769,"receiving"),
                                   (3,65537,"ready"),(3,65537,"discarded")]:
            value=stage_value(raw,state=state,received=count);value["received_bytes"]=size;self.queue(value)
            self.assertEqual(self.client.source_transfer_status(STAGE),value)
        self.assertTrue(all(row[:2]==("GET","/api/v2/project/source-transfer-stages/"+STAGE)for row in self.wire))
        self.assertEqual(len(self.closed),8)
        self.assertFalse(self.responses)

    def test_three_part_acknowledgment_requires_sent_ordinal_membership(self):
        """Final-first replay retains one-byte totals; a nonfinal singleton requires its full32768 bytes."""
        raw=three_part_raw();stage=stage_value(raw)
        for ordinal, count, size in [(2,1,32768),(0,1,1),(2,3,65536)]:
            value=ack_value(raw,ordinal,received=count);value["received_bytes"]=size;self.queue(value)
            part=raw[ordinal*STAGED_PART_BYTES:(ordinal+1)*STAGED_PART_BYTES]
            with self.subTest(ordinal=ordinal,count=count,size=size),self.assertRaises(RuntimeError):
                self.client.put_source_transfer_chunk(stage,ordinal,part)
        final=ack_value(raw,2,received=1);final["received_bytes"]=1
        nonfinal=ack_value(raw,0,received=1)
        self.queue(final,final,nonfinal)
        self.assertEqual(self.client.put_source_transfer_chunk(stage,2,raw[-1:]),final)
        self.assertEqual(self.client.put_source_transfer_chunk(stage,2,raw[-1:]),final)
        self.assertEqual(self.client.put_source_transfer_chunk(stage,0,raw[:STAGED_PART_BYTES]),nonfinal)
        self.assertEqual(self.wire[3][2],self.wire[4][2])
        self.assertEqual(final["expires_at"],EXPIRES)
        self.assertEqual(stage["received_chunk_count"],0)
        self.assertEqual(len(self.closed),6)
        self.assertFalse(self.responses)

    def test_three_part_upload_reconstructs_exact_last_byte_and_full_status(self):
        """Three once-onlyPUTs and the explicit finalGET preserve all65537 original artifact bytes."""
        raw=three_part_raw();stage=stage_value(raw);ready=stage_value(raw,state="ready",received=3)
        self.queue(ack_value(raw,0),ack_value(raw,1),ack_value(raw,2),ready)
        self.assertEqual(self.client.upload_source_transfer_stage(stage,raw),ready)
        self.assertEqual([row[0]for row in self.wire],["PUT","PUT","PUT","GET"])
        parts=[json.loads(row[2])for row in self.wire[:3]]
        self.assertEqual([part["size_bytes"]for part in parts],[32768,32768,1])
        self.assertEqual(b"".join(bytes.fromhex(part["hex"])for part in parts),raw)
        self.assertEqual([row[1].rsplit("/",1)[-1]for row in self.wire[:3]],["0","1","2"])
        self.assertEqual(len(self.closed),4)
        self.assertFalse(self.responses)


if __name__ == "__main__":
    unittest.main()
