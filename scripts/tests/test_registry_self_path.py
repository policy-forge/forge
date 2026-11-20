"""Regression: registry `self` paths must never redirect the download host."""
import pathlib
import subprocess
import sys
import unittest

VALIDATOR = pathlib.Path(__file__).resolve().parents[1] / "registry_self_path.py"


def accepts(value):
    return subprocess.run(
        [sys.executable, str(VALIDATOR), value], capture_output=True
    ).returncode == 0


class RegistrySelfPath(unittest.TestCase):
    def test_accepts_plain_catalog_path(self):
        self.assertTrue(accepts("/api/v1/catalogs/abc/resource.json"))

    def test_rejects_authority_injection(self):
        # https://registry.oscal.io@evil.example/... sends registry.oscal.io
        # to the userinfo slot and fetches from evil.example instead.
        self.assertFalse(accepts("@evil.example/x"))

    def test_rejects_protocol_relative_and_backslash(self):
        self.assertFalse(accepts("//evil.example/x"))
        self.assertFalse(accepts("\\\\evil.example\\x"))
        self.assertFalse(accepts("/api\\@evil.example"))

    def test_rejects_control_and_whitespace(self):
        self.assertFalse(accepts("/api/x\nother\ty"))
        self.assertFalse(accepts("/api/ x"))
        self.assertFalse(accepts(""))


if __name__ == "__main__":
    unittest.main()
