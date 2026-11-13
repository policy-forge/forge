#!/usr/bin/env python3
"""Deterministic controls for the staged-capacity qualification fixture builders.

These exercise the new artifact/declaration construction and check identities in
the native capacity workflow without launching a process. The native workflow
itself qualifies the real server; these controls only prove the fixtures are
strict, exact and self-consistent so a passing native run is meaningful.
"""
import hashlib
import json
import unittest

import test_workspace_staged_capacity_workflow as cap
from workspace_client import Workspace


class CapacityFixtureTests(unittest.TestCase):
    """Prove the capacity fixtures are strict Bundle4 with exact ordered identities."""

    def test_bundle_is_strict_and_client_admissible(self):
        payload = bytes(range(256)) * 4
        raw = cap.bundle(payload)
        # The maintained client decoder accepts exactly what the server requires.
        self.assertEqual(Workspace._staged_bundle(raw), raw)
        value = json.loads(raw)
        self.assertEqual(value["schema_version"], "forge.workspace-index-bundle/4")
        self.assertEqual(value["profile"], "index-and-source-hex-staged")
        self.assertIs(value["source_content_included"], True)
        self.assertEqual(value["pins"][0]["size"], len(payload))
        self.assertEqual(value["pins"][0]["sha256"], hashlib.sha256(payload).hexdigest())
        decoded = b"".join(bytes.fromhex(chunk) for chunk in value["contents"][0]["chunks"])
        self.assertEqual(decoded, payload)

    def test_declaration_matches_exact_whole_artifact(self):
        raw = cap.bundle(b"\x00\xff\r\n")
        declaration = cap.declaration(raw)
        self.assertEqual(declaration["artifact_sha256"], hashlib.sha256(raw).hexdigest())
        self.assertEqual(declaration["artifact_size_bytes"], len(raw))
        self.assertEqual(declaration["chunk_size_bytes"], cap.PART_BYTES)
        self.assertEqual(declaration["chunk_count"], (len(raw) + cap.PART_BYTES - 1) // cap.PART_BYTES)
        self.assertIs(declaration["acknowledge_sensitive_metadata"], True)
        self.assertIs(declaration["acknowledge_source_content"], True)

    def test_logical_cap_padding_is_exact_and_one_over(self):
        raw = cap.bundle(b"\x00\xff\r\n")
        self.assertLess(len(raw), cap.ARTIFACT_CAP)
        exact = raw + b" " * (cap.ARTIFACT_CAP - len(raw))
        self.assertEqual(len(exact), cap.ARTIFACT_CAP)
        self.assertEqual(Workspace._staged_bundle(exact), exact)
        over = raw + b" " * (cap.ARTIFACT_CAP + 1 - len(raw))
        self.assertEqual(len(over), cap.ARTIFACT_CAP + 1)
        with self.assertRaises(ValueError):
            Workspace._staged_bundle(over)

    def test_refused_content_is_below_transport_cap(self):
        payload = bytes(range(256)) * (cap.REFUSED_CONTENT_BYTES // 256)
        raw = cap.bundle(payload)
        self.assertEqual(len(payload), cap.REFUSED_CONTENT_BYTES)
        self.assertLess(len(raw), cap.ARTIFACT_CAP)
        self.assertEqual(Workspace._staged_bundle(raw), raw)

    def test_check_identities_are_unique_and_fixed(self):
        self.assertEqual(len(cap.CHECKS), len(set(cap.CHECKS)))
        receipt = {"checks": []}
        for name in cap.CHECKS:
            cap.checked(receipt, name)
        self.assertEqual(receipt["checks"], list(cap.CHECKS))
        with self.assertRaises(AssertionError):
            cap.checked(receipt, cap.CHECKS[0])
        with self.assertRaises(AssertionError):
            cap.checked(receipt, "unsupported-check")


if __name__ == "__main__":
    unittest.main()