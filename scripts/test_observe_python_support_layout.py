#!/usr/bin/env python3
"""Ensure the separate distro diagnostic never follows leaves or discloses targets."""
import json
from pathlib import Path
import tempfile
import unittest
import observe_python_support_layout as observer


class ObservationTests(unittest.TestCase):
    def test_known_and_unknown_link_targets_remain_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "fixed-name"
            for target, expected in [("../../known", "license-text"), ("/PRIVATE/secret", "other")]:
                path.symlink_to(target)
                result = observer.observe(path, {b"../../known": "license-text"})
                self.assertEqual(result, {"availability": "observed", "kind": "symlink", "target_class": expected})
                self.assertNotIn("PRIVATE", json.dumps(result))
                self.assertNotIn(str(path), json.dumps(result))
                path.unlink()

    def test_regular_content_is_not_read_or_reported(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "fixed-name"
            path.write_bytes(b"PRIVATE FILE CONTENT")
            self.assertEqual(observer.observe(path, {}), {"availability": "observed", "kind": "regular", "target_class": None})

    def test_absent_name_grants_no_observation(self):
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual(observer.observe(Path(directory) / "absent", {}), {"availability": "unavailable", "kind": None, "target_class": None})


if __name__ == "__main__":
    unittest.main()
