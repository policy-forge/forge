#!/usr/bin/env python3
"""Launch one owned API2 session with a due/overdue lifecycle portfolio for installed-Chrome alert verification.

Synthetic passphrase and files only; the production server reads its passphrase exclusively from its terminal.
"""
import argparse
import json
import os
from pathlib import Path
import pty
import re
import select
import signal
import subprocess
import tempfile
import time

PASSPHRASE = b"synthetic review alert browser passphrase 065\n"


def fixture(root, forge):
    """Author a real approved lifecycle record whose review is overdue at the verification date."""
    project = root / "project"
    project.mkdir()
    (project / "policy.md").write_text("# Synthetic alert policy\n\n## Access\n\n- Operators must review the supplied clause.\n")
    catalog = {"catalog": {"uuid": "77777777-7777-4777-8777-777777777777",
                           "metadata": {"title": "Synthetic alert framework", "last-modified": "2026-08-25T12:00:00Z",
                                        "version": "1.0.0", "oscal-version": "1.2.3"},
                           "groups": [{"id": "group-1", "title": "Synthetic group",
                                       "controls": [{"id": "ac-1", "title": "Synthetic control",
                                                     "parts": [{"id": "ac-1_smt", "name": "statement", "prose": "Operators must review."}]}]}]}}
    (project / "catalog.json").write_text(json.dumps(catalog))
    record = project / "lifecycle.json"
    # Review date 2026-10-12 with a 7-day due-soon window; the browser applies 2026-10-13 (overdue).
    init = [forge, "lifecycle", "init", "--source", "policy.md", "--artifact", "catalog.json",
            "--output", "lifecycle.json", "--policy-key", "alert-policy", "--version-key", "v1",
            "--title", "Synthetic alert policy", "--owner", "owner", "--party", "owner=owner,reviewer,approver",
            "--next-review", "2026-10-12", "--cadence-days", "30", "--due-soon-days", "7"]
    subprocess.run(init, cwd=project, check=True, capture_output=True)
    for state, actor, role, at in [("in-review", "owner", "reviewer", "2026-10-01T00:00:00Z"),
                                   ("approved", "owner", "approver", "2026-10-02T00:00:00Z")]:
        subprocess.run([forge, "lifecycle", "transition", "--record", "lifecycle.json", "--to", state,
                        "--actor", actor, "--role", role, "--at", at, "--rationale", "Synthetic declared transition",
                        "--apply"], cwd=project, check=True, capture_output=True)
    index = {"schema_version": "forge.workspace/2", "label": "Synthetic review-alert portfolio",
             "resources": [{"key": "source", "role": "lifecycle-source", "path": "policy.md"},
                           {"key": "catalog", "role": "oscal-catalog-artifact", "path": "catalog.json"},
                           {"key": "record", "role": "lifecycle-record", "path": "lifecycle.json"}]}
    (project / "forge.workspace.json").write_text(json.dumps(index, indent=2) + "\n")
    return project


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forge", required=True)
    parser.add_argument("--node", default="node")
    parser.add_argument("--chrome", default="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
    args = parser.parse_args()
    forge = str(Path(args.forge).resolve())
    root = Path(__file__).resolve().parents[1]
    driver = root / "ui/tests/workspace-alert-browser.cjs"
    with tempfile.TemporaryDirectory(prefix="forge-alert-browser-") as temporary:
        project = fixture(Path(temporary), forge)
        out = Path(temporary) / "out"
        out.mkdir()
        pid, terminal = pty.fork()
        if pid == 0:
            os.execv(forge, [forge, "workspace", "--project", str(project), "--api-major", "2", "--no-open"])
        try:
            output = b""
            step = 0
            origin = None
            deadline = time.monotonic() + 30
            while time.monotonic() < deadline:
                ready, _, _ = select.select([terminal], [], [], 0.2)
                if not ready:
                    continue
                data = os.read(terminal, 8192)
                if not data:
                    break
                output += data
                if step == 0 and b"Set workspace passphrase" in output:
                    time.sleep(0.1)
                    os.write(terminal, PASSPHRASE)
                    step = 1
                elif step == 1 and b"Confirm passphrase:" in output:
                    time.sleep(0.1)
                    os.write(terminal, PASSPHRASE)
                    step = 2
                match = re.search(rb"Local workspace: (http://127\.0\.0\.1:[0-9]+)", output)
                if match:
                    origin = match.group(1).decode()
                    break
            if not origin:
                raise RuntimeError("Synthetic API2 alert session did not launch")
            if PASSPHRASE.strip() in output:
                raise RuntimeError("Synthetic passphrase was echoed")
            environment = dict(os.environ)
            environment["FORGE_TEST_BROWSER_EXECUTABLE"] = args.chrome
            environment["PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD"] = "1"
            environment["PLAYWRIGHT_BROWSERS_PATH"] = "0"
            result = subprocess.run([args.node, str(driver), origin, str(project), str(out), str(root)],
                                    timeout=180, check=False, env=environment)
            print(json.dumps({"driver_exit": result.returncode, "out": str(out)}))
            raise SystemExit(result.returncode)
        finally:
            try:
                os.kill(pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                os.waitpid(pid, 0)
            except ChildProcessError:
                pass


if __name__ == "__main__":
    main()