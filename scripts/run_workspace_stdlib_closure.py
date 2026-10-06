#!/usr/bin/env python3
"""Dispatch one protected Linux UID0 synthetic filesystem campaign; never qualify real runtime denial.

Copy hashes bind selected checkout bytes only. They are not source authorization,
independent attestation, interpreter/bootstrap/ELF/libc or loaded-module trust.
The privileged program receives four fixed source bodies, not checkout paths or
caller-selected commands, and imports fixtures only after protecting the copies.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import stat
import subprocess
import sys
import time
import signal
import threading

import verify_workspace as shared
import verify_workspace_os_denial as qualifier

sys.dont_write_bytecode = True
SCHEMA = "forge.stdlib-closure-physical-controls/1"
FILES = ("test_workspace_stdlib_closure.py", "verify_workspace_os_denial.py",
         "test_workspace_os_denial.py", "verify_workspace.py")
SOURCE_LIMIT = 131072
INPUT_LIMIT = 800000
OUTPUT_LIMIT = 32768
TOOL_LIMIT = 16777216
OUTER_SECONDS = 300
# These two reviewed literals are inserted from retained support sources by the author binder.
RUNNER = '"""Run only the protected synthetic Linux closure class; emit no fixture or exception data."""\nimport contextlib\nimport importlib.util\nimport json\nimport os\nfrom pathlib import Path\nimport sys\nimport unittest\n\nsys.dont_write_bytecode = True\nEXPECTED_NAMES = (\n    "test_fixed_support_leaves_complete_chain_and_no_adjacent_credit",\n    "test_fixed_support_ownership_mutation_absence_and_budget_refuse",\n    "test_regular_inventory_and_binary_leaf_alias_agree",\n    "test_distinct_aliases_and_overlapping_roots_keep_complete_counts",\n    "test_relative_parent_components_are_actually_qualified",\n    "test_root_and_directory_links_are_never_admitted",\n    "test_external_regular_target_is_refused_without_target_byte_reads",\n    "test_dangling_cycle_and_special_targets_refuse_whole_proof",\n    "test_nonroot_link_and_regular_target_refuse",\n    "test_nonlink_writable_target_and_ancestor_refuse",\n    "test_sixteen_hops_positive_seventeen_refuses",\n    "test_retarget_after_read_refuses_even_with_same_target_bytes",\n    "test_target_growth_after_pin_refuses_complete_inventory",\n    "test_new_optional_zip_root_refuses_absence_proof",\n    "test_complete_alias_bytes_exact_limit_and_plus_one_refusal",\n    "test_logical_entry_limit_not_renewed_by_overlapping_root",\n    "test_descriptor_cleanup_fault_rejects_and_attempts_independent_closes",\n    "test_expired_budget_refuses_before_open",\n    "test_os_marker_requires_root_level_member_but_may_be_a_proved_alias",\n    "test_root_replacement_during_each_inventory_pass_refuses",\n    "test_superseded_ancestor_replacement_during_each_pass_refuses",\n    "test_retained_root_generation_is_checked_after_final_reopen",\n    "test_actual_stream_and_descriptor_share_exact_handle_boundary",\n    "test_actual_stream_close_fault_keeps_charge_and_attempts_all_owned_cleanup",\n)\nEXPECTED_IDS = sorted("leaf_physical.PhysicalLeafClosureControls." + name for name in EXPECTED_NAMES)\n\n\nclass DiscardText:\n    """Discard test prose instead of retaining tracebacks, fixture paths, source text or capabilities."""\n\n    def write(self, text):\n        """Satisfy text-stream writes without copying their content into the receipt."""\n        return len(text)\n\n    def flush(self):\n        """Provide the stream protocol without a hidden file or retained output buffer."""\n        return None\n\n\nclass ClosedResult(unittest.TestResult):\n    """Count actual unittest events and inspect each owned fixture after its registered cleanup."""\n\n    def __init__(self):\n        """Keep only fixed counters and the reviewed identifiers, never formatted exception data."""\n        super().__init__()\n        self.counts = dict(passed=0, failures=0, errors=0, skips=0,\n                           unexpected_successes=0, expected_failures=0)\n        self.fixture_checks = 0\n        self.fixture_absent = 0\n        self.ids = []\n        self.protocol_valid = True\n\n    def startTest(self, test):\n        """Record a case only when its actual ID is one of the exact reviewed twenty-two."""\n        super().startTest(test)\n        identifier = test.id()\n        if identifier not in EXPECTED_IDS or identifier in self.ids:\n            self.protocol_valid = False\n        else:\n            self.ids.append(identifier)\n\n    def addSuccess(self, test):\n        """Count genuine successful test completion, not discovery or a skipped class."""\n        self.counts["passed"] += 1\n\n    def addFailure(self, test, error):\n        """Count assertion failures without serializing the exception or its arguments."""\n        self.counts["failures"] += 1\n\n    def addError(self, test, error):\n        """Count construction, class and cleanup errors without retaining their traceback."""\n        self.counts["errors"] += 1\n\n    def addSkip(self, test, reason):\n        """A skipped physical case receives no credit, regardless of the supplied reason."""\n        self.counts["skips"] += 1\n\n    def addExpectedFailure(self, test, error):\n        """Keep expected failures separate; this exact physical campaign permits none."""\n        self.counts["expected_failures"] += 1\n\n    def addUnexpectedSuccess(self, test):\n        """Unexpected-success events invalidate the fixed no-exceptions campaign."""\n        self.counts["unexpected_successes"] += 1\n\n    def addSubTest(self, test, subtest, error):\n        """Charge every failed subtest to the appropriate outcome without copying its payload."""\n        if error is not None:\n            name = "failures" if issubclass(error[0], test.failureException) else "errors"\n            self.counts[name] += 1\n\n    def stopTest(self, test):\n        """Check only this testcase\'s known synthetic fixture after unittest has run addCleanup."""\n        private = getattr(test, "private", None)\n        name = getattr(private, "name", None)\n        if isinstance(name, str) and Path(name).parent == Path("/root") and Path(name).name.startswith("forge-leaf-closure-"):\n            self.fixture_checks += 1\n            try:\n                os.lstat(name)\n            except FileNotFoundError:\n                self.fixture_absent += 1\n            except OSError:\n                self.protocol_valid = False\n        else:\n            self.protocol_valid = False\n        super().stopTest(test)\n\n    def wasSuccessful(self):\n        """Keep unittest\'s stop decisions consistent with the redacted custom counters."""\n        return not any(value for name, value in self.counts.items() if name != "passed")\n\n\ndef case_ids(suite):\n    """Enumerate one fixed class without executing it, rejecting extra or nested unknown cases."""\n    identifiers = []\n    for test in suite:\n        if not isinstance(test, unittest.TestCase) or len(identifiers) >= 23:\n            raise ValueError("discovery")\n        identifiers.append(test.id())\n    return sorted(identifiers)\n\n\ndef main():\n    """Import only already-protected source and return closed discovery, outcome and fixture counts."""\n    report = dict(schema_version="forge.stdlib-closure-case-run/1", discovered=0, ran=0,\n                  passed=0, failures=0, errors=0, skips=0, unexpected_successes=0,\n                  expected_failures=0, exact_ids=False, fixture_checks=0, fixture_absent=0,\n                  protocol_valid=False)\n    good = False\n    try:\n        if sys.platform != "linux" or os.getuid() != 0 or os.geteuid() != 0:\n            raise ValueError("environment")\n        directory = Path(__file__).parent\n        sys.path.insert(0, str(directory))\n        with contextlib.redirect_stdout(DiscardText()), contextlib.redirect_stderr(DiscardText()):\n            specification = importlib.util.spec_from_file_location("leaf_physical", directory / "test_workspace_stdlib_closure.py")\n            module = importlib.util.module_from_spec(specification)\n            specification.loader.exec_module(module)\n            suite = unittest.TestLoader().loadTestsFromTestCase(module.PhysicalLeafClosureControls)\n            identifiers = case_ids(suite)\n            report["discovered"] = len(identifiers)\n            report["exact_ids"] = identifiers == EXPECTED_IDS\n            if not report["exact_ids"]:\n                raise ValueError("discovery")\n            result = ClosedResult()\n            suite.run(result)\n            report.update(result.counts, ran=result.testsRun, fixture_checks=result.fixture_checks,\n                          fixture_absent=result.fixture_absent,\n                          protocol_valid=result.protocol_valid and sorted(result.ids) == EXPECTED_IDS)\n            good = (report["ran"] == report["passed"] == report["fixture_checks"] == report["fixture_absent"] == 22\n                    and report["protocol_valid"] and result.wasSuccessful())\n    except BaseException:\n        good = False\n    sys.stdout.write(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\\n")\n    return 0 if good else 1\n\n\nif __name__ == "__main__":\n    raise SystemExit(main())\n'
BOOTSTRAP = '"""Protect bounded copies before imports, supervise one root child, and report cleanup without private paths."""\nimport base64\nimport hashlib\nimport json\nimport os\nimport selectors\nimport signal\nimport stat\nimport subprocess\nimport sys\nimport time\nimport threading\n\nFILES = ("test_workspace_stdlib_closure.py", "verify_workspace_os_denial.py",\n         "test_workspace_os_denial.py", "verify_workspace.py")\nSOURCE_LIMIT = 131072\nINPUT_LIMIT = 800000\nOUTPUT_LIMIT = 32768\nRUN_SECONDS = 240\n# The author binder replaces this literal with the reviewed runner, not a caller-selected program.\nRUNNER = \'"""Run only the protected synthetic Linux closure class; emit no fixture or exception data."""\\nimport contextlib\\nimport importlib.util\\nimport json\\nimport os\\nfrom pathlib import Path\\nimport sys\\nimport unittest\\n\\nsys.dont_write_bytecode = True\\nEXPECTED_NAMES = (\\n    "test_regular_inventory_and_binary_leaf_alias_agree",\\n    "test_distinct_aliases_and_overlapping_roots_keep_complete_counts",\\n    "test_relative_parent_components_are_actually_qualified",\\n    "test_root_and_directory_links_are_never_admitted",\\n    "test_external_regular_target_is_refused_without_target_byte_reads",\\n    "test_dangling_cycle_and_special_targets_refuse_whole_proof",\\n    "test_nonroot_link_and_regular_target_refuse",\\n    "test_nonlink_writable_target_and_ancestor_refuse",\\n    "test_sixteen_hops_positive_seventeen_refuses",\\n    "test_retarget_after_read_refuses_even_with_same_target_bytes",\\n    "test_target_growth_after_pin_refuses_complete_inventory",\\n    "test_new_optional_zip_root_refuses_absence_proof",\\n    "test_complete_alias_bytes_exact_limit_and_plus_one_refusal",\\n    "test_logical_entry_limit_not_renewed_by_overlapping_root",\\n    "test_descriptor_cleanup_fault_rejects_and_attempts_independent_closes",\\n    "test_expired_budget_refuses_before_open",\\n    "test_os_marker_requires_root_level_member_but_may_be_a_proved_alias",\\n    "test_root_replacement_during_each_inventory_pass_refuses",\\n    "test_superseded_ancestor_replacement_during_each_pass_refuses",\\n    "test_retained_root_generation_is_checked_after_final_reopen",\\n    "test_actual_stream_and_descriptor_share_exact_handle_boundary",\\n    "test_actual_stream_close_fault_keeps_charge_and_attempts_all_owned_cleanup",\\n)\\nEXPECTED_IDS = sorted("leaf_physical.PhysicalLeafClosureControls." + name for name in EXPECTED_NAMES)\\n\\n\\nclass DiscardText:\\n    """Discard test prose instead of retaining tracebacks, fixture paths, source text or capabilities."""\\n\\n    def write(self, text):\\n        """Satisfy text-stream writes without copying their content into the receipt."""\\n        return len(text)\\n\\n    def flush(self):\\n        """Provide the stream protocol without a hidden file or retained output buffer."""\\n        return None\\n\\n\\nclass ClosedResult(unittest.TestResult):\\n    """Count actual unittest events and inspect each owned fixture after its registered cleanup."""\\n\\n    def __init__(self):\\n        """Keep only fixed counters and the reviewed identifiers, never formatted exception data."""\\n        super().__init__()\\n        self.counts = dict(passed=0, failures=0, errors=0, skips=0,\\n                           unexpected_successes=0, expected_failures=0)\\n        self.fixture_checks = 0\\n        self.fixture_absent = 0\\n        self.ids = []\\n        self.protocol_valid = True\\n\\n    def startTest(self, test):\\n        """Record a case only when its actual ID is one of the exact reviewed twenty-two."""\\n        super().startTest(test)\\n        identifier = test.id()\\n        if identifier not in EXPECTED_IDS or identifier in self.ids:\\n            self.protocol_valid = False\\n        else:\\n            self.ids.append(identifier)\\n\\n    def addSuccess(self, test):\\n        """Count genuine successful test completion, not discovery or a skipped class."""\\n        self.counts["passed"] += 1\\n\\n    def addFailure(self, test, error):\\n        """Count assertion failures without serializing the exception or its arguments."""\\n        self.counts["failures"] += 1\\n\\n    def addError(self, test, error):\\n        """Count construction, class and cleanup errors without retaining their traceback."""\\n        self.counts["errors"] += 1\\n\\n    def addSkip(self, test, reason):\\n        """A skipped physical case receives no credit, regardless of the supplied reason."""\\n        self.counts["skips"] += 1\\n\\n    def addExpectedFailure(self, test, error):\\n        """Keep expected failures separate; this exact physical campaign permits none."""\\n        self.counts["expected_failures"] += 1\\n\\n    def addUnexpectedSuccess(self, test):\\n        """Unexpected-success events invalidate the fixed no-exceptions campaign."""\\n        self.counts["unexpected_successes"] += 1\\n\\n    def addSubTest(self, test, subtest, error):\\n        """Charge every failed subtest to the appropriate outcome without copying its payload."""\\n        if error is not None:\\n            name = "failures" if issubclass(error[0], test.failureException) else "errors"\\n            self.counts[name] += 1\\n\\n    def stopTest(self, test):\\n        """Check only this testcase\\\'s known synthetic fixture after unittest has run addCleanup."""\\n        private = getattr(test, "private", None)\\n        name = getattr(private, "name", None)\\n        if isinstance(name, str) and Path(name).parent == Path("/root") and Path(name).name.startswith("forge-leaf-closure-"):\\n            self.fixture_checks += 1\\n            try:\\n                os.lstat(name)\\n            except FileNotFoundError:\\n                self.fixture_absent += 1\\n            except OSError:\\n                self.protocol_valid = False\\n        else:\\n            self.protocol_valid = False\\n        super().stopTest(test)\\n\\n    def wasSuccessful(self):\\n        """Keep unittest\\\'s stop decisions consistent with the redacted custom counters."""\\n        return not any(value for name, value in self.counts.items() if name != "passed")\\n\\n\\ndef case_ids(suite):\\n    """Enumerate one fixed class without executing it, rejecting extra or nested unknown cases."""\\n    identifiers = []\\n    for test in suite:\\n        if not isinstance(test, unittest.TestCase) or len(identifiers) >= 23:\\n            raise ValueError("discovery")\\n        identifiers.append(test.id())\\n    return sorted(identifiers)\\n\\n\\ndef main():\\n    """Import only already-protected source and return closed discovery, outcome and fixture counts."""\\n    report = dict(schema_version="forge.stdlib-closure-case-run/1", discovered=0, ran=0,\\n                  passed=0, failures=0, errors=0, skips=0, unexpected_successes=0,\\n                  expected_failures=0, exact_ids=False, fixture_checks=0, fixture_absent=0,\\n                  protocol_valid=False)\\n    good = False\\n    try:\\n        if sys.platform != "linux" or os.getuid() != 0 or os.geteuid() != 0:\\n            raise ValueError("environment")\\n        directory = Path(__file__).parent\\n        sys.path.insert(0, str(directory))\\n        with contextlib.redirect_stdout(DiscardText()), contextlib.redirect_stderr(DiscardText()):\\n            specification = importlib.util.spec_from_file_location("leaf_physical", directory / "test_workspace_stdlib_closure.py")\\n            module = importlib.util.module_from_spec(specification)\\n            specification.loader.exec_module(module)\\n            suite = unittest.TestLoader().loadTestsFromTestCase(module.PhysicalLeafClosureControls)\\n            identifiers = case_ids(suite)\\n            report["discovered"] = len(identifiers)\\n            report["exact_ids"] = identifiers == EXPECTED_IDS\\n            if not report["exact_ids"]:\\n                raise ValueError("discovery")\\n            result = ClosedResult()\\n            suite.run(result)\\n            report.update(result.counts, ran=result.testsRun, fixture_checks=result.fixture_checks,\\n                          fixture_absent=result.fixture_absent,\\n                          protocol_valid=result.protocol_valid and sorted(result.ids) == EXPECTED_IDS)\\n            good = (report["ran"] == report["passed"] == report["fixture_checks"] == report["fixture_absent"] == 22\\n                    and report["protocol_valid"] and result.wasSuccessful())\\n    except BaseException:\\n        good = False\\n    sys.stdout.write(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\\\\n")\\n    return 0 if good else 1\\n\\n\\nif __name__ == "__main__":\\n    raise SystemExit(main())\\n\'\n\n\ndef require_wait_owner():\n    """Require default SIGCHLD and sole main-thread Python ownership before any spawn, wait or direct-child signal."""\n    if (os.name != "posix" or signal.getsignal(signal.SIGCHLD) != signal.SIG_DFL\n            or threading.current_thread() is not threading.main_thread()\n            or len(threading.enumerate()) != 1):\n        raise ValueError("wait-ownership-unqualified")\n\n\nclass DirectChild:\n    """Own only the fixed runner/sudo direct child; never adopt a PID, group or another waiter\'s exit status."""\n\n    def __init__(self, process):\n        """Bind the just-created Popen child, with no verified exit until this owner observes an actual wait status."""\n        self.process = process\n        self.pid = process.pid\n        self.reaped = False\n        self.certain = True\n        self.stop_attempted = False\n        self.exit_code = None\n\n    def observe(self):\n        """Reap once with waitpid; ECHILD or changed policy permanently forbids subsequent signaling."""\n        if self.reaped:\n            return True\n        if not self.certain:\n            raise ValueError("wait-ownership-lost")\n        try:\n            require_wait_owner()\n            waited, status = os.waitpid(self.pid, os.WNOHANG)\n            if waited == 0:\n                return False\n            if waited != self.pid:\n                raise ValueError("unexpected-wait-owner")\n            self.exit_code = os.waitstatus_to_exitcode(status)\n            self.reaped = True\n            self.process.returncode = self.exit_code\n            return True\n        except (OSError, ValueError):\n            self.certain = False\n            raise\n\n    def stop(self):\n        """Signal an original still-unreaped direct child once, then bound its actual settlement to five seconds."""\n        if self.reaped:\n            return True\n        if self.stop_attempted or not self.certain:\n            return False\n        self.stop_attempted = True\n        try:\n            if self.observe():\n                return True\n            require_wait_owner()\n            # Normal SIGCHLD and this sole wait owner keep an exited child\'s PID reserved until our wait.\n            os.kill(self.pid, signal.SIGKILL)\n            deadline = time.monotonic() + 5\n            while time.monotonic() < deadline:\n                if self.observe():\n                    return True\n                time.sleep(0.05)\n            return False\n        except (OSError, ValueError):\n            self.certain = False\n            return False\n\n\ndef generation(value):\n    """Bind a regular file\'s complete observed generation rather than its basename alone."""\n    return (value.st_dev, value.st_ino, value.st_mode, value.st_uid, value.st_gid,\n            value.st_size, value.st_mtime_ns, value.st_ctime_ns)\n\n\ndef directory_identity(value):\n    """Retain directory identity and trust fields while permitting this owner\'s entry creation/removal."""\n    return (value.st_dev, value.st_ino, value.st_mode, value.st_uid, value.st_gid)\n\n\ndef trusted_directory(value):\n    """Require actual root ownership and nonwritability by group/other for every held directory."""\n    if not stat.S_ISDIR(value.st_mode) or value.st_uid != 0 or value.st_mode & 0o022:\n        raise ValueError("directory")\n\n\ndef duplicate_free(pairs):\n    """Reject repeated JSON members before interpreting any source, pin or result field."""\n    result = {}\n    for name, value in pairs:\n        if name in result:\n            raise ValueError("duplicate")\n        result[name] = value\n    return result\n\n\ndef payload():\n    """Bound the complete stdin allocation, encoded cells and aggregate copied bytes before decode/import."""\n    raw = bytearray()\n    while True:\n        block = sys.stdin.buffer.read(min(16384, INPUT_LIMIT + 1 - len(raw)))\n        if not block:\n            break\n        raw.extend(block)\n        if len(raw) > INPUT_LIMIT:\n            raise ValueError("input")\n    value = json.loads(raw, object_pairs_hook=duplicate_free,\n                       parse_constant=lambda unused: (_ for _ in ()).throw(ValueError("constant")))\n    if not isinstance(value, dict) or set(value) != {"schema_version", "sources"} or value["schema_version"] != "forge.stdlib-closure-copy/1":\n        raise ValueError("shape")\n    sources = value["sources"]\n    if not isinstance(sources, dict) or set(sources) != set(FILES):\n        raise ValueError("files")\n    decoded = {}\n    total = 0\n    for name in FILES:\n        cell = sources[name]\n        if (not isinstance(cell, dict) or set(cell) != {"bytes", "sha256", "base64"}\n                or type(cell["bytes"]) is not int or not 0 < cell["bytes"] <= SOURCE_LIMIT\n                or not isinstance(cell["sha256"], str) or len(cell["sha256"]) != 64\n                or any(char not in "0123456789abcdef" for char in cell["sha256"])\n                or not isinstance(cell["base64"], str)\n                or len(cell["base64"]) != 4 * ((cell["bytes"] + 2) // 3)):\n            raise ValueError("source")\n        total += cell["bytes"]\n        if total > 4 * SOURCE_LIMIT:\n            raise ValueError("aggregate")\n        body = base64.b64decode(cell["base64"], validate=True)\n        if len(body) != cell["bytes"] or hashlib.sha256(body).hexdigest() != cell["sha256"]:\n            raise ValueError("pin")\n        decoded[name] = body\n    return decoded\n\n\ndef read_copy(directory, name):\n    """Read a single protected no-follow file through held-directory authority with exact EOF and generation checks."""\n    before = os.stat(name, dir_fd=directory, follow_symlinks=False)\n    if (not stat.S_ISREG(before.st_mode) or before.st_uid != 0 or before.st_mode & 0o022\n            or before.st_nlink != 1 or not 0 < before.st_size <= SOURCE_LIMIT):\n        raise ValueError("copy")\n    descriptor = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC, dir_fd=directory)\n    try:\n        if generation(os.fstat(descriptor)) != generation(before):\n            raise ValueError("generation")\n        body = bytearray()\n        while len(body) < before.st_size:\n            block = os.read(descriptor, min(16384, before.st_size - len(body)))\n            if not block:\n                raise ValueError("short")\n            body.extend(block)\n        if os.read(descriptor, 1) or generation(os.fstat(descriptor)) != generation(before):\n            raise ValueError("growth")\n        if generation(os.stat(name, dir_fd=directory, follow_symlinks=False)) != generation(before):\n            raise ValueError("replacement")\n        return {"bytes": len(body), "sha256": hashlib.sha256(body).hexdigest()}, generation(before)\n    finally:\n        os.close(descriptor)\n\n\ndef write_copy(directory, name, body):\n    """Create an exclusive 0600 root-owned copy, sync it, and verify full bytes before any fixture import."""\n    if not 0 < len(body) <= SOURCE_LIMIT:\n        raise ValueError("copy-bound")\n    descriptor = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC,\n                         0o600, dir_fd=directory)\n    try:\n        offset = 0\n        while offset < len(body):\n            written = os.write(descriptor, body[offset:offset + 16384])\n            if written <= 0:\n                raise ValueError("write")\n            offset += written\n        os.fsync(descriptor)\n    finally:\n        os.close(descriptor)\n    pin, identity = read_copy(directory, name)\n    if pin != {"bytes": len(body), "sha256": hashlib.sha256(body).hexdigest()}:\n        raise ValueError("copy-pin")\n    return pin, identity\n\n\ndef stop_owned(owner):\n    """Settle only this fixed direct child; an actual reap or uncertain owner can never trigger another signal."""\n    return owner.stop()\n\n\ndef execute(directory_path, result):\n    """Drain the fixed child under one deadline; retain only this sole owner\'s actual wait status."""\n    process = owner = selector = None\n    stdout = bytearray()\n    captured = 0\n    closed = True\n    fault = False\n    try:\n        require_wait_owner()\n        process = subprocess.Popen(["/usr/bin/python3", "-I", "-S", "-B", directory_path + "/runner.py"],\n                                   cwd=directory_path, stdin=subprocess.DEVNULL,\n                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE,\n                                   env={"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"})\n        result["started"] = True\n        owner = DirectChild(process)\n        selector = selectors.DefaultSelector()\n        for stream, kind in ((process.stdout, "stdout"), (process.stderr, "stderr")):\n            os.set_blocking(stream.fileno(), False)\n            selector.register(stream, selectors.EVENT_READ, kind)\n        deadline = time.monotonic() + RUN_SECONDS\n        while selector.get_map() or not owner.observe():\n            if time.monotonic() >= deadline:\n                result["deadline"] = True\n                fault = True\n                break\n            for key, unused in selector.select(min(0.2, max(0, deadline - time.monotonic()))):\n                block = os.read(key.fileobj.fileno(), 16384)\n                if not block:\n                    selector.unregister(key.fileobj)\n                    key.fileobj.close()\n                    continue\n                captured += len(block)\n                if captured > OUTPUT_LIMIT:\n                    result["output_bound"] = True\n                    fault = True\n                    break\n                if key.data == "stdout":\n                    stdout.extend(block)\n            if fault:\n                break\n        if fault:\n            result["forced"] = not owner.reaped\n            result["reaped"] = stop_owned(owner)\n        else:\n            if not owner.observe():\n                raise ValueError("child-wait-incomplete")\n            result["reaped"] = True\n            result["output_closed"] = True\n    except BaseException:\n        fault = True\n        if owner is not None:\n            result["forced"] = not owner.reaped\n            result["reaped"] = stop_owned(owner)\n    finally:\n        if selector is not None:\n            try:\n                selector.close()\n            except BaseException:\n                closed = False\n        if process is not None:\n            for stream in (process.stdout, process.stderr):\n                if stream is not None and not stream.closed:\n                    try:\n                        stream.close()\n                    except BaseException:\n                        closed = False\n        result["close_fault"] = not closed\n        if owner is not None:\n            result["exit_code"] = owner.exit_code\n            result["ownership_verified"] = owner.certain\n    case_report = None\n    if not fault and closed:\n        try:\n            case_report = json.loads(stdout, object_pairs_hook=duplicate_free)\n        except (ValueError, TypeError):\n            pass\n    return case_report\n\n\ndef main():\n    """Own fixed /root copies and process cleanup; report errors only as closed redacted state."""\n    report = dict(schema_version="forge.stdlib-closure-root-run/1", environment=False,\n                  source_pins={}, runner_pin=None, protected_copy=False, source_unchanged=False,\n                  execution=None, process=None, protected_directory_removed=False, fault=None)\n    parent = directory = None\n    directory_before = None\n    name = None\n    identities = {}\n    created = []\n    sources = None\n    try:\n        if sys.platform != "linux" or os.getuid() != 0 or os.geteuid() != 0:\n            raise ValueError("environment")\n        report["environment"] = True\n        sources = payload()\n        root_fd = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)\n        try:\n            trusted_directory(os.fstat(root_fd))\n            parent = os.open("root", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=root_fd)\n        finally:\n            os.close(root_fd)\n        parent_before = os.fstat(parent)\n        trusted_directory(parent_before)\n        if directory_identity(os.lstat("/root")) != directory_identity(parent_before):\n            raise ValueError("parent")\n        name = "forge-stdlib-controls-" + os.urandom(16).hex()\n        os.mkdir(name, 0o700, dir_fd=parent)\n        directory = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=parent)\n        directory_before = os.fstat(directory)\n        trusted_directory(directory_before)\n        if stat.S_IMODE(directory_before.st_mode) != 0o700:\n            raise ValueError("private-mode")\n        all_sources = dict(sources)\n        all_sources["runner.py"] = RUNNER.encode("utf-8")\n        for filename, body in all_sources.items():\n            # Remember only files actually created, including a failed partial write for cleanup.\n            created.append(filename)\n            pin, identity = write_copy(directory, filename, body)\n            identities[filename] = identity\n            if filename == "runner.py":\n                report["runner_pin"] = pin\n            else:\n                report["source_pins"][filename] = pin\n        os.fsync(directory)\n        report["protected_copy"] = True\n        # Keep ownership state in the caller before dispatch, including exceptional construction paths.\n        report["process"] = dict(exit_code=None, reaped=False, started=False, ownership_verified=False,\n                                 forced=False, deadline=False, output_closed=False, output_bound=False, close_fault=False)\n        report["execution"] = execute("/root/" + name, report["process"])\n        unchanged = directory_identity(os.fstat(parent)) == directory_identity(parent_before)\n        unchanged = unchanged and directory_identity(os.lstat("/root")) == directory_identity(parent_before)\n        unchanged = unchanged and directory_identity(os.stat(name, dir_fd=parent, follow_symlinks=False)) == directory_identity(directory_before)\n        unchanged = unchanged and directory_identity(os.fstat(directory)) == directory_identity(directory_before)\n        for filename in created:\n            pin, identity = read_copy(directory, filename)\n            expected = report["runner_pin"] if filename == "runner.py" else report["source_pins"][filename]\n            unchanged = unchanged and identity == identities[filename] and pin == expected\n        report["source_unchanged"] = unchanged\n    except BaseException:\n        report["fault"] = "protected-execution-unqualified"\n    finally:\n        process_record = report["process"]\n        may_remove = (process_record is None or process_record["started"] is False\n                      or (process_record["reaped"] is True and process_record["ownership_verified"] is True))\n        cleanup = may_remove\n        if directory is not None and may_remove:\n            try:\n                for filename in created:\n                    # A changed or incompletely pinned entry is not treated as this owner\'s removable file.\n                    current = os.stat(filename, dir_fd=directory, follow_symlinks=False)\n                    if identities.get(filename) != generation(current):\n                        cleanup = False\n                        continue\n                    os.unlink(filename, dir_fd=directory)\n                os.fsync(directory)\n            except BaseException:\n                cleanup = False\n            try:\n                os.close(directory)\n            except BaseException:\n                cleanup = False\n        if directory is not None and not may_remove:\n            try:\n                os.close(directory)\n            except BaseException:\n                cleanup = False\n        if parent is not None and name is not None and may_remove:\n            try:\n                # rmdir refuses unknown leftovers instead of deleting a foreign object recursively.\n                if directory_before is None or directory_identity(os.stat(name, dir_fd=parent, follow_symlinks=False)) != directory_identity(directory_before):\n                    raise ValueError("cleanup-identity")\n                os.rmdir(name, dir_fd=parent)\n                os.fsync(parent)\n                try:\n                    os.stat(name, dir_fd=parent, follow_symlinks=False)\n                    cleanup = False\n                except FileNotFoundError:\n                    pass\n            except BaseException:\n                cleanup = False\n        else:\n            cleanup = False\n        if parent is not None:\n            try:\n                os.close(parent)\n            except BaseException:\n                cleanup = False\n        report["protected_directory_removed"] = cleanup\n        if not cleanup:\n            report["fault"] = "protected-cleanup-unverified"\n    sys.stdout.write(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\\n")\n    return 0 if report["fault"] is None else 1\n\n\nif __name__ == "__main__":\n    raise SystemExit(main())\n'


def require_wait_owner():
    """Require default SIGCHLD and sole main-thread Python ownership before any spawn, wait or direct-child signal."""
    if (os.name != "posix" or signal.getsignal(signal.SIGCHLD) != signal.SIG_DFL
            or threading.current_thread() is not threading.main_thread()
            or len(threading.enumerate()) != 1):
        raise ValueError("wait-ownership-unqualified")


class DirectChild:
    """Own only the fixed runner/sudo direct child; never adopt a PID, group or another waiter's exit status."""

    def __init__(self, process):
        """Bind the just-created Popen child, with no verified exit until this owner observes an actual wait status."""
        self.process = process
        self.pid = process.pid
        self.reaped = False
        self.certain = True
        self.stop_attempted = False
        self.exit_code = None

    def observe(self):
        """Reap once with waitpid; ECHILD or changed policy permanently forbids subsequent signaling."""
        if self.reaped:
            return True
        if not self.certain:
            raise ValueError("wait-ownership-lost")
        try:
            require_wait_owner()
            waited, status = os.waitpid(self.pid, os.WNOHANG)
            if waited == 0:
                return False
            if waited != self.pid:
                raise ValueError("unexpected-wait-owner")
            self.exit_code = os.waitstatus_to_exitcode(status)
            self.reaped = True
            self.process.returncode = self.exit_code
            return True
        except (OSError, ValueError):
            self.certain = False
            raise

    def stop(self):
        """Signal an original still-unreaped direct child once, then bound its actual settlement to five seconds."""
        if self.reaped:
            return True
        if self.stop_attempted or not self.certain:
            return False
        self.stop_attempted = True
        try:
            if self.observe():
                return True
            require_wait_owner()
            # Normal SIGCHLD and this sole wait owner keep an exited child's PID reserved until our wait.
            os.kill(self.pid, signal.SIGKILL)
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if self.observe():
                    return True
                time.sleep(0.05)
            return False
        except (OSError, ValueError):
            self.certain = False
            return False


def duplicate_free(pairs):
    """Reject repeated root-result members instead of treating the last value as authoritative."""
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate")
        result[key] = value
    return result


def exact_pin(value):
    """Accept only a positive exact-integer byte count and lowercase SHA256 in a closed pin."""
    return (isinstance(value, dict) and set(value) == {"bytes", "sha256"}
            and type(value["bytes"]) is int and value["bytes"] > 0
            and isinstance(value["sha256"], str)
            and re.fullmatch(r"[0-9a-f]{64}", value["sha256"]) is not None)


def byte_pin(body):
    """Bind an already-bounded literal source body without claiming its authorization or loaded use."""
    return {"bytes": len(body), "sha256": hashlib.sha256(body).hexdigest()}


def fixed_tool(role):
    """Qualify only the existing fixed distro tool and bound its byte pin; absence has no fallback."""
    selected = {"python": "/usr/bin/python3", "sudo": "/usr/bin/sudo"}[role]
    path = qualifier.administration_tool(selected)
    before = path.lstat()
    if not 0 < before.st_size <= TOOL_LIMIT:
        raise ValueError("tool-bound")
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC)
    try:
        if qualifier.metadata(os.fstat(descriptor)) != qualifier.metadata(before):
            raise ValueError("tool-generation")
        digest = hashlib.sha256()
        total = 0
        while total < before.st_size:
            block = os.read(descriptor, min(16384, before.st_size - total))
            if not block:
                raise ValueError("tool-short")
            total += len(block)
            digest.update(block)
        if os.read(descriptor, 1) or qualifier.metadata(os.fstat(descriptor)) != qualifier.metadata(before):
            raise ValueError("tool-growth")
        if qualifier.metadata(path.lstat()) != qualifier.metadata(before):
            raise ValueError("tool-replacement")
    finally:
        os.close(descriptor)
    return {"selected": selected, "resolved": str(path), "pin": {"bytes": total, "sha256": digest.hexdigest()}}


def git_output(root, arguments):
    """Use the ordinary bounded command reader for a fixed Git read; retain no raw repository prose."""
    value = shared.run_command(["git", *arguments], root, 20)
    if value["failure"] is not None or value["exit_code"] != 0:
        raise ValueError("source-binding")
    return value["output"]


def capture_sources(root, arguments):
    """Bind commit parents/context and exact source bytes; hosted dispatch requires exact HEAD blobs."""
    commit = git_output(root, ["rev-parse", "HEAD"]).strip().decode("ascii")
    if not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", commit):
        raise ValueError("commit")
    parents = shared.commit_parents(git_output(root, ["cat-file", "commit", "HEAD"]), commit)
    context = shared.checkout_binding({"source_commit": commit, "ordered_parents": parents},
                                      arguments.expected_commit, event=arguments.event,
                                      checkout_kind=arguments.checkout_kind,
                                      requested_head=arguments.requested_head,
                                      requested_base=arguments.requested_base)
    bodies = {name: qualifier.read_bytes(root / "scripts" / name, SOURCE_LIMIT) for name in FILES}
    provenance_names = ["scripts/" + name for name in FILES]
    provenance_names += ["scripts/run_workspace_stdlib_closure.py", ".github/workflows/workspace-verification.yml"]
    pins = {}
    tree_matches = {}
    for name in provenance_names:
        body = bodies[Path(name).name] if name.startswith("scripts/") and Path(name).name in bodies else qualifier.read_bytes(root / name, SOURCE_LIMIT)
        pins[name] = byte_pin(body)
        captured = shared.run_command(["git", "show", commit + ":" + name], root, 20)
        tree_matches[name] = captured["failure"] is None and captured["exit_code"] == 0 and captured["output"] == body
    if arguments.event != "local" and not all(tree_matches.values()):
        raise ValueError("hosted-source-drift")
    if git_output(root, ["rev-parse", "HEAD"]).strip() != commit.encode("ascii"):
        raise ValueError("head-drift")
    return bodies, {"checkout": context, "source_pins": pins, "head_blob_matches": tree_matches,
                    "source_basis": "exact-head-blobs" if all(tree_matches.values()) else "local-explicit-working-bytes"}


def copy_payload(bodies):
    """Bound all four raw and encoded source bodies before constructing the privileged stdin envelope."""
    if set(bodies) != set(FILES):
        raise ValueError("source-set")
    total = 0
    for body in bodies.values():
        if not isinstance(body, bytes) or not 0 < len(body) <= SOURCE_LIMIT:
            raise ValueError("source-bound")
        total += len(body)
    if total > 4 * SOURCE_LIMIT:
        raise ValueError("source-aggregate")
    sources = {name: dict(byte_pin(body), base64=base64.b64encode(body).decode("ascii")) for name, body in bodies.items()}
    raw = json.dumps({"schema_version": "forge.stdlib-closure-copy/1", "sources": sources},
                     sort_keys=True, separators=(",", ":")).encode("ascii")
    if len(raw) > INPUT_LIMIT:
        raise ValueError("envelope-bound")
    return raw


def run_root(raw):
    """Supervise only the original sudo direct child with a bounded envelope, output and verified wait ownership."""
    if not isinstance(raw, bytes) or not 0 < len(raw) <= INPUT_LIMIT or not 0 < len(BOOTSTRAP.encode("utf-8")) <= SOURCE_LIMIT:
        raise ValueError("dispatch-bound")
    command = ["/usr/bin/sudo", "-n", "--", "/usr/bin/python3", "-I", "-S", "-B", "-c", BOOTSTRAP]
    process = owner = selector = None
    stdout = bytearray()
    count = sent = 0
    fault = None
    closed = True
    try:
        require_wait_owner()
        process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, env={"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"})
        owner = DirectChild(process)
        selector = selectors.DefaultSelector()
        for stream, kind, event in ((process.stdin, "stdin", selectors.EVENT_WRITE),
                                    (process.stdout, "stdout", selectors.EVENT_READ),
                                    (process.stderr, "stderr", selectors.EVENT_READ)):
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, event, kind)
        deadline = time.monotonic() + OUTER_SECONDS
        while selector.get_map() or not owner.observe():
            if time.monotonic() >= deadline:
                fault = "outer-deadline"
                break
            for key, unused in selector.select(min(0.2, max(0, deadline - time.monotonic()))):
                if key.data == "stdin":
                    sent += os.write(key.fileobj.fileno(), raw[sent:sent + 16384])
                    if sent == len(raw):
                        selector.unregister(key.fileobj)
                        key.fileobj.close()
                    continue
                block = os.read(key.fileobj.fileno(), 16384)
                if not block:
                    selector.unregister(key.fileobj)
                    key.fileobj.close()
                    continue
                count += len(block)
                if count > OUTPUT_LIMIT:
                    fault = "outer-output-bound"
                    break
                if key.data == "stdout":
                    stdout.extend(block)
            if fault is not None:
                break
        if fault is None and not owner.observe():
            fault = "outer-wait-unverified"
    except (OSError, ValueError, subprocess.SubprocessError):
        fault = "outer-dispatch-unqualified"
    finally:
        if owner is not None and not owner.reaped:
            # No worker claim about a privileged descendant follows direct-child settlement.
            if not owner.stop():
                fault = "outer-cleanup-unverified"
        if selector is not None:
            try:
                selector.close()
            except OSError:
                closed = False
        if process is not None:
            for stream in (process.stdin, process.stdout, process.stderr):
                if stream is not None and not stream.closed:
                    try:
                        stream.close()
                    except OSError:
                        closed = False
    if not closed:
        fault = "outer-close-unverified"
    return {"exit_code": None if owner is None else owner.exit_code,
            "reaped": owner is not None and owner.reaped and owner.certain,
            "output_closed": closed, "fault": fault,
            "output": bytes(stdout) if fault is None else b""}


def closed_root_result(value, pins, runner_pin):
    """Reject malformed or incomplete physical evidence, including passed-looking bytes after exit/cleanup failure."""
    keys = {"schema_version", "environment", "source_pins", "runner_pin", "protected_copy", "source_unchanged",
            "execution", "process", "protected_directory_removed", "fault"}
    if not isinstance(value, dict) or set(value) != keys or value["schema_version"] != "forge.stdlib-closure-root-run/1":
        return False
    if any(value[name] is not True for name in ("environment", "protected_copy", "source_unchanged", "protected_directory_removed")) or value["fault"] is not None:
        return False
    if value["source_pins"] != pins or value["runner_pin"] != runner_pin or not all(exact_pin(pin) for pin in pins.values()):
        return False
    process = value["process"]
    if (not isinstance(process, dict) or set(process) != {"exit_code", "reaped", "started", "ownership_verified", "forced", "deadline", "output_closed", "output_bound", "close_fault"}
            or type(process["exit_code"]) is not int or process["exit_code"] != 0
            or process["reaped"] is not True or process["started"] is not True or process["ownership_verified"] is not True or process["output_closed"] is not True
            or any(process[name] is not False for name in ("forced", "deadline", "output_bound", "close_fault"))):
        return False
    cases = value["execution"]
    expected_keys = {"schema_version", "discovered", "ran", "passed", "failures", "errors", "skips", "unexpected_successes",
                     "expected_failures", "exact_ids", "fixture_checks", "fixture_absent", "protocol_valid"}
    if not isinstance(cases, dict) or set(cases) != expected_keys or cases["schema_version"] != "forge.stdlib-closure-case-run/1":
        return False
    if cases["exact_ids"] is not True or cases["protocol_valid"] is not True:
        return False
    positive = ("discovered", "ran", "passed", "fixture_checks", "fixture_absent")
    negative = ("failures", "errors", "skips", "unexpected_successes", "expected_failures")
    return (all(type(cases[name]) is int and cases[name] == 24 for name in positive)
            and all(type(cases[name]) is int and cases[name] == 0 for name in negative))


def safe_root_result(value, pins, runner_pin):
    """Retain only closed redacted negative or positive reports; unknown fields and malformed counters are discarded."""
    keys = {"schema_version", "environment", "source_pins", "runner_pin", "protected_copy", "source_unchanged",
            "execution", "process", "protected_directory_removed", "fault"}
    if not isinstance(value, dict) or set(value) != keys or value["schema_version"] != "forge.stdlib-closure-root-run/1":
        return None
    if any(type(value[name]) is not bool for name in ("environment", "protected_copy", "source_unchanged", "protected_directory_removed")):
        return None
    if value["fault"] is not None and (not isinstance(value["fault"], str) or value["fault"] not in {"protected-execution-unqualified", "protected-cleanup-unverified"}):
        return None
    source_pins = value["source_pins"]
    if not isinstance(source_pins, dict) or not set(source_pins) <= set(pins):
        return None
    if any(not exact_pin(pin) or pin != pins[name] for name, pin in source_pins.items()):
        return None
    if value["runner_pin"] is not None and (not exact_pin(value["runner_pin"]) or value["runner_pin"] != runner_pin):
        return None
    process = value["process"]
    if process is not None:
        if not isinstance(process, dict) or set(process) != {"exit_code", "reaped", "started", "ownership_verified", "forced", "deadline", "output_closed", "output_bound", "close_fault"}:
            return None
        if process["exit_code"] is not None and (type(process["exit_code"]) is not int or not -255 <= process["exit_code"] <= 255):
            return None
        if any(type(process[name]) is not bool for name in ("reaped", "started", "ownership_verified", "forced", "deadline", "output_closed", "output_bound", "close_fault")):
            return None
    cases = value["execution"]
    if cases is not None:
        keys = {"schema_version", "discovered", "ran", "passed", "failures", "errors", "skips", "unexpected_successes",
                "expected_failures", "exact_ids", "fixture_checks", "fixture_absent", "protocol_valid"}
        if not isinstance(cases, dict) or set(cases) != keys or cases["schema_version"] != "forge.stdlib-closure-case-run/1":
            return None
        if type(cases["exact_ids"]) is not bool or type(cases["protocol_valid"]) is not bool:
            return None
        counters = keys - {"schema_version", "exact_ids", "protocol_valid"}
        if any(type(cases[name]) is not int or not 0 <= cases[name] <= 1000 for name in counters):
            return None
    return value


def outcome(transport, pins, runner_pin):
    """Accept no case credit without normal outer exit, a closed root report and all cleanup predicates."""
    if transport["fault"] is not None or transport["reaped"] is not True or transport["output_closed"] is not True or type(transport["exit_code"]) is not int:
        return None, False
    try:
        value = json.loads(transport["output"], object_pairs_hook=duplicate_free)
    except (TypeError, ValueError):
        return None, False
    retained = safe_root_result(value, pins, runner_pin)
    return retained, transport["exit_code"] == 0 and retained is not None and closed_root_result(retained, pins, runner_pin)


def requested_context(arguments):
    """Retain only supported event labels and exact requested Git identifiers, even when dispatch is unavailable."""
    value = {"event": arguments.event, "kind": arguments.checkout_kind,
             "expected_tested_commit": arguments.expected_commit,
             "requested_head": arguments.requested_head, "requested_base": arguments.requested_base}
    for name in ("expected_tested_commit", "requested_head", "requested_base"):
        if value[name] is not None and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", value[name]) is None:
            raise ValueError("requested-context")
    return value


def main():
    """Write a separate redacted receipt and fail the step whenever this physical campaign is unqualified."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--expected-commit")
    parser.add_argument("--event", default="local", choices=("local", "pull_request", "push", "workflow_dispatch"))
    parser.add_argument("--checkout-kind", default="local", choices=("local", "pull-request-merge", "head"))
    parser.add_argument("--requested-head")
    parser.add_argument("--requested-base")
    arguments = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    receipt = {"schema_version": SCHEMA, "status": "unqualified", "acceptance": False,
               "scope": "synthetic-linux-root-filesystem-controls-only", "requested_context": None, "source_identity": None,
               "source_unchanged": False, "tools": None, "tools_unchanged": False,
               "bootstrap_pin": byte_pin(BOOTSTRAP.encode("utf-8")), "runner_pin": byte_pin(RUNNER.encode("utf-8")),
               "physical_result": None, "outer": None, "failure": "dispatch-unqualified",
               "qualification": {"ip_denial": "not-evaluated", "bootstrap_trust": "not-evaluated",
                                 "loaded_bytes": "not-evaluated", "human_authorization": "not-established"}}
    good = False
    try:
        receipt["requested_context"] = requested_context(arguments)
        if sys.platform != "linux" or os.name != "posix":
            raise ValueError("unsupported")
        bodies, identity = capture_sources(root, arguments)
        receipt["source_identity"] = identity
        tools = {name: fixed_tool(name) for name in ("python", "sudo")}
        receipt["tools"] = tools
        transport = run_root(copy_payload(bodies))
        receipt["outer"] = {name: transport[name] for name in ("exit_code", "reaped", "output_closed", "fault")}
        pins = {name: byte_pin(body) for name, body in bodies.items()}
        physical, passed = outcome(transport, pins, receipt["runner_pin"])
        receipt["physical_result"] = physical
        after_bodies, after_identity = capture_sources(root, arguments)
        receipt["source_unchanged"] = bodies == after_bodies and identity == after_identity
        receipt["tools_unchanged"] = tools == {name: fixed_tool(name) for name in ("python", "sudo")}
        good = passed and receipt["source_unchanged"] and receipt["tools_unchanged"]
        receipt["failure"] = None if good else "physical-campaign-unqualified"
    except (OSError, ValueError, TypeError, KeyError, RuntimeError):
        good = False
    receipt["status"] = "passed" if good else "unqualified"
    try:
        shared.atomic_receipt(arguments.output_dir / "workspace-stdlib-closure-physical-controls.json", receipt)
    except (OSError, ValueError):
        print("Physical control receipt publication unverified", file=sys.stderr)
        return 1
    print("Physical synthetic controls: qualified" if good else "Physical synthetic controls: unqualified")
    return 0 if good else 1


if __name__ == "__main__":
    raise SystemExit(main())
