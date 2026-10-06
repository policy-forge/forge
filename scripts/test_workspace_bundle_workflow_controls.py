#!/usr/bin/env python3
"""Mocked companion-version admission controls; no Forge or loopback execution."""

from types import SimpleNamespace
import unittest
from unittest.mock import patch

import test_workspace_bundle_workflow as companion


class MetadataCompanionVersionTests(unittest.TestCase):
    """Preserve the closed metadata gate before raw transport or request pacing."""

    def test_supported_versions_return_the_exact_observed_version(self):
        """Both supported selected-major tuples retain their actual version without rewriting it."""
        for version in ("2.2.0", "2.3.0", "2.4.0"):
            with self.subTest(version=version):
                client = SimpleNamespace(_api_major=2, _contract_version=version)
                self.assertEqual(companion.require_metadata_version(client), version)

    def test_other_majors_and_versions_cannot_admit_metadata_controls(self):
        """Prior, future, mismatched and non-string tuples fail instead of silently gaining methods."""
        for major, version in (
            (1, "1.2.0"), (1, "2.2.0"), (1, "2.3.0"), (3, "2.3.0"),
            (2, "2.0.0"), (2, "2.1.0"), (2, "2.3.1"), (2, "2.12.3"),
            (2, ""), (2, None), (2, 2.3),
        ):
            with self.subTest(major=major, version=version):
                client = SimpleNamespace(_api_major=major, _contract_version=version)
                with self.assertRaisesRegex(RuntimeError, "selected API2"):
                    companion.require_metadata_version(client)

    def test_rejected_raw_gate_precedes_pacing_and_connection_creation(self):
        """An unsupported tuple cannot inspect absent pacing fields, sleep or create an HTTP connection."""
        client = SimpleNamespace(_api_major=2, _contract_version="2.1.0")
        with patch.object(companion.time, "sleep") as sleep, \
             patch.object(companion.http.client, "HTTPConnection") as connection:
            with self.assertRaisesRegex(RuntimeError, "selected API2"):
                companion.raw_import(client, b"{}")
            sleep.assert_not_called()
            connection.assert_not_called()


if __name__ == "__main__":
    unittest.main()
