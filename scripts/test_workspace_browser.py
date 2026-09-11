#!/usr/bin/env python3
"""POSIX packaged browser harness. Synthetic passphrase and files only.
The production server still reads its passphrase exclusively from its terminal.
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

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument("--forge",required=True)
parser.add_argument("--node",default="node")
parser.add_argument("--read-only",action="store_true")
args=parser.parse_args()
forge=str(Path(args.forge).resolve())
script=Path(__file__).resolve().parents[1]/"ui/tests/workspace.cjs"
with tempfile.TemporaryDirectory(prefix="forge-browser-") as root:
    project=Path(root)
    (project/"policy.md").write_text("# Synthetic policy\n\n## Access\n\n- Operators must review the supplied clause.\n")
    catalog={"catalog":{"uuid":"22222222-2222-4222-8222-222222222222","metadata":{"title":"<img src=x onerror=alert(1)>","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},"controls":[{"id":"framework-a","title":"Synthetic A"},{"id":"framework-b","title":"Synthetic B"}]}}
    (project/"framework.json").write_text(json.dumps(catalog))
    if args.read_only:
        (project/"forge.workspace.json").write_text(json.dumps({"schema_version":"forge.workspace/1","label":"Synthetic read-only project <script>","resources":[{"key":"policy","role":"policy-source","path":"policy.md"},{"key":"framework","role":"oscal-catalog-artifact","path":"framework.json"}]}))
    pid, terminal=pty.fork()
    if pid==0:
        command=[forge,"workspace","--project",root,"--no-open"]
        if args.read_only:command.append("--read-only")
        os.execv(forge,command)
    try:
        output=b"";step=0;deadline=time.monotonic()+30;url=None
        while time.monotonic()<deadline:
            ready,_,_=select.select([terminal],[],[],0.2)
            if not ready:continue
            output+=os.read(terminal,8192)
            if step==0 and b"Set workspace passphrase" in output:
                time.sleep(0.1);os.write(terminal,b"synthetic browser verification passphrase 062\n");step=1
            elif step==1 and b"Confirm passphrase:" in output:
                time.sleep(0.1);os.write(terminal,b"synthetic browser verification passphrase 062\n");step=2
            match=re.search(rb"Local workspace: (http://127\.0\.0\.1:[0-9]+)",output)
            if match:url=match.group(1).decode();break
        if not url:raise RuntimeError("Synthetic browser workspace did not launch")
        if b"synthetic browser verification passphrase" in output:raise RuntimeError("Terminal echoed the test passphrase")
        result=subprocess.run([args.node,str(script),url,"read-only" if args.read_only else "writable"],timeout=240,check=False)
        if result.returncode:raise SystemExit(result.returncode)
    finally:
        try:os.kill(pid,signal.SIGTERM)
        except ProcessLookupError:pass
        deadline=time.monotonic()+3
        while True:
            try:reaped,_=os.waitpid(pid,os.WNOHANG)
            except ChildProcessError:reaped=pid
            if reaped:break
            if time.monotonic()>=deadline:
                try:os.kill(pid,signal.SIGKILL)
                except ProcessLookupError:pass
                os.waitpid(pid,0);break
            time.sleep(0.05)
        os.close(terminal)
