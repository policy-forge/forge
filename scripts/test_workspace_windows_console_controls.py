#!/usr/bin/env python3
"""Stdlib controls for the actual Windows smoke source; no native execution credit.

Injected native/HTTP adapters test fail-closed control flow. They never establish
ConPTY, console modes, descendant cleanup or product behavior on this host.
The unsupported CLI control invokes only this helper, using no Forge process.
"""
import copy
import ctypes
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import types
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).with_name('test_workspace_windows_console.py')
SPEC = importlib.util.spec_from_file_location('windows_console_smoke', SOURCE)
smoke = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(smoke)


def session():
    """Build the current synthetic browser/read-only session for decoder controls."""
    return {'session_id':'sess_'+'a'*32,'mode':'browser','read_only':True,'api_major':1,
            'contract_version':'1.2.0','project_label':smoke.LABEL,'launched_at':'2026-10-02T12:00:00+00:00'}


def summary():
    """Build exact present-empty-index summary semantics, including all zero counts."""
    return {'version':'a'*64,'project_label':smoke.LABEL,'workspace_index_present':True,
            'health':'setup','resource_counts':dict.fromkeys(('total','valid','invalid','stale','not_validated'),0),
            'review_counts':{'total_open':0},'next_action':'register-resources'}


def passed_receipt():
    """Construct a labeled hypothetical pass only to test the serializer's guards."""
    value = smoke.new_receipt()
    pin = {'bytes':64,'sha256':'b'*64}
    value['platform'] = {'system':'Windows','build':17763,'architecture':'x64','python':'3.11.0'}
    value['identity']['provided_release_binary'] = {'before':pin.copy(),'after':pin.copy()}
    value['fixture'] = {'before':pin.copy(),'after':pin.copy(),'unchanged':True}
    for key in smoke.BOOLS:
        value['checks'][key] = key != 'forced_cleanup'
    for key in ('prompts','no_echo_modes','input_writes'):
        value['checks'][key] = 2
    value['checks']['responses'] = [{'method':m,'path':p,'status':s} for m,p,s in smoke.SEQUENCE]
    value.update(status='passed',scoped_complete=True,failure={'stage':None,'code':None})
    return value


class Response:
    """In-memory HTTP response exercising bounds/header/body handling without sockets."""

    def __init__(self, status, body, headers=None):
        """Store explicit response bytes rather than predecoded values."""
        self.status, self.body = status, body
        self.headers = headers or {'Content-Type':'application/json','Cache-Control':'no-store'}
        self.requested = None

    def read(self, count):
        """Return at most the exact requested bound and retain that request for assertions."""
        self.requested = count
        return self.body[:count]

    def getheader(self, key, default=None):
        """Expose a small ordinary mapping with the HTTPResponse default behavior."""
        return self.headers.get(key,default)


class Connection:
    """Capture method/header dispatch for transport controls; never make network calls."""

    def __init__(self, response):
        """Bind a single response and record whether cleanup closed this connection."""
        self.response, self.calls, self.closed = response, [], False

    def request(self, method, path, body=None, headers=None):
        """Record only in-memory request facts for control assertions."""
        self.calls.append((method,path,body,headers))

    def getresponse(self):
        """Deliver the explicit fake response without modifying its shape."""
        return self.response

    def close(self):
        """Record finally cleanup of each independent HTTP connection."""
        self.closed = True


class NativeFunction:
    """Typed-call test stand-in retaining argtypes/restype; no Win32 is loaded."""

    def __init__(self, implementation=None):
        """Use an explicit control implementation or ordinary boolean success."""
        self.implementation = implementation
        self.calls = []

    def __call__(self, *args):
        """Record actual NativeAPI call arguments before executing the control response."""
        self.calls.append(args)
        return self.implementation(*args) if self.implementation else 1


class NativeLibrary:
    """Fake kernel32 export table allowing real lazy ABI/signature construction."""

    def __init__(self):
        """Create exports on access, with size negotiation implemented separately."""
        self.functions = {}

    def __getattr__(self, name):
        """Supply exports without creating any OS handles or child processes."""
        if name not in self.functions:
            self.functions[name] = NativeFunction()
        return self.functions[name]


def native_api(library=None):
    """Instantiate real NativeAPI layout against a fake DLL on this non-Windows host."""
    library = library or NativeLibrary()
    with patch.object(smoke.sys,'platform','win32'), patch.object(ctypes,'WinDLL',return_value=library,create=True):
        return smoke.NativeAPI(), library


class Clock:
    """Deterministic monotonic clock for actual cleanup polling boundary controls."""

    def __init__(self):
        """Start at zero so equality and timeout cases require no sleeps."""
        self.now = 0.0

    def __call__(self):
        """Return the current injected monotonic observation."""
        return self.now

    def pause(self, duration):
        """Advance time instead of sleeping; does not pause product execution."""
        self.now += duration


class Controls(unittest.TestCase):
    """Meaningful pure/adapter regressions; every native observation remains unqualified."""

    def assert_fault(self, code, callback, stage=None):
        """Require the exact closed diagnostic, not an arbitrary exception/nonzero exit."""
        with self.assertRaises(smoke.SmokeFault) as caught:
            callback()
        self.assertEqual(caught.exception.code,code)
        if stage:
            self.assertEqual(caught.exception.stage,stage)

    def test_strict_json_nested_duplicates_nonfinite_utf8_and_raw_limit(self):
        """Reject each parser ambiguity before a receipt or response can be trusted."""
        for raw in (b'{"x":{"a":1,"a":2}}',b'{"x":NaN}',b'{"x":Infinity}',b'\xff',b'{'):
            with self.subTest(raw=raw):
                self.assert_fault('response-invalid',lambda:smoke.strict_json(raw,128))
        self.assert_fault('response-bound',lambda:smoke.strict_json(b'{} ',2))
        self.assertEqual(smoke.strict_json(b'{"x":[1,null]}',32),{'x':[1,None]})

    def test_file_pin_and_inclusive_bound(self):
        """Hash exact bytes and reject one byte beyond the bound or a nonregular input."""
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory)/'input'
            source.write_bytes(b'abcd')
            self.assertEqual(smoke.file_pin(source,4)['bytes'],4)
            self.assert_fault('binary-too-large',lambda:smoke.file_pin(source,3))
            self.assert_fault('invalid-binary',lambda:smoke.file_pin(Path(directory)))
            self.assert_fault('invalid-binary',lambda:smoke.file_pin(source.with_name('absent')))

    def test_pe_architecture_headers_are_bounded(self):
        """Accept minimal x64 headers; reject other machines, malformed offsets and magic."""
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'provided.exe'
            data = bytearray(70);data[:2]=b'MZ';data[60:64]=(64).to_bytes(4,'little');data[64:70]=b'PE\0\0\x64\x86'
            path.write_bytes(data);smoke.ensure_x64_executable(path)
            for offset, replacement in ((68,b'\x4c\x01'),(0,b'NO'),(60,(1024*1024+1).to_bytes(4,'little'))):
                bad=data.copy();bad[offset:offset+len(replacement)]=replacement;path.write_bytes(bad)
                self.assert_fault('invalid-binary',lambda:smoke.ensure_x64_executable(path))

    def test_exclusive_receipt_preserves_existing_bytes(self):
        """No-replace publication retains an existing receipt and honors the encoded cap."""
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'receipt.json';smoke.exclusive_json(path,{'x':1},64)
            before=path.read_bytes()
            with self.assertRaises(FileExistsError):smoke.exclusive_json(path,{'x':2},64)
            self.assertEqual(path.read_bytes(),before)
            self.assert_fault('response-bound',lambda:smoke.exclusive_json(path.with_name('large'),{'x':'a'*100},32))
            self.assertFalse(path.with_name('large').exists())

    def test_receipt_pass_and_mutable_claim_guards(self):
        """Reject eligibility, missing observation, wrong routes, altered pins and count forgeries."""
        base=passed_receipt();smoke.validate_receipt(base)
        mutations=(lambda x:x.update(acceptance_eligible=True),lambda x:x.update(extra='secret'),
                   lambda x:x['checks'].update(job_empty=False),lambda x:x['checks'].update(forced_cleanup=True),
                   lambda x:x['checks'].update(input_writes=True),lambda x:x['checks'].update(terminal_bytes=65537),
                   lambda x:x['platform'].update(build=17762),lambda x:x['fixture'].update(unchanged=False),
                   lambda x:x['identity']['provided_release_binary']['after'].update(sha256='c'*64),
                   lambda x:x['checks']['responses'][0].update(path='/unknown'),
                   lambda x:x['checks']['responses'][0].update(status=True),lambda x:x['failure'].update(code='raw-secret'))
        for mutate in mutations:
            value=copy.deepcopy(base);mutate(value)
            self.assert_fault('worker-result-invalid',lambda:smoke.validate_receipt(value))

    def test_failure_receipts_require_closed_failure_and_uncompleted_state(self):
        """Incomplete/failed receipts cannot claim completion or carry an arbitrary error string."""
        value=smoke.new_receipt();smoke.validate_receipt(value)
        for mutate in (lambda x:x.update(scoped_complete=True),lambda x:x['failure'].update(code=None),lambda x:x['failure'].update(stage='raw path'),lambda x:x['checks'].update(worker_job=True)):
            bad=copy.deepcopy(value);mutate(bad)
            self.assert_fault('worker-result-invalid',lambda:smoke.validate_receipt(bad))

    def test_chunked_unicode_prompts_and_vt_controls(self):
        """UTF-8 and CSI/OSC split boundaries still expose the two exact complete prompts."""
        monitor=smoke.TerminalMonitor()
        raw=('\x1b[?25h\x1b]0;safe title\x07'+smoke.PROMPTS[0]+'\x00\r\n'+smoke.PROMPTS[1]).encode()
        for byte in raw:monitor.feed(bytes([byte]))
        for prompt in smoke.PROMPTS:monitor.wait_text(__import__('re').escape(prompt),time.monotonic()+0.1)
        monitor.finish();self.assertIsNone(monitor.fault);self.assertTrue(monitor.eof)

    def test_complete_url_requires_line_termination(self):
        """A partial port is never mistaken for a complete launch descriptor."""
        monitor=smoke.TerminalMonitor();monitor.feed(b'Local workspace: http://127.0.0.1:12')
        pattern=r'Local workspace: http://127\.0\.0\.1:([0-9]{1,5})\r?\n'
        self.assert_fault('prompt-timeout',lambda:monitor.wait_text(pattern,time.monotonic()))
        monitor.feed(b'345\r\n');self.assertEqual(monitor.wait_text(pattern,time.monotonic()+1).group(1),'12345')

    def test_malformed_terminal_and_incomplete_eof_fail_closed(self):
        """Invalid UTF-8, unknown controls and incomplete escape/UTF-8 cannot earn no-echo proof."""
        for raw in (b'\xff',b'\x01',b'\x1bX',b'\x1b[',b'\xe2'):
            monitor=smoke.TerminalMonitor();monitor.feed(raw);monitor.finish()
            self.assertEqual(monitor.fault.code,'terminal-malformed')
        monitor=smoke.TerminalMonitor();monitor.feed(b'\x1b]'+b'a'*4095)
        self.assertEqual(monitor.fault.code,'terminal-malformed')

    def test_terminal_bound_is_inclusive_and_overflow_drain_sticky(self):
        """Permit exactly64KiB, latch the first extra read and stop retaining/counting later bytes."""
        monitor=smoke.TerminalMonitor()
        for _ in range(8):monitor.feed(b'a'*8192)
        self.assertEqual(monitor.total,65536);self.assertIsNone(monitor.fault)
        monitor.feed(b'b'*8192);self.assertEqual(monitor.total,73728)
        before=monitor.text;monitor.feed(b'c'*8192)
        self.assertEqual(monitor.total,73728);self.assertEqual(monitor.text,before)
        self.assertEqual(monitor.fault.code,'terminal-output-limit')

    def test_secret_echo_across_chunks_wraps_and_osc_is_rejected(self):
        """Detect synthetic credential/capability echoes including hidden OSC and line fragmentation."""
        for raw in ((smoke.PASSPHRASE.replace(' ','\r\n')).encode(),('\x1b]0;'+smoke.PASSPHRASE+'\x07').encode()):
            monitor=smoke.TerminalMonitor()
            for start in range(0,len(raw),7):monitor.feed(raw[start:start+7])
            self.assertEqual(monitor.fault.code,'terminal-echo')
        monitor=smoke.TerminalMonitor();monitor.feed(b'a'*64);monitor.forbid('a'*64)
        self.assertEqual(monitor.fault.code,'terminal-echo')

    def test_session_and_summary_closed_shape_controls(self):
        """Keep current typed fields, empty-index truth and strict bool/integer distinctions."""
        smoke.validate_session(session());smoke.validate_summary(summary())
        for value,validator,field,replacement in ((session(),smoke.validate_session,'read_only',1),
                 (session(),smoke.validate_session,'mode','machine'),(session(),smoke.validate_session,'contract_version','1.3.0'),
                 (summary(),smoke.validate_summary,'workspace_index_present',False),(summary(),smoke.validate_summary,'health','ready')):
            value[field]=replacement;self.assert_fault('response-invalid',lambda:validator(value))
        bad=summary();bad['resource_counts']['total']=False
        self.assert_fault('response-invalid',lambda:smoke.validate_summary(bad))

    def test_actual_transport_sequence_and_security_headers(self):
        """Exercise all six requests, unauthenticated boundaries and authenticated read-only shutdown."""
        bodies=[{'code':'unauthorized','message':'safe','retryable':False}]*2+[
            {'capability':'c'*64,'session':session()},session(),summary(),{'state':'shutting-down'}]
        responses=[Response(status,json.dumps(body).encode()) for (_,_,status),body in zip(smoke.SEQUENCE,bodies)]
        connections=[]
        def factory(host,port,timeout):
            """Build one bounded in-memory connection per ordered request."""
            self.assertEqual((host,port),('127.0.0.1',1234));self.assertLessEqual(timeout,5)
            value=Connection(responses[len(connections)]);connections.append(value);return value
        checks=smoke.empty_checks();monitor=smoke.TerminalMonitor()
        queries=smoke.BrowserQueries(1234,checks,monitor,time.monotonic()+5,factory)
        with patch.object(smoke.time,'sleep'):
            for index in range(6):queries.request(index,{'passphrase':smoke.PASSPHRASE} if index==2 else None)
        for index,connection in enumerate(connections):
            self.assertTrue(connection.closed);self.assertEqual(connection.response.requested,smoke.MAX_RESPONSE+1)
            method,path,body,headers=connection.calls[0]
            self.assertEqual((method,path),smoke.SEQUENCE[index][:2])
            self.assertEqual(headers['Origin'],'http://127.0.0.1:1234')
            self.assertEqual(headers['Sec-Fetch-Site'],'same-origin')
            self.assertEqual('Authorization' in headers,index>=3)
            if method=='POST':self.assertEqual(headers['Content-Type'],'application/json')
        self.assertTrue(checks['browser_mode']);self.assertTrue(checks['read_only'])
        self.assertIn('c'*64,monitor.forbidden)
        self.assert_fault('request-bound',lambda:queries.request(6))

    def test_transport_rejects_wrong_order_and_invalid_port_without_dispatch(self):
        """Reject extra/order/port/body errors before reaching an HTTP adapter."""
        for port in (0,65536,True):self.assert_fault('request-bound',lambda:smoke.BrowserQueries(port,smoke.empty_checks(),smoke.TerminalMonitor(),time.monotonic()+1))
        queries=smoke.BrowserQueries(1,smoke.empty_checks(),smoke.TerminalMonitor(),time.monotonic()+1,lambda *_a,**_k:self.fail('dispatched'))
        self.assert_fault('request-bound',lambda:queries.request(1))
        self.assert_fault('request-bound',lambda:queries.request(0,{}))
        queries.deadline=time.monotonic();self.assert_fault('worker-timeout',lambda:queries.request(0))
        queries.deadline=1
        with patch.object(smoke.time,'monotonic',side_effect=[0,1]),patch.object(smoke.time,'sleep'):
            self.assert_fault('worker-timeout',lambda:queries.request(0))

    def test_transport_bounds_malformed_response_status_and_finally_cleanup(self):
        """Reject redirects, wrong media/no-store, duplicate bodies and oversized data exactly."""
        cases=[(Response(302,b'{}'),'unexpected-response'),(Response(401,b'{"x":1,"x":2}'),'response-invalid'),
               (Response(401,b'a'*(smoke.MAX_RESPONSE+1)),'response-bound'),
               (Response(401,b'{}',{'Content-Type':'text/plain','Cache-Control':'no-store'}),'response-invalid'),
               (Response(401,b'{}',{'Content-Type':'application/json','Cache-Control':'private'}),'response-invalid')]
        for response,code in cases:
            connection=Connection(response);queries=smoke.BrowserQueries(1,smoke.empty_checks(),smoke.TerminalMonitor(),time.monotonic()+1,lambda *_a,**_k:connection)
            self.assert_fault(code,lambda:queries.request(0));self.assertTrue(connection.closed)

    def test_native_layout_signatures_and_symbolic_attribute_values(self):
        """Construct actual ctypes structures and pointer-width signatures with fake exports only."""
        api,library=native_api()
        self.assertEqual(ctypes.sizeof(api.StartupEx),112);self.assertEqual(ctypes.sizeof(api.Accounting),48)
        self.assertEqual(ctypes.sizeof(api.Extended),144)
        self.assertEqual(library.CreatePseudoConsole.restype,ctypes.c_int32)
        self.assertEqual(library.CreateProcessW.argtypes[4],ctypes.c_int32)
        self.assertEqual(library.UpdateProcThreadAttribute.argtypes[2],ctypes.c_size_t)

    def test_native_host_architecture_rejects_emulation_and_non_x64(self):
        """Read the real host-check function through fake outputs; architecture strings alone cannot pass."""
        api,library=native_api()
        for process,native,accepted in ((0,0x8664,True),(0x8664,0xaa64,False),(0x14c,0x8664,False),(0,0xaa64,False)):
            def architecture(_handle,process_result,native_result):
                """Populate independent native/process machine codes for this control case."""
                process_result._obj.value=process;native_result._obj.value=native;return 1
            library.IsWow64Process2.implementation=architecture
            if accepted:api.require_native_x64()
            else:self.assert_fault('unsupported-architecture',api.require_native_x64)
        library.IsWow64Process2.implementation=lambda *_a:0
        self.assert_fault('native-api-failed',api.require_native_x64)

    def test_owned_job_kill_flag_no_breakaway_and_failed_configuration_closes(self):
        """Real Job setup call sets only kill-on-close; failed configuration releases the handle."""
        api,library=native_api();library.CreateJobObjectW.implementation=lambda *_a:99
        seen=[]
        def configure(handle,kind,pointer,size):
            """Record the actual native limits supplied by the production helper."""
            seen.append((handle,kind,pointer._obj.BasicLimitInformation.LimitFlags,size));return 1
        library.SetInformationJobObject.implementation=configure
        self.assertEqual(api.new_job(),99);self.assertEqual(seen,[(99,9,0x2000,144)])
        library.SetInformationJobObject.implementation=lambda *_a:0
        self.assert_fault('native-api-failed',api.new_job)
        self.assertEqual(library.CloseHandle.calls[-1],(99,))

    def test_atomic_attribute_create_flags_and_lifetime(self):
        """Exercise STARTUPINFOEX JOB_LIST creation without any assignment fallback or shell."""
        api,library=native_api()
        def initialize(_buffer,_count,_flags,size):
            """Implement the real two-call size negotiation with a fixed test allocation."""
            size._obj.value=64;return 1
        library.InitializeProcThreadAttributeList.implementation=initialize
        library.CreateProcessW.implementation=lambda *_a:1
        jobs=(ctypes.c_void_p*1)(99)
        api.create(Path('/fake/python.exe'),['worker','--fixture','fixture'],Path('/fake'),0x2000d,ctypes.byref(jobs),ctypes.sizeof(jobs),True)
        call=library.CreateProcessW.calls[-1]
        self.assertFalse(call[4]);self.assertEqual(call[5],4|0x80000|0x400|0x08000000)
        self.assertEqual(call[8]._obj.StartupInfo.cb,112)
        self.assertEqual(library.UpdateProcThreadAttribute.calls[-1][2],0x2000d)
        self.assertEqual(len(library.DeleteProcThreadAttributeList.calls),1)
        self.assertNotIn('AssignProcessToJobObject',library.functions)
        library.UpdateProcThreadAttribute.implementation=lambda *_a:0
        self.assert_fault('native-api-failed',lambda:api.create(Path('/fake'),[],Path('/fake'),0x20016,1,8))
        self.assertEqual(len(library.DeleteProcThreadAttributeList.calls),2)

    def test_containment_failure_terminates_suspended_child_without_resume(self):
        """An unverified atomic assignment cannot run product code even under a fake adapter."""
        api,library=native_api();library.IsProcessInJob.implementation=lambda *_a:1
        library.WaitForSingleObject.implementation=lambda *_a:0
        process=types.SimpleNamespace(hProcess=10,hThread=11)
        self.assert_fault('job-membership-unverified',lambda:smoke.contain_before_resume(api,process,99))
        self.assertEqual(library.TerminateProcess.calls,[(10,1)]);self.assertEqual(library.ResumeThread.calls,[])
        def contained(_process,_job,result):
            """Set the observed native membership result for the positive control."""
            result._obj.value=1;return 1
        library.IsProcessInJob.implementation=contained
        smoke.contain_before_resume(api,process,99);self.assertEqual(library.ResumeThread.calls,[(11,)])

    def test_current_job_complete_membership_is_required_before_child_resume(self):
        """Any-Job membership cannot substitute for the worker's complete immediate Job set."""
        api,library=native_api();process=types.SimpleNamespace(hProcess=10,hThread=11,dwProcessId=42)
        library.WaitForSingleObject.implementation=lambda *_a:0
        def in_any_job(_process,_job,observed):
            """Report the weak any-Job fact independently of the exact member-set control."""
            observed._obj.value=1;return 1
        library.IsProcessInJob.implementation=in_any_job
        api.members=lambda _job:{100}
        self.assert_fault('job-membership-unverified',lambda:smoke.contain_before_resume(api,process,None))
        self.assertEqual(library.TerminateProcess.calls,[(10,1)]);self.assertEqual(library.ResumeThread.calls,[])
        def truncated(_job):
            """Preserve the original bound diagnostic when exact membership cannot be proven."""
            raise smoke.SmokeFault('admission','owned-process-bound')
        api.members=truncated
        self.assert_fault('owned-process-bound',lambda:smoke.contain_before_resume(api,process,None))
        self.assertEqual(library.TerminateProcess.calls,[(10,1),(10,1)]);self.assertEqual(library.ResumeThread.calls,[])
        api.members=lambda _job:{100,42}
        smoke.contain_before_resume(api,process,None);self.assertEqual(library.ResumeThread.calls,[(11,)])

    def test_job_accounting_membership_and_complete_pid_bound(self):
        """Actual parser rejects partial/over-cap Job lists and reads active accounting precisely."""
        api,library=native_api()
        def accounting(_job,kind,buffer,_size,_written):
            """Populate native outputs independently of the helper's parser."""
            if kind==1:buffer._obj.ActiveProcesses=2
            else:ctypes.memmove(buffer,struct_pack('<IIQQ',2,2,123,456),24)
            return 1
        library.QueryInformationJobObject.implementation=accounting
        self.assertEqual(api.active(99),2);self.assertEqual(api.members(99),{123,456})
        for assigned,listed in ((3,2),(17,17)):
            def incomplete(_job,_kind,buffer,_size,_written):
                """Return a deliberately incomplete native count pair."""
                ctypes.memmove(buffer,struct_pack('<II',assigned,listed),8);return 1
            library.QueryInformationJobObject.implementation=incomplete
            self.assert_fault('owned-process-bound',lambda:api.members(99))
        library.QueryInformationJobObject.implementation=lambda *_a:0
        self.assert_fault('native-api-failed',lambda:api.active(99))

    def test_cleanup_requires_actual_zero_and_boundary_timeout(self):
        """Kill requests and unsupported queries never substitute for observed active zero."""
        clock=Clock();counts=iter([2,1,0]);api=types.SimpleNamespace(active=lambda _job:next(counts))
        self.assertTrue(smoke.await_empty(api,99,1,clock,clock.pause))
        api.active=lambda _job:1;clock.now=1
        self.assertFalse(smoke.await_empty(api,99,1,clock,clock.pause))
        def failed_query(_job):
            """Simulate the typed native query failure, preserving its fail-closed meaning."""
            raise smoke.SmokeFault('cleanup','native-api-failed')
        api.active=failed_query
        self.assert_fault('native-api-failed',lambda:smoke.await_empty(api,99,1,clock,clock.pause))

    def test_owned_console_mode_is_observed_not_modified_and_always_detached(self):
        """Exercise AttachConsole/CONIN$ mode reads and cleanup without native mode mutation."""
        api,library=native_api();library.CreateFileW.implementation=lambda *_a:10
        def mode(_handle,value):
            """Set exactly the processed-only mode used by current rpassword."""
            value._obj.value=1;return 1
        library.GetConsoleMode.implementation=mode
        self.assertEqual(api.console_mode(123),1)
        self.assertEqual(library.CloseHandle.calls,[(10,)])
        self.assertEqual(len(library.FreeConsole.calls),2)
        self.assertNotIn('SetConsoleMode',library.functions)
        library.AttachConsole.implementation=lambda *_a:0
        self.assertIsNone(api.console_mode(123))

    def test_no_echo_readiness_requires_actual_processed_only_mode(self):
        """No input readiness is granted for unavailable, echoed or line-buffered console modes."""
        values=iter([None,7,3,1])
        api=types.SimpleNamespace(exited=lambda _p:None,console_mode=lambda _pid:next(values))
        process=types.SimpleNamespace(hProcess=10,dwProcessId=123)
        monitor=smoke.TerminalMonitor()
        with patch.object(smoke.time,'sleep'):
            smoke.wait_no_echo(api,process,monitor,time.monotonic()+1)
        self.assertEqual(list(values),[])
        self.assert_fault('console-mode-unverified',lambda:smoke.wait_no_echo(api,process,monitor,time.monotonic()))
        api.exited=lambda _p:0
        self.assert_fault('console-mode-unverified',lambda:smoke.wait_no_echo(api,process,monitor,time.monotonic()+1))

    def test_actual_read_loop_handles_eof_and_keeps_draining_after_overflow(self):
        """Drive real reader logic with synchronous fake ReadFile chunks, including overflow discard."""
        api,library=native_api();chunks=[b'a'*8192]*9+[b'b'*8192,b'']
        def read(_handle,buffer,_cap,count,_overlap):
            """Copy one bounded chunk into the native output buffer exactly."""
            data=chunks.pop(0);ctypes.memmove(buffer,data,len(data));count._obj.value=len(data);return 1
        library.ReadFile.implementation=read
        monitor=smoke.TerminalMonitor();smoke.read_terminal(api,10,monitor)
        self.assertEqual(chunks,[]);self.assertTrue(monitor.eof)
        self.assertEqual(monitor.total,73728);self.assertEqual(monitor.fault.code,'terminal-output-limit')

    def test_writer_partial_transfers_count_logical_inputs_only(self):
        """Exercise actual WriteFile partial-transfer loop and two logical CR submissions."""
        api,library=native_api();calls=[]
        def write(_handle,buffer,count,written,_overlap):
            """Transfer at most three bytes, forcing the real writer to finish every fragment."""
            size=min(3,count);calls.append(buffer.raw[:size]);written._obj.value=size;return 1
        library.WriteFile.implementation=write
        import queue
        inputs=queue.Queue();inputs.put(b'first\r');inputs.put(b'second\r')
        checks=smoke.empty_checks();monitor=smoke.TerminalMonitor()
        smoke.write_terminal(api,10,inputs,monitor,checks)
        self.assertEqual(b''.join(calls),b'first\rsecond\r');self.assertEqual(checks['input_writes'],2)
        self.assertIsNone(monitor.fault)

    def test_worker_campaign_launch_uses_required_project_option(self):
        """Reach the real launch adapter with fake pipes/threads and the declared long project option.

        Forge's Workspace parser declares project with #[arg(long)]. This control
        stops at api.create before product startup, console input or HTTP; fake
        teardown observations carry no native Windows or cleanup credit.
        """
        api,library=native_api();pipes=iter(((10,11),(12,13)));api.pipe=lambda:next(pipes)
        launches=[];forge=Path('/synthetic/forge.exe');fixture=Path('/synthetic/project with spaces')
        def pseudo_console(_size,_input,_output,_flags,console):
            """Provide an opaque synthetic ConPTY handle without invoking any Win32 export."""
            console._obj.value=99;return 0
        def launch(executable,arguments,directory,attribute,value,size):
            """Capture actual worker argv then stop before creating or resuming a product process."""
            launches.append((executable,arguments,directory,attribute,value,size))
            raise smoke.SmokeFault('admission','native-api-failed')
        def thread(*,target,args,daemon):
            """Model completed drain/close threads without executing their native target functions."""
            if target is smoke.read_terminal:args[2].eof=True
            return types.SimpleNamespace(start=lambda:None,join=lambda **_kwargs:None,is_alive=lambda:False)
        library.CreatePseudoConsole.implementation=pseudo_console;api.create=launch
        checks=smoke.empty_checks()
        with patch.object(smoke.threading,'Thread',side_effect=thread):
            self.assert_fault('native-api-failed',lambda:smoke.worker_campaign(api,forge,fixture,checks),'admission')
        self.assertEqual(len(launches),1)
        self.assertEqual(launches[0][0:3],(forge,['workspace','--project',fixture,'--read-only','--no-open'],fixture))
        self.assertEqual(launches[0][3:],(0x20016,99,ctypes.sizeof(ctypes.c_void_p)))
        self.assertFalse(checks['console_job']);self.assertEqual(checks['prompts'],0)
        self.assertEqual(checks['responses'],[])

    def test_controller_success_reconciles_worker_exit_and_actual_empty_job(self):
        """The real controller consumes a bounded result, checks exit0 and never forces a clean Job."""
        api,library=native_api();api.new_job=lambda:99
        process=types.SimpleNamespace(hProcess=10,hThread=11,dwProcessId=42)
        api.create=lambda *_a,**_k:process
        api.in_job=lambda *_a:True;api.resume=lambda _p:None
        api.exited=lambda _p:0;api.active=lambda _job:0
        observed=passed_receipt()['checks']
        for name in ('worker_job','worker_exit_zero','job_empty','forced_cleanup','binary_unchanged'):observed[name]=False
        with tempfile.TemporaryDirectory() as directory:
            result=Path(directory)/'worker.json';smoke.exclusive_json(result,{'checks':observed,'failure':None},16384)
            checks=smoke.empty_checks();smoke.run_controller(api,Path('/provided.exe'),Path(directory),result,checks)
        self.assertTrue(checks['worker_job']);self.assertTrue(checks['worker_exit_zero']);self.assertTrue(checks['job_empty'])
        self.assertFalse(checks['forced_cleanup']);self.assertEqual(library.TerminateJobObject.calls,[])
        self.assertEqual(library.CloseHandle.calls,[(11,),(10,),(99,)])

    def test_successful_worker_lingering_descendant_is_closed_cleanup_failure(self):
        """Successful worker exit cannot hide forced descendant termination or lose its typed failure."""
        api,library=native_api();api.new_job=lambda:99
        process=types.SimpleNamespace(hProcess=10,hThread=11,dwProcessId=42)
        api.create=lambda *_a,**_k:process;api.in_job=lambda *_a:True;api.resume=lambda _p:None
        api.exited=lambda _p:0
        counts=iter([1,0]);api.active=lambda _job:next(counts)
        observed=passed_receipt()['checks']
        for name in ('worker_job','worker_exit_zero','job_empty','forced_cleanup','binary_unchanged'):observed[name]=False
        with tempfile.TemporaryDirectory() as directory:
            result=Path(directory)/'worker.json';smoke.exclusive_json(result,{'checks':observed,'failure':None},16384)
            checks=smoke.empty_checks()
            self.assert_fault('cleanup-unverified',lambda:smoke.run_controller(api,Path('/provided.exe'),Path(directory),result,checks),'cleanup')
        self.assertTrue(checks['worker_exit_zero']);self.assertTrue(checks['forced_cleanup']);self.assertTrue(checks['job_empty'])
        self.assertEqual(library.TerminateJobObject.calls,[(99,1)])
        self.assertEqual(library.CloseHandle.calls[-1],(99,))

    def test_controller_failure_and_malformed_results_force_job_cleanup(self):
        """Worker failure, false controller observations and malformed bytes cannot escape Job termination."""
        for malformed in (False,True):
            api,library=native_api();api.new_job=lambda:99
            process=types.SimpleNamespace(hProcess=10,hThread=11,dwProcessId=42)
            api.create=lambda *_a,**_k:process;api.in_job=lambda *_a:True;api.resume=lambda _p:None
            api.exited=lambda _p:1
            counts=iter([1,0]);api.active=lambda _job:next(counts)
            with tempfile.TemporaryDirectory() as directory:
                result=Path(directory)/'worker.json';checks=smoke.empty_checks()
                if malformed:result.write_bytes(b'{"checks":{},"failure":null}')
                else:smoke.exclusive_json(result,{'checks':smoke.empty_checks(),'failure':{'stage':'terminal','code':'terminal-echo'}},16384)
                self.assert_fault('worker-result-invalid' if malformed else 'terminal-echo',lambda:smoke.run_controller(api,Path('/provided.exe'),Path(directory),result,checks))
                self.assertTrue(checks['forced_cleanup']);self.assertTrue(checks['job_empty'])
                self.assertEqual(library.TerminateJobObject.calls,[(99,1)])
                self.assertEqual(library.CloseHandle.calls[-1],(99,))

    def test_controller_timeout_and_failed_accounting_never_claim_clean_exit(self):
        """Absolute worker timeout kills the Job; cleanup query failure remains a failed observation."""
        for broken_query in (False,True):
            api,library=native_api();api.new_job=lambda:99
            process=types.SimpleNamespace(hProcess=10,hThread=11,dwProcessId=42)
            api.create=lambda *_a,**_k:process;api.in_job=lambda *_a:True;api.resume=lambda _p:None
            api.exited=lambda _p:None
            counts=iter([1,0]);api.active=lambda _job:next(counts)
            if broken_query:
                def broken(_job):
                    """Make native accounting unavailable instead of inventing active0."""
                    raise smoke.SmokeFault('cleanup','native-api-failed')
                api.active=broken
            checks=smoke.empty_checks()
            with patch.object(smoke.time,'monotonic',side_effect=[0,61,62]):
                self.assert_fault('cleanup-unverified' if broken_query else 'worker-timeout',lambda:smoke.run_controller(api,Path('/provided.exe'),Path('/fixture'),Path('/missing'),checks))
            self.assertFalse(checks['worker_exit_zero'])
            self.assertEqual(checks['job_empty'],not broken_query)
            self.assertEqual(library.CloseHandle.calls[-1],(99,))

    def test_standalone_receipt_collision_is_silent_and_no_replace(self):
        """The actual script-level exception guard suppresses publication path/errno diagnostics."""
        with tempfile.TemporaryDirectory() as directory:
            bootstrap="import runpy,sys;sys.platform='linux';sys.argv=sys.argv[1:];runpy.run_path(sys.argv[0],run_name='__main__')"
            command=[sys.executable,'-B','-c',bootstrap,str(SOURCE),'--forge','/nonexistent/forge','--output-dir',directory]
            first=subprocess.run(command,capture_output=True,timeout=5)
            self.assertEqual(first.returncode,2)
            path=Path(directory)/smoke.OUTPUT;original=path.read_bytes()
            second=subprocess.run(command,capture_output=True,timeout=5)
            self.assertEqual(second.returncode,1);self.assertEqual(second.stdout,b'');self.assertEqual(second.stderr,b'')
            self.assertEqual(path.read_bytes(),original)

    def test_unsupported_cli_is_incomplete_closed_and_secret_free(self):
        """Run only the helper on this unsupported host; no supplied executable is touched."""
        if sys.platform=='win32':
            # Inject only platform detection into the same execute boundary; no native skip.
            with tempfile.TemporaryDirectory() as directory,patch.object(smoke.sys,'platform','linux'):
                self.assertEqual(smoke.execute('/nonexistent/forge',directory),2)
                raw=(Path(directory)/smoke.OUTPUT).read_bytes()
        else:
            with tempfile.TemporaryDirectory() as directory:
                process=subprocess.run([sys.executable,'-B',str(SOURCE),'--forge','/nonexistent/forge','--output-dir',directory],capture_output=True,timeout=5)
                self.assertEqual(process.returncode,2);self.assertEqual(process.stdout,b'');self.assertEqual(process.stderr,b'')
                raw=(Path(directory)/smoke.OUTPUT).read_bytes()
        value=smoke.strict_json(raw,65536,'receipt');smoke.validate_receipt(value)
        self.assertEqual(value['status'],'incomplete');self.assertEqual(value['failure']['code'],'unsupported-platform')
        self.assertTrue(all(not value['checks'][key] for key in smoke.BOOLS))
        self.assertEqual(value['checks']['responses'],[])
        self.assertNotIn(smoke.PASSPHRASE.encode(),raw);self.assertNotIn(b'/nonexistent',raw)


def struct_pack(pattern,*values):
    """Encode independent native control bytes using explicit little-endian layouts."""
    import struct
    return struct.pack(pattern,*values)


if __name__=='__main__':
    unittest.main()
