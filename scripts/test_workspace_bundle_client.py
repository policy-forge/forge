#!/usr/bin/env python3
"""Unexecuted stdlib controls for the maintained 2.2 client; fake wire data is not native admission."""
import hashlib
import json
import unittest
from unittest.mock import patch
from workspace_client import Workspace, WorkspaceError


class BundleClientTests(unittest.TestCase):
    """Exercise public wrappers and narrow transport against explicitly authored observations."""
    def setUp(self):
        """Create one isolated negotiated client without spawning a process or retaining credentials."""
        self.client = Workspace.__new__(Workspace)
        self.client._api_major = 2
        self.client._contract_version = "2.2.0"
        self.client._api_prefix = "/api/v2"
        self.client._port = 1
        self.client._capability = "synthetic-private-capability"
        self.client._next_request_at = 0
        self.wire = []
        self.closed = []
        self.payload = b'{}'
        self.status = 200
        self.media = "application/json; charset=utf-8"
        owner = self

        class Response:
            """Read only the authored bounded bytes and response metadata."""
            @property
            def status(self):
                """Expose this control's chosen HTTP status without fabricating a native result."""
                return owner.status

            def read(self, bound):
                """Honor the actual caller's read limit so an excess byte remains observable."""
                return owner.payload[:bound]

            def getheader(self, name, default=""):
                """Supply Content-Type only; other header observations are absent."""
                return owner.media if name.lower() == "content-type" else default

        class Connection:
            """Record exact delegated transport values for one private mock connection."""
            def __init__(self, host, port, timeout):
                """Require maintained loopback transport rather than accepting a remote substitute."""
                owner.assertEqual((host, port, timeout), ("127.0.0.1", 1, 30))

            def request(self, method, path, body, headers):
                """Retain exact wire values inside the test, never a public receipt."""
                owner.wire.append((method, path, body, headers))

            def getresponse(self):
                """Return only the authored response object for the maintained decoder."""
                return Response()

            def close(self):
                """Record connection cleanup on success, typed rejection and decoder failure."""
                owner.closed.append(True)

        transport=patch("workspace_client.http.client.HTTPConnection", Connection)
        transport.start();self.addCleanup(transport.stop)
        pacing=patch("workspace_client.time.sleep")
        pacing.start();self.addCleanup(pacing.stop)

    def import_bytes(self, raw=b'{}', target=1):
        """Call the documented raw wrapper with a fixed private idempotency key."""
        return self.client.prepare_bundle_import(raw, target_index_schema_version=target,
            acknowledge_index_replacement=True, idempotency_key="synthetic-private-key")

    def test_original_raw_import_bytes_are_wrapped_once(self):
        """Preserve BOM, Unicode spelling and duplicates instead of parsing or resaving the chosen file."""
        raw='\ufeff{"key":"π","key":"duplicate"}'.encode()
        self.import_bytes(raw, 2)
        method, path, body, headers = self.wire[0]
        self.assertEqual((method, path), ("POST", "/api/v2/project/bundle-imports"))
        self.assertEqual(body, b'{"bundle":' + raw + b',"target_index_schema_version":2,"acknowledge_index_replacement":true}')
        self.assertEqual(len(body)-len(raw), 80)
        self.assertEqual(headers["Idempotency-Key"], "synthetic-private-key")
        self.assertEqual(len(self.closed), 1)

    def test_exact_import_envelope_boundary_and_one_excess(self):
        """Allow exactly1MiB including the numeric wrapper; reject one excess before dispatch."""
        self.import_bytes(b' ' * 1048496)
        self.assertEqual(len(self.wire[0][2]), 1048576)
        with self.assertRaises(ValueError):
            self.import_bytes(b' ' * 1048497)
        self.assertEqual(len(self.wire), 1)

    def test_numeric_selector_rejects_bools_floats_and_unknown_versions(self):
        """Require the exact numeric1|2 selector rather than a coercible or historical string spelling."""
        for target in (True, False, 1.0, "forge.workspace/1", 0, 3, None):
            with self.subTest(target=target), self.assertRaises(ValueError):
                self.import_bytes(target=target)
        self.assertEqual(self.wire, [])

    def test_export_and_import_require_explicit_true_acknowledgments(self):
        """Neither sensitive export nor destructive index replacement may infer acknowledgment."""
        for ack in (False, 1, "true", None):
            with self.subTest(ack=ack), self.assertRaises(ValueError):
                self.client.prepare_bundle_export("exports/metadata.json", acknowledge_sensitive_metadata=ack, idempotency_key="key")
            with self.subTest(ack=ack), self.assertRaises(ValueError):
                self.client.prepare_bundle_import(b'{}', target_index_schema_version=1, acknowledge_index_replacement=ack, idempotency_key="key")
        self.assertEqual(self.wire, [])

    def test_export_request_is_closed_and_never_commits_implicitly(self):
        """Send only the documented target and sensitivity acknowledgment; leave receipt confirmation to callers."""
        self.payload=b'{"state":"pending","operation_id":"op_synthetic000001"}'
        result=self.client.prepare_bundle_export("exports/metadata.json", acknowledge_sensitive_metadata=True, idempotency_key="key")
        self.assertEqual(result["state"], "pending")
        self.assertEqual(json.loads(self.wire[0][2]), {"target_path":"exports/metadata.json","acknowledge_sensitive_metadata":True})
        self.assertEqual(len(self.wire), 1)
        self.assertFalse(any("commits" in row[1] for row in self.wire))

    def test_wrapped_direct_preview_is_returned_without_automatic_effect(self):
        """Preserve the direct200 wrapper including complete replacement context without hiding validation."""
        reply={"validation":{"state":"valid","error_count":0,"warning_count":0,"diagnostics":[]},"preview":{"preview_id":"prev_synthetic000001"},
            "replacement":{"previous_index":None,"proposed_index":{"schema_version":"forge.workspace/1","label":"Empty","resources":[]},
            "supplied_index_sha256":"a"*64,"proposed_index_sha256":"b"*64,"removed_resource_keys":[],"consumed_file_count":0}}
        self.payload=json.dumps(reply).encode()
        self.assertEqual(self.import_bytes(), reply)
        self.assertEqual(len(self.wire), 1)

    def test_s6_requires_exact2_2_without_narrowing_numeric_major_bootstrap(self):
        """Reject S6 wrappers under older/future negotiated contracts while leaving foundation bootstrap untouched."""
        for version in ("2.0.0","2.1.0","2.12.3"):
            self.client._contract_version=version
            with self.subTest(version=version), self.assertRaises(ValueError):
                self.import_bytes()
        self.assertEqual(self.wire, [])

    def test_s3_reads_are_available_under2_1_and2_2_only(self):
        """Keep the nine explicit read wrappers usable under the successor without falling back to API1."""
        for version in ("2.1.0","2.2.0"):
            self.client._contract_version=version
            self.client.lifecycle_records(as_of="2026-10-03")
        self.assertEqual([row[1] for row in self.wire], ["/api/v2/lifecycle/records?as_of=2026-10-03&page_size=50"]*2)
        self.client._contract_version="2.12.3"
        with self.assertRaises(ValueError):
            self.client.lifecycle_records(as_of="2026-10-03")

    def test_raw_transport_rejects_other_route_missing_key_and_mixed_body(self):
        """Raw opaque bytes are confined to the documented import route and original private key."""
        for path, body, key in (("/api/v2/project/bundle-verifications",None,"key"), ("/api/v2/project/bundle-imports",None,None), ("/api/v2/project/bundle-imports",{},"key")):
            with self.subTest(path=path,body=body,key=key), self.assertRaises(ValueError):
                self.client.request("POST",path,body,idempotency_key=key,raw_json_body=b'{}')
        self.assertEqual(self.wire, [])

    def test_committed_metadata_download_checks_media_and_exact_hash(self):
        """Return exact original JSON bytes from the authenticated metadata family without implicit disk writes."""
        self.payload=b'{"label":"sensitive"}\n'
        raw=self.client.download_bundle_export("op_synthetic000001",expected_sha256=hashlib.sha256(self.payload).hexdigest())
        self.assertEqual(raw,self.payload)
        method,path,body,headers=self.wire[0]
        self.assertEqual((method,path,body),("GET","/api/v2/project/bundle-exports/op_synthetic000001/download",None))
        self.assertNotIn("Idempotency-Key",headers)
        self.assertEqual(len(self.closed),1)

    def test_download_rejects_wrong_family_hash_and_excess_bytes(self):
        """Bad media, changed bytes or a1MiB+1 response must never become accepted artifacts."""
        for payload,media,expected in ((b'{}',"text/html",hashlib.sha256(b'{}').hexdigest()),(b'{}',"application/json","a"*64),(b' '*1048577,"application/json","a"*64)):
            self.payload=payload;self.media=media
            with self.subTest(media=media,size=len(payload)),self.assertRaises(RuntimeError):
                self.client.download_bundle_export("op_synthetic000001",expected_sha256=expected)
        self.assertEqual(len(self.closed),3)

    def test_download_scope_rejects_ambiguous_operation_and_hash(self):
        """Refuse cross-route IDs and noncanonical expected fingerprints before contacting the server."""
        for operation,sha in (("../effects/commits","a"*64),("op_synthetic000001","A"*64),("op_synthetic000001",None)):
            with self.subTest(operation=operation),self.assertRaises(ValueError):
                self.client.download_bundle_export(operation,expected_sha256=sha)
        self.assertEqual(self.wire,[])

    def test_typed_readonly_rejection_remains_a_server_failure(self):
        """The client never upgrades a read-only session or downgrades a typed denial into success."""
        self.status=403;self.payload=b'{"code":"read-only-session","message":"Writes disabled","retryable":false}'
        with self.assertRaises(WorkspaceError) as caught:
            self.import_bytes()
        self.assertEqual(caught.exception.payload["code"],"read-only-session")
        self.assertEqual(len(self.closed),1)


if __name__ == "__main__":
    unittest.main()
