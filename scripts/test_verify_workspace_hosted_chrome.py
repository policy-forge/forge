#!/usr/bin/env python3
"""Synthetic stdlib hosted-Chrome controls; never launch a browser, Forge or installed-tool probe."""
import copy
import io
import json
import os
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
        return {"name":name,"status":"passed","failure":None,"observation":{"mode":"read-only" if read_only else "writable","long_fixture":long,"focus_checks":10,"unlock_requests":3,"documented_request_count":10,"synthetic_ui_fault_count":1,"reflow":[{"viewport":width,"page":width} for width in ([640,320]*(3 if long else 1))],"metadata":{"expected_resources":expected,"local_download_bytes":100,"local_download_sha256":"c"*64,"raw_envelope_bytes":111,"strict_duplicate_status":400},"capture_counts":{"actualCounterResponses":0,"distinctRenderedCounterFacts":0,"responseFactsWithoutRenderedObservation":0},"non_loopback_requests":0,"page_errors":0},"cleanup":{"forge_exit_zero":True,"node_exit_zero":True,"tree_empty":True,"terminal_eof":True,"forced":False},"terminal_bytes":100,"no_echo_modes":2}

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


if __name__=="__main__":unittest.main()
