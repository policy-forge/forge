#!/usr/bin/env python3
"""Observe fixed public distro support names without granting qualification credit.

Never emit filesystem paths, targets, exception messages or regular-file bytes.
The workflow retains this separate diagnostic; the unchanged trust gate decides.
"""
import json
import os
from pathlib import Path
import stat
import sys


def identity(info):
    """Compare the same leaf generation before and after reading a link spelling."""
    return (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


def observe(path, known_targets):
    """Return only closed facts for one fixed name, never following a leaf link."""
    result = {"availability": "unavailable", "kind": None, "target_class": None}
    try:
        before = path.lstat()
        if stat.S_ISREG(before.st_mode):
            kind, target = "regular", None
        elif stat.S_ISLNK(before.st_mode):
            kind = "symlink"
            spelling = os.readlink(os.fsencode(path))
            target = known_targets.get(spelling, "other") if len(spelling) <= 4096 else "oversized"
        else:
            kind, target = "unsupported", None
        after = path.lstat()
        if identity(before) != identity(after):
            return result
        return {"availability": "observed", "kind": kind, "target_class": target}
    except OSError:
        # An absent or unreadable fixed support name grants no observation.
        return result


def main():
    """Emit finite support-layout classifications for the actual distro interpreter."""
    if len(sys.argv) != 1:
        return 2
    version = ".".join(map(str, sys.version_info[:2]))
    architecture = getattr(sys.implementation, "_multiarch", None)
    if architecture not in {"x86_64-linux-gnu", "aarch64-linux-gnu"}:
        return 2
    root = Path("/usr/lib/python" + version)
    config = root / ("config-" + version + "-" + architecture)
    targets = {
        ("/etc/python" + version + "/sitecustomize.py").encode(): "distro-configuration",
        ("../../share/doc/python" + version + "/copyright").encode(): "license-text",
        ("../../share/doc/libpython" + version + "-stdlib/copyright").encode(): "license-text",
        ("../../" + architecture + "/libpython" + version + ".a").encode(): "development-static-library",
        ("../../" + architecture + "/libpython" + version + ".so").encode(): "development-shared-library",
        ("../../" + architecture + "/libpython" + version + ".so.1.0").encode(): "development-shared-library",
    }
    paths = {
        "license": root / "LICENSE.txt",
        "sitecustomize": root / "sitecustomize.py",
        "development-static-library": config / ("libpython" + version + ".a"),
        "development-shared-library": config / ("libpython" + version + ".so"),
        "sysconfig": root / ("_sysconfigdata__linux_" + architecture + ".py"),
    }
    report = {"schema_version": "forge.distro-support-observation/1", "acceptance_eligible": False,
              "basis": "fixed-public-support-names-only", "rows": {key: observe(path, targets) for key, path in paths.items()}}
    print(json.dumps(report, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
