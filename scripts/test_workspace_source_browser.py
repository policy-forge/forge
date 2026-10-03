#!/usr/bin/env python3
"""Prepare three real installed-Chrome source-bundle campaigns in owned synthetic projects."""

import argparse
import errno
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import select
import signal
import subprocess
import time


def digest(path):
    """Stream exact local bytes without executing the binary or retaining its contents."""
    value = hashlib.sha256()
    with Path(path).open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def write_json(path, value):
    """Create one new private receipt; refuse replacement of any previous evidence."""
    with Path(path).open("x", encoding="utf-8") as target:
        json.dump(value, target, ensure_ascii=True, indent=2)
        target.write("\n")
    Path(path).chmod(0o600)


def fixture(directory):
    """Create valid donor sources and a different recipient with one retained retired file."""
    donor = directory / "donor"
    recipient = directory / "recipient"
    donor.mkdir(mode=0o700)
    recipient.mkdir(mode=0o700)
    (donor / "refs").mkdir(mode=0o700)
    catalog = {"catalog": {
        "uuid": "22222222-2222-4222-8222-222222222222",
        "metadata": {"title": "Synthetic source-bundle framework",
                     "last-modified": "2026-09-10T00:00:00Z", "version": "1",
                     "oscal-version": "1.2.3"},
        "controls": [{"id": "framework-a", "title": "Synthetic A"},
                     {"id": "framework-b", "title": "Synthetic B"}],
    }}
    (donor / "policy.md").write_bytes(
        b"# Synthetic restored policy\n\n## Access\n\n- Operators must preserve exact source bytes.\n")
    (donor / "refs/framework.json").write_bytes(
        (json.dumps(catalog, ensure_ascii=True, indent=2) + "\n").encode("ascii"))
    proposed = {"schema_version": "forge.workspace/2", "label": "Synthetic donor complete membership",
                "resources": [{"key": "policy", "role": "policy-source", "path": "policy.md"},
                              {"key": "framework", "role": "oscal-catalog-artifact",
                               "path": "refs/framework.json"}]}
    previous = {"schema_version": "forge.workspace/1", "label": "Synthetic recipient previous membership",
                "resources": [{"key": "old-policy", "role": "policy-source", "path": "policy.md"},
                              {"key": "retired", "role": "policy-source", "path": "retired.md"}]}
    (recipient / "policy.md").write_bytes(b"# Synthetic previous policy\n\n- Keep an observed overwrite base.\n")
    (recipient / "retired.md").write_bytes(b"# Synthetic retired source\n\n- Removed membership must retain this file.\n")
    for project, index in [(donor, proposed), (recipient, previous)]:
        (project / "forge.workspace.json").write_bytes(
            (json.dumps(index, ensure_ascii=True, indent=2) + "\n").encode("ascii"))
    return donor, recipient


def wait_owned(pid, deadline):
    """Wait only the original never-reaped PTY child; never adopt or signal a scanned PID."""
    while time.monotonic() < deadline:
        observed, status = os.waitpid(pid, os.WNOHANG)
        if observed == pid:
            return status
        time.sleep(0.05)
    return None


def drain_owned(pid, terminal, passphrase, deadline, observation):
    """Drain at most 64 KiB while retaining the original child's actual reap through validation faults."""
    tail = b""
    eof = False
    try:
        while time.monotonic() < deadline:
            if observation["wait_status"] is None:
                try:
                    observed, status = os.waitpid(pid, os.WNOHANG)
                except ChildProcessError:
                    observation["ownership_lost"] = True
                    raise
                if observed == pid:
                    observation["wait_status"] = status
            if not eof:
                timeout = (0 if observation["wait_status"] is not None else
                           min(0.05, max(0, deadline - time.monotonic())))
                ready, _, _ = select.select([terminal], [], [], timeout)
                if ready:
                    available = 65536 - observation["bytes"]
                    if available <= 0:
                        raise RuntimeError("terminal-budget")
                    try:
                        block = os.read(terminal, min(8192, available))
                    except OSError as error:
                        if error.errno != errno.EIO:
                            raise
                        block = b""
                    if not block:
                        eof = True
                    else:
                        observation["bytes"] += len(block)
                        combined = tail + block
                        if passphrase and passphrase in combined:
                            raise RuntimeError("terminal-echo")
                        tail = combined[-(len(passphrase) - 1):] if len(passphrase) > 1 else b""
                        continue
                elif observation["wait_status"] is not None:
                    return observation["wait_status"]
            if observation["wait_status"] is not None:
                return observation["wait_status"]
            if eof:
                time.sleep(min(0.05, max(0, deadline - time.monotonic())))
        return observation["wait_status"]
    finally:
        tail = b""


def read_startup(terminal, passphrase, deadline):
    """Consume bounded private startup bytes, submit twice, and discard every terminal byte."""
    import termios
    captured = bytearray()
    submitted = 0
    while time.monotonic() < deadline:
        ready, _, _ = select.select([terminal], [], [], min(0.1, max(0, deadline - time.monotonic())))
        if ready:
            try:
                block = os.read(terminal, 8192)
            except OSError as error:
                if error.errno == errno.EIO:
                    break
                raise
            if not block:
                break
            if len(captured) + len(block) > 65536:
                raise RuntimeError("terminal-budget")
            captured.extend(block)
        if passphrase in captured:
            raise RuntimeError("terminal-echo")
        prompt_ready = ((submitted == 0 and b"Set workspace passphrase (" in captured)
                        or (submitted == 1 and b"Confirm passphrase:" in captured))
        if prompt_ready and not termios.tcgetattr(terminal)[3] & termios.ECHO:
            unsent = passphrase + b"\n"
            while unsent:
                if time.monotonic() >= deadline:
                    raise RuntimeError("terminal-write-expired")
                written = os.write(terminal, unsent)
                if written <= 0:
                    raise RuntimeError("terminal-write-unverified")
                unsent = unsent[written:]
            submitted += 1
        match = re.search(rb"Local workspace: (http://127\.0\.0\.1:[0-9]{1,5})", captured)
        if match:
            if submitted != 2 or passphrase in captured:
                raise RuntimeError("startup-contract")
            origin = match.group(1).decode("ascii")
            count = len(captured)
            captured.clear()
            return origin, count, submitted
    captured.clear()
    raise RuntimeError("startup-unverified")


def campaign(args, stage, project, donor, fixture_root, deadline):
    """Own one sequential server and Node child; force or uncertain closure always revokes pass."""
    import pty
    out = fixture_root.parent / stage
    out.mkdir(mode=0o700)
    row = {"stage": stage, "status": "failed", "read_only": stage == "lookup",
           "node_exit": None, "forge_exit": None, "startup_bytes": 0, "prompt_submissions": 0,
           "forced_cleanup": False, "direct_children_reaped": False,
           "descendant_tree_cleanup": "unmeasured", "failure_class": None,
           "owned_child_kind": "forge", "owned_child_pid": None, "owned_child_wait_status": None}
    passphrase = ("synthetic-source-bundle-" + secrets.token_hex(24)).encode("ascii")
    pid = None
    terminal = None
    node = None
    status = None
    drainage = {"wait_status": None, "bytes": 0, "ownership_lost": False}
    try:
        if time.monotonic() >= deadline:
            raise RuntimeError("campaign-expired")
        pid, terminal = pty.fork()
        if pid == 0:
            command = [args.forge, "workspace", "--project", str(project),
                       "--api-major", "2", "--no-open"]
            if stage == "lookup":
                command.append("--read-only")
            try:
                os.execv(args.forge, command)
            finally:
                os._exit(127)
        row["owned_child_pid"] = pid
        origin, count, submissions = read_startup(terminal, passphrase, min(deadline, time.monotonic() + 30))
        row["startup_bytes"] = count
        row["prompt_submissions"] = submissions
        environment = os.environ.copy()
        environment["FORGE_SOURCE_BROWSER_PASSPHRASE"] = passphrase.decode("ascii")
        command = [args.node, args.driver, origin, stage, str(project), str(donor),
                   str(fixture_root.parent), str(out / "browser"), args.source_root, args.chrome]
        if args.rtk:
            command = [args.rtk, "proxy", *command]
        if time.monotonic() >= deadline:
            raise RuntimeError("node-expired")
        node = subprocess.Popen(command, env=environment, stdin=subprocess.DEVNULL,
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        environment.pop("FORGE_SOURCE_BROWSER_PASSPHRASE", None)
        row["node_exit"] = node.wait(timeout=max(0.001, deadline - time.monotonic()))
        # Keep the actual Forge PTY open and consume bounded shutdown output until reap.
        status = drain_owned(pid, terminal, passphrase,
                             min(deadline, time.monotonic() + 8), drainage)
        if status is not None:
            row["forge_exit"] = os.waitstatus_to_exitcode(status)
        raw = (out / "browser/receipt.json").read_bytes()
        if len(raw) > 131072:
            raise RuntimeError("driver-receipt-budget")
        browser = json.loads(raw)
        row["driver_receipt_sha256"] = hashlib.sha256(raw).hexdigest()
        if row["node_exit"] == 0 and row["forge_exit"] == 0 and browser.get("status") == "passed":
            row["status"] = "passed"
    except Exception as error:
        row["failure_class"] = type(error).__name__
    finally:
        status = drainage["wait_status"] if drainage["wait_status"] is not None else status
        if node is not None and node.poll() is None:
            row["forced_cleanup"] = True
            row["status"] = "failed"
            try:
                node.terminate()
                node.wait(timeout=3)
            except subprocess.TimeoutExpired:
                try:
                    node.kill()
                    node.wait(timeout=3)
                except Exception:
                    pass
            except Exception:
                pass
        if pid is not None and pid != 0 and status is None:
            row["forced_cleanup"] = True
            row["status"] = "failed"
            for action in [signal.SIGTERM, signal.SIGKILL]:
                if drainage["ownership_lost"] or drainage["wait_status"] is not None:
                    break
                try:
                    try:
                        os.kill(pid, action)
                    except ProcessLookupError:
                        pass
                    cleanup_deadline = time.monotonic() + 3
                    try:
                        drain_owned(pid, terminal, passphrase, cleanup_deadline, drainage)
                    except Exception:
                        row["status"] = "failed"
                        if not drainage["ownership_lost"] and drainage["wait_status"] is None:
                            drainage["wait_status"] = wait_owned(pid, cleanup_deadline)
                except ChildProcessError:
                    drainage["ownership_lost"] = True
                except Exception:
                    row["status"] = "failed"
                status = drainage["wait_status"]
                if status is not None:
                    break
        if terminal is not None:
            try:
                os.close(terminal)
            except OSError:
                row["status"] = "failed"
        row["shutdown_terminal_bytes"] = drainage["bytes"]
        row["owned_child_wait_status"] = status
        row["direct_children_reaped"] = status is not None and (node is None or node.returncode is not None)
        if not row["direct_children_reaped"] or row["forced_cleanup"]:
            row["status"] = "failed"
        passphrase = b""
        write_json(out / "launch-receipt.json", row)
    return row


def run(args):
    """Bind runtime inputs, preserve private fixtures, and abstain after any failed campaign."""
    if os.name != "posix":
        raise RuntimeError("POSIX-PTY-only")
    for name in ["forge", "node", "chrome", "source_root", "driver", "out"]:
        setattr(args, name, str(Path(getattr(args, name)).resolve(strict=name != "out")))
    args.rtk = str(Path(args.rtk).resolve(strict=True)) if args.rtk else None
    out = Path(args.out)
    out.mkdir(mode=0o700)
    out.chmod(0o700)
    fixture_root = out / "fixtures"
    fixture_root.mkdir(mode=0o700)
    donor, recipient = fixture(fixture_root)
    inputs = {name: digest(getattr(args, name)) for name in ["forge", "node", "chrome", "driver"]}
    if args.rtk:
        inputs["rtk"] = digest(args.rtk)
    inputs["launcher"] = digest(__file__)
    for name in ["ui/workspace.js", "ui/workspace.css", "ui/node_modules/playwright/package.json",
                 "ui/node_modules/playwright-core/package.json"]:
        inputs[name] = digest(Path(args.source_root) / name)
    receipt = {"format": "forge.s6-source-native-browser-launch/1", "status": "failed",
               "api_major": 2, "api_version": "2.3.0", "inputs": inputs, "campaigns": [],
               "node_wrapper": "rtk-proxy" if args.rtk else "direct-node",
               "limitations": ["Source-only preparation is not an executed result.",
                   "Successful direct-child closure does not measure an empty descendant tree.",
                   "Native per-user journal state is not overridden, inventoried or deleted by this launcher.",
                   "Page routing does not qualify OS network denial; screenshots are not human, AT or platform acceptance."]}
    deadline = time.monotonic() + 600
    for stage, project in [("export", donor), ("restore", recipient), ("lookup", recipient)]:
        row = campaign(args, stage, project, donor, fixture_root, min(deadline, time.monotonic() + 180))
        receipt["campaigns"].append(row)
        if row["status"] != "passed":
            break
    if len(receipt["campaigns"]) == 3 and all(row["status"] == "passed" for row in receipt["campaigns"]):
        receipt["status"] = "passed"
    for name, expected in inputs.items():
        target = (__file__ if name == "launcher" else
                  str(Path(args.source_root) / name) if name.startswith("ui/") else getattr(args, name))
        if digest(target) != expected:
            receipt["status"] = "failed"
            receipt["input_stability"] = "changed"
    if "input_stability" not in receipt:
        receipt["input_stability"] = "unchanged"
    write_json(out / "receipt.json", receipt)
    print(json.dumps({"status": receipt["status"], "campaigns": len(receipt["campaigns"])}))
    return 0 if receipt["status"] == "passed" else 1


def main():
    """Require approved executables and fresh output; optionally bind RTK for Node wrapping only."""
    parser = argparse.ArgumentParser()
    for name in ["forge", "node", "chrome", "source-root", "driver", "out"]:
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--rtk")
    return run(parser.parse_args())


if __name__ == "__main__":
    try:
        exit_code = main()
    except Exception:
        exit_code = 1
    raise SystemExit(exit_code)
