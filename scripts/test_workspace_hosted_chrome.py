#!/usr/bin/env python3
"""Bounded Linux installed-Chrome producer; four synthetic campaigns do not establish full acceptance."""
import argparse
import ctypes
import json
import math
import os
from pathlib import Path
import re
import select
import signal
import subprocess
import sys
import tempfile
import time

import verify_workspace as shared
from workspace_browser_fixtures import synthetic_framework_catalog, synthetic_long_project_label

sys.dont_write_bytecode=True
SCHEMA="forge.hosted-chrome-smoke/3"
OUTPUT="hosted-chrome-smoke.json"
CAMPAIGNS=("default-read-only","default-writable","long-read-only","long-writable")
PASSPHRASE=b"synthetic browser verification passphrase 062"
MAX_OUTPUT=262144
ROOT=Path(__file__).resolve().parents[1]

# Failure protocol enums mirror the consumed pure CJS helper and are pinned/drift-checked by controls.
BROWSER_FAILURE_SCHEMA="forge.workspace-browser-failure/2"
BROWSER_FAILURE_STAGES=frozenset(("browser-setup","asset-binding","input-style-binding","unlock","navigation-recovery","resource-authoring","conversion-recovery","framework-workflow","decision-authoring","trace-export","metadata-preview","metadata-long-label","metadata-download","metadata-file-selection","metadata-comparison","metadata-duplicate","metadata-refresh","final-reflow","storage-checks","session-shutdown","counter-correlation","request-page-errors","browser-cleanup"))
BROWSER_FAILURE_AWAIT_STEPS=frozenset(("prepare","activate","response","ready","acknowledgment","download-state","focus","dialog"))
BROWSER_FAILURE_CATEGORIES=frozenset(("assertion","timeout","unclassified"))
BROWSER_FAILURE_OPERATORS=frozenset(("strictEqual","deepStrictEqual","match","=="))
CAPTURE_INPUTS=("ui/workspace.js","ui/workspace.css","ui/tests/workspace.cjs","ui/tests/workspace_failure.cjs","ui/tests/workspace_failure.test.cjs","scripts/test_workspace_hosted_chrome.py","scripts/workspace_browser_tool_probe.cjs","scripts/workspace_browser_fixtures.py")


def validate_browser_failure(value):
    """Validate closed failed-row facts and a nullable fixed refresh await; diagnostics never qualify a pass."""
    closed(value,("stage","category","assertion_operator","node_exit_code","await_step"))
    if type(value["stage"]) is not str or value["stage"] not in BROWSER_FAILURE_STAGES:raise ValueError("browser-failure-stage")
    if type(value["category"]) is not str or value["category"] not in BROWSER_FAILURE_CATEGORIES:raise ValueError("browser-failure-category")
    operator=value["assertion_operator"]
    if operator is not None and (type(operator) is not str or operator not in BROWSER_FAILURE_OPERATORS):raise ValueError("browser-failure-operator")
    if value["category"]!="assertion" and operator is not None:raise ValueError("browser-failure-category")
    step=value["await_step"]
    if step is not None and (type(step) is not str or step not in BROWSER_FAILURE_AWAIT_STEPS or value["stage"]!="metadata-refresh"):raise ValueError("browser-failure-await-step")
    code=value["node_exit_code"]
    if type(code) is not int or code==0 or not -2147483648<=code<=2147483647:raise ValueError("browser-failure-exit")
    return value


def decode_browser_failure(raw,node_exit_code):
    """Decode one complete fixed-step envelope and bind actual Popen status; malformed output stays unavailable."""
    try:
        value=strict_json(raw,maximum=1024)
        closed(value,("schema_version","stage","category","assertion_operator","await_step"))
        if value["schema_version"]!=BROWSER_FAILURE_SCHEMA:raise ValueError("browser-failure-version")
        return validate_browser_failure({"stage":value["stage"],"category":value["category"],"assertion_operator":value["assertion_operator"],"node_exit_code":node_exit_code,"await_step":value["await_step"]})
    except Exception:return None


def closed(value,keys):
    """Reject open receipt objects without copying arbitrary private members into diagnostics."""
    if not isinstance(value,dict) or set(value)!=set(keys):raise ValueError("closed-object")


def strict_json(raw,maximum=MAX_OUTPUT):
    """Read bounded JSON with duplicate decoded keys and nonfinite values rejected at every nesting depth."""
    if not raw or len(raw)>maximum:raise ValueError("json-bound")
    def unique(pairs):
        """Preserve evidence by refusing repeated decoded keys instead of selecting a last value."""
        result={}
        for key,value in pairs:
            if key in result:raise ValueError("duplicate-field")
            result[key]=value
        return result
    def nonfinite(_value):
        """Reject nonfinite literals using a fixed code rather than the supplied text."""
        raise ValueError("nonfinite")
    return json.loads(raw,object_pairs_hook=unique,parse_constant=nonfinite)


def integer(value,maximum=10000,minimum=0):
    """Use exact nonboolean integral denominators with explicit inclusive bounds."""
    if type(value) is not int or not minimum<=value<=maximum:raise ValueError("counter-bound")
    return value


def clean_environment(node,chrome,screenshot=None,long=False):
    """Clear Node and Playwright overrides, require installed Chrome, and keep screenshots in fresh private storage."""
    environment=os.environ.copy()
    for key in list(environment):
        if key.startswith(("PLAYWRIGHT_","NODE_","CHROME_","DYLD_")) or key in {"LD_PRELOAD","LD_LIBRARY_PATH","FORGE_TEST_SCREENSHOT","FORGE_TEST_LONG_METADATA","FORGE_TEST_BROWSER_EXECUTABLE"}:environment.pop(key,None)
    environment.update(PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD="1",PLAYWRIGHT_BROWSERS_PATH="0",FORGE_TEST_BROWSER_EXECUTABLE=str(chrome),PATH=str(Path(node).parent)+os.pathsep+environment.get("PATH",""))
    if screenshot is not None:
        environment["FORGE_TEST_SCREENSHOT"]=str(screenshot)
        for key in ("TMPDIR","TEMP","TMP"):environment[key]=str(Path(screenshot).parent)
    if long:environment["FORGE_TEST_LONG_METADATA"]="1"
    return environment


def process_identity(pid):
    """Read one bounded Linux PID/parent/start identity; vanished processes supply no signal authorization."""
    try:
        with open(Path("/proc")/str(pid)/"stat","rb") as stream:raw=stream.read(4097)
        if len(raw)>4096:raise ValueError("process-stat-bound")
        parts=raw.rsplit(b") ",1)[1].split()
        return int(parts[1]),int(parts[19])
    except (FileNotFoundError,ProcessLookupError):return None
    except PermissionError:raise ValueError("process-visibility-unverified")


COMMAND_FAILURES=frozenset({"subreaper-unavailable","pidfd-unavailable","process-scan-bound","process-stat-bound","process-visibility-unverified","owned-process-bound","child-exit-unverified","command-timeout","output-bound","cleanup-unverified","execution-unverified"})
TOOL_STEPS=frozenset({"resolve-node","resolve-npm","resolve-chrome","chrome-engine","node-version","npm-version","chrome-version","package-probe","package-validation","node-pin","npm-pin","chrome-pin"})
TOOL_FAILURES=COMMAND_FAILURES|frozenset({"executable-unavailable","chrome-path","chrome-engine","tool-exit-nonzero","tool-version","chrome-product","tool-scope","package-graph","package-scope","package-pin","closed-fields","integer","chrome-file-bound","chrome-file-changed","tool-capture-unverified"})


class ToolCaptureError(ValueError):
    """Carry only a validated tool step/reason and exact bounded process status; never retain raw command output."""
    def __init__(self,step,reason,exit_code=None):
        """Reject unapproved diagnostic strings and bool/unbounded exit values before storing the three safe facts."""
        if type(step) is not str or step not in TOOL_STEPS or type(reason) is not str or reason not in TOOL_FAILURES:raise ValueError("diagnostic-invalid")
        if exit_code is not None and (type(exit_code) is not int or not -(2**31)<=exit_code<2**31):raise ValueError("diagnostic-invalid")
        super().__init__("tool-capture-failed");self.step=step;self.reason=reason;self.exit_code=exit_code


def tool_diagnostic(error):
    """Revalidate an internally typed failure and return one closed, path/output/exception-free public record."""
    checked=ToolCaptureError(error.step,error.reason,error.exit_code)
    return {"phase":"browser-tool-capture","step":checked.step,"reason":checked.reason,"exit_code":checked.exit_code}


def command_failure(error):
    """Preserve known producer-owned lifecycle codes; arbitrary exceptions are represented only by a fixed fallback."""
    code=error.args[0] if type(error) is ValueError and len(error.args)==1 and type(error.args[0]) is str else None
    return code if code in COMMAND_FAILURES else "execution-unverified"


def tool_capture_step(step,callback,exit_code=None):
    """Associate one bounded preflight with its safe phase; raw exceptions cannot cross the typed diagnostic boundary."""
    try:return callback()
    except ToolCaptureError:raise
    except Exception as error:
        code=error.args[0] if type(error) is ValueError and len(error.args)==1 and type(error.args[0]) is str else None
        reason=code if code in TOOL_FAILURES else "executable-unavailable" if isinstance(error,FileNotFoundError) else "tool-capture-unverified"
        raise ToolCaptureError(step,reason,exit_code) from None


class OwnedTree:
    """Observe this Linux producer's descendants by PID/start identity; never kill unrelated or recycled processes."""
    def __init__(self):
        """Adopt orphaned descendants before any campaign starts; unavailable Linux confinement fails closed."""
        try:available=ctypes.CDLL(None,use_errno=True).prctl(36,1,0,0,0)==0
        except Exception:available=False
        if not available:raise ValueError("subreaper-unavailable")
        if not hasattr(os,"pidfd_open") or not hasattr(signal,"pidfd_send_signal"):raise ValueError("pidfd-unavailable")
        self.owner=os.getpid();self.seen={};self.forced=False;self.processes={};self.children=set();self.exits={};self.handles={}

    def observe(self):
        """Read bounded process identities and track only descendants or children adopted by this producer."""
        rows={};count=0
        with os.scandir("/proc") as entries:
            for entry in entries:
                count+=1
                if count>20000:raise ValueError("process-scan-bound")
                if not entry.name.isdigit():continue
                pid=int(entry.name);identity=process_identity(pid)
                if identity is not None:rows[pid]=identity
        owned={self.owner};changed=True
        while changed:
            changed=False
            for pid,(parent,started) in rows.items():
                if pid not in owned and parent in owned:
                    if pid not in self.seen and len(self.seen)>=128:raise ValueError("owned-process-bound")
                    prior=self.seen.get(pid);owned.add(pid);self.seen[pid]=started;changed=True
                    if pid not in self.handles or prior!=started:
                        try:handle=os.pidfd_open(pid)
                        except ProcessLookupError:continue
                        try:verified=process_identity(pid)
                        except Exception:
                            os.close(handle);raise
                        if verified is not None and verified[1]==started and verified[0] in owned:
                            if pid in self.handles:os.close(self.handles[pid])
                            self.handles[pid]=handle
                        else:os.close(handle)
        if len(self.seen)>128:raise ValueError("owned-process-bound")
        return {pid:started for pid,started in self.seen.items() if pid in owned and pid in rows and rows[pid][1]==started}

    def track_process(self,process):
        """Reserve a direct Popen child so only its owner consumes and interprets its actual wait status."""
        if len(self.processes)+len(self.children)>=128:raise ValueError("owned-process-bound")
        self.processes[process.pid]=process

    def track_child(self,pid):
        """Reserve the direct PTY child and retain its actual exit status separately from adopted descendants."""
        if len(self.processes)+len(self.children)>=128:raise ValueError("owned-process-bound")
        self.children.add(pid)

    def poll_child(self,pid):
        """Return an observed PTY child status; already-reaped or unknown status cannot be inferred as zero."""
        if pid not in self.children:raise ValueError("child-exit-unverified")
        if pid not in self.exits:
            try:reaped,status=os.waitpid(pid,os.WNOHANG)
            except ChildProcessError:raise ValueError("child-exit-unverified")
            if reaped:self.exits[pid]=os.waitstatus_to_exitcode(status)
        return self.exits.get(pid)

    def reap(self):
        """Preserve direct owners' actual statuses and reap only separately observed adopted descendants."""
        for process in self.processes.values():process.poll()
        for pid in self.children:
            if pid not in self.exits:self.poll_child(pid)
        for pid in self.seen:
            if pid in self.processes or pid in self.children:continue
            try:os.waitpid(pid,os.WNOHANG)
            except ChildProcessError:continue

    def settle(self,seconds=5):
        """Require actual descendant absence after a bounded natural grace window; no forced cleanup earns pass."""
        deadline=time.monotonic()+seconds
        while True:
            self.reap()
            if not self.observe():return True
            if time.monotonic()>=deadline:return False
            time.sleep(.02)

    def stop(self):
        """Force verified descendants and direct unreaped children even if global scanning fails; never infer empty cleanup."""
        self.forced=True;deadline=time.monotonic()+5
        while True:
            # Retained pidfds name only previously verified owned process instances, independent of a later scan failure.
            for handle in self.handles.values():
                try:signal.pidfd_send_signal(handle,signal.SIGKILL)
                except OSError:pass
            try:targets=self.observe()
            except Exception:targets={}
            # Direct children cannot reuse their PIDs before their sole owner's wait; pidfds fix the signal target.
            for pid,process in self.processes.items():
                if process.poll() is None:targets[pid]=None
            for pid in self.children:
                try:
                    if self.poll_child(pid) is None:targets[pid]=None
                except ValueError:continue
            for pid,started in targets.items():
                handle=None
                try:
                    handle=os.pidfd_open(pid)
                    if started is None:
                        current=self.processes[pid].poll() if pid in self.processes else self.poll_child(pid)
                        owned=current is None
                    else:owned=self.observe().get(pid)==started
                    if owned:signal.pidfd_send_signal(handle,signal.SIGKILL)
                except (OSError,ValueError):pass
                finally:
                    if handle is not None:os.close(handle)
            try:self.reap()
            except ValueError:pass
            try:
                direct_complete=all(process.poll() is not None for process in self.processes.values()) and all(self.poll_child(pid) is not None for pid in self.children)
                if not self.observe() and direct_complete:return True
            except Exception:pass
            if time.monotonic()>=deadline:return False
            time.sleep(.02)

    def close(self):
        """Release bounded retained pidfds only after lifecycle reconciliation; closing handles is not cleanup proof."""
        for handle in self.handles.values():os.close(handle)
        self.handles.clear()


def command(command_line,root,timeout,environment):
    """Drain both owned pipes to EOF under one byte/deadline budget; return only stdout and discard private stderr."""
    tree=None;process=None;captured=bytearray();failure=None
    try:
        tree=OwnedTree()
        process=subprocess.Popen(command_line,cwd=root,env=environment,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
        tree.track_process(process);streams=[process.stdout,process.stderr];deadline=time.monotonic()+timeout;observed=0
        for stream in streams:os.set_blocking(stream.fileno(),False)
        while process.poll() is None or streams:
            tree.observe()
            if time.monotonic()>=deadline:raise ValueError("command-timeout")
            for stream in select.select(streams,[],[],.05)[0]:
                block=os.read(stream.fileno(),8192)
                if not block:streams.remove(stream);continue
                observed+=len(block)
                if observed>MAX_OUTPUT:raise ValueError("output-bound")
                if stream is process.stdout:captured.extend(block)
        if not tree.settle():raise ValueError("cleanup-unverified")
    except Exception as error:
        failure=command_failure(error)
        if process is not None:
            try:
                if not tree.stop():failure="cleanup-unverified"
            except Exception:failure="cleanup-unverified"
            try:process.wait(timeout=5)
            except Exception:failure="cleanup-unverified"
        captured.clear()
    finally:
        try:
            if tree is not None:tree.close()
        except Exception:failure="cleanup-unverified";captured.clear()
        if process is not None:
            for stream in (process.stdout,process.stderr):
                try:
                    if stream is not None:stream.close()
                except Exception:failure="cleanup-unverified";captured.clear()
    return {"exit_code":None if process is None else process.returncode,"failure":failure,"output":bytes(captured)}


def validate_packages(value):
    """Require the exact UI-local approved package pair and bounded full installed-tree observations."""
    closed(value,("schema_version","node","packages","optional_omitted","installed_lock"))
    if value["schema_version"]!="forge.hosted-chrome-tools/1" or value["node"]!="24.19.0" or value["optional_omitted"] is not True:raise ValueError("tool-scope")
    if not isinstance(value["packages"],list) or [item.get("name") for item in value["packages"]]!=["playwright","playwright-core"]:raise ValueError("package-graph")
    for item in value["packages"]:
        closed(item,("name","version","relative_path","manifest","entry","files","bytes","tree_sha256"))
        if item["version"]!="1.62.1" or item["relative_path"]!="ui/node_modules/"+item["name"]:raise ValueError("package-scope")
        integer(item["files"],10000,1);integer(item["bytes"],67108864,1)
        for field in ("manifest","entry"):
            closed(item[field],("bytes","sha256"));integer(item[field]["bytes"],16777216,1)
            if not re.fullmatch("[0-9a-f]{64}",item[field]["sha256"]):raise ValueError("package-pin")
        if not re.fullmatch("[0-9a-f]{64}",item["tree_sha256"]):raise ValueError("package-pin")
    closed(value["installed_lock"],("bytes","sha256"));integer(value["installed_lock"]["bytes"],1048576,1)
    if not re.fullmatch("[0-9a-f]{64}",value["installed_lock"]["sha256"]):raise ValueError("package-pin")
    return value


def large_file_pin(path):
    """Hash an unchanged regular Chrome engine up to 512 MiB using bounded streaming, without retaining bytes."""
    import hashlib,stat
    path=Path(path);initial=path.lstat()
    if not stat.S_ISREG(initial.st_mode) or path.is_symlink() or initial.st_size>536870912:raise ValueError("chrome-file-bound")
    digest=hashlib.sha256();count=0
    with path.open("rb") as stream:
        while True:
            block=stream.read(1048576)
            if not block:break
            count+=len(block)
            if count>536870912:raise ValueError("chrome-file-bound")
            digest.update(block)
    final=path.lstat()
    if (initial.st_dev,initial.st_ino,initial.st_size,initial.st_mtime_ns,initial.st_ctime_ns)!=(final.st_dev,final.st_ino,final.st_size,final.st_mtime_ns,final.st_ctime_ns) or count!=initial.st_size:raise ValueError("chrome-file-changed")
    return {"bytes":count,"sha256":digest.hexdigest()}


def capture_tools(root,node,npm,chrome):
    """Bind approved local tools; a failure exposes only its allowlisted phase/reason and exact command status."""
    node=tool_capture_step("resolve-node",lambda:Path(node).resolve(strict=True))
    npm=tool_capture_step("resolve-npm",lambda:Path(npm).resolve(strict=True))
    chrome=tool_capture_step("resolve-chrome",lambda:Path(chrome).resolve(strict=True))
    if str(chrome)!="/opt/google/chrome/chrome":raise ToolCaptureError("chrome-engine","chrome-path")
    def check_engine():
        """Read only the existing four-byte ELF gate before any command; no launcher fallback or engine download is allowed."""
        with chrome.open("rb") as stream:
            if stream.read(4)!=b"\x7fELF":raise ValueError("chrome-engine")
    tool_capture_step("chrome-engine",check_engine)
    environment=clean_environment(node,chrome);observations=[]
    for step,arguments in (("node-version",[node,"--version"]),("npm-version",[node,npm,"--version"]),("chrome-version",[chrome,"--version"]),("package-probe",[node,root/"scripts/workspace_browser_tool_probe.cjs"])):
        row=tool_capture_step(step,lambda:command([str(item) for item in arguments],root,30,environment))
        if row["failure"] is not None:raise ToolCaptureError(step,row["failure"] if row["failure"] in COMMAND_FAILURES else "execution-unverified",row["exit_code"])
        if row["exit_code"]!=0:raise ToolCaptureError(step,"tool-exit-nonzero",row["exit_code"])
        observations.append(row["output"])
    if observations[0].strip()!=b"v24.19.0":raise ToolCaptureError("node-version","tool-version",0)
    if observations[1].strip()!=b"11.17.0":raise ToolCaptureError("npm-version","tool-version",0)
    match=re.fullmatch(rb"Google Chrome ([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+)\s*",observations[2])
    if not match:raise ToolCaptureError("chrome-version","chrome-product",0)
    packages=tool_capture_step("package-validation",lambda:validate_packages(strict_json(observations[3])),0)
    node_pin=tool_capture_step("node-pin",lambda:shared.hash_file(node))
    npm_pin=tool_capture_step("npm-pin",lambda:shared.hash_file(npm))
    chrome_pin=tool_capture_step("chrome-pin",lambda:large_file_pin(chrome))
    return {"node":{"version":"24.19.0",**node_pin},"npm":{"version":"11.17.0",**npm_pin},
            "chrome":{"product":"Google Chrome","version":match[1].decode(),"executable_path":"/opt/google/chrome/chrome",**chrome_pin},"packages":packages}


def make_fixture(project,name):
    """Preserve exact baseline default/long read-only/writable fixture semantics before real UI operations."""
    if name not in CAMPAIGNS:raise ValueError("campaign")
    read_only=name.endswith("read-only");long=name.startswith("long-")
    (project/"policy.md").write_text("# Synthetic policy\n\n## Access\n\n- Operators must review the supplied clause.\n")
    (project/"framework.json").write_text(json.dumps(synthetic_framework_catalog()))
    if read_only or long:
        label=synthetic_long_project_label() if long else "Synthetic read-only project <script>"
        resources=[{"key":"policy","role":"policy-source","path":"policy.md"},{"key":"framework","role":"oscal-catalog-artifact","path":"framework.json"}] if read_only else []
        (project/"forge.workspace.json").write_text(json.dumps({"schema_version":"forge.workspace/1","label":label,"resources":resources}))
    return read_only,long


def safe_observation(value,name,source,tools):
    """Retain only bounded numeric/hash browser facts after unchanged real CJS assertions and child exit zero."""
    closed(value,("mode","browserVersion","embeddedAssetSha256","embeddedStyleSha256","inputBorderContrast","focusChecks","reflowChecks","syntheticFaults","captureConsumerObservation","metadataConsumerObservation","unlockRequests","documentedRequests","nonLoopbackRequests","pageErrors"))
    expected_mode="read-only" if name.endswith("read-only") else "writable"
    if value["mode"]!=expected_mode or value["browserVersion"]!=tools["chrome"]["version"]:raise ValueError("browser-binding")
    for field,key in (("embeddedAssetSha256","ui/workspace.js"),("embeddedStyleSha256","ui/workspace.css")):
        if value[field]!=source[key]["sha256"]:raise ValueError("asset-binding")
    if type(value["nonLoopbackRequests"]) is not int or value["nonLoopbackRequests"]!=0 or type(value["pageErrors"]) is not int or value["pageErrors"]!=0:raise ValueError("browser-error")
    if not isinstance(value["documentedRequests"],list) or not isinstance(value["syntheticFaults"],list):raise ValueError("browser-denominator")
    metadata=value["metadataConsumerObservation"]
    expected=2 if expected_mode=="read-only" else 6
    if integer(metadata["previewRegistrations"],1000)!=expected or integer(metadata["comparisonExpected"],1000)!=expected or integer(metadata["strictDuplicateStatus"],599)!=400:raise ValueError("metadata-denominator")
    download=integer(metadata["localDownloadBytes"],1048565,1)
    if integer(metadata["rawEnvelopeBytes"],1048576)!=download+11 or not re.fullmatch("[0-9a-f]{64}",metadata["localDownloadSha256"]):raise ValueError("download-binding")
    widths=[]
    rows=value["reflowChecks"]
    if not isinstance(rows,list) or len(rows)!=2:raise ValueError("reflow-denominator")
    if name.startswith("long-"):
        if not isinstance(metadata.get("longContent"),dict):raise ValueError("long-fixture")
        rows=metadata["longContent"]["metadataReflow"]+rows
    for row in rows:
        viewport=integer(row["viewportWidth"],640,320);page=integer(row["pageWidth"],640,1)
        if viewport not in {320,640} or page>viewport:raise ValueError("page-overflow")
        widths.append({"viewport":viewport,"page":page})
    if [row["viewport"] for row in widths]!=[640,320]*(3 if name.startswith("long-") else 1):raise ValueError("reflow-denominator")
    captures=value["captureConsumerObservation"]
    capture_counts={key:integer(captures[key]) for key in ("actualCounterResponses","distinctRenderedCounterFacts","responseFactsWithoutRenderedObservation")}
    if any(capture_counts[key]>capture_counts["actualCounterResponses"] for key in ("distinctRenderedCounterFacts","responseFactsWithoutRenderedObservation")):raise ValueError("capture-denominator")
    return {"mode":expected_mode,"long_fixture":name.startswith("long-"),"focus_checks":integer(value["focusChecks"],1000,1),"unlock_requests":integer(value["unlockRequests"],1000,1),
            "documented_request_count":integer(len(value["documentedRequests"]),1000,1),"synthetic_ui_fault_count":integer(len(value["syntheticFaults"]),100),
            "reflow":widths,"metadata":{"expected_resources":expected,"local_download_bytes":download,"local_download_sha256":metadata["localDownloadSha256"],"raw_envelope_bytes":download+11,"strict_duplicate_status":400},
            "capture_counts":capture_counts,"non_loopback_requests":0,"page_errors":0}


def terminal_read(terminal,state):
    """Bound full-lifetime terminal bytes and reject an echoed synthetic credential across chunk boundaries."""
    try:block=os.read(terminal,8192)
    except OSError as error:
        if error.errno==5:state["eof"]=True;return b""
        raise
    if not block:state["eof"]=True;return b""
    state["bytes"]+=len(block)
    if state["bytes"]>65536:raise ValueError("terminal-bound")
    combined=state["tail"]+block
    if PASSPHRASE in combined:raise ValueError("terminal-echo")
    state["tail"]=combined[-len(PASSPHRASE):]
    return block


def private_size(directory):
    """Bound observed private artifact totals at polling fences; this is not an OS aggregate disk quota."""
    count=total=0
    for parent,dirs,files in os.walk(directory,followlinks=False):
        count+=len(dirs)+len(files)
        if count>10000:raise ValueError("private-file-bound")
        for name in files:
            try:total+=(Path(parent)/name).lstat().st_size
            except FileNotFoundError:continue
            if total>134217728:raise ValueError("private-byte-bound")
    return total


def file_limits():
    """Apply a Linux per-file write ceiling to owned runtime children without changing build-tool limits."""
    import resource
    resource.setrlimit(resource.RLIMIT_FSIZE,(16777216,16777216))


def terminal_submit(terminal):
    """Wait for actual no-echo mode before one complete synthetic submission; partial writes fail closed."""
    import termios
    deadline=time.monotonic()+2
    while termios.tcgetattr(terminal)[3]&termios.ECHO:
        if time.monotonic()>=deadline:raise ValueError("terminal-echo-mode")
        time.sleep(.01)
    if os.write(terminal,PASSPHRASE+b"\n")!=len(PASSPHRASE)+1:raise ValueError("terminal-write")


def exec_forge_child(forge,project,read_only):
    """Replace only the forked child; every failed setup/exec exits immediately without controller cleanup or publication."""
    try:
        file_limits()
        arguments=[str(forge),"workspace","--project",str(project),"--no-open"]
        if read_only:arguments.append("--read-only")
        os.execv(str(forge),arguments)
    finally:os._exit(127)


def failure_code(error):
    """Publish only fixed producer-owned phase codes; arbitrary exception text and CJS stderr remain private."""
    allowed={"startup-timeout","startup-closed","terminal-bound","terminal-echo","terminal-echo-mode","terminal-write","prompt-denominator","browser-timeout","browser-failed","browser-output-bound","cleanup-unverified","forge-shutdown-timeout","child-exit-unverified","private-file-bound","private-byte-bound","owned-process-bound","process-scan-bound","process-stat-bound","process-visibility-unverified","asset-binding","browser-binding","browser-error","browser-denominator","download-binding","long-fixture","chrome-binding","reflow-denominator","reflow-order","page-overflow","capture-denominator","metadata-denominator","raw-envelope","json-bound","duplicate-field","nonfinite","counter-bound","closed-object"}
    if type(error) is ValueError and len(error.args)==1 and type(error.args[0]) is str and error.args[0] in allowed:return error.args[0]
    return "campaign-unverified"


def campaign(root,forge,node,chrome,name,source,tools):
    """Run one actual fixture with bounded PTY/pipes, natural zero exits and an observed empty owned descendant tree."""
    import pty
    import termios
    tree=OwnedTree();pid=None;terminal=None;process=None;forge_code=None;stdout=bytearray();stderr_bytes=0
    state={"bytes":0,"tail":b"","eof":False};row={"name":name,"status":"failed","failure":"campaign-unverified","observation":None,"cleanup":{"forge_exit_zero":False,"node_exit_zero":False,"tree_empty":False,"terminal_eof":False,"forced":False},"terminal_bytes":0,"no_echo_modes":0,"browser_failure":None}
    with tempfile.TemporaryDirectory(prefix="forge-hosted-chrome-") as private:
        project=Path(private)/"project";project.mkdir();read_only,long=make_fixture(project,name)
        try:
            pid,terminal=pty.fork()
            if pid==0:exec_forge_child(forge,project,read_only)
            tree.track_child(pid);os.set_blocking(terminal,False);startup=bytearray();step=0;url=None;deadline=time.monotonic()+30
            while url is None:
                tree.observe()
                if time.monotonic()>=deadline:raise ValueError("startup-timeout")
                if select.select([terminal],[],[],.05)[0]:
                    startup.extend(terminal_read(terminal,state))
                    if state["eof"]:raise ValueError("startup-closed")
                    prompt=b"Set workspace passphrase" if step==0 else b"Confirm passphrase:"
                    if step<2 and prompt in startup:
                        # Read the actual PTY mode immediately before each credential write.
                        terminal_submit(terminal);step+=1;row["no_echo_modes"]+=1;startup.clear()
                    match=re.search(rb"Local workspace: (http://127\.0\.0\.1:[0-9]+)",startup)
                    if match:url=match[1].decode()
            if step!=2:raise ValueError("prompt-denominator")
            environment=clean_environment(node,chrome,Path(private)/"private-screenshot.png",long)
            process=subprocess.Popen([str(node),str(root/"ui/tests/workspace.cjs"),url,"read-only" if read_only else "writable"],cwd=root,env=environment,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True,preexec_fn=file_limits)
            tree.track_process(process)
            for stream in (process.stdout,process.stderr):os.set_blocking(stream.fileno(),False)
            open_streams={process.stdout,process.stderr};deadline=time.monotonic()+240;next_private_check=time.monotonic()
            while process.poll() is None or open_streams:
                tree.observe()
                if time.monotonic()>=next_private_check:
                    private_size(private);next_private_check=time.monotonic()+1
                if time.monotonic()>=deadline:raise ValueError("browser-timeout")
                descriptors=list(open_streams)+([] if state["eof"] else [terminal])
                for stream in select.select(descriptors,[],[],.05)[0]:
                    if stream==terminal:terminal_read(terminal,state);continue
                    block=os.read(stream.fileno(),8192)
                    if not block:open_streams.remove(stream);continue
                    if stream==process.stdout:stdout.extend(block)
                    else:stderr_bytes+=len(block)
                    if len(stdout)+stderr_bytes>MAX_OUTPUT:raise ValueError("browser-output-bound")
            if process.returncode!=0:
                row["browser_failure"]=decode_browser_failure(stdout,process.returncode)
                raise ValueError("browser-failed")
            # Linux-only producer: drain the complete PTY stream and require observed EOF before closing.
            deadline=time.monotonic()+5
            while forge_code is None or not state["eof"]:
                if forge_code is None:
                    forge_code=tree.poll_child(pid)
                if not state["eof"] and select.select([terminal],[],[],.02)[0]:terminal_read(terminal,state)
                tree.observe()
                if time.monotonic()>=deadline:raise ValueError("forge-shutdown-timeout")
            os.close(terminal);terminal=None
            if forge_code!=0 or not tree.settle():raise ValueError("cleanup-unverified")
            private_size(private)
            observation=safe_observation(strict_json(stdout),name,source,tools)
            row.update(status="passed",failure=None,observation=observation,cleanup={"forge_exit_zero":True,"node_exit_zero":True,"tree_empty":True,"terminal_eof":True,"forced":False})
        except Exception as error:
            row["failure"]=failure_code(error)
            if terminal is not None:os.close(terminal);terminal=None
            row["cleanup"]["terminal_eof"]=state["eof"]
            row["cleanup"]["forced"]=True
            try:row["cleanup"]["tree_empty"]=tree.stop()
            except Exception:row["cleanup"]["tree_empty"]=False
            if process is not None:
                try:process.wait(timeout=5)
                except subprocess.TimeoutExpired:pass
                row["cleanup"]["node_exit_zero"]=process.returncode==0
            if pid is not None:
                try:
                    forge_code=tree.poll_child(pid)
                except ValueError:pass
                row["cleanup"]["forge_exit_zero"]=forge_code==0
        finally:
            tree.close()
            if terminal is not None:os.close(terminal)
            if process is not None:
                for stream in (process.stdout,process.stderr):stream.close()
            row["terminal_bytes"]=state["bytes"]
    return row


def produce(root,forge,node,npm,chrome,output_dir):
    """Publish a closed four-campaign result; missing tools, unsupported hosts and failures never become pass."""
    receipt={"schema_version":SCHEMA,"scope":"installed-google-chrome-linux-prerequisite","truth_state":"synthetic-development","acceptance_eligible":False,"status":"failed","failure":"producer-unverified","tools":None,"input_stability":"unverified","campaigns":[]}
    try:
        if sys.platform!="linux":receipt.update(status="incomplete",failure="unsupported-platform")
        else:
            before={name:shared.hash_file(root/name) for name in CAPTURE_INPUTS}
            before["provided_release_binary"]=shared.hash_file(forge);tools=capture_tools(root,node,npm,chrome);receipt["tools"]=tools
            receipt["campaigns"]=[campaign(root,forge,node,chrome,name,before,tools) for name in CAMPAIGNS]
            after={name:shared.hash_file(root/name) for name in before if name!="provided_release_binary"};after["provided_release_binary"]=shared.hash_file(forge)
            tools_after=capture_tools(root,node,npm,chrome)
            receipt["input_stability"]="unchanged" if before==after and tools==tools_after else "changed"
            receipt["status"]="passed" if receipt["input_stability"]=="unchanged" and all(row["status"]=="passed" for row in receipt["campaigns"]) else "failed"
            receipt["failure"]=None if receipt["status"]=="passed" else "campaign-or-input-unverified"
    except Exception:receipt.update(status="failed",failure="producer-unverified")
    destination=Path(output_dir);destination.mkdir(parents=True,exist_ok=True)
    shared.atomic_receipt(destination/OUTPUT,receipt)
    return receipt


def main():
    """Expose only explicit executable inputs and a fresh result directory; redact all runtime exceptions."""
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ("forge","node","npm","chrome","output-dir"):parser.add_argument("--"+name,required=True,type=Path)
    args=parser.parse_args()
    value=produce(ROOT,args.forge.resolve(),args.node.resolve(),args.npm.resolve(),args.chrome.resolve(),args.output_dir)
    return {"passed":0,"failed":1,"incomplete":2}[value["status"]]


if __name__=="__main__":
    try:exit_code=main()
    except Exception:exit_code=1
    sys.exit(exit_code)
