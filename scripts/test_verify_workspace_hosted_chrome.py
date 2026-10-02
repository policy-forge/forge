#!/usr/bin/env python3
"""Synthetic stdlib hosted-Chrome controls; never launch a browser, Forge or installed-tool probe."""
import copy
import io
import json
import os
import re
from pathlib import Path
import tempfile
import unittest
from unittest import mock
import verify_workspace_hosted_chrome as wrapper
import test_workspace_hosted_chrome as browser


class HostedChromeControls(unittest.TestCase):
    """Use real bounded artifacts and explicitly mocked native/tool outcomes; this is not hosted acceptance."""
    def setUp(self):
        """Create isolated fake source/release bytes and immutable expectation copies for every test."""
        self.private=tempfile.TemporaryDirectory(prefix="forge-chrome-controls-");self.addCleanup(self.private.cleanup)
        self.root=Path(self.private.name);self.forge=self.root/"fake-forge";self.forge.write_bytes(b"synthetic-release")
        for name in wrapper.EXTRA_INPUTS:
            path=self.root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_text("synthetic:"+name)
        for name in ("ui/workspace.js","ui/workspace.css","ui/tests/workspace.cjs","scripts/test_workspace_hosted_chrome.py","scripts/workspace_browser_tool_probe.cjs","scripts/workspace_browser_fixtures.py"):
            path=self.root/name;path.parent.mkdir(parents=True,exist_ok=True)
            if not path.exists():path.write_text("synthetic:"+name)
        self.pin={"bytes":5,"sha256":"a"*64};self.tools={"node":{"version":"24.19.0",**self.pin},"npm":{"version":"11.17.0",**self.pin},"chrome":{"product":"Google Chrome","version":"150.1.2.3","executable_path":"/opt/google/chrome/chrome",**self.pin},"packages":self.package_fixture()}
        self.identity={"source_commit":"a"*40,"ordered_parents":[],"tracked_source_clean":True,"provided_release_binary":wrapper.shared.hash_file(self.forge),"inputs":{}}
        self.rust={"cargo":{"version":"1.99.0"},"rustc":{"version":"1.99.0"},"rust_host":"x86_64-unknown-linux-gnu"}
        self.output=self.root/"result"

    def package_fixture(self):
        """Build synthetic approved-local package facts with distinct hash expectations and no installed modules."""
        packages=[]
        for name in ("playwright","playwright-core"):
            packages.append({"name":name,"version":"1.62.1","relative_path":"ui/node_modules/"+name,"manifest":copy.deepcopy(self.pin),"entry":copy.deepcopy(self.pin),"files":1,"bytes":5,"tree_sha256":"b"*64})
        return {"schema_version":"forge.hosted-chrome-tools/1","node":"24.19.0","packages":packages,"optional_omitted":True,"installed_lock":copy.deepcopy(self.pin)}

    def campaign_fixture(self,name):
        """Author explicitly synthetic closed pass rows with the actual two/six fixture resource denominators."""
        long=name.startswith("long-");read_only=name.endswith("read-only");expected=2 if read_only else 6
        return {"name":name,"status":"passed","failure":None,"observation":{"mode":"read-only" if read_only else "writable","long_fixture":long,"focus_checks":10,"unlock_requests":3,"documented_request_count":10,"synthetic_ui_fault_count":1,"reflow":[{"viewport":width,"page":width} for width in ([640,320]*(3 if long else 1))],"metadata":{"expected_resources":expected,"local_download_bytes":100,"local_download_sha256":"c"*64,"raw_envelope_bytes":111,"strict_duplicate_status":400},"capture_counts":{"actualCounterResponses":0,"distinctRenderedCounterFacts":0,"responseFactsWithoutRenderedObservation":0},"non_loopback_requests":0,"page_errors":0},"cleanup":{"forge_exit_zero":True,"node_exit_zero":True,"tree_empty":True,"terminal_eof":True,"forced":False},"terminal_bytes":100,"no_echo_modes":2,"browser_failure":None}

    def producer_fixture(self,status="passed"):
        """Return a fresh independent expectation tree so malformed cases cannot taint later controls."""
        value={"schema_version":browser.SCHEMA,"scope":"installed-google-chrome-linux-prerequisite","truth_state":"synthetic-development","acceptance_eligible":False,"status":status,"failure":None,"tools":copy.deepcopy(self.tools),"input_stability":"unchanged","campaigns":[self.campaign_fixture(name) for name in browser.CAMPAIGNS]}
        if status=="failed":value.update(failure="campaign-or-input-unverified");value["campaigns"][0].update(status="failed",failure="campaign-unverified",observation=None)
        if status=="incomplete":value.update(failure="unsupported-platform",tools=None,input_stability="unverified",campaigns=[])
        return value

    def read(self,value,code=0):
        """Pass real bounded bytes through the closed receipt reader using synthetic tool expectations."""
        path=self.root/"producer.json";path.write_bytes(wrapper.shared.canonical_bytes(value))
        return wrapper.read_receipt(path,code,self.tools)

    def run_outer(self,*,status="passed",code=0,build="success",npm="success",after=None,tools_after=None,dirty=False,missing=False,command_failure=None):
        """Mock all external executions while retaining real source hashing, strict receipt reading and publication."""
        before=copy.deepcopy(self.identity);before["tracked_source_clean"]=not dirty
        def producer(arguments,root,timeout,environment):
            """Create only a synthetic artifact; never invoke a tool, browser or Forge subprocess."""
            private=Path(arguments[arguments.index("--output-dir")+1])
            if not missing:(private/browser.OUTPUT).write_bytes(wrapper.shared.canonical_bytes(self.producer_fixture(status)))
            return {"exit_code":code,"failure":command_failure,"output":b"PRIVATE exception path token"}
        with mock.patch.object(wrapper.sys,"platform","linux"),mock.patch.object(wrapper.shared,"capture_identity",side_effect=[before,copy.deepcopy(before if after is None else after)]),mock.patch.object(wrapper.shared,"tool_versions",return_value=self.rust),mock.patch.object(browser,"capture_tools",side_effect=[self.tools,self.tools if tools_after is None else tools_after]),mock.patch.object(browser,"command",side_effect=producer) as command:
            value=wrapper.verify(self.root,self.forge,self.root/"node",self.root/"npm",self.root/"chrome",self.output,build_outcome=build,npm_outcome=npm)
        return value,command.call_count

    def test_four_campaign_pass_preserves_scope(self):
        """Exercise outer reconciliation without native credit and require all four fixed synthetic campaign slots."""
        value,count=self.run_outer();self.assertEqual(count,1);self.assertEqual(value["status"],"passed")
        self.assertFalse(value["acceptance_eligible"]);self.assertEqual(len(value["producer"]["receipt"]["campaigns"]),4)
        self.assertNotIn("PRIVATE",(self.output/wrapper.OUTPUT).read_text());self.assertEqual(value["input_stability"],"unchanged")

    def test_producer_exit_reconciliation(self):
        """A passed-looking artifact with nonzero exit, missing file or failed child drainage never earns pass."""
        for index,arguments in enumerate(({"code":1},{"code":None},{"missing":True},{"command_failure":"execution-unverified"})):
            self.output=self.root/f"exit-{index}";value,_=self.run_outer(**arguments)
            self.assertEqual(value["status"],"failed");self.assertIsNone(value["producer"]["receipt"])

    def test_failed_and_incomplete_are_distinct(self):
        """Retain closed failed/unsupported evidence without treating either producer outcome as acceptance."""
        for status,code in (("failed",1),("incomplete",2)):
            self.output=self.root/status;value,_=self.run_outer(status=status,code=code)
            self.assertEqual(value["status"],status);self.assertFalse(value["acceptance_eligible"])

    def test_build_install_and_dirty_preconditions(self):
        """Missing or failed build/install assertions and dirty sources prevent any browser producer launch."""
        for index,arguments in enumerate(({"dirty":True},{"build":"unrecorded"},{"build":"failure"},{"npm":"skipped"},{"npm":"cancelled"})):
            self.output=self.root/f"pre-{index}";value,count=self.run_outer(**arguments)
            self.assertNotEqual(value["status"],"passed");self.assertEqual(count,0)

    def test_source_and_tool_drift_revoke_pass(self):
        """Post-run source and browser-tree differences override even a synthetic producer pass."""
        after=copy.deepcopy(self.identity);after["source_commit"]="b"*40
        value,_=self.run_outer(after=after);self.assertEqual(value["status"],"failed")
        self.output=self.root/"tool-drift";changed=copy.deepcopy(self.tools);changed["packages"]["packages"][0]["tree_sha256"]="d"*64
        value,_=self.run_outer(tools_after=changed);self.assertEqual(value["status"],"failed")

    def test_campaign_order_and_complete_denominator(self):
        """Reject prefix success, duplicate/reordered slots and partial cleanup instead of shrinking the four-campaign gate."""
        for kind in ("prefix","order","duplicate","cleanup","echo","forced","terminal"):
            value=self.producer_fixture()
            if kind=="prefix":value["campaigns"].pop()
            elif kind=="order":value["campaigns"].reverse()
            elif kind=="duplicate":value["campaigns"][1]=copy.deepcopy(value["campaigns"][0])
            elif kind=="cleanup":value["campaigns"][0]["cleanup"]["tree_empty"]=False
            elif kind=="echo":value["campaigns"][0]["no_echo_modes"]=1
            elif kind=="forced":value["campaigns"][0]["cleanup"]["forced"]=True
            else:value["campaigns"][0]["cleanup"]["terminal_eof"]=False
            with self.subTest(kind=kind),self.assertRaises(ValueError):self.read(value)

    def test_closed_keys_and_exact_integer_types(self):
        """Reject raw private additions and bool/float counter substitutions in final published evidence."""
        for location in ("root","row","observation","metadata","cleanup"):
            value=self.producer_fixture();row=value["campaigns"][0]
            target={"root":value,"row":row,"observation":row["observation"],"metadata":row["observation"]["metadata"],"cleanup":row["cleanup"]}[location]
            target["private"]="SECRET"
            with self.assertRaises(ValueError):self.read(value)
        for count in (True,10.0,-1):
            value=self.producer_fixture();value["campaigns"][0]["observation"]["focus_checks"]=count
            with self.assertRaises(ValueError):self.read(value)

    def test_reflow_and_raw_envelope_denominators(self):
        """Reject overflowing global page widths, missing long-width observations and collapsed raw envelope facts."""
        for kind in ("overflow","missing-long","envelope","resources","hash"):
            value=self.producer_fixture();row=value["campaigns"][2];observation=row["observation"]
            if kind=="overflow":observation["reflow"][1]["page"]=321
            elif kind=="missing-long":observation["reflow"].pop()
            elif kind=="envelope":observation["metadata"]["raw_envelope_bytes"]=110
            elif kind=="resources":observation["metadata"]["expected_resources"]=6
            else:observation["metadata"]["local_download_sha256"]="PRIVATE"
            with self.subTest(kind=kind),self.assertRaises(ValueError):self.read(value)

    def test_unsupported_cannot_invent_observations(self):
        """Unsupported-platform receipts have no tools or campaign observations and cannot claim native success."""
        value=self.producer_fixture("incomplete");value["campaigns"]=[self.campaign_fixture(browser.CAMPAIGNS[0])]
        with self.assertRaises(ValueError):self.read(value,2)
        with mock.patch.object(browser.sys,"platform","unsupported"),mock.patch.object(browser,"capture_tools") as tools:
            result=browser.produce(self.root,self.forge,self.root/"node",self.root/"npm",self.root/"chrome",self.root/"unsupported")
        self.assertEqual(result["status"],"incomplete");tools.assert_not_called()

    def test_raw_duplicate_nonfinite_size_and_special_files(self):
        """Reject decoded duplicate JSON keys, nonfinite values, oversized artifacts and nonregular input before open."""
        raw=wrapper.shared.canonical_bytes(self.producer_fixture())
        path=self.root/"malformed"
        for data in (raw.replace(b'"focus_checks":10',b'"focus_checks":10,"focus_chec\\u006bs":10'),raw.replace(b'"terminal_bytes":100',b'"terminal_bytes":NaN'),b" "*262145):
            path.write_bytes(data)
            with self.assertRaises(ValueError):wrapper.read_receipt(path,0,self.tools)
        with mock.patch.object(wrapper.os,"open") as opened,self.assertRaises(ValueError):wrapper.read_bytes(self.root)
        opened.assert_not_called()

    def test_fixture_isolation(self):
        """Malformed fixture mutation cannot alter tool expectations or later synthetic valid receipts."""
        original=copy.deepcopy(self.tools);bad=self.producer_fixture();bad["tools"]["chrome"]["sha256"]="f"*64
        self.assertEqual(self.tools,original)
        with self.assertRaises(ValueError):self.read(bad)
        value,_=self.read(self.producer_fixture());self.assertEqual(value["tools"],original)

    def test_fixture_semantics_and_long_environment(self):
        """Preserve all four actual authored index/policy/catalog inputs and clear inherited long/global tool overrides."""
        for name in browser.CAMPAIGNS:
            project=self.root/name;project.mkdir();read_only,long=browser.make_fixture(project,name)
            catalog=json.loads((project/"framework.json").read_text());self.assertEqual(len(catalog["catalog"]["controls"]),53)
            index=project/"forge.workspace.json";self.assertEqual(index.exists(),read_only or long)
            if index.exists():
                value=json.loads(index.read_text());self.assertEqual(len(value["resources"]),2 if read_only else 0)
                self.assertEqual(len(value["label"]),200 if long else len("Synthetic read-only project <script>"))
        with mock.patch.dict(os.environ,{"NODE_PATH":"GLOBAL","NODE_OPTIONS":"PRIVATE","PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH":"fallback","FORGE_TEST_LONG_METADATA":"1","FORGE_TEST_SCREENSHOT":"/PRIVATE"}):
            environment=browser.clean_environment(self.root/"node",self.root/"chrome",self.root/"private/screenshot.png")
        self.assertNotIn("NODE_PATH",environment);self.assertNotIn("NODE_OPTIONS",environment);self.assertNotIn("FORGE_TEST_LONG_METADATA",environment)
        self.assertEqual(environment["FORGE_TEST_BROWSER_EXECUTABLE"],str(self.root/"chrome"));self.assertEqual(environment["TMPDIR"],str(self.root/"private"))

    def test_local_graph_must_be_exact(self):
        """Reject wrong versions/global locations, optional native packages and malformed installed tree counters."""
        for kind in ("version","path","optional","order","count"):
            value=self.package_fixture()
            if kind=="version":value["packages"][0]["version"]="1.62.0"
            elif kind=="path":value["packages"][0]["relative_path"]="GLOBAL"
            elif kind=="optional":value["optional_omitted"]=False
            elif kind=="order":value["packages"].reverse()
            else:value["packages"][0]["files"]=True
            with self.subTest(kind=kind),self.assertRaises(ValueError):browser.validate_packages(value)
        self.assertEqual(browser.validate_packages(self.package_fixture())["node"],"24.19.0")

    def test_terminal_echo_overflow_eof_and_short_submission(self):
        """Exercise cross-chunk no-echo/full-stream bounds and require actual no-echo mode before complete writes."""
        state={"bytes":0,"tail":b"","eof":False}
        with mock.patch.object(browser.os,"read",side_effect=[browser.PASSPHRASE[:20],browser.PASSPHRASE[20:]]):
            browser.terminal_read(1,state)
            with self.assertRaises(ValueError):browser.terminal_read(1,state)
        state={"bytes":65536,"tail":b"","eof":False}
        with mock.patch.object(browser.os,"read",return_value=b"x"),self.assertRaises(ValueError):browser.terminal_read(1,state)
        state={"bytes":0,"tail":b"","eof":False}
        with mock.patch.object(browser.os,"read",side_effect=OSError(5,"PRIVATE")):browser.terminal_read(1,state)
        self.assertTrue(state["eof"])
        import termios
        with mock.patch.object(termios,"tcgetattr",return_value=[0,0,0,0]),mock.patch.object(browser.os,"write",return_value=1),self.assertRaises(ValueError):browser.terminal_submit(1)
        with mock.patch.object(termios,"tcgetattr",return_value=[0,0,0,0]),mock.patch.object(browser.os,"write",return_value=len(browser.PASSPHRASE)+1):browser.terminal_submit(1)

    def test_private_artifact_size_and_streamed_pin(self):
        """Bound actual temporary metadata sizes and require stable regular engine bytes, without launching Chrome."""
        directory=self.root/"files";directory.mkdir();(directory/"sample").write_bytes(b"12345")
        self.assertEqual(browser.private_size(directory),5)
        self.assertEqual(browser.large_file_pin(directory/"sample")["bytes"],5)
        with self.assertRaises(ValueError):browser.large_file_pin(directory)

    def test_owned_pidfd_cleanup_does_not_signal_recycled_identity(self):
        """Mock native process observation and prove a vanished/replaced identity is never signaled through its pidfd."""
        tree=browser.OwnedTree.__new__(browser.OwnedTree);tree.forced=False;tree.processes={};tree.children=set();tree.exits={};tree.seen={};tree.handles={}
        with mock.patch.object(tree,"observe",side_effect=[{12:99},{},{}]),mock.patch.object(tree,"reap"),mock.patch.object(browser.os,"pidfd_open",return_value=7,create=True),mock.patch.object(browser.os,"close"),mock.patch.object(browser.signal,"pidfd_send_signal",create=True) as sent:
            self.assertTrue(tree.stop());sent.assert_not_called()
        self.assertTrue(tree.forced)

    def test_scan_failure_still_cleans_direct_child_without_false_empty_credit(self):
        """Mock a permanently unavailable process scan while retaining sole wait ownership and actual nonzero child status."""
        tree=browser.OwnedTree.__new__(browser.OwnedTree);tree.forced=False;tree.processes={};tree.children=set();tree.exits={};tree.seen={};tree.handles={}
        process=mock.Mock(pid=11,returncode=None)
        def poll():
            """Expose the direct child's current observed status without generic reaping or an invented zero."""
            return process.returncode
        def killed(*_arguments):
            """Model only the owned direct child's actual SIGKILL status after a pidfd signal."""
            process.returncode=-9
        process.poll.side_effect=poll;tree.track_process(process)
        with mock.patch.object(tree,"observe",side_effect=ValueError("process-scan-bound")),mock.patch.object(browser.os,"pidfd_open",return_value=7,create=True),mock.patch.object(browser.os,"close"),mock.patch.object(browser.signal,"pidfd_send_signal",side_effect=killed,create=True) as sent,mock.patch.object(browser.time,"monotonic",side_effect=[0,6]):
            self.assertFalse(tree.stop())
        sent.assert_called_once_with(7,browser.signal.SIGKILL);self.assertEqual(process.returncode,-9);self.assertTrue(tree.forced)

    def test_retained_verified_pidfds_survive_later_scan_failure(self):
        """Mock genuine observed ancestry, then a failed global scan; exact held descendant instances are still signaled."""
        tree=browser.OwnedTree.__new__(browser.OwnedTree);tree.owner=100;tree.forced=False;tree.processes={};tree.children=set();tree.exits={};tree.seen={};tree.handles={}
        entries=[mock.Mock(name="entry") for _ in range(2)]
        entries[0].name="101";entries[1].name="102"
        scan=mock.MagicMock();scan.__enter__.return_value=iter(entries)
        identities={101:(100,10),102:(101,20)}
        with mock.patch.object(browser.os,"scandir",return_value=scan),mock.patch.object(browser,"process_identity",side_effect=lambda pid:identities[pid]),mock.patch.object(browser.os,"pidfd_open",side_effect=[7,8],create=True):
            self.assertEqual(tree.observe(),{101:10,102:20})
        self.assertEqual(tree.handles,{101:7,102:8})
        with mock.patch.object(tree,"observe",side_effect=ValueError("process-scan-bound")),mock.patch.object(tree,"reap"),mock.patch.object(browser.signal,"pidfd_send_signal",create=True) as sent,mock.patch.object(browser.time,"monotonic",side_effect=[0,6]):self.assertFalse(tree.stop())
        self.assertEqual(sent.call_args_list,[mock.call(7,browser.signal.SIGKILL),mock.call(8,browser.signal.SIGKILL)])
        with mock.patch.object(browser.os,"close") as closed:tree.close()
        self.assertEqual(closed.call_args_list,[mock.call(7),mock.call(8)]);self.assertEqual(tree.handles,{})
        with mock.patch("builtins.open",side_effect=FileNotFoundError):self.assertIsNone(browser.process_identity(101))

    def cjs_fixture(self,name="default-read-only"):
        """Build a bounded synthetic CJS success envelope for redacted projection controls only."""
        long=name.startswith("long-");read_only=name.endswith("read-only");expected=2 if read_only else 6
        metadata={"previewRegistrations":expected,"comparisonExpected":expected,"localDownloadBytes":100,"localDownloadSha256":"c"*64,"rawEnvelopeBytes":111,"strictDuplicateStatus":400}
        if long:metadata["longContent"]={"metadataReflow":[{"viewportWidth":width,"pageWidth":width} for width in [640,320]*2]}
        return {"mode":"read-only" if read_only else "writable","browserVersion":"150.1.2.3","embeddedAssetSha256":"d"*64,"embeddedStyleSha256":"e"*64,"inputBorderContrast":{},"focusChecks":10,"reflowChecks":[{"viewportWidth":640,"pageWidth":640},{"viewportWidth":320,"pageWidth":320}],"syntheticFaults":[{"message":"PRIVATE"}],"captureConsumerObservation":{"actualCounterResponses":0,"distinctRenderedCounterFacts":0,"responseFactsWithoutRenderedObservation":0},"metadataConsumerObservation":metadata,"unlockRequests":3,"documentedRequests":["PRIVATE"],"nonLoopbackRequests":0,"pageErrors":0}

    def test_actual_cjs_long_reflow_shape_requires_all_six_observations(self):
        """Model unchanged CJS preview and selected-file measurements plus final widths; reject incomplete/extra evidence."""
        source={"ui/workspace.js":{"sha256":"d"*64},"ui/workspace.css":{"sha256":"e"*64}}
        for name in ("long-read-only","long-writable"):
            value=self.cjs_fixture(name)
            self.assertEqual(len(value["metadataConsumerObservation"]["longContent"]["metadataReflow"]),4)
            observed=browser.safe_observation(value,name,source,self.tools)
            self.assertEqual([row["viewport"] for row in observed["reflow"]],[640,320]*3)
            for count in (2,3,5):
                bad=copy.deepcopy(value);bad["metadataConsumerObservation"]["longContent"]["metadataReflow"]=bad["metadataConsumerObservation"]["longContent"]["metadataReflow"][:count] if count<4 else bad["metadataConsumerObservation"]["longContent"]["metadataReflow"]+[{"viewportWidth":640,"pageWidth":640}]
                with self.subTest(name=name,count=count),self.assertRaises(ValueError):browser.safe_observation(bad,name,source,self.tools)

    def test_failed_fork_exec_always_exits_without_controller_return(self):
        """Mock setup and exec failures and require the child-only exit fence rather than receipt publication."""
        for setup_failure in (True,False):
            with mock.patch.object(browser,"file_limits",side_effect=ValueError("PRIVATE") if setup_failure else None),mock.patch.object(browser.os,"execv",side_effect=OSError("PRIVATE")) as executed,mock.patch.object(browser.os,"_exit",side_effect=SystemExit(127)) as exited:
                with self.assertRaises(SystemExit) as result:browser.exec_forge_child(self.forge,self.root,True)
            self.assertEqual(result.exception.code,127);exited.assert_called_once_with(127)
            self.assertEqual(executed.call_count,0 if setup_failure else 1)

    def test_allowlisted_phase_codes_never_publish_arbitrary_exception_text(self):
        """Keep actionable producer phases but redact unrecognized ValueErrors and arbitrary exception classes."""
        self.assertEqual(browser.failure_code(ValueError("startup-timeout")),"startup-timeout")
        self.assertEqual(browser.failure_code(ValueError("PRIVATE path token")),"campaign-unverified")
        self.assertEqual(browser.failure_code(OSError("startup-timeout")),"campaign-unverified")
        value=self.producer_fixture("failed");value["campaigns"][0]["failure"]="browser-failed"
        self.assertEqual(self.read(value,1)[0]["campaigns"][0]["failure"],"browser-failed")
        value["campaigns"][0]["failure"]="PRIVATE"
        with self.assertRaises(ValueError):self.read(value,1)

    def test_reaping_preserves_direct_child_nonzero_and_unknown_status(self):
        """Mock killed direct children and prove adopted-child reaping never invents Popen or PTY success."""
        tree=browser.OwnedTree.__new__(browser.OwnedTree);tree.processes={};tree.children=set();tree.exits={};tree.seen={11:1,12:2,13:3};tree.handles={}
        process=mock.Mock(pid=11,returncode=None)
        def actual_status():
            """Represent the direct owner's real SIGKILL status instead of CPython's unknown-child fallback."""
            process.returncode=-9
            return -9
        process.poll.side_effect=actual_status;tree.track_process(process);tree.track_child(12)
        with mock.patch.object(browser.os,"waitpid",side_effect=[(12,9),(13,0)]) as waited:tree.reap()
        self.assertEqual(process.returncode,-9);self.assertEqual(tree.poll_child(12),-9)
        self.assertEqual([call.args[0] for call in waited.call_args_list],[12,13])
        tree.track_child(14)
        with mock.patch.object(browser.os,"waitpid",side_effect=ChildProcessError),self.assertRaisesRegex(ValueError,"child-exit-unverified"):tree.poll_child(14)
        self.assertNotIn(14,tree.exits)

    def test_cjs_projection_never_retains_raw_private_fields(self):
        """Project only bounded numeric/hash facts; synthetic messages/route strings never enter retained observation."""
        source={"ui/workspace.js":{"sha256":"d"*64},"ui/workspace.css":{"sha256":"e"*64}}
        observation=browser.safe_observation(self.cjs_fixture(),"default-read-only",source,self.tools)
        self.assertNotIn("PRIVATE",json.dumps(observation));self.assertEqual(observation["metadata"]["expected_resources"],2)
        for kind in ("browser","asset","counter","capture","envelope"):
            value=self.cjs_fixture()
            if kind=="browser":value["browserVersion"]="OTHER"
            elif kind=="asset":value["embeddedStyleSha256"]="f"*64
            elif kind=="counter":value["focusChecks"]=True
            elif kind=="capture":value["captureConsumerObservation"]["distinctRenderedCounterFacts"]=1
            else:value["metadataConsumerObservation"]["rawEnvelopeBytes"]=110
            with self.subTest(kind=kind),self.assertRaises(ValueError):browser.safe_observation(value,"default-read-only",source,self.tools)

    def test_true_google_chrome_engine_and_exact_tools_required(self):
        """Reject a launcher/global tool path and wrong exact Node/npm versions before any browser launch."""
        for filename in ("node","npm","shell-launcher"):(self.root/filename).write_text("synthetic")
        with self.assertRaises(ValueError):browser.capture_tools(self.root,self.root/"node",self.root/"npm",self.root/"shell-launcher")
        chrome=Path("/opt/google/chrome/chrome")
        responses=[{"exit_code":0,"failure":None,"output":row} for row in (b"v24.18.0\n",b"11.17.0\n",b"Google Chrome 150.1.2.3\n",wrapper.shared.canonical_bytes(self.package_fixture()))]
        with mock.patch.object(Path,"resolve",side_effect=lambda **_kwargs:chrome),mock.patch.object(Path,"open",return_value=io.BytesIO(b"\x7fELF")),mock.patch.object(browser,"command",side_effect=responses),self.assertRaises(ValueError):browser.capture_tools(self.root,chrome,chrome,chrome)

    def test_final_capture_counts_cannot_exceed_responses(self):
        """Reject invented DOM facts in a passed-looking receipt; zero actual counters explicitly supplies no capture proof."""
        value=self.producer_fixture();value["campaigns"][0]["observation"]["capture_counts"]["distinctRenderedCounterFacts"]=1
        with self.assertRaises(ValueError):self.read(value)
        actual,_=self.read(self.producer_fixture());self.assertEqual(actual["campaigns"][0]["observation"]["capture_counts"]["actualCounterResponses"],0)

    def test_hosted_wrong_parent_context_stops_before_producer(self):
        """Wrong synthetic merge-parent order cannot downgrade to local checkout or invoke any native work."""
        before=copy.deepcopy(self.identity);before["ordered_parents"]=["b"*40,"c"*40]
        with mock.patch.object(wrapper.shared,"capture_identity",return_value=before),mock.patch.object(wrapper.shared,"tool_versions",return_value=self.rust),mock.patch.object(browser,"capture_tools") as tools:
            value=wrapper.verify(self.root,self.forge,self.root/"node",self.root/"npm",self.root/"chrome",self.output,expected_commit="a"*40,build_outcome="success",npm_outcome="success",event="pull_request",checkout_kind="pull-request-merge",requested_head="b"*40,requested_base="c"*40)
        self.assertEqual(value["status"],"failed");tools.assert_not_called()

    def test_producer_drift_and_partial_campaign_failure(self):
        """A failed row remains in the full four-slot producer result, and source/tool drift revokes complete synthetic success."""
        rows=[self.campaign_fixture(name) for name in browser.CAMPAIGNS];rows[1].update(status="failed",failure="campaign-unverified",observation=None)
        with mock.patch.object(browser.sys,"platform","linux"),mock.patch.object(browser,"capture_tools",return_value=self.tools),mock.patch.object(browser,"campaign",side_effect=rows):
            value=browser.produce(self.root,self.forge,self.root/"node",self.root/"npm",self.root/"chrome",self.root/"producer-failure")
        self.assertEqual(value["status"],"failed");self.assertEqual(len(value["campaigns"]),4)
        changed=copy.deepcopy(self.tools);changed["chrome"]["sha256"]="f"*64
        with mock.patch.object(browser.sys,"platform","linux"),mock.patch.object(browser,"capture_tools",side_effect=[self.tools,changed]),mock.patch.object(browser,"campaign",side_effect=[self.campaign_fixture(name) for name in browser.CAMPAIGNS]):
            value=browser.produce(self.root,self.forge,self.root/"node",self.root/"npm",self.root/"chrome",self.root/"producer-drift")
        self.assertEqual(value["status"],"failed");self.assertEqual(value["input_stability"],"changed")

    def test_public_helper_main_unsupported_is_fixed_nonzero(self):
        """Unsupported helper invocation produces no native claims and returns its distinct incomplete exit mapping."""
        arguments=["helper","--forge",str(self.forge),"--node","node","--npm","npm","--chrome","chrome","--output-dir",str(self.root/"cli-unsupported")]
        with mock.patch.object(browser.sys,"argv",arguments),mock.patch.object(browser,"produce",return_value={"status":"incomplete"}) as produced:self.assertEqual(browser.main(),2)
        self.assertEqual(produced.call_args.args[-1],self.root/"cli-unsupported")

    def test_no_replacement_and_publication_failure_are_nonzero(self):
        """Existing evidence is never replaced, and arbitrary publisher failure text never enters public diagnostics."""
        self.output.mkdir();sentinel=self.output/wrapper.OUTPUT;sentinel.write_bytes(b"sentinel")
        with self.assertRaises(ValueError):wrapper.verify(self.root,self.forge,self.root/"node",self.root/"npm",self.root/"chrome",self.output)
        self.assertEqual(sentinel.read_bytes(),b"sentinel")
        arguments=["verify","--forge",str(self.forge),"--node","node","--npm","npm","--chrome","chrome","--output-dir",str(self.output)]
        with mock.patch.object(wrapper.sys,"argv",arguments),mock.patch.object(wrapper,"verify",side_effect=OSError("PRIVATE")),mock.patch("sys.stderr",new_callable=io.StringIO) as diagnostic:
            self.assertEqual(wrapper.main(),2);self.assertNotIn("PRIVATE",diagnostic.getvalue())

    def test_main_forwards_hosted_context(self):
        """Forward all hash/ordered-parent context and provisioning outcomes rather than silently granting local credit."""
        arguments=["verify","--forge",str(self.forge),"--node","node","--npm","npm","--chrome","chrome","--output-dir",str(self.output),"--expected-commit","a"*40,"--requested-head","b"*40,"--requested-base","c"*40,"--event","pull_request","--checkout-kind","pull-request-merge","--build-outcome","success","--npm-outcome","success"]
        with mock.patch.object(wrapper.sys,"argv",arguments),mock.patch.object(wrapper,"verify",return_value={"status":"passed"}) as verify,mock.patch("sys.stdout",new_callable=io.StringIO):self.assertEqual(wrapper.main(),0)
        self.assertEqual(verify.call_args.kwargs,{"event":"pull_request","checkout_kind":"pull-request-merge","requested_head":"b"*40,"requested_base":"c"*40})
        self.assertEqual(verify.call_args.args[6:],("a"*40,"success","success"))


    def tool_command_rows(self):
        """Create fresh mocked successful command outputs; these are not installed-tool observations."""
        return [{"exit_code":0,"failure":None,"output":value} for value in (b"v24.19.0\n",b"11.17.0\n",b"Google Chrome 150.1.2.3\n",wrapper.shared.canonical_bytes(self.package_fixture()))]

    def capture_with_mocked_tools(self,rows):
        """Mock every executable/path/hash/command boundary while exercising the actual closed tool-capture control flow."""
        chrome=Path("/opt/google/chrome/chrome")
        with mock.patch.object(Path,"resolve",side_effect=lambda **_kwargs:chrome),mock.patch.object(Path,"open",side_effect=lambda *_args,**_kwargs:io.BytesIO(b"\x7fELF")),mock.patch.object(browser,"command",side_effect=rows),mock.patch.object(browser.shared,"hash_file",return_value=self.pin),mock.patch.object(browser,"large_file_pin",return_value=self.pin):
            return browser.capture_tools(self.root,chrome,chrome,chrome)

    def test_tool_diagnostics_are_closed_exact_and_output_free(self):
        """Require exact safe keys and signed integer statuses; reject arbitrary strings, bools and out-of-range values."""
        for code in (None,-(2**31),-9,0,127,2**31-1):
            value=browser.tool_diagnostic(browser.ToolCaptureError("node-version","tool-exit-nonzero",code))
            self.assertEqual(value,{"phase":"browser-tool-capture","step":"node-version","reason":"tool-exit-nonzero","exit_code":code})
        for step,reason,code in (("PRIVATE","tool-version",0),("node-version","PRIVATE",0),("node-version","tool-version",True),("node-version","tool-version",1.0),("node-version","tool-version",2**31),("node-version","tool-version",-(2**31)-1)):
            with self.subTest(step=step,reason=reason,code=code),self.assertRaises(ValueError):browser.ToolCaptureError(step,reason,code)
        self.assertEqual(str(browser.ToolCaptureError("package-probe","output-bound",-9)),"tool-capture-failed")

    def test_each_tool_command_failure_keeps_phase_and_actual_exit(self):
        """Distinguish all four command preflights and actual nonzero/signal exits without retaining private output."""
        for index,step in enumerate(("node-version","npm-version","chrome-version","package-probe")):
            for code,failure,expected in ((23,None,"tool-exit-nonzero"),(-9,"command-timeout","command-timeout"),(None,"pidfd-unavailable","pidfd-unavailable")):
                rows=self.tool_command_rows();rows[index]={"exit_code":code,"failure":failure,"output":b"PRIVATE path token"}
                with self.subTest(step=step,code=code),self.assertRaises(browser.ToolCaptureError) as caught:self.capture_with_mocked_tools(rows)
                value=browser.tool_diagnostic(caught.exception)
                self.assertEqual((value["step"],value["reason"],value["exit_code"]),(step,expected,code));self.assertNotIn("PRIVATE",json.dumps(value))

    def test_tool_output_validation_keeps_zero_exit_and_exact_phase(self):
        """Successful commands with wrong versions/product or malformed package JSON still fail at their own gate."""
        for index,step,raw,reason in ((0,"node-version",b"v24.18.0\n","tool-version"),(1,"npm-version",b"11.16.0\n","tool-version"),(2,"chrome-version",b"Chromium PRIVATE\n","chrome-product"),(3,"package-validation",b'{"PRIVATE":',"tool-capture-unverified")):
            rows=self.tool_command_rows();rows[index]["output"]=raw
            with self.subTest(step=step),self.assertRaises(browser.ToolCaptureError) as caught:self.capture_with_mocked_tools(rows)
            self.assertEqual(browser.tool_diagnostic(caught.exception),{"phase":"browser-tool-capture","step":step,"reason":reason,"exit_code":0})
        self.assertEqual(self.capture_with_mocked_tools(self.tool_command_rows()),self.tools)

    def test_tool_resolution_engine_and_pin_failures_are_phase_bound(self):
        """Expose no path/error text for missing tools, wrong ELF bytes and changed final hash input."""
        chrome=Path("/opt/google/chrome/chrome")
        for index,step in enumerate(("resolve-node","resolve-npm","resolve-chrome")):
            resolved=[chrome]*3;resolved[index]=FileNotFoundError("PRIVATE path token")
            with mock.patch.object(Path,"resolve",side_effect=resolved),self.assertRaises(browser.ToolCaptureError) as caught:browser.capture_tools(self.root,chrome,chrome,chrome)
            self.assertEqual(browser.tool_diagnostic(caught.exception),{"phase":"browser-tool-capture","step":step,"reason":"executable-unavailable","exit_code":None})
        with mock.patch.object(Path,"resolve",return_value=chrome),mock.patch.object(Path,"open",return_value=io.BytesIO(b"text")),self.assertRaises(browser.ToolCaptureError) as caught:browser.capture_tools(self.root,chrome,chrome,chrome)
        self.assertEqual(browser.tool_diagnostic(caught.exception)["reason"],"chrome-engine")
        with mock.patch.object(Path,"resolve",return_value=chrome),mock.patch.object(Path,"open",return_value=io.BytesIO(b"\x7fELF")),mock.patch.object(browser,"command",side_effect=self.tool_command_rows()),mock.patch.object(browser.shared,"hash_file",side_effect=OSError("PRIVATE path token")),self.assertRaises(browser.ToolCaptureError) as caught:browser.capture_tools(self.root,chrome,chrome,chrome)
        self.assertEqual(browser.tool_diagnostic(caught.exception),{"phase":"browser-tool-capture","step":"node-pin","reason":"tool-capture-unverified","exit_code":None})

    def test_command_unavailable_native_lifecycle_is_typed_before_spawn(self):
        """Mock unavailable Linux facilities before Popen; null status records no command execution or native proof."""
        for reason in ("subreaper-unavailable","pidfd-unavailable"):
            with mock.patch.object(browser,"OwnedTree",side_effect=ValueError(reason)),mock.patch.object(browser.subprocess,"Popen") as spawn:
                row=browser.command(["PRIVATE"],self.root,1,{})
            self.assertEqual(row,{"exit_code":None,"failure":reason,"output":b""});spawn.assert_not_called()
        with mock.patch.object(browser.ctypes,"CDLL",return_value=object()),self.assertRaisesRegex(ValueError,"subreaper-unavailable"):browser.OwnedTree()
        for error in (RuntimeError("subreaper-unavailable PRIVATE"),ValueError("PRIVATE path token"),OSError("output-bound")):
            self.assertEqual(browser.command_failure(error),"execution-unverified")

    def test_command_failure_discards_bytes_and_preserves_forced_exit(self):
        """Mock actual child ownership/cleanup status; forced cleanup remains failed and cannot manufacture a zero exit."""
        for reason,clean in (("process-scan-bound",True),("command-timeout",True),("output-bound",True),("child-exit-unverified",False)):
            tree=mock.Mock();tree.observe.side_effect=ValueError(reason);tree.stop.return_value=clean
            process=mock.Mock();process.returncode=None;process.poll.return_value=None;process.stdout.fileno.return_value=10
            def waited(timeout):
                """Model only the direct owner's observed SIGKILL wait result after cleanup; no real process is launched."""
                process.returncode=-9;return -9
            process.wait.side_effect=waited
            with mock.patch.object(browser,"OwnedTree",return_value=tree),mock.patch.object(browser.subprocess,"Popen",return_value=process),mock.patch.object(browser.os,"set_blocking"):
                row=browser.command(["PRIVATE"],self.root,1,{})
            self.assertEqual(row,{"exit_code":-9,"failure":reason if clean else "cleanup-unverified","output":b""});tree.stop.assert_called_once();tree.close.assert_called_once();process.stdout.close.assert_called_once()
        self.assertEqual(browser.command_failure(ValueError("process-visibility-unverified")),"process-visibility-unverified")

    def test_nonzero_command_and_cleanup_failure_cannot_pass(self):
        """Keep a completed child's real nonzero exit, and discard bytes when natural cleanup or handle closure fails."""
        for clean,close_failure in ((True,False),(False,False),(True,True)):
            tree=mock.Mock();tree.settle.return_value=clean
            if close_failure:tree.close.side_effect=OSError("PRIVATE path token")
            process=mock.Mock(returncode=17);process.poll.return_value=17;process.stdout.fileno.return_value=10;process.stderr.fileno.return_value=11
            with mock.patch.object(browser,"OwnedTree",return_value=tree),mock.patch.object(browser.subprocess,"Popen",return_value=process),mock.patch.object(browser.os,"set_blocking"),mock.patch.object(browser.select,"select",side_effect=[([process.stdout,process.stderr],[],[]),([process.stdout],[],[])]),mock.patch.object(browser.os,"read",side_effect=[b"PRIVATE output",b"",b""]):
                row=browser.command(["PRIVATE"],self.root,1,{})
            self.assertEqual(row["exit_code"],17)
            if clean and not close_failure:self.assertEqual(row,{"exit_code":17,"failure":None,"output":b"PRIVATE output"})
            else:self.assertEqual(row,{"exit_code":17,"failure":"cleanup-unverified","output":b""})

    def test_wrapper_v2_preflight_diagnostic_never_starts_producer(self):
        """Publish a typed tool error with no producer/campaign credit; arbitrary errors and tampered records stay redacted."""
        for index,error in enumerate((browser.ToolCaptureError("package-probe","tool-exit-nonzero",23),RuntimeError("PRIVATE path token"),browser.ToolCaptureError("node-version","command-timeout",-9))):
            if index==2:error.reason="PRIVATE path token"
            destination=self.root/f"diagnostic-{index}"
            with mock.patch.object(wrapper.sys,"platform","linux"),mock.patch.object(wrapper.shared,"capture_identity",return_value=self.identity),mock.patch.object(wrapper.shared,"tool_versions",return_value=self.rust),mock.patch.object(browser,"capture_tools",side_effect=error),mock.patch.object(wrapper,"native_run") as native:
                value=wrapper.verify(self.root,self.forge,self.root/"node",self.root/"npm",self.root/"chrome",destination,build_outcome="success",npm_outcome="success")
            self.assertEqual(value["schema_version"],"forge.workspace-hosted-chrome-verification/3");self.assertEqual(value["status"],"failed");native.assert_not_called()
            self.assertEqual(value["producer"]["status"],"not-run");self.assertIsNone(value["browser_tools"]);self.assertNotIn("PRIVATE",(destination/wrapper.OUTPUT).read_text())
            if index==0:self.assertEqual(value["diagnostic"],{"phase":"browser-tool-capture","step":"package-probe","reason":"tool-exit-nonzero","exit_code":23});self.assertEqual(value["failure"],"browser-tool-capture-failed")
            else:self.assertIsNone(value["diagnostic"]);self.assertEqual(value["failure"],"verification-input-invalid")

    def test_after_tool_failure_revokes_synthetic_success(self):
        """A post-campaign typed tool failure preserves history yet revokes outer pass and leaves stability unverified."""
        value,_=self.run_outer(tools_after=browser.ToolCaptureError("chrome-pin","chrome-file-changed"))
        self.assertEqual(value["status"],"failed");self.assertEqual(value["failure"],"browser-tool-capture-failed")
        self.assertEqual(value["producer"]["status"],"passed");self.assertEqual(value["tool_stability"],"unverified")
        self.assertEqual(value["diagnostic"],{"phase":"browser-tool-capture","step":"chrome-pin","reason":"chrome-file-changed","exit_code":None})
        self.output=self.root/"ordinary-success";value,_=self.run_outer();self.assertIsNone(value["diagnostic"]);self.assertEqual(value["status"],"passed")


    def drain_mocked_command(self,events,*,exit_code=0,poll_values=None,timeout=30,clock_step=.001,clean=True,close_failure=None,read_failure=None):
        """Exercise the actual drain with two distinct fake pipes and an advancing clock; never create native processes."""
        tree=mock.Mock();tree.settle.return_value=clean;tree.stop.return_value=True
        process=mock.Mock();process.returncode=exit_code;process.stdout.fileno.return_value=10;process.stderr.fileno.return_value=11
        if poll_values is None:process.poll.return_value=exit_code
        else:process.poll.side_effect=poll_values
        if close_failure=="tree":tree.close.side_effect=OSError("PRIVATE cleanup path")
        elif close_failure is not None:getattr(process,close_failure).close.side_effect=OSError("PRIVATE cleanup path")
        pending=list(events);clock=[-clock_step]
        def monotonic():
            """Advance one deterministic absolute budget; readiness and bytes never reset the deadline."""
            clock[0]+=clock_step;return clock[0]
        def ready(streams,_write,_error,_timeout):
            """Return only a currently open fake pipe scheduled by the test, with empty readiness after its last event."""
            if not pending:return [],[],[]
            descriptor=pending[0][0];matching=[stream for stream in streams if stream.fileno()==descriptor]
            self.assertEqual(len(matching),1);return matching,[],[]
        def read(descriptor,maximum):
            """Consume one real-sized scheduled block on its own descriptor; a mock cannot invent an oversized read."""
            expected,block=pending.pop(0);self.assertEqual(descriptor,expected);self.assertEqual(maximum,8192)
            self.assertLessEqual(len(block),maximum)
            if read_failure is not None and descriptor==11:raise OSError("PRIVATE stderr path token")
            return block
        def waited(_timeout=None,**_kwargs):
            """Observe a mock direct-child exit after failure; a running child becomes signal-terminated, never zero."""
            if process.returncode is None:process.returncode=-9
            return process.returncode
        process.wait.side_effect=waited
        with mock.patch.object(browser,"OwnedTree",return_value=tree),mock.patch.object(browser.subprocess,"Popen",return_value=process) as spawn,mock.patch.object(browser.os,"set_blocking") as blocking,mock.patch.object(browser.select,"select",side_effect=ready),mock.patch.object(browser.os,"read",side_effect=read) as reads,mock.patch.object(browser.time,"monotonic",side_effect=monotonic):
            row=browser.command(["PRIVATE executable"],self.root,timeout,{"SYNTHETIC":"true"})
        return row,tree,process,spawn.call_args.kwargs,blocking.call_args_list,reads.call_args_list

    def bounded_pipe_events(self,descriptor,raw):
        """Split synthetic bytes into blocks no larger than the production read size, then schedule that pipe's own EOF."""
        return [(descriptor,raw[offset:offset+8192]) for offset in range(0,len(raw),8192)]+[(descriptor,b"")]

    def test_dualpipe_warning_cannot_contaminate_version_stdout(self):
        """Drain private stderr separately while the unchanged full-match product parser accepts only actual stdout."""
        version=b"Google Chrome 150.1.2.3\n";warning=b"PRIVATE synthetic channel warning\n"
        row,tree,process,spawn,blocking,reads=self.drain_mocked_command([(11,warning),(10,version),(11,b""),(10,b"")])
        self.assertEqual(row,{"exit_code":0,"failure":None,"output":version});self.assertNotIn(warning,row["output"])
        self.assertEqual(spawn["stdin"],browser.subprocess.DEVNULL);self.assertEqual(spawn["stdout"],browser.subprocess.PIPE);self.assertEqual(spawn["stderr"],browser.subprocess.PIPE);self.assertTrue(spawn["start_new_session"])
        self.assertEqual(blocking,[mock.call(10,False),mock.call(11,False)]);self.assertEqual([call.args[0] for call in reads],[11,10,11,10])
        tree.settle.assert_called_once();tree.stop.assert_not_called();process.stdout.close.assert_called_once();process.stderr.close.assert_called_once()
        rows=self.tool_command_rows();rows[2]=row;self.assertEqual(self.capture_with_mocked_tools(rows),self.tools)

    def test_google_only_on_stderr_cannot_spoof_stdout_brand(self):
        """A zero-exit Chromium stdout stays rejected even when private stderr contains a valid Google product line."""
        row,*_=self.drain_mocked_command([(10,b"Chromium 150.1.2.3\n"),(11,b"Google Chrome 150.1.2.3\n"),(10,b""),(11,b"")])
        self.assertEqual(row["output"],b"Chromium 150.1.2.3\n");rows=self.tool_command_rows();rows[2]=row
        with self.assertRaises(browser.ToolCaptureError) as caught:self.capture_with_mocked_tools(rows)
        self.assertEqual(browser.tool_diagnostic(caught.exception),{"phase":"browser-tool-capture","step":"chrome-version","reason":"chrome-product","exit_code":0})

    def test_aggregate_stdout_stderr_byte_limit_is_inclusive(self):
        """Count discarded stderr toward the same exact byte budget; one extra byte clears all retained stdout."""
        for extra in (0,1):
            events=[(10,b"good")]+self.bounded_pipe_events(11,b"x"*(browser.MAX_OUTPUT-4+extra))+[(10,b"")]
            row,tree,process,*_=self.drain_mocked_command(events)
            self.assertEqual(row,{"exit_code":0,"failure":"output-bound" if extra else None,"output":b"" if extra else b"good"})
            if extra:tree.stop.assert_called_once();process.wait.assert_called_once()
            else:tree.settle.assert_called_once();tree.stop.assert_not_called()
            process.stdout.close.assert_called_once();process.stderr.close.assert_called_once()

    def test_each_pipe_requires_eof_even_after_zero_exit(self):
        """A finished child and one pipe's EOF cannot grant pass while the other pipe remains open without data."""
        for eof_descriptor,waiting in ((10,11),(11,10)):
            row,tree,process,*_=self.drain_mocked_command([(10,b"PRIVATE captured"),(eof_descriptor,b"")],timeout=1,clock_step=.25)
            self.assertEqual(row,{"exit_code":0,"failure":"command-timeout","output":b""});tree.settle.assert_not_called();tree.stop.assert_called_once()
            process.stdout.close.assert_called_once();process.stderr.close.assert_called_once();self.assertIn(waiting,(10,11))

    def test_both_eofs_do_not_infer_child_completion(self):
        """A still-running child remains under the same deadline after both EOFs, and its forced signal exit is retained."""
        row,tree,process,*_=self.drain_mocked_command([(10,b""),(11,b"")],exit_code=None,timeout=1,clock_step=.25)
        self.assertEqual(row,{"exit_code":-9,"failure":"command-timeout","output":b""});tree.stop.assert_called_once();process.wait.assert_called_once();tree.settle.assert_not_called()

    def test_stderr_activity_never_renews_absolute_deadline(self):
        """Continuous small stderr readiness still reaches the original deadline and cannot extend a running command."""
        row,tree,process,*_=self.drain_mocked_command([(11,b"PRIVATE")]*10,exit_code=None,timeout=1,clock_step=.25)
        self.assertEqual(row,{"exit_code":-9,"failure":"command-timeout","output":b""});tree.stop.assert_called_once();process.wait.assert_called_once()

    def test_dualpipe_cleanup_failure_clears_output_and_closes_both(self):
        """Natural tree or any handle-close failure revokes output, while every owned pipe receives its independent close."""
        for clean,close_failure in ((False,None),(True,"tree"),(True,"stdout"),(True,"stderr")):
            row,tree,process,*_=self.drain_mocked_command([(11,b"PRIVATE stderr"),(10,b"PRIVATE stdout"),(10,b""),(11,b"")],clean=clean,close_failure=close_failure)
            self.assertEqual(row,{"exit_code":0,"failure":"cleanup-unverified","output":b""});tree.close.assert_called_once()
            process.stdout.close.assert_called_once();process.stderr.close.assert_called_once()
            if not clean:tree.stop.assert_called_once();process.wait.assert_called_once()
            else:tree.stop.assert_not_called()

    def test_stderr_read_failure_is_redacted_and_cannot_pass(self):
        """An unexpected second-pipe I/O exception discards already-read stdout and publishes no private exception text."""
        row,tree,process,*_=self.drain_mocked_command([(10,b"PRIVATE stdout"),(11,b"PRIVATE stderr")],read_failure=True)
        self.assertEqual(row,{"exit_code":0,"failure":"execution-unverified","output":b""});tree.stop.assert_called_once();process.wait.assert_called_once()
        process.stdout.close.assert_called_once();process.stderr.close.assert_called_once()


    def browser_failure_fixture(self,*,stage="metadata-file-selection",category="assertion",operator="strictEqual"):
        """Provide independent literal fixed protocol facts without importing the CJS diagnostic helper."""
        return {"schema_version":"forge.workspace-browser-failure/1","stage":stage,"category":category,"assertion_operator":operator}

    def failed_browser_receipt(self,diagnostic=None):
        """Keep the complete four-row failed denominator while attaching only explicitly synthetic safe facts."""
        value=self.producer_fixture("failed");row=value["campaigns"][0]
        row.update(failure="browser-failed",browser_failure=diagnostic)
        row["cleanup"].update(node_exit_zero=False,forced=True)
        return value

    def test_browser_failure_decoder_binds_actual_exit_and_fixed_operators(self):
        """Capture declared safe facts for actual nonzero statuses; assert(...) uses the fixed Node '==' operator."""
        for operator in ("strictEqual","deepStrictEqual","match","==",None):
            raw=wrapper.shared.canonical_bytes(self.browser_failure_fixture(operator=operator))
            for code in (1,-9,-2147483648,2147483647):
                expected={"stage":"metadata-file-selection","category":"assertion","assertion_operator":operator,"node_exit_code":code}
                self.assertEqual(browser.decode_browser_failure(raw,code),expected)
        raw=wrapper.shared.canonical_bytes(self.browser_failure_fixture())
        padded=raw+b" "*(1024-len(raw))
        self.assertEqual(browser.decode_browser_failure(padded,1)["node_exit_code"],1)
        self.assertIsNone(browser.decode_browser_failure(padded+b" ",1))
        for category in ("timeout","unclassified"):
            raw=wrapper.shared.canonical_bytes(self.browser_failure_fixture(category=category,operator=None))
            self.assertEqual(browser.decode_browser_failure(raw,3)["category"],category)
        for code in (0,None,True,1.0,-2147483649,2147483648):
            self.assertIsNone(browser.decode_browser_failure(wrapper.shared.canonical_bytes(self.browser_failure_fixture()),code))

    def test_browser_failure_decoder_rejects_open_unknown_and_ambiguous_stdout(self):
        """Do not select a line, last object or arbitrary enum from private/malformed child output."""
        valid=self.browser_failure_fixture();cases=[]
        for field,value in (("schema_version","private/version"),("stage","PRIVATE path token"),("category","SECRET"),("assertion_operator","PRIVATE"),("node_exit_code",7),("private","SECRET")):
            row=copy.deepcopy(valid);row[field]=value;cases.append(wrapper.shared.canonical_bytes(row))
        row=copy.deepcopy(valid);row.update(category="timeout",assertion_operator="match");cases.append(wrapper.shared.canonical_bytes(row))
        for field in valid:
            row=copy.deepcopy(valid);del row[field];cases.append(wrapper.shared.canonical_bytes(row))
        raw=wrapper.shared.canonical_bytes(valid)
        cases.extend((b"",b"PRIVATE warning\n"+raw,raw+raw,raw[:-2],b" "*1025,b"\xff",b"[]",raw.replace(b'"stage":',br'"st\u0061ge":"unlock","stage":'),raw.replace(b'"assertion_operator":"strictEqual"',b'"assertion_operator":NaN')))
        for data in cases:
            with self.subTest(size=len(data)):self.assertIsNone(browser.decode_browser_failure(data,1))

    def test_failed_browser_receipt_retains_safe_diagnostic_without_pass_credit(self):
        """A real strict artifact read can retain the fixed failed-row facts and actual status without passing it."""
        diagnostic={"stage":"metadata-file-selection","category":"assertion","assertion_operator":"==","node_exit_code":-9}
        value,pin=self.read(self.failed_browser_receipt(diagnostic),1)
        self.assertEqual(value["status"],"failed");self.assertEqual(len(value["campaigns"]),4)
        self.assertEqual(value["campaigns"][0]["failure"],"browser-failed")
        self.assertEqual(value["campaigns"][0]["browser_failure"],diagnostic);self.assertIsNone(value["campaigns"][0]["observation"])
        self.assertFalse(value["acceptance_eligible"]);self.assertGreater(pin["bytes"],0)
        for row in self.read(self.producer_fixture())[0]["campaigns"]:self.assertIsNone(row["browser_failure"])

    def test_browser_failure_reader_refuses_contradictory_closed_facts(self):
        """Refuse unknown/private nested facts, false child status, old versions and diagnostics attached to success."""
        diagnostic={"stage":"metadata-file-selection","category":"assertion","assertion_operator":"match","node_exit_code":1}
        for field,wrong in (("stage",None),("stage",{}),("category","PRIVATE"),("assertion_operator","PRIVATE"),("node_exit_code",0),("node_exit_code",True),("node_exit_code",None),("node_exit_code",1.0),("node_exit_code",2147483648),("private","SECRET")):
            row=copy.deepcopy(diagnostic);row[field]=wrong
            with self.subTest(field=field),self.assertRaises(ValueError):self.read(self.failed_browser_receipt(row),1)
        for field in diagnostic:
            row=copy.deepcopy(diagnostic);del row[field]
            with self.subTest(missing=field),self.assertRaises(ValueError):self.read(self.failed_browser_receipt(row),1)
        value=self.failed_browser_receipt(diagnostic);value["campaigns"][0]["cleanup"]["node_exit_zero"]=True
        with self.assertRaises(ValueError):self.read(value,1)
        value=self.failed_browser_receipt(diagnostic);value["campaigns"][0]["failure"]="browser-timeout"
        with self.assertRaises(ValueError):self.read(value,1)
        value=self.producer_fixture();value["campaigns"][0]["browser_failure"]=diagnostic
        with self.assertRaises(ValueError):self.read(value)
        value=self.failed_browser_receipt({**diagnostic,"category":"timeout"})
        with self.assertRaises(ValueError):self.read(value,1)
        value=self.producer_fixture();value["schema_version"]="forge.hosted-chrome-smoke/1"
        with self.assertRaises(ValueError):self.read(value)
        value=self.producer_fixture();del value["campaigns"][0]["browser_failure"]
        with self.assertRaises(ValueError):self.read(value)

    def exercise_failed_browser_campaign(self,raw,*,exit_code=17):
        """Drive the real campaign to nonzero Node completion with fake PTY/pipes/process ownership only."""
        process=mock.Mock();process.returncode=exit_code;process.poll.return_value=exit_code
        process.stdout=mock.Mock();process.stdout.fileno.return_value=10
        process.stderr=mock.Mock();process.stderr.fileno.return_value=11
        tree=mock.Mock();tree.stop.return_value=True;tree.poll_child.return_value=-9
        terminal=[b"Set workspace passphrase (",b"Confirm passphrase:",b"Local workspace: http://127.0.0.1:12345"]
        pipes={10:[raw[index:index+8192] for index in range(0,len(raw),8192)]+[b""],11:[b"SECRET private stderr path token",b""]}
        clock=[0.0]
        def monotonic():
            """Advance a synthetic monotonic clock without waiting or asserting native timing."""
            clock[0]+=.001;return clock[0]
        def terminal_block(_descriptor,state):
            """Supply synthetic startup chunks and count bytes without touching an actual terminal."""
            block=terminal.pop(0);state["bytes"]+=len(block);return block
        def ready(streams,_writes,_errors,_timeout):
            """Expose ready fake pipes; the incomplete Forge terminal becomes idle after startup."""
            return ([stream for stream in streams if stream!=23 or terminal],[],[])
        def read_pipe(descriptor,maximum):
            """Supply bounded fake stdout/stderr bytes to the actual production drain loop."""
            block=pipes[descriptor].pop(0)
            if len(block)>maximum:raise AssertionError("Synthetic block exceeds read bound")
            return block
        with mock.patch("pty.fork",return_value=(24,23)),mock.patch.object(browser,"OwnedTree",return_value=tree),mock.patch.object(browser.subprocess,"Popen",return_value=process) as spawn,mock.patch.object(browser.os,"set_blocking"),mock.patch.object(browser.os,"close") as closed,mock.patch.object(browser,"terminal_read",side_effect=terminal_block),mock.patch.object(browser,"terminal_submit") as submitted,mock.patch.object(browser.select,"select",side_effect=ready),mock.patch.object(browser.os,"read",side_effect=read_pipe),mock.patch.object(browser.time,"monotonic",side_effect=monotonic),mock.patch.object(browser,"clean_environment",return_value={}):
            row=browser.campaign(self.root,self.forge,self.root/"node",self.root/"chrome","long-writable",{},self.tools)
        return row,tree,process,spawn,submitted,closed

    def test_actual_failed_campaign_attaches_only_closed_stdout_and_actual_exit(self):
        """The production campaign consumer retains safe failure facts while its original forced failure stays intact."""
        raw=wrapper.shared.canonical_bytes(self.browser_failure_fixture())
        row,tree,process,spawn,submitted,closed=self.exercise_failed_browser_campaign(raw,exit_code=-9)
        self.assertEqual(row["status"],"failed");self.assertEqual(row["failure"],"browser-failed");self.assertIsNone(row["observation"])
        self.assertEqual(row["browser_failure"],{"stage":"metadata-file-selection","category":"assertion","assertion_operator":"strictEqual","node_exit_code":-9})
        self.assertEqual(row["cleanup"],{"forge_exit_zero":False,"node_exit_zero":False,"tree_empty":True,"terminal_eof":False,"forced":True})
        self.assertNotIn("SECRET",json.dumps(row));self.assertEqual(submitted.call_count,2)
        tree.stop.assert_called_once();tree.close.assert_called_once();process.wait.assert_called_once_with(timeout=5)
        process.stdout.close.assert_called_once();process.stderr.close.assert_called_once();spawn.assert_called_once();self.assertEqual(sum(call.args==(23,) for call in closed.call_args_list),1)

    def test_actual_failed_campaign_keeps_primary_failure_when_diagnostic_unavailable(self):
        """Malformed/private/oversized failure stdout cannot become safe facts or weaken the real campaign result."""
        valid=self.browser_failure_fixture();claimed={**valid,"node_exit_code":0}
        raw=wrapper.shared.canonical_bytes(valid)
        for data in (b"PRIVATE",raw+raw,wrapper.shared.canonical_bytes(claimed),b" "*1025):
            row,*_=self.exercise_failed_browser_campaign(data)
            self.assertEqual(row["failure"],"browser-failed");self.assertEqual(row["status"],"failed")
            self.assertIsNone(row["browser_failure"]);self.assertIsNone(row["observation"])
            self.assertTrue(row["cleanup"]["forced"]);self.assertFalse(row["cleanup"]["node_exit_zero"])

    def test_failure_helper_protocol_mirrors_and_input_pins_are_consumed(self):
        """Read only helper literals and prove both new helper/control paths enter wrapper and producer source capture."""
        helper=(Path(browser.__file__).resolve().parents[1]/"ui/tests/workspace_failure.cjs").read_text()
        stage_body=re.search(r"const STAGES = Object\.freeze\(\[(.*?)\]\);",helper,re.S).group(1)
        operator_body=re.search(r"const OPERATORS = Object\.freeze\(\[(.*?)\]\);",helper,re.S).group(1)
        self.assertEqual(set(re.findall(r'"([^"\n]+)"',stage_body)),set(browser.BROWSER_FAILURE_STAGES))
        self.assertEqual(set(re.findall(r'"([^"\n]+)"',operator_body)),set(browser.BROWSER_FAILURE_OPERATORS))
        self.assertEqual(set(re.findall(r'category = "([^"\n]+)"',helper)),set(browser.BROWSER_FAILURE_CATEGORIES))
        self.assertIn('schema_version: "'+browser.BROWSER_FAILURE_SCHEMA+'"',helper)
        paths=("ui/tests/workspace_failure.cjs","ui/tests/workspace_failure.test.cjs")
        for name in paths:self.assertIn(name,wrapper.EXTRA_INPUTS);self.assertIn(name,browser.CAPTURE_INPUTS)
        original=wrapper.shared.hash_file
        with mock.patch.object(browser.sys,"platform","linux"),mock.patch.object(browser,"capture_tools",return_value=self.tools),mock.patch.object(browser,"campaign",side_effect=[self.campaign_fixture(name) for name in browser.CAMPAIGNS]),mock.patch.object(wrapper.shared,"hash_file",wraps=original) as hashed:
            value=browser.produce(self.root,self.forge,self.root/"node",self.root/"npm",self.root/"chrome",self.root/"helper-pins")
        self.assertEqual(value["status"],"passed")
        for name in paths:self.assertEqual(sum(call.args==(self.root/name,) for call in hashed.call_args_list),2)


if __name__=="__main__":unittest.main()
