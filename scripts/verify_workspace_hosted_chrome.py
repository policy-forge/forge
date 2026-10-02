#!/usr/bin/env python3
"""Bind four installed-Chrome campaigns to an approved local tool graph, exact checkout and release bytes."""
import argparse
import json
import os
from pathlib import Path
import stat
import sys
import tempfile

import verify_workspace as shared
import test_workspace_hosted_chrome as browser

sys.dont_write_bytecode=True
SCHEMA="forge.workspace-hosted-chrome-verification/2"
OUTPUT="workspace-hosted-chrome-verification.json"
EXTRA_INPUTS=("ui/tests/workspace.cjs","ui/package.json","ui/package-lock.json","scripts/workspace_browser_fixtures.py","scripts/workspace_browser_tool_probe.cjs","scripts/test_workspace_hosted_chrome.py","scripts/verify_workspace_hosted_chrome.py","scripts/test_verify_workspace_hosted_chrome.py","docs/development-tools/hosted-chrome.json")


def read_bytes(path):
    """Capture bounded unchanged regular receipt bytes, rejecting links and special files before opening."""
    path=Path(path);initial=path.lstat()
    if not stat.S_ISREG(initial.st_mode) or path.is_symlink():raise ValueError("receipt-regular")
    descriptor=os.open(path,os.O_RDONLY|getattr(os,"O_NOFOLLOW",0)|getattr(os,"O_NONBLOCK",0))
    try:
        before=os.fstat(descriptor)
        def identity(item):
            """Compare bounded descriptor/path metadata without reading arbitrary replacement content."""
            return (item.st_dev,item.st_ino,item.st_size,item.st_mtime_ns,item.st_ctime_ns)
        if not stat.S_ISREG(before.st_mode) or before.st_size>262144 or identity(initial)!=identity(before):raise ValueError("receipt-bound")
        with os.fdopen(descriptor,"rb",closefd=False) as stream:raw=stream.read(262145)
        if not raw or len(raw)>262144 or identity(before)!=identity(os.fstat(descriptor)) or identity(before)!=identity(path.lstat()):raise ValueError("receipt-changed")
        return raw
    finally:os.close(descriptor)


def read_receipt(path,exit_code,tools):
    """Reject partial, contradictory or passed-looking nonzero producer evidence while retaining safe failed/incomplete rows."""
    raw=read_bytes(path);value=browser.strict_json(raw)
    browser.closed(value,("schema_version","scope","truth_state","acceptance_eligible","status","failure","tools","input_stability","campaigns"))
    if value["schema_version"]!=browser.SCHEMA or value["scope"]!="installed-google-chrome-linux-prerequisite" or value["truth_state"]!="synthetic-development" or value["acceptance_eligible"] is not False:raise ValueError("scope")
    expected_exit={"passed":0,"failed":1,"incomplete":2}
    if value["status"] not in expected_exit or type(exit_code) is not int or exit_code!=expected_exit[value["status"]]:raise ValueError("producer-exit")
    if value["input_stability"] not in {"unchanged","changed","unverified"}:raise ValueError("stability")
    if value["failure"] not in {None,"unsupported-platform","producer-unverified","campaign-or-input-unverified"}:raise ValueError("failure")
    if value["tools"] is not None and value["tools"]!=tools:raise ValueError("tool-binding")
    rows=value["campaigns"]
    if not isinstance(rows,list) or len(rows) not in {0,4}:raise ValueError("campaign-denominator")
    if rows and [row.get("name") for row in rows]!=list(browser.CAMPAIGNS):raise ValueError("campaign-order")
    for row in rows:
        browser.closed(row,("name","status","failure","observation","cleanup","terminal_bytes","no_echo_modes"))
        browser.closed(row["cleanup"],("forge_exit_zero","node_exit_zero","tree_empty","terminal_eof","forced"))
        if any(type(flag) is not bool for flag in row["cleanup"].values()):raise ValueError("cleanup-type")
        browser.integer(row["terminal_bytes"],73728);browser.integer(row["no_echo_modes"],2)
        if row["status"] not in {"passed","failed"} or row["failure"] is not None and browser.failure_code(ValueError(row["failure"]))!=row["failure"]:raise ValueError("campaign-status")
        if row["status"]=="passed":
            if row["failure"] is not None or row["cleanup"]!={"forge_exit_zero":True,"node_exit_zero":True,"tree_empty":True,"terminal_eof":True,"forced":False} or row["terminal_bytes"]>65536 or row["no_echo_modes"]!=2:raise ValueError("campaign-cleanup")
            observation=row["observation"]
            browser.closed(observation,("mode","long_fixture","focus_checks","unlock_requests","documented_request_count","synthetic_ui_fault_count","reflow","metadata","capture_counts","non_loopback_requests","page_errors"))
            if observation["mode"]!=("read-only" if row["name"].endswith("read-only") else "writable") or observation["long_fixture"] is not row["name"].startswith("long-"):raise ValueError("campaign-fixture")
            for key in ("focus_checks","unlock_requests","documented_request_count"):browser.integer(observation[key],1000,1)
            browser.integer(observation["synthetic_ui_fault_count"],100)
            if type(observation["non_loopback_requests"]) is not int or observation["non_loopback_requests"]!=0 or type(observation["page_errors"]) is not int or observation["page_errors"]!=0:raise ValueError("browser-errors")
            browser.closed(observation["metadata"],("expected_resources","local_download_bytes","local_download_sha256","raw_envelope_bytes","strict_duplicate_status"))
            expected=2 if observation["mode"]=="read-only" else 6
            if browser.integer(observation["metadata"]["expected_resources"],1000)!=expected or browser.integer(observation["metadata"]["strict_duplicate_status"],599)!=400:raise ValueError("metadata-count")
            local=browser.integer(observation["metadata"]["local_download_bytes"],1048565,1)
            if browser.integer(observation["metadata"]["raw_envelope_bytes"],1048576)!=local+11:raise ValueError("raw-envelope")
            if not browser.re.fullmatch("[0-9a-f]{64}",observation["metadata"]["local_download_sha256"]):raise ValueError("download-pin")
            browser.closed(observation["capture_counts"],("actualCounterResponses","distinctRenderedCounterFacts","responseFactsWithoutRenderedObservation"))
            for count in observation["capture_counts"].values():browser.integer(count)
            if any(observation["capture_counts"][key]>observation["capture_counts"]["actualCounterResponses"] for key in ("distinctRenderedCounterFacts","responseFactsWithoutRenderedObservation")):raise ValueError("capture-denominator")
            widths=observation["reflow"]
            if not isinstance(widths,list) or len(widths)!=(6 if observation["long_fixture"] else 2):raise ValueError("reflow-count")
            if [width.get("viewport") for width in widths]!=([640,320]* (3 if observation["long_fixture"] else 1)):raise ValueError("reflow-order")
            for width in widths:
                browser.closed(width,("viewport","page"));browser.integer(width["viewport"],640,320);browser.integer(width["page"],640,1)
                if width["page"]>width["viewport"]:raise ValueError("page-overflow")
        elif row["failure"] is None or row["observation"] is not None:raise ValueError("failed-observation")
    if value["status"]=="passed" and (value["failure"] is not None or value["tools"]!=tools or value["input_stability"]!="unchanged" or len(rows)!=4 or any(row["status"]!="passed" for row in rows)):raise ValueError("pass-denominator")
    if value["status"]=="incomplete" and (value["failure"]!="unsupported-platform" or rows or value["tools"] is not None):raise ValueError("unsupported-observation")
    if value["status"]=="failed" and value["failure"] is None:raise ValueError("failed-outcome")
    return value,{"bytes":len(raw),"sha256":shared.hashlib.sha256(raw).hexdigest()}


def capture_identity(root,forge,timeout):
    """Supplement exact shared Git/source capture without extending its closed three-suite API /2 receipt."""
    identity=shared.capture_identity(root,forge,timeout)
    identity["inputs"].update({name:shared.hash_file(root/name) for name in EXTRA_INPUTS})
    return identity


def native_run(root,forge,node,npm,chrome,tools):
    """Run the one predefined bounded producer in fresh private storage, discarding all arbitrary child output."""
    with tempfile.TemporaryDirectory(prefix="forge-hosted-chrome-receipt-") as private:
        environment=browser.clean_environment(node,chrome)
        completed=browser.command([sys.executable,"-B",str(root/"scripts/test_workspace_hosted_chrome.py"),"--forge",str(forge),"--node",str(node),"--npm",str(npm),"--chrome",str(chrome),"--output-dir",private],root,1200,environment)
        row={"status":"failed","producer_exit_code":completed["exit_code"],"failure":"producer-unverified","receipt":None,"receipt_pin":None}
        if completed["failure"] is None:
            try:
                value,pin=read_receipt(Path(private)/browser.OUTPUT,completed["exit_code"],tools)
                row.update(status=value["status"],failure=value["failure"],receipt=value,receipt_pin=pin)
            except Exception:row["failure"]="invalid-producer-receipt"
        return row


def verify(root,forge,node,npm,chrome,output_dir,expected_commit=None,build_outcome="unrecorded",npm_outcome="unrecorded",*,event="local",checkout_kind="local",requested_head=None,requested_base=None):
    """Reconcile source/tool/release/checkout and provisioning assertions before giving four-campaign prerequisite credit."""
    if build_outcome not in shared.BUILD_OUTCOMES or npm_outcome not in shared.BUILD_OUTCOMES:raise ValueError("outcome")
    destination=Path(output_dir)
    if destination.exists() and (not destination.is_dir() or any(destination.iterdir())):raise ValueError("fresh-output")
    destination.mkdir(parents=True,exist_ok=True)
    receipt={"schema_version":SCHEMA,"scope":"f04-installed-chrome-prerequisite","truth_state":"synthetic-development","acceptance_eligible":False,"status":"failed","failure":"verification-unverified","diagnostic":None,"identity":None,"checkout":None,"tools":None,"browser_tools":None,"input_stability":"unverified","tool_stability":"unverified","producer":{"status":"not-run","producer_exit_code":None,"failure":"not-run","receipt":None,"receipt_pin":None},"build":{"outcome":build_outcome,"profile":"release","features":"default","locked":True,"offline":True,"binding":"workflow-step-assertion"},"npm":{"outcome":npm_outcome,"ignore_scripts":True,"omit_optional":True,"browser_downloads":False,"binding":"workflow-step-assertion"},"pending_gates":list(shared.PENDING)}
    try:
        before=capture_identity(root,forge,30);receipt["identity"]=before
        tools=shared.tool_versions(root,30);receipt["tools"]=tools
        receipt["checkout"]=shared.checkout_binding(before,expected_commit,event=event,checkout_kind=checkout_kind,requested_head=requested_head,requested_base=requested_base)
        if not before["tracked_source_clean"]:receipt["failure"]="tracked-source-unclean"
        elif any(tools.get(name) is None for name in ("cargo","rustc","rust_host")):receipt["failure"]="tool-identity-unavailable"
        elif sys.platform!="linux":receipt.update(status="incomplete",failure="unsupported-platform")
        elif build_outcome!="success" or npm_outcome!="success":
            receipt["status"]="failed" if "failure" in (build_outcome,npm_outcome) or "cancelled" in (build_outcome,npm_outcome) else "incomplete"
            receipt["failure"]="provisioning-not-qualified"
        else:
            browser_tools=browser.capture_tools(root,node,npm,chrome);receipt["browser_tools"]=browser_tools
            receipt["producer"]=native_run(root,forge,node,npm,chrome,browser_tools)
            receipt["status"]=receipt["producer"]["status"];receipt["failure"]=receipt["producer"]["failure"]
        after=capture_identity(root,forge,30);tools_after=shared.tool_versions(root,30)
        browser_after=browser.capture_tools(root,node,npm,chrome) if receipt["browser_tools"] is not None else None
        receipt["input_stability"]="unchanged" if before==after else "changed"
        receipt["tool_stability"]="unchanged" if tools==tools_after and receipt["browser_tools"]==browser_after else "changed"
        if receipt["input_stability"]!="unchanged" or receipt["tool_stability"]!="unchanged":receipt.update(status="failed",failure="verification-input-changed")
    except browser.ToolCaptureError as error:
        try:diagnostic=browser.tool_diagnostic(error)
        except Exception:receipt.update(status="failed",failure="verification-input-invalid")
        else:receipt.update(status="failed",failure="browser-tool-capture-failed",diagnostic=diagnostic)
    except Exception:receipt.update(status="failed",failure="verification-input-invalid")
    shared.atomic_receipt(destination/OUTPUT,receipt)
    return receipt


def main():
    """Expose explicit executables and hosted context; fixed public diagnostics cannot contain raw failures or paths."""
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ("forge","node","npm","chrome","output-dir"):parser.add_argument("--"+name,required=True,type=Path)
    parser.add_argument("--expected-commit");parser.add_argument("--requested-head");parser.add_argument("--requested-base")
    parser.add_argument("--event",choices=("local","push","pull_request","workflow_dispatch"),default="local")
    parser.add_argument("--checkout-kind",choices=("local","head","pull-request-merge"),default="local")
    for name in ("build-outcome","npm-outcome"):parser.add_argument("--"+name,choices=shared.BUILD_OUTCOMES,default="unrecorded")
    args=parser.parse_args()
    try:value=verify(Path(__file__).resolve().parents[1],args.forge.resolve(),args.node.resolve(),args.npm.resolve(),args.chrome.resolve(),args.output_dir,args.expected_commit,args.build_outcome,args.npm_outcome,event=args.event,checkout_kind=args.checkout_kind,requested_head=args.requested_head,requested_base=args.requested_base)
    except Exception:
        print("Hosted Chrome verification could not publish a fresh receipt.",file=sys.stderr);return 2
    print("Hosted Chrome prerequisite: "+value["status"]+"; acceptance gates remain open.")
    return {"passed":0,"incomplete":1,"failed":2}[value["status"]]


if __name__=="__main__":sys.exit(main())
