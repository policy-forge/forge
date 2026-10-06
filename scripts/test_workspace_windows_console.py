#!/usr/bin/env python3
"""Bounded stdlib ConPTY browser-session smoke; no browser or platform acceptance.

Only native x64 CPython 3.11+ on Windows build 17763+ can execute the product.
The controller owns a non-inheritable kill-on-close Job; JOB_LIST assigns its
worker atomically at creation. The worker drains ConPTY independently and
observes the owned Forge console mode before each synthetic passphrase input.
Receipts contain closed observations only, never terminal text, URLs, tokens,
paths, PIDs, credentials, response bodies or arbitrary exception messages.
Before/after binary hashes identify supplied bytes; they are not build or
loader attestations. The orchestrator must separately bind source and build.
"""
import argparse
import codecs
import hashlib
import http.client
import json
import os
from pathlib import Path
import platform
import queue
import re
import struct
import subprocess
import sys
import tempfile
import threading
import time
try:
    import ctypes
except ImportError:
    ctypes = None

SCHEMA = 'forge.windows-console-smoke/1'
OUTPUT = 'windows-console-smoke.json'
LABEL = 'Synthetic Windows console smoke'
PASSPHRASE = 'synthetic browser verification passphrase 062'
# Match an ASCII prefix independent of console rendering of the numeric en dash.
# The separately observed owned no-echo mode still gates every input submission.
PROMPTS = ('Set workspace passphrase (', 'Confirm passphrase: ')
MAX_BINARY = 256 * 1024 * 1024
MAX_TERMINAL = 65536
MAX_RESPONSE = 4 * 1024 * 1024
MAX_PROCESSES = 16
SEQUENCE = (('GET', '/api/v1/project/summary', 401),
            ('GET', '/api/v1/session', 401),
            ('POST', '/api/v1/session/unlock', 200),
            ('GET', '/api/v1/session', 200),
            ('GET', '/api/v1/project/summary', 200),
            ('POST', '/api/v1/session/shutdown', 200))
BOOLS = ('worker_job', 'console_job', 'terminal_bounded', 'no_echo_observed',
         'browser_mode', 'read_only', 'forge_exit_zero', 'conpty_closed',
         'terminal_eof', 'worker_exit_zero', 'job_empty', 'forced_cleanup',
         'binary_unchanged')
STAGES = ('platform', 'binary', 'admission', 'terminal', 'unlock', 'query',
          'shutdown', 'cleanup', 'receipt')
CODES = ('unsupported-platform', 'unsupported-architecture', 'missing-ctypes',
         'missing-native-api', 'invalid-binary', 'binary-too-large', 'native-api-failed',
         'job-membership-unverified', 'owned-process-bound', 'worker-timeout',
         'worker-result-invalid', 'prompt-timeout', 'terminal-malformed',
         'terminal-output-limit', 'terminal-echo', 'console-mode-unverified',
         'request-bound', 'response-invalid', 'response-bound', 'request-failed',
         'unexpected-response', 'forge-exit-failed', 'console-close-timeout',
         'cleanup-unverified', 'fixture-changed', 'binary-changed', 'internal-control-error')


class SmokeFault(Exception):
    """Carry only a closed safe classification; arbitrary exception text is excluded."""

    def __init__(self, stage, code, incomplete=False):
        """Reject unrecognized classifications before crossing the receipt boundary."""
        if stage not in STAGES or code not in CODES:
            raise ValueError('invalid safe classification')
        self.stage, self.code, self.incomplete = stage, code, incomplete
        super().__init__(code)


def exact(value, keys, stage='query'):
    """Require a JSON object with exactly the named fields, including nested values."""
    if type(value) is not dict or set(value) != set(keys):
        raise SmokeFault(stage, 'worker-result-invalid' if stage == 'receipt' else 'response-invalid')
    return value


def strict_json(raw, cap, stage='query'):
    """Bound raw bytes before strict UTF-8/JSON parsing; reject duplicates and NaN."""
    if type(raw) is not bytes or len(raw) > cap:
        raise SmokeFault(stage, 'response-bound')

    def pairs(items):
        """Reject repeated members at every decoded object level."""
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError('duplicate member')
            result[key] = value
        return result

    def constant(_value):
        """Reject non-finite JSON constants without retaining their original spelling."""
        raise ValueError('nonfinite')

    try:
        return json.loads(raw.decode('utf-8', 'strict'), object_pairs_hook=pairs,
                          parse_constant=constant)
    except (ValueError, UnicodeError, RecursionError):
        raise SmokeFault(stage, 'response-invalid') from None


def file_pin(path, cap=MAX_BINARY):
    """Hash a bounded regular file by streaming; never expose the filename."""
    try:
        with open(path, 'rb') as source:
            before = os.fstat(source.fileno())
            import stat
            if not stat.S_ISREG(before.st_mode):
                raise SmokeFault('binary', 'invalid-binary')
            if before.st_size > cap:
                raise SmokeFault('binary', 'binary-too-large')
            digest, count = hashlib.sha256(), 0
            while True:
                chunk = source.read(min(65536, cap + 1 - count))
                if not chunk:
                    break
                count += len(chunk)
                if count > cap:
                    raise SmokeFault('binary', 'binary-too-large')
                digest.update(chunk)
            after = os.fstat(source.fileno())
            if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (
                    after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns) or count != before.st_size:
                raise SmokeFault('binary', 'binary-changed')
            return {'bytes': count, 'sha256': digest.hexdigest()}
    except OSError:
        raise SmokeFault('binary', 'invalid-binary') from None


def ensure_x64_executable(path):
    """Reject non-PE and non-x64 supplied files using bounded DOS/COFF headers."""
    try:
        with open(path, 'rb') as source:
            head = source.read(64)
            if len(head) != 64 or head[:2] != b'MZ':
                raise SmokeFault('binary', 'invalid-binary')
            offset = struct.unpack_from('<I', head, 60)[0]
            if not 64 <= offset <= 1024 * 1024:
                raise SmokeFault('binary', 'invalid-binary')
            source.seek(offset)
            header = source.read(6)
            if len(header) != 6 or header[:4] != b'PE\0\0' or struct.unpack('<H', header[4:])[0] != 0x8664:
                raise SmokeFault('binary', 'invalid-binary')
    except OSError:
        raise SmokeFault('binary', 'invalid-binary') from None


def empty_checks():
    """Initialize every observation as unmeasured; unsupported is never a pass."""
    value = {key: False for key in BOOLS}
    value.update(prompts=0, no_echo_modes=0, input_writes=0, terminal_bytes=0, responses=[])
    return value


def new_receipt():
    """Create the closed public envelope without inferring native observations."""
    windows = sys.platform == 'win32'
    return {'schema_version': SCHEMA, 'scope': 'browser-session-console-smoke',
            'truth_state': 'synthetic-development', 'acceptance_eligible': False,
            'status': 'incomplete', 'scoped_complete': False,
            'platform': {'system': 'Windows' if windows else 'unsupported',
                         'build': sys.getwindowsversion().build if windows else None,
                         'architecture': 'unsupported',
                         'python': '.'.join(map(str, sys.version_info[:3]))},
            'identity': {'harness': file_pin(__file__, 131072),
                         'provided_release_binary': {'before': None, 'after': None}},
            'fixture': {'before': None, 'after': None, 'unchanged': False},
            'checks': empty_checks(),
            'failure': {'stage': 'platform', 'code': 'unsupported-platform'}}


def validate_checks(value, stage='receipt'):
    """Validate worker/public observation types and the exact ordered route prefix."""
    exact(value, list(BOOLS) + ['prompts', 'no_echo_modes', 'input_writes', 'terminal_bytes', 'responses'], stage)
    if any(type(value[key]) is not bool for key in BOOLS):
        raise SmokeFault(stage, 'worker-result-invalid')
    for key, bound in (('prompts', 2), ('no_echo_modes', 2), ('input_writes', 2), ('terminal_bytes', 73728)):
        if type(value[key]) is not int or not 0 <= value[key] <= bound:
            raise SmokeFault(stage, 'worker-result-invalid')
    if type(value['responses']) is not list or len(value['responses']) > 6:
        raise SmokeFault(stage, 'worker-result-invalid')
    for index, item in enumerate(value['responses']):
        exact(item, ['method', 'path', 'status'], stage)
        if (item['method'], item['path']) != SEQUENCE[index][:2] or type(item['status']) is not int or not 100 <= item['status'] <= 599:
            raise SmokeFault(stage, 'worker-result-invalid')


def validate_receipt(value):
    """Fail closed on mutable direct-library receipts before serializing pass claims."""
    exact(value, ['schema_version', 'scope', 'truth_state', 'acceptance_eligible', 'status',
                  'scoped_complete', 'platform', 'identity', 'fixture', 'checks', 'failure'], 'receipt')
    if (value['schema_version'], value['scope'], value['truth_state'], value['acceptance_eligible']) != (
            SCHEMA, 'browser-session-console-smoke', 'synthetic-development', False) or type(value['acceptance_eligible']) is not bool:
        raise SmokeFault('receipt', 'worker-result-invalid')
    exact(value['platform'], ['system', 'build', 'architecture', 'python'], 'receipt')
    host = value['platform']
    if host['system'] not in ('Windows', 'unsupported') or host['architecture'] not in ('x64', 'unsupported') or not re.fullmatch(r'\d+\.\d+\.\d+', host['python']):
        raise SmokeFault('receipt', 'worker-result-invalid')
    if (host['system'] == 'Windows' and (type(host['build']) is not int or host['build'] <= 0)) or (host['system'] != 'Windows' and host['build'] is not None):
        raise SmokeFault('receipt', 'worker-result-invalid')
    exact(value['identity'], ['harness', 'provided_release_binary'], 'receipt')
    exact(value['identity']['provided_release_binary'], ['before', 'after'], 'receipt')
    exact(value['fixture'], ['before', 'after', 'unchanged'], 'receipt')
    pins = [value['identity']['harness'], *value['identity']['provided_release_binary'].values(),
            value['fixture']['before'], value['fixture']['after']]
    for pin in pins:
        if pin is None:
            continue
        exact(pin, ['bytes', 'sha256'], 'receipt')
        if type(pin['bytes']) is not int or not 0 <= pin['bytes'] <= MAX_BINARY or type(pin['sha256']) is not str or not re.fullmatch(r'[0-9a-f]{64}', pin['sha256']):
            raise SmokeFault('receipt', 'worker-result-invalid')
    if value['identity']['harness'] is None or not 0 < value['identity']['harness']['bytes'] <= 131072 or type(value['fixture']['unchanged']) is not bool:
        raise SmokeFault('receipt', 'worker-result-invalid')
    validate_checks(value['checks'])
    exact(value['failure'], ['stage', 'code'], 'receipt')
    if type(value['scoped_complete']) is not bool or value['status'] not in ('passed', 'failed', 'incomplete'):
        raise SmokeFault('receipt', 'worker-result-invalid')
    if value['status'] == 'passed':
        checks, binary = value['checks'], value['identity']['provided_release_binary']
        if not (value['scoped_complete'] and host['system'] == 'Windows' and host['architecture'] == 'x64' and host['build'] >= 17763
                and tuple(map(int, host['python'].split('.'))) >= (3, 11, 0)
                and value['failure'] == {'stage': None, 'code': None}
                and all(checks[key] for key in BOOLS if key != 'forced_cleanup') and not checks['forced_cleanup']
                and all(checks[key] == 2 for key in ('prompts', 'no_echo_modes', 'input_writes'))
                and checks['terminal_bytes'] <= MAX_TERMINAL
                and checks['responses'] == [{'method': m, 'path': p, 'status': s} for m, p, s in SEQUENCE]
                and binary['before'] is not None and binary['before'] == binary['after']
                and value['fixture']['before'] is not None and value['fixture']['before'] == value['fixture']['after'] and value['fixture']['unchanged']):
            raise SmokeFault('receipt', 'worker-result-invalid')
    elif value['scoped_complete'] or value['failure']['stage'] not in STAGES or value['failure']['code'] not in CODES:
        raise SmokeFault('receipt', 'worker-result-invalid')
    if value['status'] == 'incomplete' and (any(value['checks'][key] for key in BOOLS)
            or any(value['checks'][key] for key in ('prompts','no_echo_modes','input_writes','terminal_bytes'))
            or value['checks']['responses']):
        raise SmokeFault('receipt', 'worker-result-invalid')
    return value


def exclusive_json(path, value, cap):
    """Publish one bounded closed JSON file with O_EXCL; never replace evidence."""
    raw = (json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + '\n').encode('utf-8')
    if len(raw) > cap:
        raise SmokeFault('receipt', 'response-bound')
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, 'wb') as output:
        output.write(raw)
        output.flush()
        os.fsync(output.fileno())


class TerminalMonitor:
    """Incrementally strip bounded VT controls, detect prompts and forbid secret echo.

    This recognizer is not a terminal emulator. Printable output exists only in
    memory; unknown escapes and truncated UTF-8/VT at EOF fail closed. NUL emitted
    by the current rpassword Windows prompt writer is accepted and ignored.
    """

    def __init__(self):
        """Initialize isolated decoder, sticky fault and complete-lifetime byte count."""
        self.decoder = codecs.getincrementaldecoder('utf-8')('strict')
        self.text, self.raw_compact, self.escape, self.state = '', '', '', 'text'
        self.total, self.fault, self.eof = 0, None, False
        self.forbidden = [PASSPHRASE.replace(' ', '')]
        self.condition = threading.Condition()

    def forbid(self, secret):
        """Add the in-memory minted capability to the lifetime echo scan."""
        with self.condition:
            self.forbidden.append(secret)
            self._scan()

    def _fail(self, code):
        """Latch the first failure while permitting the reader to drain and discard."""
        if self.fault is None:
            self.fault = SmokeFault('terminal', code)
        self.condition.notify_all()

    def _scan(self):
        """Detect whitespace-fragmented literal secrets in the entire printable trace."""
        compact = ''.join(self.text.split())
        if any(secret in compact or secret in self.raw_compact for secret in self.forbidden):
            self._fail('terminal-echo')

    def feed(self, raw):
        """Bound each read and lifetime allocation before incremental decoding."""
        with self.condition:
            if self.fault is not None:
                return
            if type(raw) is not bytes or len(raw) > 8192:
                self._fail('terminal-output-limit')
                return
            self.total += len(raw)
            if self.total > MAX_TERMINAL:
                self._fail('terminal-output-limit')
                return
            try:
                text = self.decoder.decode(raw)
                self.raw_compact += ''.join(text.split())
                for char in text:
                    self._character(char)
                self._scan()
            except (UnicodeError, ValueError):
                self._fail('terminal-malformed')
            self.condition.notify_all()

    def _character(self, char):
        """Consume complete CSI/OSC sequences without letting escapes grow unbounded."""
        if self.state == 'text':
            if char == '\x1b':
                self.state, self.escape = 'escape', char
            elif char in '\r\n\t' or char >= ' ':
                self.text += char
            elif char not in ('\x00', '\x07', '\b'):
                raise ValueError('unsupported control')
        else:
            self.escape += char
            if len(self.escape.encode('utf-8')) > 4096:
                raise ValueError('escape bound')
            if self.state == 'escape':
                if char == '[':
                    self.state = 'csi'
                elif char == ']':
                    self.state = 'osc'
                elif char in ('7', '8', '=', '>'):
                    self.state, self.escape = 'text', ''
                else:
                    raise ValueError('unsupported escape')
            elif self.state == 'csi':
                if '@' <= char <= '~':
                    if not re.fullmatch(r'\x1b\[[0-?]*[ -/]*[@-~]', self.escape):
                        raise ValueError('malformed csi')
                    self.state, self.escape = 'text', ''
                elif not (' ' <= char <= '?'):
                    raise ValueError('malformed csi')
            elif self.state == 'osc':
                if char == '\x07' or self.escape.endswith('\x1b\\'):
                    self.state, self.escape = 'text', ''

    def finish(self, eof=True):
        """Finalize decoding; only a verified pipe-end observation records actual EOF."""
        with self.condition:
            self.eof = eof
            if self.fault is None:
                try:
                    self.decoder.decode(b'', final=True)
                    if self.state != 'text':
                        raise ValueError('unfinished escape')
                except (UnicodeError, ValueError):
                    self._fail('terminal-malformed')
            self.condition.notify_all()

    def wait_text(self, pattern, deadline):
        """Wait for a complete fixed prompt/URL without exposing retained output."""
        with self.condition:
            while True:
                if self.fault:
                    raise self.fault
                found = re.search(pattern, self.text)
                if found:
                    return found
                remaining = deadline - time.monotonic()
                if remaining <= 0 or self.eof:
                    raise SmokeFault('terminal', 'prompt-timeout')
                self.condition.wait(min(remaining, 0.1))


def validate_session(value):
    """Require the exact browser/read-only session shape for this synthetic project."""
    exact(value, ['session_id', 'mode', 'read_only', 'api_major', 'contract_version', 'project_label', 'launched_at'])
    if not (type(value['session_id']) is str and re.fullmatch(r'sess_[a-z0-9]{8,64}', value['session_id'])
            and value['mode'] == 'browser' and value['read_only'] is True
            and type(value['api_major']) is int and value['api_major'] == 1
            and value['contract_version'] == '1.2.0' and value['project_label'] == LABEL
            and type(value['launched_at']) is str and re.fullmatch(r'\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?(?:Z|[+-]\d\d:\d\d)', value['launched_at'])):
        raise SmokeFault('query', 'response-invalid')


def validate_summary(value):
    """Pin current empty-index semantics and zero denominators rather than pruning."""
    exact(value, ['version', 'project_label', 'workspace_index_present', 'health', 'resource_counts', 'review_counts', 'next_action'])
    exact(value['resource_counts'], ['total', 'valid', 'invalid', 'stale', 'not_validated'])
    exact(value['review_counts'], ['total_open'])
    if not (type(value['version']) is str and 8 <= len(value['version']) <= 128
            and value['project_label'] == LABEL and value['workspace_index_present'] is True
            and value['health'] == 'setup' and value['next_action'] == 'register-resources'
            and all(type(number) is int and number == 0 for number in value['resource_counts'].values())
            and type(value['review_counts']['total_open']) is int and value['review_counts']['total_open'] == 0):
        raise SmokeFault('query', 'response-invalid')


class BrowserQueries:
    """Consume only the six frozen requests with paced direct-loopback HTTP."""

    def __init__(self, port, checks, monitor, deadline, connection=http.client.HTTPConnection):
        """Keep port/capability memory-only; allow injection for pure transport controls."""
        if type(port) is not int or not 1 <= port <= 65535:
            raise SmokeFault('query', 'request-bound')
        self.port, self.checks, self.monitor, self.deadline = port, checks, monitor, deadline
        self.connection, self.capability, self.last = connection, None, 0.0

    def request(self, index, body=None):
        """Reject out-of-order/extra requests before dispatch; bound response before parse."""
        if type(index) is not int or index != len(self.checks['responses']) or not 0 <= index < 6:
            raise SmokeFault('query', 'request-bound')
        now = time.monotonic()
        if now >= self.deadline:
            raise SmokeFault('query', 'worker-timeout')
        time.sleep(max(0, 0.1 - (now - self.last)))
        if time.monotonic() >= self.deadline:
            raise SmokeFault('query', 'worker-timeout')
        method, path, expected = SEQUENCE[index]
        raw = None if body is None else json.dumps(body, separators=(',', ':'), allow_nan=False).encode('utf-8')
        if (index == 2 and body != {'passphrase': PASSPHRASE}) or (index != 2 and body is not None) or (raw is not None and len(raw) > 4096):
            raise SmokeFault('query', 'request-bound')
        headers = {'Accept': 'application/json', 'Origin': f'http://127.0.0.1:{self.port}',
                   'Sec-Fetch-Site': 'same-origin', 'Sec-Fetch-Mode': 'cors', 'Sec-Fetch-Dest': 'empty'}
        if method == 'POST':
            headers['Content-Type'] = 'application/json'
        if index >= 3:
            if self.capability is None:
                raise SmokeFault('query', 'request-bound')
            headers['Authorization'] = 'Bearer ' + self.capability
        client = self.connection('127.0.0.1', self.port, timeout=min(5, max(0.01, self.deadline-time.monotonic())))
        try:
            client.request(method, path, body=raw, headers=headers)
            response = client.getresponse()
            if type(response.status) is not int or not 100 <= response.status <= 599:
                raise SmokeFault('query', 'response-invalid')
            self.checks['responses'].append({'method': method, 'path': path, 'status': response.status})
            data = response.read(MAX_RESPONSE + 1)
            if len(data) > MAX_RESPONSE:
                raise SmokeFault('query', 'response-bound')
            if response.getheader('Content-Type', '').split(';')[0].strip().lower() != 'application/json' or response.getheader('Cache-Control') != 'no-store':
                raise SmokeFault('query', 'response-invalid')
            value = strict_json(data, MAX_RESPONSE)
            if response.status != expected:
                raise SmokeFault('query', 'unexpected-response')
        except (OSError, http.client.HTTPException):
            raise SmokeFault('query', 'request-failed') from None
        finally:
            client.close()
            self.last = time.monotonic()
        if index < 2:
            exact(value, ['code', 'message', 'retryable'])
            if value['code'] != 'unauthorized' or type(value['message']) is not str or not 1 <= len(value['message']) <= 500 or type(value['retryable']) is not bool:
                raise SmokeFault('query', 'response-invalid')
        elif index == 2:
            exact(value, ['capability', 'session'])
            if type(value['capability']) is not str or not re.fullmatch(r'[0-9a-f]{64}', value['capability']):
                raise SmokeFault('unlock', 'response-invalid')
            validate_session(value['session'])
            self.capability = value['capability']
            self.monitor.forbid(self.capability)
            self.checks['browser_mode'] = self.checks['read_only'] = True
        elif index == 3:
            validate_session(value)
        elif index == 4:
            validate_summary(value)
        else:
            exact(value, ['state'])
            if value['state'] != 'shutting-down':
                raise SmokeFault('shutdown', 'response-invalid')
        return value


class NativeAPI:
    """Typed Win32 calls with x64 structure checks; constructed only on Windows.

    JOB_LIST is 0x2000d (SDK enum 13); PSEUDOCONSOLE is 0x20016 (enum 22).
    The owned Job never enables either breakaway flag. Failure to atomically
    assign a worker, or to prove later containment, terminates rather than
    falling back to post-creation assignment.
    """

    def __init__(self):
        """Bind every consumed function and verify native ABI before spawning."""
        if ctypes is None:
            raise SmokeFault('platform', 'missing-ctypes', True)
        if sys.platform != 'win32':
            raise SmokeFault('platform', 'unsupported-platform', True)
        c = ctypes
        self.c, self.dll = c, c.WinDLL('kernel32.dll', use_last_error=True)
        handle, dword, word, size = c.c_void_p, c.c_uint32, c.c_uint16, c.c_size_t

        class Coord(c.Structure):
            """Win32 signed 16-bit COORD passed by value to CreatePseudoConsole."""
            _fields_ = [('X', c.c_int16), ('Y', c.c_int16)]

        class Startup(c.Structure):
            """Native STARTUPINFOW including pointer-width handle fields."""
            _fields_ = [('cb', dword), ('lpReserved', c.c_wchar_p), ('lpDesktop', c.c_wchar_p), ('lpTitle', c.c_wchar_p),
                        ('dwX', dword), ('dwY', dword), ('dwXSize', dword), ('dwYSize', dword), ('dwXCountChars', dword),
                        ('dwYCountChars', dword), ('dwFillAttribute', dword), ('dwFlags', dword),
                        ('wShowWindow', word), ('cbReserved2', word), ('lpReserved2', handle),
                        ('hStdInput', handle), ('hStdOutput', handle), ('hStdError', handle)]

        class StartupEx(c.Structure):
            """STARTUPINFOEXW retains its live attribute-list allocation until creation."""
            _fields_ = [('StartupInfo', Startup), ('lpAttributeList', handle)]

        class Process(c.Structure):
            """PROCESS_INFORMATION handles and identifiers; identifiers never leave memory."""
            _fields_ = [('hProcess', handle), ('hThread', handle), ('dwProcessId', dword), ('dwThreadId', dword)]

        class Limits(c.Structure):
            """JOBOBJECT_BASIC_LIMIT_INFORMATION for kill-on-last-handle ownership."""
            _fields_ = [('PerProcessUserTimeLimit', c.c_int64), ('PerJobUserTimeLimit', c.c_int64),
                        ('LimitFlags', dword), ('MinimumWorkingSetSize', size), ('MaximumWorkingSetSize', size),
                        ('ActiveProcessLimit', dword), ('Affinity', size), ('PriorityClass', dword), ('SchedulingClass', dword)]

        class Io(c.Structure):
            """IO_COUNTERS layout required by extended Job limits."""
            _fields_ = [(name, c.c_uint64) for name in ('ReadOperationCount', 'WriteOperationCount', 'OtherOperationCount', 'ReadTransferCount', 'WriteTransferCount', 'OtherTransferCount')]

        class Extended(c.Structure):
            """JOBOBJECT_EXTENDED_LIMIT_INFORMATION without breakaway or quota changes."""
            _fields_ = [('BasicLimitInformation', Limits), ('IoInfo', Io), ('ProcessMemoryLimit', size),
                        ('JobMemoryLimit', size), ('PeakProcessMemoryUsed', size), ('PeakJobMemoryUsed', size)]

        class Accounting(c.Structure):
            """Actual active-process count; requested termination is not completion."""
            _fields_ = [('TotalUserTime', c.c_int64), ('TotalKernelTime', c.c_int64), ('ThisPeriodTotalUserTime', c.c_int64),
                        ('ThisPeriodTotalKernelTime', c.c_int64), ('TotalPageFaultCount', dword),
                        ('TotalProcesses', dword), ('ActiveProcesses', dword), ('TotalTerminatedProcesses', dword)]

        self.Coord, self.StartupEx, self.Process, self.Extended, self.Accounting = Coord, StartupEx, Process, Extended, Accounting
        for klass, expected in ((Startup,104),(StartupEx,112),(Process,24),(Limits,64),(Io,48),(Extended,144),(Accounting,48)):
            if c.sizeof(klass) != expected:
                raise SmokeFault('platform', 'unsupported-architecture', True)
        ptr = c.c_void_p
        signatures = {
            'CreateJobObjectW': (handle, [ptr, c.c_wchar_p]),
            'SetInformationJobObject': (c.c_int32, [handle, c.c_int32, ptr, dword]),
            'QueryInformationJobObject': (c.c_int32, [handle, c.c_int32, ptr, dword, ptr]),
            'TerminateJobObject': (c.c_int32, [handle,dword]),
            'IsProcessInJob': (c.c_int32, [handle,handle,ptr]),
            'CloseHandle': (c.c_int32, [handle]),
            'CreatePipe': (c.c_int32,[ptr,ptr,ptr,dword]),
            'CreatePseudoConsole': (c.c_int32,[Coord,handle,handle,dword,ptr]),
            'ClosePseudoConsole': (None,[handle]),
            'InitializeProcThreadAttributeList': (c.c_int32,[ptr,dword,dword,ptr]),
            'UpdateProcThreadAttribute': (c.c_int32,[ptr,dword,size,ptr,size,ptr,ptr]),
            'DeleteProcThreadAttributeList': (None,[ptr]),
            'CreateProcessW': (c.c_int32,[c.c_wchar_p,ptr,ptr,ptr,c.c_int32,dword,ptr,c.c_wchar_p,ptr,ptr]),
            'ResumeThread': (dword,[handle]),
            'WaitForSingleObject': (dword,[handle,dword]),
            'GetExitCodeProcess': (c.c_int32,[handle,ptr]),
            'TerminateProcess': (c.c_int32,[handle,dword]),
            'ReadFile': (c.c_int32,[handle,ptr,dword,ptr,ptr]),
            'WriteFile': (c.c_int32,[handle,ptr,dword,ptr,ptr]),
            'FreeConsole': (c.c_int32,[]), 'AttachConsole': (c.c_int32,[dword]),
            'CreateFileW': (handle,[c.c_wchar_p,dword,dword,ptr,dword,dword,handle]),
            'GetConsoleMode': (c.c_int32,[handle,ptr]),
            'GetCurrentProcess': (handle,[]), 'IsWow64Process2': (c.c_int32,[handle,ptr,ptr])}
        try:
            for name, (result, args) in signatures.items():
                function = getattr(self.dll, name)
                function.restype, function.argtypes = result, args
        except AttributeError:
            raise SmokeFault('platform', 'missing-native-api', True) from None

    def require_native_x64(self):
        """Reject emulated/32-bit processes and non-x64 native hosts using actual Win32 facts."""
        process, native = self.c.c_uint16(), self.c.c_uint16()
        self.require(self.dll.IsWow64Process2(self.dll.GetCurrentProcess(),self.c.byref(process),self.c.byref(native)),'platform')
        if process.value != 0 or native.value != 0x8664:
            raise SmokeFault('platform','unsupported-architecture',True)

    def require(self, result, stage='admission'):
        """Convert a failed native boolean into a safe classification only."""
        if not result:
            raise SmokeFault(stage, 'native-api-failed')
        return result

    def close(self, handle):
        """Close an owned non-null ordinary handle, never an HPCON."""
        if handle:
            self.require(self.dll.CloseHandle(handle), 'cleanup')

    def new_job(self):
        """Create an unnamed noninheritable kill-on-close Job with no breakaway."""
        handle = self.require(self.dll.CreateJobObjectW(None,None))
        limits = self.Extended()
        limits.BasicLimitInformation.LimitFlags = 0x2000
        try:
            self.require(self.dll.SetInformationJobObject(handle,9,self.c.byref(limits),self.c.sizeof(limits)))
            return handle
        except BaseException:
            self.close(handle)
            raise

    def active(self, job):
        """Read actual Job accounting; failed query cannot mean empty."""
        accounting = self.Accounting()
        self.require(self.dll.QueryInformationJobObject(job,1,self.c.byref(accounting),self.c.sizeof(accounting),None),'cleanup')
        if accounting.ActiveProcesses > MAX_PROCESSES:
            raise SmokeFault('cleanup','owned-process-bound')
        return accounting.ActiveProcesses

    def members(self, job=None):
        """Read the complete bounded PID set of this Job, refusing truncated results."""
        buffer = self.c.create_string_buffer(8+MAX_PROCESSES*self.c.sizeof(self.c.c_size_t))
        self.require(self.dll.QueryInformationJobObject(job,3,buffer,len(buffer),None))
        assigned, listed = struct.unpack_from('<II',buffer.raw)
        if assigned != listed or listed > MAX_PROCESSES:
            raise SmokeFault('admission','owned-process-bound')
        return set(struct.unpack_from('<'+'Q'*listed,buffer.raw,8))

    def in_job(self, process, job):
        """Prove exact controller Job membership, or current inherited membership."""
        observed = self.c.c_int32()
        self.require(self.dll.IsProcessInJob(process,job,self.c.byref(observed)))
        return bool(observed.value)

    def terminate(self, job):
        """Request termination of all owned processes; callers still await active zero."""
        self.require(self.dll.TerminateJobObject(job,1),'cleanup')

    def exited(self, process):
        """Return an actual exit code after a signaled handle; null means still running."""
        wait = self.dll.WaitForSingleObject(process,0)
        if wait == 258:
            return None
        if wait != 0:
            raise SmokeFault('cleanup','native-api-failed')
        code = self.c.c_uint32()
        self.require(self.dll.GetExitCodeProcess(process,self.c.byref(code)),'cleanup')
        return code.value

    def create(self, executable, arguments, cwd, attribute, value, size, worker=False):
        """Create suspended with a live JOB_LIST/PSEUDOCONSOLE attribute, no shell."""
        c, native = self.c, self.dll
        allocation = c.c_size_t()
        native.InitializeProcThreadAttributeList(None,1,0,c.byref(allocation))
        if not 0 < allocation.value <= 65536:
            raise SmokeFault('admission','native-api-failed')
        buffer = c.create_string_buffer(allocation.value)
        self.require(native.InitializeProcThreadAttributeList(buffer,1,0,c.byref(allocation)))
        try:
            self.require(native.UpdateProcThreadAttribute(buffer,0,attribute,value,size,None,None))
            startup, process = self.StartupEx(), self.Process()
            startup.StartupInfo.cb = c.sizeof(startup)
            startup.lpAttributeList = c.cast(buffer,c.c_void_p)
            command = c.create_unicode_buffer(subprocess.list2cmdline([str(executable),*map(str,arguments)]))
            # No credential/capability inherited in environment. SystemRoot is required by Windows.
            environment = {key: os.environ[key] for key in ('SystemRoot','WINDIR','TEMP','TMP') if key in os.environ}
            env = c.create_unicode_buffer('\0'.join(key+'='+environment[key] for key in sorted(environment))+'\0\0')
            flags = 4|0x80000|0x400|(0x08000000 if worker else 0)
            self.require(native.CreateProcessW(str(executable),command,None,None,False,flags,env,str(cwd),c.byref(startup),c.byref(process)))
            return process
        finally:
            native.DeleteProcThreadAttributeList(buffer)

    def resume(self, process):
        """Release a proven-contained suspended process exactly once."""
        if self.dll.ResumeThread(process.hThread) == 0xffffffff:
            raise SmokeFault('admission','native-api-failed')

    def pipe(self):
        """Create non-inherited synchronous endpoints for independent reader/writer threads."""
        read, write = self.c.c_void_p(), self.c.c_void_p()
        self.require(self.dll.CreatePipe(self.c.byref(read),self.c.byref(write),None,0))
        return read.value, write.value

    def console_mode(self, pid):
        """Observe the owned child's console input flags; never alter mode or inject input."""
        self.dll.FreeConsole()
        attached, handle = False, None
        try:
            attached = bool(self.dll.AttachConsole(pid))
            if not attached:
                return None
            handle = self.dll.CreateFileW('CONIN$',0x80000000,3,None,3,0,None)
            if handle == self.c.c_void_p(-1).value or not handle:
                handle = None
                return None
            mode = self.c.c_uint32()
            if not self.dll.GetConsoleMode(handle,self.c.byref(mode)):
                return None
            return mode.value
        finally:
            if handle:
                self.close(handle)
            if attached:
                self.dll.FreeConsole()


def await_empty(api, job, deadline, clock=time.monotonic, pause=time.sleep):
    """Await actual zero accounting; neither a kill request nor timeout is success."""
    while True:
        if api.active(job) == 0:
            return True
        if clock() >= deadline:
            return False
        pause(0.02)


def contain_before_resume(api, process, job):
    """Prove the exact owned/current Job before resume; failed queries terminate the suspended child."""
    failure = None
    try:
        contained = api.in_job(process.hProcess,job)
        if job is None and contained:
            contained = process.dwProcessId in api.members(None)
    except SmokeFault as error:
        contained, failure = False, error
    if not contained:
        api.require(api.dll.TerminateProcess(process.hProcess,1))
        if api.dll.WaitForSingleObject(process.hProcess,3000) != 0:
            raise SmokeFault('cleanup','cleanup-unverified')
        raise failure or SmokeFault('admission','job-membership-unverified')
    api.resume(process)


def read_terminal(api, handle, monitor):
    """Drain output; zero bytes/broken pipe alone establish EOF, not reader failure."""
    eof = False
    try:
        while True:
            buffer, count = api.c.create_string_buffer(8192), api.c.c_uint32()
            if not api.dll.ReadFile(handle,buffer,8192,api.c.byref(count),None):
                if api.c.get_last_error() in (109,232):
                    eof = True
                    break
                raise SmokeFault('terminal','native-api-failed')
            if count.value == 0:
                eof = True
                break
            monitor.feed(buffer.raw[:count.value])
    except BaseException:
        with monitor.condition:
            monitor._fail('native-api-failed')
    finally:
        monitor.finish(eof)


def write_terminal(api, handle, inputs, monitor, checks, stop=None, deadline=None):
    """Submit two CR inputs; idle queue polls honor cancellation and one absolute deadline.

    Empty bounded polls are readiness observations, never native API failures.
    Cancellation cannot preempt an in-progress synchronous WriteFile; teardown
    still requires the writer to finish and records failure if it remains live.
    """
    stop = threading.Event() if stop is None else stop
    deadline = time.monotonic() + 60 if deadline is None else deadline
    try:
        for _ in range(2):
            while True:
                if stop.is_set() or monitor.fault is not None:
                    return
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise SmokeFault('terminal', 'worker-timeout')
                try:
                    data = inputs.get(timeout=min(0.1, remaining))
                except queue.Empty:
                    continue
                break
            offset = 0
            while offset < len(data):
                if stop.is_set() or monitor.fault is not None:
                    return
                buffer, count = api.c.create_string_buffer(data[offset:]), api.c.c_uint32()
                api.require(api.dll.WriteFile(handle,buffer,len(data)-offset,api.c.byref(count),None),'terminal')
                if not 0 < count.value <= len(data)-offset:
                    raise SmokeFault('terminal','native-api-failed')
                offset += count.value
            checks['input_writes'] += 1
    except SmokeFault as error:
        with monitor.condition:
            monitor._fail(error.code)
    except BaseException:
        with monitor.condition:
            monitor._fail('native-api-failed')


def wait_no_echo(api, process, monitor, deadline):
    """Poll actual owned CONIN$ mode until processed on and echo/line disabled."""
    while time.monotonic() < deadline:
        if monitor.fault:
            raise monitor.fault
        if api.exited(process.hProcess) is not None:
            break
        if api.console_mode(process.dwProcessId) == 1:
            return
        time.sleep(0.02)
    raise SmokeFault('terminal','console-mode-unverified')


def worker_campaign(api, forge, fixture, checks):
    """Run owned ConPTY smoke; preserve the first failure and cancel idle writer teardown.

    Prompt rendering grants no input authority: actual owned console mode must
    be processed-only before each submission. Cleanup observations remain
    required for pass even when an earlier closed failure keeps diagnostic priority.
    """
    c, native = api.c, api.dll
    deadline = time.monotonic()+60
    startup = min(deadline,time.monotonic()+30)
    monitor, inputs, stop = TerminalMonitor(), queue.Queue(maxsize=2), threading.Event()
    in_read = in_write = out_read = out_write = hpc = process = None
    reader = writer = closer = None
    failure = None
    try:
        in_read,in_write = api.pipe()
        out_read,out_write = api.pipe()
        console = c.c_void_p()
        if native.CreatePseudoConsole(api.Coord(160,40),in_read,out_write,0,c.byref(console)) != 0:
            raise SmokeFault('admission','native-api-failed')
        hpc = console.value
        reader = threading.Thread(target=read_terminal,args=(api,out_read,monitor),daemon=True)
        reader.start()
        process = api.create(forge,['workspace','--project',fixture,'--read-only','--no-open'],fixture,0x20016,hpc,c.sizeof(c.c_void_p))
        contain_before_resume(api,process,None)
        checks['console_job'] = True
        api.close(in_read); in_read = None
        api.close(out_write); out_write = None
        writer = threading.Thread(target=write_terminal,args=(api,in_write,inputs,monitor,checks,stop,deadline),daemon=True)
        writer.start()
        for prompt in PROMPTS:
            monitor.wait_text(re.escape(prompt),startup)
            checks['prompts'] += 1
            wait_no_echo(api,process,monitor,startup)
            checks['no_echo_modes'] += 1
            inputs.put_nowait((PASSPHRASE+'\r').encode('utf-8'))
        found = monitor.wait_text(r'Local workspace: http://127\.0\.0\.1:([0-9]{1,5})\r?\n',startup)
        queries = BrowserQueries(int(found.group(1)),checks,monitor,deadline)
        for index in range(6):
            queries.request(index,{'passphrase':PASSPHRASE} if index == 2 else None)
        grace = min(deadline,time.monotonic()+3)
        while api.exited(process.hProcess) is None and time.monotonic()<grace:
            time.sleep(0.02)
        if api.exited(process.hProcess) != 0:
            raise SmokeFault('shutdown','forge-exit-failed')
        checks['forge_exit_zero'] = True
    except SmokeFault as error:
        failure = error
    except BaseException:
        failure = SmokeFault('terminal','internal-control-error')
    finally:
        stop.set()
        native.FreeConsole()
        if writer:
            writer.join(timeout=0.2)
        if writer is None or not writer.is_alive():
            if in_write:
                api.close(in_write); in_write = None
        if hpc:
            # Never close on the drain thread: older inbox ConPTY may block.
            closer = threading.Thread(target=native.ClosePseudoConsole,args=(hpc,),daemon=True)
            closer.start()
            closer.join(timeout=3)
            checks['conpty_closed'] = not closer.is_alive()
        if reader:
            reader.join(timeout=3)
            checks['terminal_eof'] = monitor.eof and not reader.is_alive()
        checks['terminal_bytes'] = monitor.total
        checks['terminal_bounded'] = monitor.fault is None and monitor.total <= MAX_TERMINAL and checks['terminal_eof']
        checks['no_echo_observed'] = checks['terminal_bounded'] and checks['no_echo_modes'] == 2
        if failure is None and monitor.fault is not None:
            failure = monitor.fault
        if failure is None and (not checks['conpty_closed'] or not checks['terminal_eof'] or (writer and writer.is_alive())):
            failure = SmokeFault('cleanup','console-close-timeout')
        for handle in (in_read,out_write,in_write if writer is None or not writer.is_alive() else None):
            if handle:
                api.close(handle)
        if out_read and (reader is None or not reader.is_alive()):
            api.close(out_read)
        if process:
            api.close(process.hThread); api.close(process.hProcess)
    if failure:
        raise failure


def worker_main(args):
    """Write one nonsecret worker result; controller owns final Job cleanup proof."""
    checks, failure = empty_checks(), None
    try:
        api = NativeAPI()
        if os.getpid() not in api.members(None):
            raise SmokeFault('admission','job-membership-unverified')
        worker_campaign(api,Path(args.forge),Path(args.fixture),checks)
    except SmokeFault as error:
        failure = {'stage':error.stage,'code':error.code}
    except BaseException:
        failure = {'stage':'terminal','code':'internal-control-error'}
    try:
        validate_checks(checks)
        exclusive_json(Path(args.worker_result),{'checks':checks,'failure':failure},16384)
    except BaseException:
        return 1
    return 0 if failure is None else 1


def run_controller(api, forge, fixture, result_path, checks):
    """Own worker Job admission, bounded wait, forced fallback and observed active zero."""
    c = api.c
    job, process, fault = api.new_job(), None, None
    result = None
    try:
        jobs = (c.c_void_p*1)(job)
        process = api.create(Path(sys.executable).resolve(),[Path(__file__).resolve(),'--worker',
                             '--forge',forge,'--fixture',fixture,'--worker-result',result_path],
                             fixture,0x2000d,c.byref(jobs),c.sizeof(jobs),worker=True)
        contain_before_resume(api,process,job)
        checks['worker_job'] = True
        deadline = time.monotonic()+60
        while api.exited(process.hProcess) is None:
            api.active(job)
            if time.monotonic() >= deadline:
                raise SmokeFault('admission','worker-timeout')
            time.sleep(0.02)
        code = api.exited(process.hProcess)
        try:
            with open(result_path,'rb') as source:
                raw = source.read(16385)
            result = strict_json(raw,16384,'receipt')
            exact(result,['checks','failure'],'receipt')
            validate_checks(result['checks'])
            observed = result['checks']
            # Only controller sets admission/exit/Job-empty/binary facts.
            if any(observed[key] for key in ('worker_job','worker_exit_zero','job_empty','forced_cleanup','binary_unchanged')):
                raise SmokeFault('receipt','worker-result-invalid')
            if result['failure'] is None:
                if code != 0:
                    raise SmokeFault('receipt','worker-result-invalid')
            else:
                exact(result['failure'],['stage','code'],'receipt')
                if code != 1 or result['failure']['stage'] not in STAGES or result['failure']['code'] not in CODES:
                    raise SmokeFault('receipt','worker-result-invalid')
            checks.update(observed)
            checks['worker_job'], checks['worker_exit_zero'] = True, code == 0
            if result['failure']:
                raise SmokeFault(result['failure']['stage'],result['failure']['code'])
        except OSError:
            raise SmokeFault('receipt','worker-result-invalid') from None
    except SmokeFault as error:
        fault = error
    except BaseException:
        fault = SmokeFault('admission','internal-control-error')
    finally:
        try:
            # All failed campaigns force cleanup, even if a child already exited.
            if fault is not None or api.active(job) != 0:
                checks['forced_cleanup'] = True
                if fault is None:
                    fault = SmokeFault('cleanup','cleanup-unverified')
                api.terminate(job)
            checks['job_empty'] = await_empty(api,job,time.monotonic()+10)
            if not checks['job_empty']:
                fault = SmokeFault('cleanup','cleanup-unverified')
        except BaseException:
            fault = SmokeFault('cleanup','cleanup-unverified')
        finally:
            try:
                if process:
                    try:
                        api.close(process.hThread)
                    finally:
                        api.close(process.hProcess)
            finally:
                api.close(job)
    if fault:
        raise fault


def execute(forge, output_dir):
    """Publish a closed result; unsupported hosts never spawn product or worker."""
    receipt = new_receipt()
    try:
        if sys.platform != 'win32' or sys.version_info < (3,11) or platform.python_implementation() != 'CPython':
            raise SmokeFault('platform','unsupported-platform',True)
        if platform.machine().lower() not in ('amd64','x86_64') or struct.calcsize('P') != 8:
            raise SmokeFault('platform','unsupported-architecture',True)
        if receipt['platform']['build'] < 17763:
            raise SmokeFault('platform','missing-native-api',True)
        api = NativeAPI()
        api.require_native_x64()
        receipt['platform']['architecture'] = 'x64'
        forge = Path(forge).resolve(strict=True)
        receipt['identity']['provided_release_binary']['before'] = file_pin(forge)
        ensure_x64_executable(forge)
        with tempfile.TemporaryDirectory(prefix='forge-windows-console-') as temporary:
            fixture = Path(temporary)/'project'
            fixture.mkdir()
            index = fixture/'forge.workspace.json'
            exclusive_json(index,{'schema_version':'forge.workspace/1','label':LABEL,'resources':[]},4096)
            receipt['fixture']['before'] = file_pin(index,4096)
            try:
                run_controller(api,forge,fixture,Path(temporary)/'worker.json',receipt['checks'])
            finally:
                receipt['fixture']['after'] = file_pin(index,4096)
                receipt['fixture']['unchanged'] = receipt['fixture']['before'] == receipt['fixture']['after'] and sorted(p.name for p in fixture.iterdir()) == ['forge.workspace.json']
                receipt['identity']['provided_release_binary']['after'] = file_pin(forge)
                receipt['checks']['binary_unchanged'] = receipt['identity']['provided_release_binary']['before'] == receipt['identity']['provided_release_binary']['after']
            if not receipt['fixture']['unchanged']:
                raise SmokeFault('query','fixture-changed')
            if not receipt['checks']['binary_unchanged']:
                raise SmokeFault('binary','binary-changed')
        receipt.update(status='passed',scoped_complete=True,failure={'stage':None,'code':None})
    except SmokeFault as error:
        receipt.update(status='incomplete' if error.incomplete else 'failed',scoped_complete=False,
                       failure={'stage':error.stage,'code':error.code})
    except BaseException:
        receipt.update(status='failed',scoped_complete=False,failure={'stage':'receipt','code':'internal-control-error'})
    validate_receipt(receipt)
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True,exist_ok=True)
    exclusive_json(output_dir/OUTPUT,receipt,65536)
    return {'passed':0,'failed':1,'incomplete':2}[receipt['status']]


def main(argv=None):
    """Parse public inputs or the private Job-owned worker invocation without secrets."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--forge',required=True)
    parser.add_argument('--output-dir')
    parser.add_argument('--worker',action='store_true',help=argparse.SUPPRESS)
    parser.add_argument('--fixture',help=argparse.SUPPRESS)
    parser.add_argument('--worker-result',help=argparse.SUPPRESS)
    args = parser.parse_args(argv)
    if args.worker:
        if args.output_dir or not args.fixture or not args.worker_result:
            parser.error('invalid worker inputs')
        return worker_main(args)
    if not args.output_dir or args.fixture or args.worker_result:
        parser.error('output directory required')
    return execute(args.forge,args.output_dir)


if __name__ == '__main__':
    try:
        exit_code = main()
    except Exception:
        # No raw exception/argument/terminal content reaches stdout or stderr.
        exit_code = 1
    sys.exit(exit_code)
