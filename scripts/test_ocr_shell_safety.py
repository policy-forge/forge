#!/usr/bin/env python3
"""Offline regression tests for OCR shell-safety remediations."""

import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class ShellSafety(unittest.TestCase):
    def test_installer_preserves_existing_hooks_and_symlink_targets(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            subprocess.run(["git", "init", "-q", str(repo)], check=True)
            scripts = repo / "scripts"
            scripts.mkdir()
            shutil.copyfile(ROOT / "scripts/install-hooks.sh", scripts / "install-hooks.sh")
            delegate = scripts / "pre-commit.sh"
            delegate.write_text("#!/bin/sh\nexit 0\n")
            delegate.chmod(0o755)
            hook = repo / ".git/hooks/pre-commit"

            def install():
                return subprocess.run(["bash", str(scripts / "install-hooks.sh")],
                                      capture_output=True)

            self.assertEqual(install().returncode, 0)
            self.assertTrue(os.access(hook, os.X_OK))
            original = hook.read_bytes()
            self.assertEqual(install().returncode, 0)
            self.assertEqual(hook.read_bytes(), original)
            hook.chmod(0o644)
            result = install()
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(b"not executable", result.stderr)
            hook.chmod(0o755)
            hook.write_bytes(b"custom hook\n")
            self.assertNotEqual(install().returncode, 0)
            self.assertEqual(hook.read_bytes(), b"custom hook\n")
            hook.unlink()
            target = repo / "protected"
            target.write_bytes(b"do not overwrite")
            hook.symlink_to(target)
            self.assertNotEqual(install().returncode, 0)
            self.assertEqual(target.read_bytes(), b"do not overwrite")
            self.assertTrue(hook.is_symlink())
            self.assertEqual(list(hook.parent.glob(".pre-commit.*")), [])

    def test_clean_refuses_unmarked_and_protected_directories_before_network(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary)
            sentinel = destination / "keep"
            sentinel.write_bytes(b"must survive")
            for path in [destination, ROOT, Path.home(), Path("/")]:
                result = subprocess.run(
                    ["bash", str(ROOT / "scripts/fetch-oscal-catalogs.sh"),
                     "--clean", "--dest", str(path)], capture_output=True, timeout=10)
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertIn(b"refusing to clean", result.stderr)
            self.assertEqual(sentinel.read_bytes(), b"must survive")


if __name__ == "__main__":
    unittest.main()
