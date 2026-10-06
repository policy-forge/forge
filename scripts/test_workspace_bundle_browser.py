#!/usr/bin/env python3
"""Launch owned POSIX browser fixtures for installed-Chrome metadata-bundle controls."""
import argparse
import errno
import hashlib
import json
import os
from pathlib import Path
import re
import select
import signal
import subprocess
import tempfile
import time


def digest(path):
    """Bind exact script/executable bytes privately without invoking their contents."""
    value=hashlib.sha256()
    with Path(path).open("rb") as source:
        for block in iter(lambda:source.read(1024*1024),b""):
            value.update(block)
    return value.hexdigest()


def fixture(directory):
    """Author only real synthetic confined files and an explicit complete supplied metadata bundle."""
    project=directory/"project";project.mkdir();chosen=directory/"chosen";chosen.mkdir()
    catalog={"catalog":{"uuid":"22222222-2222-4222-8222-222222222222","metadata":{"title":"<img src=x onerror=alert(1)>","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},
        "controls":[{"id":"framework-a","title":"Synthetic A"},{"id":"framework-b","title":"Synthetic B"}]+[{"id":f"framework-extra-{number:03}","title":f"Synthetic pagination control{number}"} for number in range(1,52)]}}
    (project/"policy.md").write_bytes(b"# Synthetic policy\n\n## Access\n\n- Operators must review the supplied clause.\n")
    (project/"incoming.md").write_bytes(b"# Synthetic incoming policy\n\n## Records\n\n- Operators must preserve the supplied record.\n")
    (project/"framework.json").write_text(json.dumps(catalog,ensure_ascii=False),encoding="utf-8")
    initial={"schema_version":"forge.workspace/1","label":"<script>"+"L"*192,"resources":[{"key":"policy","role":"policy-source","path":"policy.md"},{"key":"framework","role":"oscal-catalog-artifact","path":"framework.json"}]}
    (project/"forge.workspace.json").write_text(json.dumps(initial,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
    incoming={"schema_version":"forge.workspace/1","label":"<script>"+"R"*192,"resources":[{"key":"incoming","role":"policy-source","path":"incoming.md"},{"key":"framework","role":"oscal-catalog-artifact","path":"framework.json"}]}
    normalized=(json.dumps(incoming,ensure_ascii=False,indent=2)+"\n").encode()
    bundle={"schema_version":"forge.workspace-index-bundle/1","content_profile":"index-and-hashes","index":incoming,"index_sha256":hashlib.sha256(normalized).hexdigest(),
        "pins":[{"key":row["key"],"sha256":digest(project/row["path"]),"size_bytes":(project/row["path"]).stat().st_size} for row in incoming["resources"]]}
    supplied=chosen/("F"*180+".json");supplied.write_text(json.dumps(bundle,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
    return project,supplied


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


def run(args):
    """Launch real browser mode, feed only the synthetic terminal passphrase and retain actual child outcomes."""
    if os.name!="posix":
        raise RuntimeError("This PTY launcher is POSIX-only; no Windows terminal proof")
    import pty
    out=Path(args.out).resolve();out.mkdir(exist_ok=False)
    forge=Path(args.forge).resolve();node=Path(args.node).resolve();root=Path(args.source_root).resolve();driver=Path(args.driver).resolve() if args.driver else root/"ui"/"tests"/"workspace-bundle-browser.cjs"
    receipt={"format":"forge.s6-native-browser-launch/1","status":"error","mode":args.mode,"api_major":2,"contract_version":None,"forge_sha256":digest(forge),"node_sha256":digest(node),"driver_sha256":digest(driver),"launcher_sha256":digest(__file__),"native_driver_exit":None,"normal_server_exit":False,"forced_cleanup":False,"limitations":["POSIX PTY browser launch only; no machine-session capability sharing.","Page request restrictions do not establish OS network denial."]}
    with tempfile.TemporaryDirectory(prefix="forge-s6-Chrome-fixture-") as temporary:
        project,incoming=fixture(Path(temporary));pid,terminal=pty.fork()
        if pid==0:
            command=[str(forge),"workspace","--project",str(project),"--api-major","2","--no-open"]
            if args.mode=="read-only":command.append("--read-only")
            os.execv(str(forge),command)
        receipt["owned_child_kind"]="forge";receipt["owned_child_pid"]=pid
        status=None
        drainage={"wait_status":None,"bytes":0,"ownership_lost":False}
        try:
            output=b"";step=0;deadline=time.monotonic()+30;origin=None
            while time.monotonic()<deadline:
                ready,_,_=select.select([terminal],[],[],0.2)
                if not ready:continue
                data=os.read(terminal,8192)
                if not data:break
                output+=data
                if len(output)>65536:raise RuntimeError("Bounded synthetic browser launch output exceeded")
                if step==0 and b"Set workspace passphrase" in output:
                    time.sleep(0.1);os.write(terminal,b"synthetic S6 browser bundle passphrase 062\n");step=1
                elif step==1 and b"Confirm passphrase:" in output:
                    time.sleep(0.1);os.write(terminal,b"synthetic S6 browser bundle passphrase 062\n");step=2
                match=re.search(rb"Local workspace: (http://127\.0\.0\.1:[0-9]+)",output)
                if match:origin=match.group(1).decode();break
            if not origin:raise RuntimeError("Synthetic browser-mode session did not launch")
            if b"synthetic S6 browser bundle passphrase 062" in output:raise RuntimeError("Synthetic passphrase was echoed")
            output=b""
            result=subprocess.run([str(node),str(driver),origin,args.mode,str(project),str(incoming),str(out/"browser"),str(root)],timeout=360,check=False)
            receipt["native_driver_exit"]=result.returncode
            driver_raw=(out/"browser"/"receipt.json").read_bytes()
            if len(driver_raw)>131072:raise RuntimeError("Browser receipt bound exceeded")
            observed=json.loads(driver_raw)
            version=observed.get("contract_version")
            if version in ("2.2.0","2.3.0","2.4.0") and type(observed.get("api_major")) is int and observed["api_major"]==2:
                receipt["contract_version"]=version
            elif result.returncode==0:raise RuntimeError("Browser did not establish a supported metadata contract")
            # Keep the native PTY open and discard bounded shutdown output until actual reap.
            status=drain_owned(pid,terminal,b"synthetic S6 browser bundle passphrase 062",
                               time.monotonic()+5,drainage)
            if status is not None:
                receipt["server_wait_status"]=status;receipt["normal_server_exit"]=os.waitstatus_to_exitcode(status)==0
            if result.returncode==0 and receipt["normal_server_exit"] and observed.get("status")=="passed" and receipt["contract_version"] in ("2.2.0","2.3.0","2.4.0"):
                receipt["status"]="passed"
        except Exception as error:
            receipt["error_class"]=type(error).__name__
        finally:
            status=drainage["wait_status"] if drainage["wait_status"] is not None else status
            if status is None:
                receipt["forced_cleanup"]=True
                receipt["status"]="error"
                for action in [signal.SIGTERM,signal.SIGKILL]:
                    if drainage["ownership_lost"] or drainage["wait_status"] is not None:break
                    try:
                        try:os.kill(pid,action)
                        except ProcessLookupError:pass
                        cleanup_deadline=time.monotonic()+3
                        try:
                            drain_owned(pid,terminal,b"synthetic S6 browser bundle passphrase 062",
                                        cleanup_deadline,drainage)
                        except Exception:
                            receipt["status"]="error"
                            if not drainage["ownership_lost"] and drainage["wait_status"] is None:
                                drainage["wait_status"]=wait_owned(pid,cleanup_deadline)
                    except ChildProcessError:drainage["ownership_lost"]=True
                    except Exception:receipt["status"]="error"
                    status=drainage["wait_status"]
                    if status is not None:break
                receipt["cleanup_wait_status"]=status
            if terminal is not None:
                try:os.close(terminal)
                except OSError:receipt["status"]="error"
            receipt["shutdown_terminal_bytes"]=drainage["bytes"]
            receipt["owned_child_wait_status"]=status if status is not None else receipt.get("cleanup_wait_status")
            receipt["direct_children_reaped"]=receipt["owned_child_wait_status"] is not None
            if receipt["forced_cleanup"] or not receipt["direct_children_reaped"]:receipt["status"]="error"
            (out/"launch-receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
    print(json.dumps({"status":receipt["status"],"receipt":str(out/"launch-receipt.json")}))
    return 0 if receipt["status"]=="passed" else 1


def main():
    """Require explicit local binary/runtime/source/output inputs and never install or download tools."""
    parser=argparse.ArgumentParser();parser.add_argument("--driver");parser.add_argument("--forge",required=True);parser.add_argument("--node",required=True);parser.add_argument("--source-root",required=True);parser.add_argument("--out",required=True);parser.add_argument("--mode",choices=("writable","read-only"),required=True)
    return run(parser.parse_args())


if __name__=="__main__":
    raise SystemExit(main())
