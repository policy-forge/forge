#!/usr/bin/env python3
"""Stdlib adapter controls for the OS-denial helper/wrapper; no native qualification.

These controls call real controller, reader and publication bodies. Native
operations are replaced explicitly; successful mocks do not calibrate a network.
"""
import contextlib
import copy
import ctypes
import errno
import importlib.util
import io
import json
import os
from pathlib import Path
import select
import signal
import stat
import sys
import tempfile
import time
import types
import unittest
from unittest import mock

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))


def load(name, path):
    """Load only the assigned Python control/runner source without invoking its main guard."""
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


native = load("os_denial_native_controls", HERE / "test_workspace_os_denial.py")
wrapper = load("os_denial_wrapper_controls", HERE / "verify_workspace_os_denial.py")


def plan():
    """Return fixed private mock inputs with a live absolute budget and four pins."""
    pin = {"bytes": 4, "sha256": "a" * 64}
    return {"schema": "forge.os-denial-private-plan/1", "root": "/mock/root",
            "forge": "/mock/forge", "output_dir": "/mock/output", "ip": "/usr/sbin/ip",
            "python": "/usr/bin/python3.11", "target_uid": 1001, "target_gid": 1001,
            "deadline_monotonic_ns": time.monotonic_ns() + 590000000000,
            "source_pins": {key: dict(pin) for key in native.SOURCES}, "release_pin": dict(pin)}


def client():
    """Construct a full closed mock client denominator for reader controls only."""
    declared = {"operation_%02d" % n for n in range(39)}
    value = {"schema_version": "forge.workspace-client-verification/2", "status": "passed",
             "checks": sorted(native.CHECKS), "check_count": 16, "declared_operation_count": 39,
             "observed_operations": ["operation_00"], "unobserved_operations": sorted(declared - {"operation_00"}),
             "operation_outcomes": {"operation_00": {"succeeded": 1, "rejected": 0, "transport_failed": 0}}}
    return value, declared


def proof_record():
    """Provide a structurally native-shaped blocked observation for actual fence controls."""
    status = {"Uid": "1001\t1001\t1001\t1001", "Gid": "1001\t1001\t1001\t1001",
              "Groups": "", "Threads": "1", "NoNewPrivs": "1"}
    status.update(dict.fromkeys(("CapInh", "CapPrm", "CapEff", "CapBnd", "CapAmb"), "0000000000000000"))
    return {"pid": 41, "start": 29, "net": (8, 7), "status": status,
            "fds": {0: "/dev/null", 1: "pipe:[42]", 2: "pipe:[42]", 9: "pipe:[43]"}}


def bare_native():
    """Build adapter state without constructing libc or making any native call."""
    api = native.Native.__new__(native.Native)
    api.plan = plan()
    api.end, api.work_end, api.setup_end = time.monotonic() + 10, time.monotonic() + 9, time.monotonic() + 8
    api.in_setup = True
    api.children, api.observed, api.names, api.netfds, api.servers = [], {}, {}, {}, {}
    api.names_attempted, api.links_attempted, api.pending_fds = [], [], set()
    api.links = {}
    api.local_reference_failure, api.forced, api.scan_complete = False, False, True
    api.created, api.base, api.workspace = True, None, Path("/mock/worker")
    return api


class FakeNative:
    """Replace individual native operations while retaining actual controller ordering/gates."""

    def __init__(self, inputs, fault=None, cleanup=None):
        """Keep a call journal and fault boundary rather than fabricate a final receipt."""
        self.inputs, self.fault, self.events = inputs, fault, []
        self.cleanup_value = cleanup or {"state": "verified", "owned_processes_empty": True,
            "owned_references_closed": True, "owned_names_absent": True, "virtual_links_absent": True, "forced": False}
        self.account_value = {"instances": 3, "namespace_checked": 3, "privilege_checked": 3,
                              "scan_complete": True, "scope": "observed-process-instances-only"}

    def step(self, name):
        """Inject a failure at one actual phase boundary with no native call."""
        self.events.append(name)
        if self.fault and self.fault[0] == name:
            raise self.fault[1]

    def qualify(self):
        """Return safe mocked platform metadata after journaling qualification."""
        self.step("qualify")
        return {"os": "Linux", "machine": "x86_64", "kernel_release": "mock-6.17", "python_version": "3.11.9"}

    def open_output(self):
        """Journal the actual controller's fixed private output reservation phase."""
        self.step("output")

    def protect(self):
        """Return mock identities matching the input pins; no executable is copied or run."""
        self.step("protect")
        return self.inputs["release_pin"], self.inputs["source_pins"]

    def setup(self):
        """Journal resource setup for ordering and cancellation controls."""
        self.step("setup")

    def dut_only(self):
        """Journal separate before/after DUT inventories."""
        self.step("dut-only")

    def server(self, role):
        """Journal each fixed fixture role without creating a socket/process."""
        self.step("server:" + role)

    def finish_setup(self):
        """Journal the nonrenewing setup transition."""
        self.step("setup-finished")

    def probe(self, family, role):
        """Return an individual adapter result; actual controller validates/order-counts it."""
        self.step("probe:" + family + ":" + role)
        return {"outcome": "denied", "errno": 101, "challenge_verified": False} if role == "dut-external" else {
            "outcome": "connected", "errno": None, "challenge_verified": True}

    def run_client(self):
        """Validate actual mock raw client bytes using the real helper reader."""
        self.step("client")
        value, declared = client()
        raw = native.canonical(value)
        parsed = native.client_receipt(raw, declared)
        return 0, parsed, {"bytes": len(raw), "sha256": native.hashlib.sha256(raw).hexdigest()}

    def stable(self):
        """Journal before/after identity reconciliation."""
        self.step("stable")

    def cleanup(self):
        """Return independently selected cleanup facts after journaling cleanup."""
        self.step("cleanup")
        return dict(self.cleanup_value)

    def accounting(self):
        """Return selected observed-instance facts after actual controller cleanup."""
        self.step("accounting")
        return dict(self.account_value)


class FakeStream:
    """Represent one explicitly identified command pipe; no descriptor operation is real."""

    def __init__(self, fd):
        """Keep a stable mocked file number and closure count."""
        self.fd, self.closes = fd, 0

    def fileno(self):
        """Return the private mock descriptor relation used by the real command body."""
        return self.fd

    def close(self):
        """Record closure rather than close any host descriptor."""
        self.closes += 1


class FakeSelector:
    """Schedule independently registered stdout/stderr/stdin events without host IO."""

    def __init__(self):
        """Start with an empty relation inventory and deterministic iteration counter."""
        self.active, self.ticks = {}, 0

    def register(self, stream, events):
        """Record the actual requested direction for each stream."""
        self.active[stream.fd] = (stream, events)

    def unregister(self, stream):
        """Remove only the actual EOF/closed-input stream."""
        del self.active[stream.fd]

    def get_map(self):
        """Expose pending independent stream ownership to the actual command loop."""
        return self.active

    def select(self, timeout):
        """Advance one bounded clock tick and service only still-registered streams."""
        self.ticks += 1
        return [(types.SimpleNamespace(fileobj=stream), events) for stream, events in self.active.values()]

    def close(self):
        """Close the private mocked selection inventory."""
        self.active.clear()


class FakeCommandChild:
    """Own a mocked actual completion result independently of pipe EOF."""

    def __init__(self, selector, exit_after=2, input_bytes=None):
        """Create separate stdout/stderr and optional stdin pipe relations."""
        self.selector, self.exit_after = selector, exit_after
        self.stdout, self.stderr = FakeStream(31), FakeStream(32)
        self.stdin = FakeStream(33) if input_bytes is not None else None
        self.killed, self.waits = False, []

    def poll(self):
        """Return natural0, runningNone or actual mocked forced-9 completion."""
        return -9 if self.killed else 0 if self.selector.ticks >= self.exit_after else None

    def kill(self):
        """Latch forced completion without signalling any process."""
        self.killed = True

    def wait(self, timeout):
        """Retain the actual wait call and return this child's real mock status."""
        self.waits.append(timeout)
        return self.poll()


class NativeControls(unittest.TestCase):
    """Exercise the actual native controller/parsers/fences through explicit adapters."""

    def test_plan_canonical_exact_keys_and_types(self):
        """Reject bool IDs, unknown targets and noncanonical private input."""
        value = plan()
        self.assertEqual(native.parse_plan(native.canonical(value), time.monotonic_ns()), value)
        for change in ({"target_uid": True}, {"arbitrary_command": "secret"}, {"deadline_monotonic_ns": 1}):
            bad = dict(value, **change)
            with self.subTest(change=change), self.assertRaises(native.Failure):
                native.parse_plan(native.canonical(bad), time.monotonic_ns())
        with self.assertRaises(native.Failure):
            native.parse_plan(json.dumps(value).encode(), time.monotonic_ns())

    def test_strict_json_bounds_duplicate_nonfinite_and_depth(self):
        """Close decoded duplicates, nonfinite constants, byte caps and depth limits."""
        for raw in (b'{"a":1,"\\u0061":2}', b'{"v":NaN}', b"[" * 18 + b"0" + b"]" * 18,
                    b" " * (native.MAX_PLAN + 1)):
            with self.subTest(raw=raw[:20]), self.assertRaises(native.Failure):
                native.decode(raw, native.MAX_PLAN)

    def test_canonical_matches_actual_immutable_shared_publisher(self):
        """Bind plan/client encoding to real compact ASCII shared bytes, including Unicode."""
        for value in (plan(), client()[0], {"text": "é雪", "bool": False}):
            self.assertEqual(native.canonical(value), wrapper.shared.canonical_bytes(value))

    def test_closed_failure_never_accepts_raw_text(self):
        """A private secret/path cannot become an allowlisted failure code."""
        with self.assertRaises(ValueError):
            native.Failure("secret /private/path")

    def test_actual_controller_orders_twelve_rows_and_client(self):
        """Actual controller surrounds one unchanged client adapter with six probes per phase."""
        p, api = plan(), FakeNative(plan())
        api.inputs = p
        value = native.execute(p, api)
        self.assertEqual(value["status"], "passed")
        self.assertEqual(len(value["probes"]), 12)
        self.assertTrue(all(row["status"] == "passed" for row in value["probes"]))
        before = api.events[:api.events.index("client")]
        self.assertEqual(sum(x.startswith("probe:") for x in before), 6)
        self.assertLess(api.events.index("cleanup"), api.events.index("accounting"))
        self.assertEqual(value["attempted_egress"], {"state": "unmeasured", "count": None})

    def test_first_probe_failure_stops_client_and_preserves_cleanup_cause(self):
        """A calibration exception prevents client execution; independent cleanup does not mask it."""
        p = plan()
        cleanup = {"state": "unverified", "owned_processes_empty": False, "owned_references_closed": False,
                   "owned_names_absent": False, "virtual_links_absent": False, "forced": True}
        api = FakeNative(p, ("probe:ipv4:calibration-external", native.Failure("probe-calibration-failed")), cleanup)
        value = native.execute(p, api)
        self.assertNotIn("client", api.events)
        self.assertEqual(value["failure"], "probe-calibration-failed")
        self.assertEqual(value["cleanup_failure"], "cleanup-unverified")
        self.assertEqual(value["status"], "failed")

    def test_unverified_denial_refusal_timeout_have_no_credit(self):
        """Only exact ENETUNREACH101/no challenge passes a DUT external row."""
        row = {"role": "dut-external"}
        for result in ({"outcome": "unverified", "errno": None, "challenge_verified": False},
                       {"outcome": "denied", "errno": True, "challenge_verified": False},
                       {"outcome": "connected", "errno": None, "challenge_verified": True}):
            self.assertFalse(native.expected_probe(row, result))

    def test_forced_cleanup_revokes_success(self):
        """Even otherwise complete mock runtime work fails if any cleanup was forced."""
        p, api = plan(), FakeNative(plan())
        api.cleanup_value["forced"] = True
        value = native.execute(p, api)
        self.assertEqual((value["status"], value["cleanup_failure"]), ("failed", "forced-cleanup"))

    def test_verified_label_cannot_replace_individual_owned_cleanup_facts(self):
        """Actual controller revokes a verified label when any independently required owned fact is false."""
        for key in ("owned_processes_empty", "owned_references_closed", "owned_names_absent", "virtual_links_absent"):
            p, api = plan(), FakeNative(plan())
            api.cleanup_value[key] = False
            value = native.execute(p, api)
            self.assertEqual(value["status"], "failed")
            self.assertEqual(value["cleanup"]["state"], "unverified")
            self.assertEqual(value["cleanup_failure"], "cleanup-unverified")

    def test_incomplete_before_resources_keeps_false_native_facts(self):
        """A qualified missing facility retains not-created evidence without invented native observations."""
        p = plan()
        cleanup = native.initial_receipt(p)["cleanup"]
        cleanup["state"] = "verified-not-created"
        api = FakeNative(p, ("qualify", native.Failure("facility-unavailable", True)), cleanup)
        value = native.execute(p, api)
        self.assertEqual(value["status"], "incomplete")
        self.assertFalse(any(value["qualification"].values()))
        self.assertEqual(value["observed_runtime"]["instances"], 0)
        self.assertFalse(value["observed_runtime"]["scan_complete"])
        self.assertTrue(all(row["status"] == "not-run" for row in value["probes"]))

    def test_unknown_startup_failure_is_failed_unverified(self):
        """An unknown pre-resource exception is not optimistic incomplete/not-created."""
        p = plan()
        cleanup = native.initial_receipt(p)["cleanup"]
        cleanup["state"] = "verified-not-created"
        value = native.execute(p, FakeNative(p, ("qualify", RuntimeError("secret")), cleanup))
        self.assertEqual(value["status"], "failed")
        self.assertEqual(value["cleanup"]["state"], "unverified")
        self.assertNotIn(b"secret", native.canonical(value))

    def test_scan_failure_and_wrong_accounting_revokes_pass(self):
        """Observed counts and scan-complete truth cannot be inferred from phase completion."""
        for change in ({"scan_complete": False}, {"namespace_checked": 2}, {"instances": 0}):
            p, api = plan(), FakeNative(plan())
            api.account_value.update(change)
            self.assertEqual(native.execute(p, api)["status"], "failed")

    def test_blocked_fence_actual_record_accepts_only_exact_identity(self):
        """Actual fence compares PID/start/netns and exact FD relations before release."""
        record = proof_record()
        child = native.Child(41, 29, 42)
        native.blocked_proof(record, child, (8, 7), 1001, 1001, record["fds"])
        for key, value in (("start", 30), ("net", (8, 8)), ("pid", 42), ("fds", {0: "socket:[1]"})):
            bad = copy.deepcopy(record)
            bad[key] = value
            with self.subTest(key=key), self.assertRaises(native.Failure):
                native.blocked_proof(bad, child, (8, 7), 1001, 1001, record["fds"])

    def test_blocked_fence_checks_all_caps_ids_groups_nnp_and_threads(self):
        """Retained root/saved/filesystem IDs or any capability/group/thread cannot pass."""
        for key, value in (("Uid", "1001 1001 0 1001"), ("Gid", "1001 1001 1001 0"),
                           ("Groups", "1001"), ("NoNewPrivs", "0"), ("Threads", "2"),
                           *((name, "1") for name in ("CapInh", "CapPrm", "CapEff", "CapBnd", "CapAmb"))):
            record = proof_record()
            record["status"][key] = value
            with self.subTest(key=key), self.assertRaises(native.Failure):
                native.blocked_proof(record, native.Child(41, 29, 42), (8, 7), 1001, 1001, record["fds"])

    def test_client_reader_closed_denominator_and_aggregate(self):
        """Validate all16 checks/39partition and reject combined outcomes over10000."""
        value, declared = client()
        self.assertEqual(native.client_receipt(native.canonical(value), declared), value)
        for mutation in ("bool", "missing", "unknown", "aggregate"):
            bad = copy.deepcopy(value)
            if mutation == "bool": bad["check_count"] = True
            elif mutation == "missing": bad["checks"].pop()
            elif mutation == "unknown": bad["unobserved_operations"].append("foreign")
            else: bad["operation_outcomes"]["operation_00"] = {"succeeded": 5001, "rejected": 5000, "transport_failed": 0}
            with self.subTest(mutation=mutation), self.assertRaises(native.Failure):
                native.client_receipt(native.canonical(bad), declared)

    def test_client_reader_does_not_turn_missing_operations_into_observed(self):
        """Unobserved operations remain explicit; unknown/overlapping identifiers fail."""
        value, declared = client()
        actual = native.client_receipt(native.canonical(value), declared)
        self.assertEqual(len(actual["observed_operations"]), 1)
        self.assertEqual(len(actual["unobserved_operations"]), 38)
        value["unobserved_operations"].insert(0, "operation_00")
        with self.assertRaises(native.Failure): native.client_receipt(native.canonical(value), declared)

    def test_remaining_deadline_does_not_renew(self):
        """Every retry uses the same absolute end and expires at its boundary."""
        self.assertEqual(native.remaining(9, lambda: 8), 1)
        with self.assertRaises(native.Failure): native.remaining(9, lambda: 9)

    def test_expired_native_budget_abstains_before_creation_and_dispatch(self):
        """Actual protect/setup/server/launch/admin entries cannot create resources or spawn after expiry."""
        api = bare_native()
        api.end = api.work_end = api.setup_end = time.monotonic() - 1
        api.owner_proof, api.pipe = mock.Mock(return_value=True), mock.Mock()
        with mock.patch.object(native.subprocess,"Popen") as popen, mock.patch.object(native.os,"fork") as fork, \
             mock.patch.object(native.tempfile,"mkdtemp") as temporary, mock.patch.object(native.os,"urandom") as random:
            for operation in (api.protect, api.setup, lambda: api.server("dut"),
                              lambda: api.launch("dut",lambda end: None,api.end),
                              lambda: api.administrative(["netns","add","fixed"],api.end)):
                with self.assertRaises(native.Failure): operation()
            popen.assert_not_called();fork.assert_not_called();temporary.assert_not_called()
            random.assert_not_called();api.pipe.assert_not_called()
        api = bare_native();api.owner_proof = mock.Mock(return_value=True)
        with mock.patch.object(native,"remaining",side_effect=[1,native.Failure("command-timeout")]), \
             mock.patch.object(native.subprocess,"Popen") as popen:
            with self.assertRaises(native.Failure): api.administrative(["netns","add","fixed"])
        popen.assert_not_called()

    def test_drop_adapter_clears_bounding_before_ids_and_nnp(self):
        """Actual drop body calls all capability bounds/groups/IDs/capset/NNP in order."""
        api = native.Native.__new__(native.Native)
        api.plan, api.lib = plan(), types.SimpleNamespace(prctl=mock.Mock(return_value=0), capset=mock.Mock(return_value=0))
        events = []
        api.prctl = lambda option, value=0: events.append(("prctl", option, value))
        api.call = lambda method, *args: events.append(("call", method, args))
        with mock.patch.object(native, "read_proc", return_value=b"2\n"), \
             mock.patch.object(native.os, "setgroups", side_effect=lambda x: events.append(("groups", x))), \
             mock.patch.object(native.os, "setresgid", side_effect=lambda *x: events.append(("gid", x)), create=True), \
             mock.patch.object(native.os, "setresuid", side_effect=lambda *x: events.append(("uid", x)), create=True):
            api.drop()
        self.assertEqual(events[:3], [("prctl", 24, n) for n in range(3)])
        self.assertLess(next(i for i,x in enumerate(events) if x[0] == "groups"), next(i for i,x in enumerate(events) if x[0] == "uid"))
        self.assertEqual(events[-1], ("prctl", 38, 1))

    def test_owner_proof_reads_hidden_sigaction_flag(self):
        """Actual owner eligibility rejects SA_NOCLDWAIT even when the handler is default."""
        api = native.Native.__new__(native.Native)
        flags = [0]
        def action(_sig, _input, output):
            """Write a fake ABI-shaped action into the actual output buffer."""
            row = ctypes.cast(output, ctypes.POINTER(native.SigAction)).contents
            row.flags = flags[0]
            return 0
        api.lib = types.SimpleNamespace(sigaction=action)
        with mock.patch.object(native, "process_record", return_value=proof_record()):
            self.assertTrue(api.owner_proof())
            flags[0] = 2
            self.assertFalse(api.owner_proof())

    def test_libc_qualifier_rejects_unknown_abi_before_sigaction(self):
        """Actual qualifier rejects missing/invalid GNU identity without any sigaction query."""
        for value in (None, b"musl", b"2.39 secret", b"9" * 32):
            lib = types.SimpleNamespace(gnu_get_libc_version=mock.Mock(return_value=value), sigaction=mock.Mock())
            with self.assertRaises(native.Failure) as caught: native.qualify_libc(lib)
            self.assertEqual(caught.exception.code, "facility-unavailable")
            self.assertTrue(caught.exception.unavailable)
            lib.sigaction.assert_not_called()
        lib = types.SimpleNamespace(sigaction=mock.Mock())
        with self.assertRaises(native.Failure): native.qualify_libc(lib)
        lib.sigaction.assert_not_called()
        lib.gnu_get_libc_version = mock.Mock(return_value=b"2.39")
        self.assertEqual(native.qualify_libc(lib), b"2.39")

    def test_rollback_signal_requires_exact_original_owner_proof(self):
        """Approved direct-only rollback latches force before signalling and abstains without proof."""
        api = native.Native.__new__(native.Native)
        api.forced, api.owner_proof = False, mock.Mock(return_value=True)
        child = native.Child(41, None, None, rollback=True)
        with mock.patch.object(native.os, "kill") as kill:
            api.stop_direct(child)
            self.assertTrue(api.forced)
            kill.assert_called_once_with(41, signal.SIGKILL)
        for rollback, proof in ((False, True), (True, False)):
            child.rollback, api.owner_proof.return_value = rollback, proof
            with mock.patch.object(native.os, "kill") as kill, self.assertRaises(native.Failure):
                api.stop_direct(child)
            kill.assert_not_called()

    def test_child_poll_uses_specific_wait_and_preserves_status(self):
        """Actual fork wait owner never consumes waitpid(-1) or fabricates a clean exit."""
        child = native.Child(41, 29, 42)
        with mock.patch.object(native.os, "waitpid", return_value=(41, 7 << 8)) as wait:
            self.assertEqual(child.poll(), 7)
            self.assertEqual(child.poll(), 7)
            wait.assert_called_once_with(41, os.WNOHANG)

    def test_popen_child_wait_owner_and_pipe_close(self):
        """Popen owns its status/file object; raw os.close cannot leave a double-close destructor."""
        owner = types.SimpleNamespace(poll=mock.Mock(return_value=-9), stdout=mock.Mock())
        child = native.Child(41, 29, None, owner, output=42)
        with mock.patch.object(native.os, "close") as close:
            self.assertEqual(child.poll(), -9)
            child.close()
            owner.stdout.close.assert_called_once()
            close.assert_not_called()

    def test_capture_bound_and_eof_are_separate(self):
        """Actual child drain rejects overflow and records EOF separately from exit status."""
        child = native.Child(41, 29, None, output=42)
        with mock.patch.object(native.os, "read", side_effect=[b"abc", b""]):
            child.drain()
            self.assertFalse(child.eof)
            child.drain()
            self.assertTrue(child.eof)
            self.assertIsNone(child.exit)
        child = native.Child(41, 29, None, output=42)
        child.captured.extend(b"x" * native.MAX_CAPTURE)
        with mock.patch.object(native.os, "read", return_value=b"x"), self.assertRaises(native.Failure): child.drain()
        self.assertEqual(child.captured, b"")

    def test_launch_reservation_failure_keeps_owned_pipe_nonblocking(self):
        """Actual launch sets nonblocking before fork/reserve and retains partial-child handles."""
        for at_start in (False, True):
            api = bare_native()
            api.owner_proof = mock.Mock(return_value=True)
            events = []
            def blocking(fd, value):
                """Journal the pre-reservation read-side nonblocking fence."""
                events.append(("blocking", fd, value))
            def fork():
                """Return a mock direct PID after the read-side configuration."""
                events.append(("fork",))
                return 41
            with mock.patch.object(native.os, "pipe2", side_effect=[(30,31),(32,33),(34,35)], create=True), \
                 mock.patch.object(native.os, "set_blocking", side_effect=blocking), \
                 mock.patch.object(native.os, "readlink", side_effect=lambda p: "pipe:["+p.rsplit("/",1)[-1]+"]"), \
                 mock.patch.object(native.os, "fork", side_effect=fork), \
                 mock.patch.object(native.os, "pidfd_open", side_effect=None if at_start else OSError("secret"), return_value=44, create=True), \
                 mock.patch.object(native, "start_identity", side_effect=OSError("secret")), \
                 mock.patch.object(native.os, "close"):
                with self.assertRaises(OSError): api.launch("dut", lambda end: None, api.work_end)
            self.assertEqual(events[:2], [("blocking", 34, False), ("fork",)])
            self.assertEqual(len(api.children), 1)
            self.assertEqual(api.children[0].output, 34)
            self.assertEqual(api.children[0].pidfd, 44 if at_start else None)
            self.assertEqual(api.pending_fds, set())

    def test_administrative_pipe_setup_failure_abstains_before_popen(self):
        """Actual admin pipe setup occurs before spawn and independently closes both reserved ends."""
        api = bare_native();api.owner_proof = mock.Mock(return_value=True)
        with mock.patch.object(native.os,"pipe2",return_value=(30,31),create=True), \
             mock.patch.object(native.os,"set_blocking",side_effect=OSError("private")), \
             mock.patch.object(native.os,"close") as close, mock.patch.object(native.subprocess,"Popen") as popen:
            with self.assertRaises(OSError): api.administrative(["netns","list"])
        popen.assert_not_called()
        self.assertEqual([row.args[0] for row in close.call_args_list],[30,31])
        self.assertEqual(api.pending_fds,set());self.assertEqual(api.children,[])

    def test_administrative_popen_failure_closes_prepared_pipe(self):
        """A failed original Popen call leaves no reserved read/write pipe or invented child owner."""
        api = bare_native();api.owner_proof = mock.Mock(return_value=True);events = []
        def blocking(fd,value):
            """Journal the actual pre-spawn nonblocking read-side fence."""
            events.append(("blocking",fd,value))
        def spawn(*args,**kwargs):
            """Reject only the mocked process creation after checking the prepared write relation."""
            events.append(("spawn",kwargs["stdout"]))
            raise OSError("private")
        with mock.patch.object(native.os,"pipe2",return_value=(30,31),create=True), \
             mock.patch.object(native.os,"set_blocking",side_effect=blocking), \
             mock.patch.object(native.os,"close") as close, mock.patch.object(native.subprocess,"Popen",side_effect=spawn):
            with self.assertRaises(OSError): api.administrative(["netns","list"])
        self.assertEqual(events,[("blocking",30,False),("spawn",31)])
        self.assertEqual([row.args[0] for row in close.call_args_list],[30,31])
        self.assertEqual(api.pending_fds,set());self.assertEqual(api.children,[])

    def test_administrative_reservation_failure_retains_original_owner_for_cleanup(self):
        """Pidfd/start failure keeps the prepared pipe and exact Popen owner for failed/forced cleanup."""
        for at_start in (False,True):
            api = bare_native();api.owner_proof = mock.Mock(return_value=True);events = [];status = [None]
            owner = types.SimpleNamespace(pid=41,stdout=None,poll=mock.Mock(side_effect=lambda:status[0]))
            def blocking(fd,value):
                """Journal pre-spawn nonblocking state without touching a host descriptor."""
                events.append(("blocking",fd,value))
            def spawn(*args,**kwargs):
                """Return the original fake Popen owner for the prepared output descriptor."""
                events.append(("spawn",kwargs["stdout"]))
                return owner
            def stop(*args):
                """Complete only this fake owner's status after the actual forced-stop path."""
                status[0] = -9
            api.scan = mock.Mock(side_effect=native.Failure("execution-unverified"));api.scan_complete = False
            with mock.patch.object(native.os,"pipe2",return_value=(30,31),create=True), \
                 mock.patch.object(native.os,"set_blocking",side_effect=blocking), \
                 mock.patch.object(native.subprocess,"Popen",side_effect=spawn), \
                 mock.patch.object(native.os,"pidfd_open",side_effect=None if at_start else OSError("private"),return_value=44,create=True), \
                 mock.patch.object(native,"start_identity",side_effect=OSError("private")), \
                 mock.patch.object(native.os,"close") as close, mock.patch.object(native.os,"read",return_value=b""), \
                 mock.patch.object(native.os,"kill",side_effect=stop) as kill, \
                 mock.patch.object(native.signal,"pidfd_send_signal",side_effect=stop,create=True) as pidfd_kill:
                with self.assertRaises(OSError): api.administrative(["netns","list"])
                self.assertEqual(events,[("blocking",30,False),("spawn",31)])
                self.assertEqual(len(api.children),1);child = api.children[0]
                self.assertIs(child.owner,owner);self.assertEqual(child.output,30)
                self.assertEqual(child.pidfd,44 if at_start else None);self.assertEqual(api.pending_fds,set())
                cleanup = api.cleanup()
                (pidfd_kill if at_start else kill).assert_called_once_with(44 if at_start else 41,signal.SIGKILL)
                (kill if at_start else pidfd_kill).assert_not_called()
                self.assertIsNone(child.output);self.assertIsNone(child.pidfd)
                self.assertIn(30,[row.args[0] for row in close.call_args_list])
            self.assertGreaterEqual(owner.poll.call_count,2);self.assertEqual(child.exit,-9)
            self.assertEqual(cleanup["state"],"unverified");self.assertTrue(cleanup["forced"])
            self.assertFalse(cleanup["owned_processes_empty"])

    def test_administrative_actual_exit_eof_and_owned_descriptor_close(self):
        """A natural mock command requires original-owner status and EOF, then closes the explicit readfd."""
        api = bare_native();api.owner_proof = mock.Mock(return_value=True)
        owner = types.SimpleNamespace(pid=41,stdout=None,poll=mock.Mock(side_effect=[None,0,0]))
        with mock.patch.object(native.os,"pipe2",return_value=(30,31),create=True), \
             mock.patch.object(native.os,"set_blocking") as blocking, \
             mock.patch.object(native.subprocess,"Popen",return_value=owner) as popen, \
             mock.patch.object(native.os,"pidfd_open",return_value=44,create=True), \
             mock.patch.object(native,"start_identity",return_value=29), \
             mock.patch.object(native.os,"read",side_effect=[b"bounded",b""]), \
             mock.patch.object(native.os,"close") as close, mock.patch.object(native.time,"sleep"):
            self.assertEqual(api.administrative(["netns","list"]),b"bounded")
            blocking.assert_called_once_with(30,False);self.assertEqual(popen.call_args.kwargs["stdout"],31)
            child = api.children[0];self.assertIs(child.owner,owner)
            self.assertEqual(child.exit,0);self.assertTrue(child.eof);self.assertEqual(child.captured,b"")
            child.close()
        self.assertEqual([row.args[0] for row in close.call_args_list],[31,30,44])
        self.assertEqual(api.pending_fds,set())

    def test_proc_scan_stops_streaming_at_entry_cap_before_topology(self):
        """Actual scanner stops after cap+1 rows and closes the context before any stat/topology allocation."""
        api = bare_native();consumed = []
        def entries():
            """Yield many synthetic directory rows so eager consumption would violate the control."""
            for number in range(100):
                consumed.append(number)
                yield types.SimpleNamespace(name=str(number))
        directory = mock.MagicMock();directory.__enter__.return_value = entries()
        with mock.patch.object(native,"MAX_PROC",3), mock.patch.object(native.os,"scandir",return_value=directory) as scan, \
             mock.patch.object(native,"read_proc") as read:
            with self.assertRaises(native.Failure): api.scan()
        scan.assert_called_once_with("/proc");read.assert_not_called()
        self.assertEqual(consumed,[0,1,2,3]);directory.__exit__.assert_called_once()
        self.assertFalse(api.scan_complete)

    def test_process_fd_inventory_stops_at_cap_before_any_readlink(self):
        """Actual blocked-record inventory consumes cap+1 rows and never follows an oversized FD list."""
        consumed = []
        def entries():
            """Yield a synthetic large FD directory without consulting actual proc state."""
            for number in range(100):
                consumed.append(number)
                yield types.SimpleNamespace(name=str(number))
        directory = mock.MagicMock();directory.__enter__.return_value = entries()
        fields = [b"S",b"1"] + [b"0"] * 17 + [b"29"]
        raw = b"41 (mock) " + b" ".join(fields)
        identity = types.SimpleNamespace(st_dev=8,st_ino=7)
        with mock.patch.object(native,"MAX_FDS",3), mock.patch.object(native.os,"scandir",return_value=directory) as scan, \
             mock.patch.object(native,"read_proc",side_effect=[raw,b"Threads:\t1\n"]), \
             mock.patch.object(native.Path,"stat",return_value=identity), mock.patch.object(native.os,"readlink") as link:
            with self.assertRaises(native.Failure): native.process_record(41,True)
        scan.assert_called_once_with(Path("/proc/41/fd"));link.assert_not_called()
        self.assertEqual(consumed,[0,1,2,3]);directory.__exit__.assert_called_once()

    def test_shared_fd_inventory_requires_bounded_canonical_numeric_names(self):
        """The actual child/parent FD-name helper rejects foreign/duplicate/out-of-range names before use."""
        for names in (["0","1","42"],["0","foreign"],["0","01"],["0","2147483648"],["0","0"]):
            directory = mock.MagicMock();directory.__enter__.return_value = iter(types.SimpleNamespace(name=n) for n in names)
            with mock.patch.object(native.os,"scandir",return_value=directory):
                if names == ["0","1","42"]: self.assertEqual(native.bounded_fd_names("/mock/fd"),[0,1,42])
                else:
                    with self.assertRaises(native.Failure): native.bounded_fd_names("/mock/fd")
            directory.__exit__.assert_called_once()

    def test_child_close_attempts_every_handle_after_first_failure(self):
        """A failed output close cannot skip the independently owned stop/pidfd handles."""
        child = native.Child(41, 29, 44, output=42, stop=43)
        with mock.patch.object(native.os, "close", side_effect=[OSError("secret"), None, None]) as close:
            with self.assertRaises(native.Failure): child.close()
        self.assertEqual([c.args[0] for c in close.call_args_list], [42,43,44])
        self.assertEqual(child.output, 42)
        self.assertIsNone(child.stop)
        self.assertIsNone(child.pidfd)

    def test_cleanup_scan_failure_still_reaches_direct_pidfd_fallback(self):
        """Actual cleanup stops a reserved direct child despite unavailable complete proc scanning."""
        api = bare_native()
        child = native.Child(41, 29, 44)
        api.children = [child]
        child.poll = mock.Mock(side_effect=lambda: child.exit)
        def stop(_fd, _signal):
            """Return a real mock forced status after the actual fallback signal."""
            child.exit = -9
        api.scan = mock.Mock(side_effect=native.Failure("execution-unverified"))
        api.scan_complete = False
        with mock.patch.object(native.signal, "pidfd_send_signal", side_effect=stop, create=True) as kill, \
             mock.patch.object(native.os, "close"):
            cleanup = api.cleanup()
        kill.assert_called_once_with(44, signal.SIGKILL)
        self.assertEqual(cleanup["state"], "unverified")
        self.assertTrue(cleanup["forced"])

    def test_veth_cleanup_measures_affected_namespaces_before_release(self):
        """Actual cleanup deletes only the identity-qualified endpoint and queries both scopes twice."""
        api = bare_native()
        api.links_attempted = ["left", "right"]
        api.names = {"canary": ("own-canary", 8, 7), "calibration": ("own-calibration", 8, 9)}
        api.netfds = {"canary": 41, "calibration": 42}
        api.links = {("canary", "left"): (3,"aa:bb:cc:dd:ee:01"),
                     ("calibration", "right"): (4,"aa:bb:cc:dd:ee:02")}
        rows = {"canary": [{"ifname":"left","ifindex":3,"address":"aa:bb:cc:dd:ee:01","linkinfo":{"info_kind":"veth"}}],
                "calibration": [{"ifname":"right","ifindex":4,"address":"aa:bb:cc:dd:ee:02","linkinfo":{"info_kind":"veth"}}]}
        queries, commands = [], []
        def inventory(role, end, cleanup):
            """Record actual affected-scope queries while the namespace handles remain retained."""
            self.assertEqual(set(api.netfds), {"canary","calibration"})
            queries.append(role)
            return copy.deepcopy(rows[role])
        def delete(argv, end, cleanup):
            """Simulate kernel paired-veth deletion only after actual identity validation."""
            commands.append(argv)
            rows["canary"].clear();rows["calibration"].clear()
        api.namespace_links, api.administrative = inventory, delete
        self.assertTrue(api.cleanup_links())
        self.assertEqual(commands, [["-n","own-canary","link","delete","dev","left"]])
        self.assertEqual(queries, ["canary","calibration","canary","calibration"])

    def test_veth_mismatch_or_affected_scope_failure_cannot_receive_absence_credit(self):
        """Wrong private link identity forbids deletion and failed affected queries cannot use host absence."""
        api = bare_native()
        api.links_attempted, api.names = ["left"], {"canary": ("own",8,7), "calibration": ("other",8,9)}
        api.links = {("canary","left"):(3,"aa:bb:cc:dd:ee:01")}
        api.namespace_links = mock.Mock(return_value=[{"ifname":"left","ifindex":99,
            "address":"aa:bb:cc:dd:ee:01","linkinfo":{"info_kind":"veth"}}])
        api.administrative = mock.Mock()
        self.assertFalse(api.cleanup_links())
        api.administrative.assert_not_called()
        api.namespace_links.side_effect = native.Failure("namespace-unverified")
        self.assertFalse(api.cleanup_links())
        api.administrative.assert_not_called()

    def test_namespace_link_query_requires_actual_name_and_handle_identity(self):
        """Actual affected query refuses a changed namespace before issuing any ip command."""
        api = bare_native()
        api.names, api.netfds = {"canary": ("own",8,7)}, {"canary":41}
        good = types.SimpleNamespace(st_dev=8, st_ino=7)
        bad = types.SimpleNamespace(st_dev=8, st_ino=99)
        api.administrative = mock.Mock(return_value=native.canonical([{"ifname":"lo"}]))
        with mock.patch.object(native.Path, "stat", return_value=good), mock.patch.object(native.os, "fstat", return_value=bad):
            with self.assertRaises(native.Failure): api.namespace_links("canary", api.end, True)
        api.administrative.assert_not_called()
        with mock.patch.object(native.Path, "stat", return_value=good), mock.patch.object(native.os, "fstat", return_value=good):
            self.assertEqual(api.namespace_links("canary", api.end, True), [{"ifname":"lo"}])
        api.administrative.assert_called_once_with(["-n","own","-j","-d","link","show"],api.end,True)

    def test_detailed_link_argv_supplies_required_veth_delete_authority(self):
        """Actual endpoint capture/cleanup request details; missing kind still refuses capture/delete authority."""
        api = bare_native();api.names = {"canary":("own-canary",8,7),"calibration":("own-calibration",8,7)}
        api.netfds = {"canary":41,"calibration":42};api.links_attempted = ["left","right"]
        rows = {"own-canary":[{"ifname":"left","ifindex":3,"address":"aa:bb:cc:dd:ee:01"}],
                "own-calibration":[{"ifname":"right","ifindex":4,"address":"aa:bb:cc:dd:ee:02"}]}
        missing = [False];queries,deleted = [],[]
        def administration(argv,end,cleanup=False):
            """Model upstream linkinfo emission only for the fixed detailed JSON query."""
            if "delete" in argv:
                deleted.append(argv);rows["own-canary"].clear();rows["own-calibration"].clear()
                return b""
            queries.append(argv);value = copy.deepcopy(rows[argv[1]])
            if "-d" in argv and not missing[0]:
                for row in value: row["linkinfo"] = {"info_kind":"veth"}
            return native.canonical(value)
        api.administrative = administration
        identity = types.SimpleNamespace(st_dev=8,st_ino=7)
        with mock.patch.object(native.Path,"stat",return_value=identity), mock.patch.object(native.os,"fstat",return_value=identity):
            api.links[("canary","left")] = api.capture_link("canary","left",api.end)
            api.links[("calibration","right")] = api.capture_link("calibration","right",api.end)
            self.assertEqual(api.links[("canary","left")],(3,"aa:bb:cc:dd:ee:01"))
            missing[0] = True
            with self.assertRaises(native.Failure): api.capture_link("canary","left",api.end)
            self.assertFalse(api.cleanup_links());self.assertEqual(deleted,[])
            missing[0] = False
            self.assertTrue(api.cleanup_links())
        self.assertTrue(queries);self.assertTrue(all(argv[2:] == ["-j","-d","link","show"] for argv in queries))
        self.assertEqual(deleted,[["-n","own-canary","link","delete","dev","left"]])

    def test_private_cleanup_is_bounded_and_never_follows_worker_symlink(self):
        """Actual dirfd cleanup preserves an outside TEMP sentinel and fails closed at entry bounds."""
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            sentinel = root / "outside";sentinel.write_bytes(b"preserve")
            owned = root / "owned";owned.mkdir();(owned / "nested").mkdir()
            (owned / "nested/file").write_bytes(b"mock")
            (owned / "link").symlink_to(sentinel)
            native.remove_private(owned,time.monotonic()+10)
            self.assertFalse(owned.exists())
            self.assertEqual(sentinel.read_bytes(),b"preserve")
            owned.mkdir()
            for n in range(3): (owned / str(n)).write_bytes(b"mock")
            with mock.patch.object(native,"MAX_PROC",1), self.assertRaises(native.Failure):
                native.remove_private(owned,time.monotonic()+10)
            self.assertTrue(owned.exists())
    def test_publication_is_readable_no_replacement_and_private_dirfd(self):
        """Actual TEMP publication preserves a collision and has readable redacted bytes."""
        with tempfile.TemporaryDirectory() as root:
            api = native.Native.__new__(native.Native)
            api.output_fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY)
            try:
                api.publish("os-denial-smoke.json", b"{}\n")
                p = Path(root) / "os-denial-smoke.json"
                self.assertEqual(p.read_bytes(), b"{}\n")
                self.assertTrue(p.stat().st_mode & 0o004)
                with self.assertRaises(FileExistsError): api.publish("os-denial-smoke.json", b"other")
                self.assertEqual(p.read_bytes(), b"{}\n")
                self.assertEqual([f.name for f in Path(root).iterdir()], [p.name])
            finally:
                api.finish_output()

    def test_protected_package_is_readable_and_nonwritable_under_umask077(self):
        """Actual source-copy body fixes only new protected hierarchy modes under restrictive umask."""
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            api = bare_native()
            api.plan.update(root=str(root), forge=str(root / "release"))
            for key in native.SOURCES:
                path = root / key
                path.parent.mkdir(parents=True, exist_ok=True)
                raw = key.encode()
                path.write_bytes(raw)
                api.plan["source_pins"][key] = {"bytes": len(raw), "sha256": native.hashlib.sha256(raw).hexdigest()}
            (root / "release").write_bytes(b"mock release")
            api.plan["release_pin"] = {"bytes": 12, "sha256": native.hashlib.sha256(b"mock release").hexdigest()}
            old = os.umask(0o077)
            try:
                with mock.patch.object(native.os, "chown"):
                    release, inputs = api.protect()
                self.assertEqual(release, api.plan["release_pin"])
                self.assertEqual(inputs, api.plan["source_pins"])
                for directory in (api.package, api.package / "scripts", api.package / "docs", api.package / "docs/api"):
                    self.assertEqual(directory.stat().st_mode & 0o777, 0o755)
                self.assertEqual((api.package / "forge").stat().st_mode & 0o777, 0o555)
                self.assertTrue(all((api.package / key).stat().st_mode & 0o777 == 0o444 for key in native.SOURCES))
            finally:
                os.umask(old)
                if api.base is not None:
                    native.shutil.rmtree(api.base)

    def test_actual_client_body_preserves_raw_file_and_fixed_argv(self):
        """Actual client body reads/pins/preserves the real canonical file and runs no optimized Python."""
        with tempfile.TemporaryDirectory() as root:
            api = bare_native()
            api.package, api.workspace = Path(root) / "package", Path(root) / "worker"
            api.package.mkdir();api.workspace.mkdir()
            value, declared = client()
            raw = wrapper.shared.canonical_bytes(value)
            (api.workspace / "client-verification.json").write_bytes(raw)
            contract = api.package / native.SOURCES[3]
            contract.parent.mkdir(parents=True)
            contract.write_bytes(b"\n".join(b"      operationId: " + x.encode() for x in sorted(declared)))
            fake_child = types.SimpleNamespace(captured=bytearray())
            def launch(role, callback, end):
                """Invoke only the fixed exec callback under the explicit execve mock."""
                self.assertEqual(role, "dut")
                callback(end)
                return fake_child
            api.launch = launch
            api.await_child = mock.Mock(return_value=(0, b"private discarded stdout"))
            api.publish = mock.Mock()
            with mock.patch.object(native.os, "execve") as execute:
                code, parsed, pin = api.run_client()
            self.assertEqual(code, 0)
            self.assertEqual(parsed, value)
            api.publish.assert_called_once_with("client-verification.json", raw)
            self.assertEqual(pin, {"bytes": len(raw), "sha256": native.hashlib.sha256(raw).hexdigest()})
            self.assertEqual(execute.call_args.args[1][1:5], ["-E", "-s", "-S", "-B"])
            self.assertNotIn("-O", execute.call_args.args[1])

    def test_actual_probe_adapter_distinguishes_denial_from_other_native_errors(self):
        """Actual connect body classifies only101 as denial, with no real sockets or namespace launch."""
        for error_number in (101, 111, 110, 97):
            api = bare_native()
            api.ports, api.challenge = {"canary": [41,42], "dut": [43,44]}, b"S" * 32
            raw = []
            def launch(role, callback, end):
                """Execute the fixed connect callback using fake socket and write adapters."""
                callback(end)
                return object()
            api.launch, api.await_child = launch, lambda child, end: (0, raw[-1])
            sock = mock.MagicMock()
            sock.__enter__.return_value = sock
            sock.connect.side_effect = OSError(error_number, "private secret")
            with mock.patch.object(native.socket, "socket", return_value=sock), \
                 mock.patch.object(native.os, "write", side_effect=lambda fd, data: raw.append(data)), \
                 mock.patch.object(native.errno, "ENETUNREACH", 101):
                result = api.probe("ipv6", "dut-external")
            self.assertEqual(result["outcome"], "denied" if error_number == 101 else "unverified")
            self.assertEqual(result["errno"], 101 if error_number == 101 else None)
            self.assertFalse(result["challenge_verified"])
            self.assertNotIn(b"private", raw[-1])

    def test_main_post_publication_failure_returns_nonzero_without_raw_error(self):
        """Actual public guard reconciles a later output-close failure with failed exit even after passed bytes."""
        p, api = plan(), FakeNative(plan())
        api.inputs = p
        api.publish, api.finish_output = mock.Mock(), mock.Mock(side_effect=OSError("private secret"))
        with mock.patch.object(native.sys, "argv", ["helper", "--plan-stdin"]), \
             mock.patch.object(native, "read_plan", return_value=native.canonical(p)), \
             mock.patch.object(native, "Native", return_value=api):
            self.assertEqual(native.main(), 1)
        self.assertEqual(api.publish.call_args.args[0], "os-denial-smoke.json")
        self.assertNotIn(b"private secret", api.publish.call_args.args[1])

    def test_plan_read_has_eof_bytecap_and_nonrenewing_startup_deadline(self):
        """Actual stdin body requires EOF, rejects excess bytes and never renews its30-second startup cap."""
        stream = types.SimpleNamespace(fileno=lambda: 31)
        with mock.patch.object(native.os, "set_blocking"), \
             mock.patch.object(native.select, "select", return_value=([31], [], [])), \
             mock.patch.object(native.os, "read", side_effect=[b"a", b""]):
            self.assertEqual(native.read_plan(stream), b"a")
        with mock.patch.object(native.os, "set_blocking"), \
             mock.patch.object(native.select, "select", return_value=([31], [], [])), \
             mock.patch.object(native.os, "read", return_value=b"x" * (native.MAX_PLAN+1)), self.assertRaises(native.Failure):
            native.read_plan(stream)
        with mock.patch.object(native.time, "monotonic", return_value=0), \
             mock.patch.object(native, "remaining", side_effect=[30, 29, native.Failure("command-timeout")]) as left, \
             mock.patch.object(native.os, "set_blocking"), \
             mock.patch.object(native.select, "select", return_value=([31], [], [])), \
             mock.patch.object(native.os, "read", return_value=b"x"), self.assertRaises(native.Failure):
            native.read_plan(stream)
        self.assertEqual([call.args[0] for call in left.call_args_list], [30,30,30])


class WrapperControls(unittest.TestCase):
    """Invoke actual ordinary receipt reader with private real files and adversarial metadata."""

    def read(self, mutation=None, code=0, raw_mutation=None):
        """Write a mock-produced raw client/native pair and call the actual wrapper reader."""
        p, api = plan(), FakeNative(plan())
        api.inputs = p
        value = native.execute(p, api)
        if mutation: mutation(value)
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            raw = native.canonical(value["client"]["receipt"])
            (root / wrapper.CLIENT_OUTPUT).write_bytes(raw if raw_mutation is None else raw_mutation(raw))
            destination = root / wrapper.NATIVE_OUTPUT
            destination.write_bytes(native.canonical(value))
            return wrapper.read_native(destination, code, p["release_pin"], p["source_pins"], client()[1])[0]

    def test_actual_reader_revalidates_raw_client_and_all_rows(self):
        """A full mock pair survives actual shared reader revalidation, without native credit."""
        value = self.read()
        self.assertEqual(value["status"], "passed")
        self.assertEqual(value["attempted_egress"]["count"], None)

    def test_reader_actual_exit_overrides_passed_looking_file(self):
        """A passed-looking receipt after nonzero/boolean completion cannot pass."""
        for code in (1, False, None):
            with self.subTest(code=code), self.assertRaises(wrapper.GateError): self.read(code=code)

    def test_reader_requires_actual_raw_client_byte_equality(self):
        """A nested dictionary/pin cannot replace the actual private canonical file."""
        with self.assertRaises((wrapper.GateError, ValueError)):
            self.read(raw_mutation=lambda raw: raw.rstrip())

    def test_reader_rejects_client_pin_and_nested_mismatch(self):
        """The raw file, shared result, nested result and byte pin must all agree."""
        for mutation in (lambda v: v["client"]["receipt_pin"].update(sha256="b" * 64),
                         lambda v: v["client"].update(exit_code=1),
                         lambda v: v["client"]["receipt"].update(check_count=True)):
            with self.subTest(mutation=mutation), self.assertRaises((wrapper.GateError, ValueError)): self.read(mutation)

    def test_reader_order_and_errno_noattempt_closedness(self):
        """Missing/reordered rows, bool errno and attempted-egress0 cannot acquire credit."""
        for mutation in (lambda v: v["probes"].reverse(), lambda v: v["probes"].pop(),
                         lambda v: v["probes"][1].update(errno=True),
                         lambda v: v["attempted_egress"].update(count=0)):
            with self.subTest(mutation=mutation), self.assertRaises(wrapper.GateError): self.read(mutation)

    def test_reader_false_qualification_and_observed_scope(self):
        """A missing fence/scan proof or exhaustive scope claim revokes pass."""
        for mutation in (lambda v: v["qualification"].update(client_preexec_fence=False),
                         lambda v: v["observed_runtime"].update(scan_complete=False),
                         lambda v: v["observed_runtime"].update(scope="all-processes")):
            with self.subTest(mutation=mutation), self.assertRaises(wrapper.GateError): self.read(mutation)

    def test_reader_forced_and_unverified_cleanup_cannot_pass(self):
        """Cleanup status must match actual failure state even after12 rows and client pass."""
        for mutation in (lambda v: v["cleanup"].update(forced=True),
                         lambda v: v["cleanup"].update(state="unverified"),
                         lambda v: v["cleanup"].update(owned_names_absent=False)):
            with self.subTest(mutation=mutation), self.assertRaises(wrapper.GateError): self.read(mutation)

    def test_reader_retains_mixed_forced_unverified_failure_priority(self):
        """A validated failed receipt preserves primary cause and unverified-cleanup priority when forced too."""
        def failure(value):
            """Inject the actual mixed failure shape without turning it into success."""
            value.update(status="failed", failure="probe-calibration-failed", cleanup_failure="cleanup-unverified")
            value["cleanup"].update(state="unverified", forced=True, owned_processes_empty=False)
        value = self.read(failure, code=1)
        self.assertEqual(value["failure"], "probe-calibration-failed")
        self.assertEqual(value["cleanup_failure"], "cleanup-unverified")

    def test_reader_incomplete_not_created_has_no_native_facts(self):
        """Actual reader accepts a genuinely not-created shape and rejects optimistic booleans."""
        p = plan()
        cleanup = native.initial_receipt(p)["cleanup"]
        cleanup["state"] = "verified-not-created"
        value = native.execute(p, FakeNative(p, ("qualify", native.Failure("facility-unavailable", True)), cleanup))
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / wrapper.NATIVE_OUTPUT
            path.write_bytes(native.canonical(value))
            self.assertEqual(wrapper.read_native(path, 2, p["release_pin"], p["source_pins"], client()[1])[0]["status"], "incomplete")
            value["cleanup"]["owned_names_absent"] = True
            path.write_bytes(native.canonical(value))
            with self.assertRaises(wrapper.GateError): wrapper.read_native(path, 2, p["release_pin"], p["source_pins"], client()[1])

    def command_adapter(self, stdout, stderr, *, end=100, exit_after=2, input_bytes=None, privileged=False, cap=None):
        """Call actual command with descriptor/selector/clock adapters; no child/tool is executed."""
        selection = FakeSelector()
        child = FakeCommandChild(selection, exit_after, input_bytes)
        reads = {31: list(stdout), 32: list(stderr)}
        written = bytearray()
        def read(fd, limit):
            """Supply the next independent mock pipe block or terminal EOF."""
            return reads[fd].pop(0) if reads[fd] else b""
        def write(fd, raw):
            """Retain exact bytes the actual input-plan loop attempts to send."""
            self.assertEqual(fd, 33)
            written.extend(raw)
            return len(raw)
        with mock.patch.object(wrapper.selectors, "DefaultSelector", return_value=selection), \
             mock.patch.object(wrapper.subprocess, "Popen", return_value=child), \
             mock.patch.object(wrapper.os, "set_blocking"), mock.patch.object(wrapper.os, "read", side_effect=read), \
             mock.patch.object(wrapper.os, "write", side_effect=write), \
             mock.patch.object(wrapper.time, "monotonic", side_effect=lambda: selection.ticks), \
             mock.patch.object(wrapper, "MAX_CAPTURE", cap if cap is not None else wrapper.MAX_CAPTURE):
            result = wrapper.command(["mock"], Path("/mock"), end, input_bytes, privileged)
        return result, child, bytes(written)

    def test_command_returns_only_stdout_and_drains_warning_stderr(self):
        """Actual dual-pipe command excludes private stderr warning from returned stdout."""
        result, child, _ = self.command_adapter([b"tool\n", b""], [b"private warning", b""])
        self.assertEqual(result, {"exit_code": 0, "failure": None, "output": b"tool\n"})
        self.assertGreaterEqual(child.stdout.closes, 1)
        self.assertGreaterEqual(child.stderr.closes, 1)

    def test_command_aggregate_bound_includes_discarded_stderr(self):
        """Combined pipe bytes over the inclusive cap fail and discard all output."""
        result, _, _ = self.command_adapter([b"abc", b""], [b"def", b""], cap=5)
        self.assertEqual(result["failure"], "output-bound")
        self.assertEqual(result["output"], b"")
        result, _, _ = self.command_adapter([b"abc", b""], [b"de", b""], cap=5)
        self.assertIsNone(result["failure"])

    def test_command_independent_eof_cannot_replace_actual_exit(self):
        """Both pipes at EOF still wait for actual completion; timed-out ordinary child returns-9."""
        result, child, _ = self.command_adapter([b""], [b""], end=3, exit_after=99)
        self.assertEqual(result["failure"], "command-timeout")
        self.assertEqual(result["exit_code"], -9)
        self.assertTrue(child.killed)
        self.assertEqual(result["output"], b"")

    def test_command_privileged_timeout_never_guesses_native_cleanup(self):
        """The wrapper cannot kill a sudo proxy then infer native experiment resources are gone."""
        result, child, _ = self.command_adapter([b""], [b""], end=3, exit_after=99, privileged=True)
        self.assertEqual(result["failure"], "command-timeout")
        self.assertIsNone(result["exit_code"])
        self.assertFalse(child.killed)

    def test_command_writes_exact_plan_and_closes_input_at_eof(self):
        """Actual input loop sends the exact canonical plan without a shell or trailing reconstruction."""
        raw = native.canonical(plan())
        result, child, written = self.command_adapter([b""], [b""], input_bytes=raw)
        self.assertEqual(written, raw)
        self.assertIsNone(result["failure"])
        self.assertGreaterEqual(child.stdin.closes, 1)

    def test_command_expired_or_exact_boundary_never_starts_child(self):
        """Actual command fences both entry and pre-Popen boundary at equality without a native/tool child."""
        with mock.patch.object(wrapper.time,"monotonic",return_value=10), \
             mock.patch.object(wrapper.subprocess,"Popen") as spawn, \
             mock.patch.object(wrapper.selectors,"DefaultSelector") as selection:
            value = wrapper.command(["mock"],Path("/mock"),10)
        spawn.assert_not_called();selection.assert_not_called()
        self.assertEqual(value,{"exit_code":None,"failure":"command-timeout","output":b""})
        selection = FakeSelector()
        with mock.patch.object(wrapper.time,"monotonic",side_effect=[0,10]), \
             mock.patch.object(wrapper.subprocess,"Popen") as spawn, \
             mock.patch.object(wrapper.selectors,"DefaultSelector",return_value=selection):
            value = wrapper.command(["mock"],Path("/mock"),10)
        spawn.assert_not_called()
        self.assertEqual(value["failure"],"command-timeout")

    def test_native_dispatch_expired_boundaries_never_call_bootstrap(self):
        """Actual ordinary dispatcher checks before temp resources and again before privileged command."""
        p = plan();identity = {"provided_release_binary":p["release_pin"],"inputs":dict(p["source_pins"])}
        identity["inputs"][wrapper.EXTRA_INPUTS[0]] = {"bytes":4,"sha256":"c"*64}
        paths = {"python":Path("/usr/bin/python3.11"),"ip":Path("/usr/sbin/ip"),"sudo":Path("/usr/bin/sudo")}
        with mock.patch.object(wrapper.time,"monotonic",return_value=10), \
             mock.patch.object(wrapper,"command") as command, mock.patch.object(wrapper.tempfile,"TemporaryDirectory") as directory:
            value = wrapper.native_run(Path("/mock"),Path("/mock/forge"),identity,paths,10)
        command.assert_not_called();directory.assert_not_called()
        self.assertEqual((value["status"],value["failure"]),("failed","command-timeout"))
        with mock.patch.object(wrapper.time,"monotonic",side_effect=[0,10]), mock.patch.object(wrapper,"command") as command:
            value = wrapper.native_run(Path("/mock"),Path("/mock/forge"),identity,paths,10)
        command.assert_not_called()
        self.assertEqual(value["failure"],"command-timeout")

    def test_capture_tools_completion_refuses_expired_qualification(self):
        """Actual qualification cannot return tools after its final shared deadline even with successful adapters."""
        pin = plan()["release_pin"];expired = [False]
        def ordinary(*args):
            """Simulate a bounded ordinary qualification ending exactly at the deadline."""
            expired[0] = True
            return {"cargo":"1.99.0","rustc":"1.99.0","rust_host":"x86_64-unknown-linux-gnu"}
        outputs = [wrapper.shared.canonical_bytes({"version":"3.11.9","paths":["/usr/lib/python3.11"]}),
                   b"ip utility, iproute2-6.17.0\n",b"Sudo version 1.9.15p5\n"]
        with mock.patch.object(wrapper.sys,"platform","linux"), mock.patch.object(wrapper.os,"uname",return_value=types.SimpleNamespace(machine="x86_64")), \
             mock.patch.object(wrapper.os,"getuid",return_value=1001), mock.patch.object(wrapper.os,"geteuid",return_value=1001), \
             mock.patch.object(wrapper.os,"getgid",return_value=1001), mock.patch.object(wrapper.os,"getegid",return_value=1001), \
             mock.patch.object(wrapper,"administration_tool",side_effect=lambda p:p), mock.patch.object(wrapper.shutil,"which",side_effect=lambda n:"/usr/bin/"+n), \
             mock.patch.object(wrapper,"command",side_effect=[{"exit_code":0,"failure":None,"output":r} for r in outputs]), \
             mock.patch.object(wrapper,"stdlib_inventory",return_value={"pin":pin,"entries":10}), \
             mock.patch.object(wrapper.shared,"hash_file",return_value=pin), mock.patch.object(wrapper.shared,"tool_versions",side_effect=ordinary), \
             mock.patch.object(wrapper.time,"monotonic",side_effect=lambda:1 if expired[0] else 0):
            with self.assertRaises(wrapper.GateError) as caught: wrapper.capture_tools(Path("/mock"),1)
        self.assertEqual(caught.exception.code,"command-timeout")
        self.assertTrue(caught.exception.incomplete)

    def test_verify_initial_capture_failure_is_unverified(self):
        """Failure before identity capture does not invent a changed-byte comparison."""
        with tempfile.TemporaryDirectory() as root, \
             mock.patch.object(wrapper, "capture_identity", side_effect=[RuntimeError("secret"), {"tracked_source_clean": True}]):
            value = wrapper.verify(Path(root), Path(root) / "forge", Path(root) / "out")
        self.assertEqual(value["input_stability"], "unverified")
        self.assertEqual(value["status"], "failed")
        self.assertNotIn(b"secret", native.canonical(value))

    def test_verify_after_capture_failure_revokes_pass_without_changed_claim(self):
        """A failed after read rejects a mock producer pass and keeps comparison unverified."""
        identity = {"tracked_source_clean": True}
        producer = {"status": "passed", "failure": None, "receipt": {"cleanup": {"state": "verified", "forced": False}},
                    "receipt_pin": None, "exit_code": 0}
        with tempfile.TemporaryDirectory() as root, \
             mock.patch.object(wrapper, "capture_identity", side_effect=[identity, RuntimeError("secret")]), \
             mock.patch.object(wrapper.shared, "checkout_binding", return_value={}), \
             mock.patch.object(wrapper.sys, "platform", "linux"), \
             mock.patch.object(wrapper, "capture_tools", return_value=({}, {})), \
             mock.patch.object(wrapper, "native_run", return_value=producer):
            value = wrapper.verify(Path(root), Path(root) / "forge", Path(root) / "out", build_outcome="success")
        self.assertEqual(value["status"], "failed")
        self.assertEqual(value["input_stability"], "unverified")
        self.assertEqual(value["failure"], "verification-input-invalid")

    def verify_adapter(self, *, build="success", dirty=False, source_change=False, tool_change=False,
                       native_error=False, tool_missing=False):
        """Call complete actual outer verification with isolated identity/tool/producer adapters."""
        p = plan()
        receipt = native.execute(p, FakeNative(p))
        producer = {"status": "passed", "failure": None, "exit_code": 0,
                    "receipt": receipt, "receipt_pin": wrapper.pin_bytes(native.canonical(receipt))}
        identity = {"tracked_source_clean": not dirty, "mock_identity": "before"}
        after = dict(identity, mock_identity="after") if source_change else dict(identity)
        pin = p["release_pin"]
        tools = {"ordinary": {"cargo": "1.99.0", "rustc": "1.99.0", "rust_host": "x86_64-unknown-linux-gnu"},
                 "privileged_python": {"pin": pin, "version": "3.11.9", "root_trust": True,
                                       "stdlib_pin": pin, "stdlib_entries": 10},
                 "ip": {"pin": pin, "version": "6.17.0", "root_trust": True},
                 "sudo": {"pin": pin, "version": "1.9.15p5", "root_trust": True}}
        after_tools = copy.deepcopy(tools)
        if tool_change: after_tools["ip"]["pin"] = {"bytes": 4, "sha256": "b" * 64}
        calls = []
        def dispatch(*args):
            """Journal attempted producer execution or raise an unknown private exception."""
            calls.append(args)
            if native_error: raise RuntimeError("private secret")
            return producer
        with tempfile.TemporaryDirectory() as root, \
             mock.patch.object(wrapper, "capture_identity", side_effect=[identity, after]), \
             mock.patch.object(wrapper.shared, "checkout_binding", return_value={"state": "mock-bound"}), \
             mock.patch.object(wrapper.sys, "platform", "linux"), \
             mock.patch.object(wrapper, "capture_tools", side_effect=wrapper.GateError("tool-unavailable", True)
                               if tool_missing else [(tools, {}), (after_tools, {})]), \
             mock.patch.object(wrapper, "native_run", side_effect=dispatch):
            out = Path(root) / "output"
            value = wrapper.verify(Path(root), Path(root) / "forge", out, build_outcome=build)
            self.assertEqual((out / wrapper.OUTPUT).read_bytes(), wrapper.shared.canonical_bytes(value))
        return value, calls

    def test_complete_wrapper_positive_route_preserves_pending_gates(self):
        """Complete actual outer route requires producer/context/tool/source/cleanup facts and retains wider gates."""
        value, calls = self.verify_adapter()
        self.assertEqual(value["status"], "passed")
        self.assertEqual(len(calls), 1)
        self.assertEqual(value["input_stability"], "unchanged")
        self.assertEqual(value["tool_stability"], "unchanged")
        self.assertFalse(value["acceptance_eligible"])
        self.assertEqual(value["attempted_egress"], {"state": "unmeasured", "count": None})
        self.assertTrue(value["pending_gates"])

    def test_build_unqualified_dirty_context_or_missing_tool_never_dispatches(self):
        """Actual outer controller skips native launch for failed/unrecorded builds, dirty context or missing tools."""
        for kwargs, expected in (({"build": "failure"}, "failed"), ({"build": "cancelled"}, "failed"),
                                 ({"build": "unrecorded"}, "incomplete"), ({"build": "skipped"}, "incomplete"),
                                 ({"dirty": True}, "failed"), ({"tool_missing": True}, "incomplete")):
            value, calls = self.verify_adapter(**kwargs)
            self.assertEqual(value["status"], expected)
            self.assertEqual(calls, [])
            self.assertEqual(value["producer"]["status"], "not-run")
            self.assertEqual(value["cleanup"], {"state": "verified-not-created", "forced": False})

    def test_changed_source_or_tool_after_revoke_mock_runtime_success(self):
        """Actual byte/tool comparison revokes otherwise successful mock native work."""
        for kwargs, code in (({"source_change": True}, "source-changed"), ({"tool_change": True}, "tool-identity-changed")):
            value, calls = self.verify_adapter(**kwargs)
            self.assertEqual(value["status"], "failed")
            self.assertEqual(value["failure"], code)
            self.assertEqual(len(calls), 1)

    def test_unknown_dispatched_exception_has_failed_producer_and_unverified_cleanup(self):
        """Unknown native_run failure retains attempted dispatch rather than fictitious not-run credit."""
        value, calls = self.verify_adapter(native_error=True)
        self.assertEqual(len(calls), 1)
        self.assertEqual(value["status"], "failed")
        self.assertEqual(value["producer"]["status"], "failed")
        self.assertEqual(value["cleanup"]["state"], "unverified")
        self.assertIsNone(value["producer"]["exit_code"])
        self.assertNotIn(b"private secret", native.canonical(value))

    def test_fresh_outer_output_collision_preserves_existing_bytes(self):
        """Actual verifier rejects a nonfresh destination before any identity/tool/native call."""
        with tempfile.TemporaryDirectory() as root:
            out = Path(root) / "out";out.mkdir()
            path = out / wrapper.OUTPUT;path.write_bytes(b"historical")
            with mock.patch.object(wrapper, "capture_identity") as capture, self.assertRaises(ValueError):
                wrapper.verify(Path(root), Path(root) / "forge", out)
            capture.assert_not_called()
            self.assertEqual(path.read_bytes(), b"historical")

    def test_outer_main_publication_failure_returns_nonzero(self):
        """Actual CLI guard catches publication failure without exposing private exception text."""
        with mock.patch.object(wrapper.sys, "argv", ["wrapper", "--forge", "/mock/forge", "--output-dir", "/mock/out"]), \
             mock.patch.object(wrapper, "verify", side_effect=OSError("private secret")), \
             mock.patch.object(wrapper.sys, "stderr", io.StringIO()) as stream:
            self.assertEqual(wrapper.main(), 1)
            self.assertNotIn("private secret", stream.getvalue())

    def test_native_run_fixed_bootstrap_actual_private_raw_reader(self):
        """Actual ordinary dispatch builds fixed bootstrap argv and revalidates privately written mock raw files."""
        p = plan()
        identity = {"provided_release_binary": p["release_pin"], "inputs": dict(p["source_pins"])}
        identity["inputs"][wrapper.EXTRA_INPUTS[0]] = {"bytes": 4, "sha256": "c" * 64}
        record = []
        def dispatch(argv, root, deadline, raw, privileged):
            """Mock only privileged execution while retaining its actual plan/receipt byte exchange."""
            record.append((argv, raw, privileged))
            inputs = json.loads(raw)
            value = native.execute(inputs, FakeNative(inputs))
            private = Path(inputs["output_dir"])
            (private / wrapper.CLIENT_OUTPUT).write_bytes(native.canonical(value["client"]["receipt"]))
            (private / wrapper.NATIVE_OUTPUT).write_bytes(native.canonical(value))
            return {"exit_code": 0, "failure": None, "output": b""}
        paths = {"python": Path("/usr/bin/python3.11"), "ip": Path("/usr/sbin/ip"), "sudo": Path("/usr/bin/sudo")}
        routes = [("GET", None, x) for x in client()[1]]
        with mock.patch.object(wrapper, "command", side_effect=dispatch), \
             mock.patch.object(wrapper.os, "getuid", return_value=1001), mock.patch.object(wrapper.os, "getgid", return_value=1001), \
             mock.patch.object(wrapper.shared, "contract_routes", return_value=routes):
            result = wrapper.native_run(Path("/mock/root"), Path("/mock/forge"), identity, paths, time.monotonic()+600)
        self.assertEqual(result["status"], "passed")
        argv, raw, privileged = record[0]
        self.assertEqual(argv[:3], ["/usr/bin/sudo", "-n", "--"])
        self.assertIn(wrapper.BOOTSTRAP, argv)
        self.assertEqual(raw, wrapper.shared.canonical_bytes(json.loads(raw)))
        self.assertTrue(privileged)


class WrapperCommandCleanupControls(unittest.TestCase):
    """Exercise actual ordinary-command close failures with synthetic pipes and no privileged child."""

    def cleanup_command(self, *, selector_fault=False, stream_fault=None, cap=None):
        """Run the actual dual-pipe loop and spy every owned close under fixed mocked status and clock."""
        selection = FakeSelector()
        child = FakeCommandChild(selection, 2, b"")
        reads = {31: [b"safe stdout", b""], 32: [b"private stderr", b""]}

        def read(fd, _limit):
            """Deliver independent bounded stdout/stderr blocks then EOF without native I/O."""
            return reads[fd].pop(0) if reads[fd] else b""

        faults = {}
        for name in ("stdin", "stdout", "stderr"):
            if stream_fault == name:
                faults[name] = [None, OSError("private close detail")] if name == "stdin" else OSError("private close detail")
        with mock.patch.object(wrapper.selectors, "DefaultSelector", return_value=selection), \
             mock.patch.object(wrapper.subprocess, "Popen", return_value=child), \
             mock.patch.object(wrapper.os, "set_blocking"), mock.patch.object(wrapper.os, "read", side_effect=read), \
             mock.patch.object(wrapper.time, "monotonic", side_effect=lambda: selection.ticks), \
             mock.patch.object(wrapper, "MAX_CAPTURE", wrapper.MAX_CAPTURE if cap is None else cap), \
             mock.patch.object(selection, "close", side_effect=RuntimeError("private selector detail") if selector_fault else None,
                               wraps=selection.close) as selector_close, \
             mock.patch.object(child.stdin, "close", side_effect=faults.get("stdin"), wraps=child.stdin.close) as stdin_close, \
             mock.patch.object(child.stdout, "close", side_effect=faults.get("stdout"), wraps=child.stdout.close) as stdout_close, \
             mock.patch.object(child.stderr, "close", side_effect=faults.get("stderr"), wraps=child.stderr.close) as stderr_close:
            value = wrapper.command(["mock"], Path("/mock"), 100, input_bytes=b"")
        return value, child, {"selector": selector_close.call_count, "stdin": stdin_close.call_count,
                              "stdout": stdout_close.call_count, "stderr": stderr_close.call_count}

    def test_selector_close_failure_still_attempts_every_pipe_and_revokes_success(self):
        """Actual selector cleanup failure cannot skip any owned stream or retain a successful output."""
        value, child, closes = self.cleanup_command(selector_fault=True)
        self.assertEqual(value, {"exit_code": 0, "failure": "cleanup-unverified", "output": b""})
        self.assertEqual(closes["selector"], 1)
        self.assertGreaterEqual(closes["stdin"], 2)
        self.assertEqual(closes["stdout"], 1)
        self.assertEqual(closes["stderr"], 1)
        self.assertFalse(child.killed)
        self.assertNotIn("private selector detail", repr(value))

    def test_each_pipe_close_failure_discards_output_and_keeps_other_close_attempts(self):
        """Each final stdin/stdout/stderr close error independently rejects success with all sibling attempts retained."""
        for name in ("stdin", "stdout", "stderr"):
            with self.subTest(stream=name):
                value, child, closes = self.cleanup_command(stream_fault=name)
                self.assertEqual(value, {"exit_code": 0, "failure": "cleanup-unverified", "output": b""})
                self.assertEqual(closes["selector"], 1)
                self.assertGreaterEqual(closes["stdin"], 2)
                self.assertEqual(closes["stdout"], 1)
                self.assertEqual(closes["stderr"], 1)
                self.assertFalse(child.killed)
                self.assertNotIn("private close detail", repr(value))

    def test_primary_output_bound_failure_survives_later_selector_close_failure(self):
        """A later cleanup error cannot erase the first bounded-output failure or restore private bytes."""
        value, child, closes = self.cleanup_command(selector_fault=True, cap=1)
        self.assertEqual(value, {"exit_code": 0, "failure": "output-bound", "output": b""})
        self.assertEqual(closes["selector"], 1)
        self.assertGreaterEqual(closes["stdin"], 2)
        self.assertEqual(closes["stdout"], 1)
        self.assertEqual(closes["stderr"], 1)
        self.assertFalse(child.killed)


class ToolDiagnosticControls(unittest.TestCase):
    """Exercise real ordinary-wrapper qualification seams with synthetic metadata and no native/tool execution."""

    def phase_pairs(self):
        """Supply independent closed vocabulary expectations rather than deriving expected pairs from the candidate."""
        paths=("missing","not-absolute","not-root-owned","worker-writable","unsupported-link","not-directory","not-regular","link-bound","not-executable","path-observation-unverified")
        commands=("command-failed","command-timeout","output-bound","command-cleanup-unverified")
        return {"python-path":paths,"ip-path":paths,"sudo-path":paths,
            "python-probe":commands+("probe-json-invalid","probe-shape-invalid","version-invalid"),
            "stdlib-roots":("root-shape-invalid","missing-root","not-root-owned","worker-writable","unsupported-link","not-directory","not-regular","dynload-missing","path-observation-unverified"),
            "stdlib-entry":("not-root-owned","worker-writable","unsupported-link","not-directory","not-regular","unsupported-kind","entry-bound","depth-bound","byte-bound","entry-observation-unverified"),
            "ip-version":commands+("version-invalid","identity-observation-unverified"),
            "sudo-version":commands+("version-invalid","identity-observation-unverified"),
            "ordinary-tools":("required-identity-missing","identity-observation-unverified"),
            "qualification-budget":("deadline-expired",)}

    def failed_step(self, phase, callback, *args, **kwargs):
        """Call the actual fixed-component adapter and return its typed failure for exact consequence assertions."""
        with self.assertRaises(wrapper.GateError) as caught:
            wrapper.tool_step(phase,callback,*args,**kwargs)
        return caught.exception

    @contextlib.contextmanager
    def capture_context(self, *, observations=None, path_fault=None, inventory_fault=None,
                        ordinary=None, hash_fault=None, clock=0):
        """Replace every ordinary executable/identity observation while retaining actual qualification control flow."""
        pin=plan()["release_pin"]
        if observations is None:
            observations=[{"exit_code":0,"failure":None,"output":wrapper.shared.canonical_bytes({"version":"3.11.9","paths":["/mock/stdlib","/mock/stdlib/lib-dynload"]})},
                {"exit_code":0,"failure":None,"output":b"ip utility, iproute2-6.17.0\n"},
                {"exit_code":0,"failure":None,"output":b"Sudo version 1.9.15p5\n"}]
        if ordinary is None:ordinary={"cargo":"1.99.0","rustc":"1.99.0","rust_host":"x86_64-unknown-linux-gnu"}
        def path_check(path):
            """Return a synthetic existing executable or fail only the explicitly selected component."""
            component="python" if path.name=="python3" else path.name
            if path_fault is not None and component==path_fault[0]:raise path_fault[1]
            return path
        with contextlib.ExitStack() as stack:
            for target,name,value in ((wrapper.sys,"platform","linux"),(wrapper.os,"uname",types.SimpleNamespace(machine="x86_64")),
                                      (wrapper.os,"getuid",1001),(wrapper.os,"geteuid",1001),(wrapper.os,"getgid",1001),(wrapper.os,"getegid",1001)):
                stack.enter_context(mock.patch.object(target,name,return_value=value) if name!="platform" else mock.patch.object(target,name,value))
            stack.enter_context(mock.patch.object(wrapper.time,"monotonic",side_effect=clock if callable(clock) else None,return_value=clock if not callable(clock) else None))
            stack.enter_context(mock.patch.object(wrapper.shutil,"which",side_effect=lambda name:"/mock/"+name))
            paths=stack.enter_context(mock.patch.object(wrapper,"administration_tool",side_effect=path_check))
            command=stack.enter_context(mock.patch.object(wrapper,"command",side_effect=observations))
            inventory=stack.enter_context(mock.patch.object(wrapper,"stdlib_inventory",side_effect=inventory_fault,return_value={"pin":pin,"entries":10}))
            ordinary_call=stack.enter_context(mock.patch.object(wrapper.shared,"tool_versions",return_value=ordinary))
            hashed=stack.enter_context(mock.patch.object(wrapper.shared,"hash_file",side_effect=hash_fault,return_value=pin))
            yield {"command":command,"paths":paths,"inventory":inventory,"ordinary":ordinary_call,"hash":hashed}

    @contextlib.contextmanager
    def inventory_context(self, entries, *, max_entries=None, max_bytes=None, scandir_fault=None):
        """Supply actual traversal code with qualified synthetic roots and explicit fake directory entries only."""
        def scan(path):
            """Expose one root inventory and an empty dynload directory without scanning the host."""
            if scandir_fault is not None:raise scandir_fault
            return contextlib.nullcontext(iter(entries if path==Path("/mock/stdlib") else []))
        with contextlib.ExitStack() as stack:
            stack.enter_context(mock.patch.object(wrapper.Path,"exists",return_value=True))
            stack.enter_context(mock.patch.object(wrapper,"root_trusted",side_effect=lambda path,directory=False:path))
            stack.enter_context(mock.patch.object(wrapper.time,"monotonic",return_value=0))
            scanned=stack.enter_context(mock.patch.object(wrapper.os,"scandir",side_effect=scan))
            hashed=stack.enter_context(mock.patch.object(wrapper.shared,"hash_file",return_value=plan()["release_pin"]))
            if max_entries is not None:stack.enter_context(mock.patch.object(wrapper,"MAX_ENTRIES",max_entries))
            if max_bytes is not None:stack.enter_context(mock.patch.object(wrapper,"MAX_STDLIB_BYTES",max_bytes))
            yield scanned,hashed

    def verify_faults(self, *, first_fault=None, after_identity_fault=None, after_tool_fault=None, build="success"):
        """Publish actual ordinary-wrapper bytes using only independent synthetic identity/tool/native adapters."""
        identity={"tracked_source_clean":True,"mock":"stable"}
        producer={"status":"passed","failure":None,"exit_code":0,"receipt":{"cleanup":{"state":"verified","forced":False}},"receipt_pin":None}
        tools={"mock":"stable"};identities=[identity,after_identity_fault or dict(identity)]
        captures=first_fault if first_fault is not None else [(tools,{}),after_tool_fault if after_tool_fault is not None else (dict(tools),{})]
        with tempfile.TemporaryDirectory() as private, mock.patch.object(wrapper,"capture_identity",side_effect=identities), \
             mock.patch.object(wrapper.shared,"checkout_binding",return_value={}),mock.patch.object(wrapper.sys,"platform","linux"), \
             mock.patch.object(wrapper,"capture_tools",side_effect=captures) as capture,mock.patch.object(wrapper,"native_run",return_value=producer) as native_run:
            output=Path(private)/"output"
            value=wrapper.verify(Path(private),Path(private)/"forge",output,build_outcome=build)
            raw=(output/wrapper.OUTPUT).read_bytes()
            self.assertEqual(raw,wrapper.shared.canonical_bytes(value))
        return value,raw,capture,native_run

    def test_all_fixed_diagnostic_pairs_are_closed_copied_and_nullable(self):
        """Validate every independently enumerated pair and keep public output to exactly three fixed fields."""
        expected=self.phase_pairs();self.assertEqual(wrapper.TOOL_PHASE_REASONS,expected);self.assertEqual(len(expected),10)
        self.assertIsNone(wrapper.validate_tool_diagnostic(None))
        for phase,reasons in expected.items():
            for reason in reasons:
                original={"phase":phase,"reason":reason,"exit_code":None}
                value=wrapper.validate_tool_diagnostic(original)
                self.assertEqual(value,original);self.assertIsNot(value,original)
                self.assertEqual(wrapper.tool_diagnostic(phase,reason),value)
                original["reason"]="PRIVATE exception path"
                self.assertEqual(value,{"phase":phase,"reason":reason,"exit_code":None})

    def test_diagnostic_refuses_open_fields_wrong_pairs_bool_and_status_coercion(self):
        """Reject private fields and unknown stage/reason/type combinations instead of coercing hostile metadata."""
        base={"phase":"python-probe","reason":"command-failed","exit_code":None}
        for field,value in (("phase","PRIVATE"),("phase",True),("reason","missing"),("reason",[]),("exit_code",True),("exit_code",1.0),("exit_code",-256),("exit_code",256),("private","PRIVATE")):
            record=dict(base);record[field]=value
            with self.subTest(field=field),self.assertRaises(ValueError):wrapper.validate_tool_diagnostic(record)
        for field in base:
            record=dict(base);del record[field]
            with self.subTest(missing=field),self.assertRaises(ValueError):wrapper.validate_tool_diagnostic(record)
        for value in (False,[],"PRIVATE",1):
            with self.assertRaises(ValueError):wrapper.validate_tool_diagnostic(value)
        for phase,reasons in self.phase_pairs().items():
            if phase not in {"python-probe","ip-version","sudo-version"}:
                with self.assertRaises(ValueError):wrapper.tool_diagnostic(phase,reasons[0],0)

    def test_command_status_bounds_and_parsed_zero_status_are_distinct(self):
        """Only actual bounded exact command statuses survive; parsed-output faults cannot claim nonzero status."""
        for phase in ("python-probe","ip-version","sudo-version"):
            for code in (-255,-9,0,1,255):
                self.assertEqual(wrapper.tool_diagnostic(phase,"command-failed",code)["exit_code"],code)
            for code in (None,False,True,1.0,-256,256):self.assertIsNone(wrapper.observed_exit(code))
            for reason in (("probe-json-invalid","probe-shape-invalid","version-invalid") if phase=="python-probe" else ("version-invalid",)):
                self.assertIsNone(wrapper.tool_diagnostic(phase,reason)["exit_code"])
                self.assertEqual(wrapper.tool_diagnostic(phase,reason,0)["exit_code"],0)
                for code in (-9,1):
                    with self.assertRaises(ValueError):wrapper.tool_diagnostic(phase,reason,code)

    def test_duplicate_nonfinite_nested_diagnostic_json_never_becomes_safe_fact(self):
        """Apply the actual strict parser before typed validation so duplicate/nonfinite public fields cannot be selected."""
        for raw in (b'{"diagnostic":{"phase":"python-probe","phase":"ip-version","reason":"command-failed","exit_code":1}}',
                    b'{"diagnostic":{"phase":"python-probe","reason":"command-failed","exit_code":NaN}}'):
            with self.assertRaises(ValueError):wrapper.strict_json(raw)
        raw=wrapper.shared.canonical_bytes({"diagnostic":{"phase":"python-probe","reason":"command-failed","exit_code":1,"private":"PRIVATE"}})
        with self.assertRaises(ValueError):wrapper.validate_tool_diagnostic(wrapper.strict_json(raw)["diagnostic"])

    def test_gate_error_and_successful_step_keep_existing_callback_contract(self):
        """The original two-argument error and unchanged callback signature stay usable without diagnostics or dispatch."""
        error=wrapper.GateError("tool-unavailable",True)
        self.assertEqual((error.code,error.incomplete,str(error)),("tool-unavailable",True,"tool-unavailable"))
        self.assertIsNone(error.diagnostic);self.assertIsNone(error.tool_reason)
        marker=object();callback=mock.Mock(return_value=marker)
        self.assertIs(wrapper.tool_step("python-path",callback,1,2),marker);callback.assert_called_once_with(1,2)
        for phase,reason in (("PRIVATE",None),("python-path","version-invalid")):
            callback.reset_mock()
            with self.assertRaises(ValueError):wrapper.tool_step(phase,callback,reason=reason)
            callback.assert_not_called()

    def test_typed_step_retains_error_identity_status_and_first_component(self):
        """An attached first fault survives nested qualification while mismatched internal reasons use fixed defaults."""
        error=wrapper.GateError("tool-untrusted",True,tool_reason="not-root-owned")
        callback=mock.Mock(side_effect=error);caught=self.failed_step("python-path",callback,Path("/mock/tool"))
        self.assertIs(caught,error);self.assertEqual((caught.code,caught.incomplete),("tool-untrusted",True))
        self.assertEqual(caught.diagnostic,{"phase":"python-path","reason":"not-root-owned","exit_code":None})
        caught=self.failed_step("ordinary-tools",mock.Mock(side_effect=error))
        self.assertEqual(caught.diagnostic,{"phase":"python-path","reason":"not-root-owned","exit_code":None})
        wrong=wrapper.GateError("tool-untrusted",True,tool_reason="byte-bound")
        self.assertEqual(self.failed_step("sudo-path",mock.Mock(side_effect=wrong)).diagnostic,{"phase":"sudo-path","reason":"path-observation-unverified","exit_code":None})

    def test_unknown_step_faults_stay_failed_and_redacted(self):
        """Unclassified callback faults keep generic failed classification and never leak exception text or chains."""
        for phase in self.phase_pairs():
            error=self.failed_step(phase,mock.Mock(side_effect=RuntimeError("PRIVATE path token credential")))
            self.assertEqual(error.code,"verification-input-invalid");self.assertFalse(error.incomplete)
            self.assertEqual(error.diagnostic["phase"],phase);self.assertIsNone(error.diagnostic["exit_code"])
            self.assertNotIn("PRIVATE",str(error));self.assertNotIn(b"PRIVATE",wrapper.shared.canonical_bytes(error.diagnostic))
            self.assertTrue(error.__suppress_context__)

    def test_actual_root_trust_predicates_keep_fixed_priority(self):
        """Exercise original root/mode/link/kind gates through synthetic lstat records without reading host ownership."""
        good=types.SimpleNamespace(st_uid=0,st_mode=stat.S_IFREG|0o755)
        directory=types.SimpleNamespace(st_uid=0,st_mode=stat.S_IFDIR|0o755)
        cases=((types.SimpleNamespace(st_uid=1001,st_mode=stat.S_IFLNK|0o777),False,"not-root-owned"),
               (types.SimpleNamespace(st_uid=0,st_mode=stat.S_IFLNK|0o777),False,"worker-writable"),
               (types.SimpleNamespace(st_uid=0,st_mode=stat.S_IFLNK|0o755),False,"unsupported-link"),
               (good,True,"not-directory"),(directory,False,"not-regular"))
        for info,wants_directory,reason in cases:
            with mock.patch.object(wrapper.Path,"lstat",return_value=info):
                error=self.failed_step("python-path",wrapper.root_trusted,Path("/mock/tool"),wants_directory)
            self.assertEqual(error.diagnostic,{"phase":"python-path","reason":reason,"exit_code":None});self.assertEqual(error.code,"tool-untrusted");self.assertFalse(error.incomplete)
        with mock.patch.object(wrapper.Path,"lstat") as observed:
            error=self.failed_step("ip-path",wrapper.root_trusted,Path("relative"))
        observed.assert_not_called();self.assertEqual(error.diagnostic["reason"],"not-absolute")
        with mock.patch.object(wrapper.Path,"lstat",side_effect=[good,directory,directory]):
            self.assertEqual(wrapper.root_trusted(Path("/mock/tool")),Path("/mock/tool"))

    def test_actual_administration_missing_owner_mode_and_exec_gates(self):
        """Component diagnostics preserve already-present executable trust failures and abstain from running tools."""
        good=types.SimpleNamespace(st_uid=0,st_mode=stat.S_IFREG|0o755)
        cases=((False,good,True,"missing","tool-unavailable"),(True,types.SimpleNamespace(st_uid=1001,st_mode=stat.S_IFREG|0o755),True,"not-root-owned","tool-untrusted"),
               (True,types.SimpleNamespace(st_uid=0,st_mode=stat.S_IFREG|0o777),True,"worker-writable","tool-untrusted"),(True,good,False,"not-executable","tool-untrusted"))
        for exists,info,executable,reason,code in cases:
            with mock.patch.object(wrapper.Path,"exists",return_value=exists),mock.patch.object(wrapper.Path,"lstat",return_value=info), \
                 mock.patch.object(wrapper,"root_trusted",side_effect=lambda path,directory=False:path),mock.patch.object(wrapper.Path,"resolve",return_value=Path("/mock/tool")),mock.patch.object(wrapper.os,"access",return_value=executable):
                error=self.failed_step("sudo-path",wrapper.administration_tool,Path("/mock/tool"))
            self.assertEqual(error.code,code);self.assertTrue(error.incomplete);self.assertEqual(error.diagnostic,{"phase":"sudo-path","reason":reason,"exit_code":None})

    def test_actual_administration_link_bound_and_observation_failure(self):
        """Resolve at most the fixed link bound and classify unreadable metadata without exposing a path."""
        link=types.SimpleNamespace(st_uid=0,st_mode=stat.S_IFLNK|0o777)
        with mock.patch.object(wrapper.Path,"exists",return_value=True),mock.patch.object(wrapper.Path,"lstat",return_value=link), \
             mock.patch.object(wrapper,"root_trusted",side_effect=lambda path,directory=False:path),mock.patch.object(wrapper.os,"readlink",return_value="/mock/tool") as readlink,mock.patch.object(wrapper.os,"access") as access:
            error=self.failed_step("ip-path",wrapper.administration_tool,Path("/mock/tool"))
        self.assertEqual(readlink.call_count,16);access.assert_not_called();self.assertEqual(error.diagnostic["reason"],"link-bound")
        with mock.patch.object(wrapper.Path,"exists",side_effect=OSError("PRIVATE path")):
            error=self.failed_step("python-path",wrapper.administration_tool,Path("/mock/tool"))
        self.assertEqual(error.code,"verification-input-invalid");self.assertFalse(error.incomplete);self.assertEqual(error.diagnostic["reason"],"path-observation-unverified")

    def test_actual_capture_paths_stop_before_probe_and_native_dispatch(self):
        """All three caller-selected executable phases remain distinct and fail before later command observations."""
        for component in ("python","ip","sudo"):
            error=wrapper.GateError("tool-untrusted",True,tool_reason="not-root-owned")
            with self.capture_context(path_fault=(component,error)) as calls:
                with self.assertRaises(wrapper.GateError) as caught:wrapper.capture_tools(Path("/mock"),100)
                calls["command"].assert_not_called();calls["inventory"].assert_not_called();calls["ordinary"].assert_not_called()
            self.assertEqual(caught.exception.diagnostic,{"phase":component+"-path","reason":"not-root-owned","exit_code":None})

    def test_actual_capture_probe_malformed_shape_and_version_zero_status(self):
        """Successful command status cannot qualify invalid JSON/keys/version and each unchanged rejection stays typed."""
        cases=((b'PRIVATE',"probe-json-invalid","verification-input-invalid",False),
               (b'{"version":"3.11.9","version":"3.11.9","paths":[]}',"probe-json-invalid","execution-unverified",False),
               (b'{"version":"3.11.9","paths":NaN}',"probe-json-invalid","execution-unverified",False),
               (b'{"version":"3.11.9","paths":[],"private":"PRIVATE"}',"probe-shape-invalid","execution-unverified",False),
               (wrapper.shared.canonical_bytes({"version":"3.10.9","paths":[]}),"version-invalid","tool-untrusted",True))
        for raw,reason,code,incomplete in cases:
            with self.capture_context(observations=[{"exit_code":0,"failure":None,"output":raw}]) as calls:
                with self.assertRaises(wrapper.GateError) as caught:wrapper.capture_tools(Path("/mock"),100)
                calls["inventory"].assert_not_called();calls["ordinary"].assert_not_called()
            self.assertEqual((caught.exception.code,caught.exception.incomplete),(code,incomplete))
            self.assertEqual(caught.exception.diagnostic,{"phase":"python-probe","reason":reason,"exit_code":0})
            self.assertNotIn(b"PRIVATE",wrapper.shared.canonical_bytes(caught.exception.diagnostic))

    def test_actual_python_probe_selector_failure_never_claims_json_observation(self):
        """A real command adapter failure before spawn/parse retains generic failure and a null observed status."""
        actual_command = wrapper.command
        with self.capture_context() as calls, \
             mock.patch.object(wrapper.selectors, "DefaultSelector", side_effect=RuntimeError("PRIVATE selector path")) as selector, \
             mock.patch.object(wrapper.subprocess, "Popen") as spawn, \
             mock.patch.object(wrapper, "strict_json") as parse, \
             mock.patch.object(wrapper, "native_run") as native_run:
            calls["command"].side_effect = actual_command
            with self.assertRaises(wrapper.GateError) as caught:
                wrapper.capture_tools(Path("/mock"), 100)
            selector.assert_called_once_with()
            spawn.assert_not_called()
            parse.assert_not_called()
            native_run.assert_not_called()
            calls["inventory"].assert_not_called()
            calls["ordinary"].assert_not_called()
            self.assertEqual(calls["command"].call_count, 1)
        self.assertEqual((caught.exception.code, caught.exception.incomplete), ("verification-input-invalid", False))
        self.assertEqual(caught.exception.diagnostic, {"phase": "python-probe", "reason": "command-failed", "exit_code": None})
        self.assertNotIn(b"PRIVATE", wrapper.shared.canonical_bytes(caught.exception.diagnostic))
        self.assertTrue(caught.exception.__suppress_context__)

    def test_actual_version_commands_keep_exit_failure_and_stdout_binding(self):
        """Both fixed version commands retain actual nonzero/timeout/bound/cleanup facts without publishing output."""
        for name,flag in (("ip","-V"),("sudo","--version")):
            for failure,code,reason in ((None,-9,"command-failed"),("command-timeout",None,"command-timeout"),("output-bound",0,"output-bound"),("cleanup-unverified",0,"command-cleanup-unverified")):
                with mock.patch.object(wrapper,"command",return_value={"failure":failure,"exit_code":code,"output":b"PRIVATE path credential"}) as command,mock.patch.object(wrapper.time,"monotonic",return_value=0),mock.patch.object(wrapper.shared,"hash_file") as hashed:
                    with self.assertRaises(wrapper.GateError) as caught:wrapper.capture_tool_version(name,Path("/mock/"+name),Path("/mock"),100)
                self.assertEqual(command.call_args.args[0],["/mock/"+name,flag]);hashed.assert_not_called()
                self.assertEqual(caught.exception.diagnostic,{"phase":name+"-version","reason":reason,"exit_code":code})
                self.assertEqual((caught.exception.code,caught.exception.incomplete),("tool-unavailable",True))
                self.assertNotIn(b"PRIVATE",wrapper.shared.canonical_bytes(caught.exception.diagnostic))
            with mock.patch.object(wrapper,"command",return_value={"failure":None,"exit_code":0,"output":b"PRIVATE"}),mock.patch.object(wrapper.time,"monotonic",return_value=0):
                with self.assertRaises(wrapper.GateError) as caught:wrapper.capture_tool_version(name,Path("/mock/"+name),Path("/mock"),100)
            self.assertEqual(caught.exception.diagnostic,{"phase":name+"-version","reason":"version-invalid","exit_code":0})

    def test_actual_capture_command_faults_include_python_probe_actual_status(self):
        """The initial Python command independently retains all four allowlisted lifecycle/output failures."""
        for failure,code,reason in ((None,-9,"command-failed"),("command-timeout",None,"command-timeout"),("output-bound",0,"output-bound"),("cleanup-unverified",0,"command-cleanup-unverified")):
            with self.capture_context(observations=[{"failure":failure,"exit_code":code,"output":b"PRIVATE"}]) as calls:
                with self.assertRaises(wrapper.GateError) as caught:wrapper.capture_tools(Path("/mock"),100)
                calls["inventory"].assert_not_called();calls["ordinary"].assert_not_called()
            self.assertEqual(caught.exception.diagnostic,{"phase":"python-probe","reason":reason,"exit_code":code})

    def test_actual_stdlib_root_shape_missing_dynload_and_owner_faults(self):
        """Root inventories preserve exact shape/existence/dynload/trust predicates without scanning host directories."""
        for paths in (None,[],["/mock/a"],["relative","/mock/lib-dynload"],["/mock/a","/mock/a"]):
            with mock.patch.object(wrapper.Path,"exists") as exists:
                with self.assertRaises(wrapper.GateError) as caught:wrapper.stdlib_inventory(paths,100)
            exists.assert_not_called();self.assertEqual(caught.exception.diagnostic["reason"],"root-shape-invalid")
        with mock.patch.object(wrapper.Path,"exists",return_value=False):
            with self.assertRaises(wrapper.GateError) as caught:wrapper.stdlib_inventory(["/mock/a","/mock/lib-dynload"],100)
        self.assertEqual(caught.exception.diagnostic,{"phase":"stdlib-roots","reason":"missing-root","exit_code":None})
        with mock.patch.object(wrapper.Path,"exists",return_value=True),mock.patch.object(wrapper,"root_trusted",side_effect=lambda path,directory=False:path),mock.patch.object(wrapper.os,"scandir") as scan:
            with self.assertRaises(wrapper.GateError) as caught:wrapper.stdlib_inventory(["/mock/a","/mock/b"],100)
        scan.assert_not_called();self.assertEqual(caught.exception.diagnostic["reason"],"dynload-missing")
        with mock.patch.object(wrapper.Path,"exists",return_value=True),mock.patch.object(wrapper,"root_trusted",side_effect=wrapper.GateError("tool-untrusted",tool_reason="not-root-owned")):
            with self.assertRaises(wrapper.GateError) as caught:wrapper.stdlib_inventory(["/mock/a","/mock/lib-dynload"],100)
        self.assertEqual(caught.exception.diagnostic,{"phase":"stdlib-roots","reason":"not-root-owned","exit_code":None})

    def test_actual_stdlib_entry_predicates_keep_owner_mode_link_kind_priority(self):
        """Apply real streamed entry checks to synthetic metadata and forbid hashing unqualified private paths."""
        cases=((1001,stat.S_IFLNK|0o777,"not-root-owned"),(0,stat.S_IFLNK|0o777,"worker-writable"),
               (0,stat.S_IFLNK|0o755,"unsupported-link"),(0,stat.S_IFIFO|0o600,"unsupported-kind"))
        for uid,mode,reason in cases:
            entry=types.SimpleNamespace(path="/mock/stdlib/private.py",stat=mock.Mock(return_value=types.SimpleNamespace(st_uid=uid,st_mode=mode,st_size=1)))
            with self.inventory_context([entry]) as (scan,hashed):
                with self.assertRaises(wrapper.GateError) as caught:wrapper.stdlib_inventory(["/mock/stdlib","/mock/stdlib/lib-dynload"],100)
            hashed.assert_not_called();entry.stat.assert_called_once_with(follow_symlinks=False)
            self.assertEqual(caught.exception.diagnostic,{"phase":"stdlib-entry","reason":reason,"exit_code":None});self.assertTrue(caught.exception.incomplete)

    def test_actual_stdlib_entry_and_byte_bounds_stop_before_hashing(self):
        """Both configured bounds fail before retaining an over-limit file hash or claiming a qualified inventory."""
        entry=types.SimpleNamespace(path="/mock/stdlib/private.py",stat=mock.Mock(return_value=types.SimpleNamespace(st_uid=0,st_mode=stat.S_IFREG|0o644,st_size=64)))
        with self.inventory_context([entry],max_entries=0) as (scan,hashed):
            with self.assertRaises(wrapper.GateError) as caught:wrapper.stdlib_inventory(["/mock/stdlib","/mock/stdlib/lib-dynload"],100)
        entry.stat.assert_not_called();hashed.assert_not_called();self.assertEqual(caught.exception.diagnostic["reason"],"entry-bound")
        with self.inventory_context([entry],max_bytes=63) as (scan,hashed):
            with self.assertRaises(wrapper.GateError) as caught:wrapper.stdlib_inventory(["/mock/stdlib","/mock/stdlib/lib-dynload"],100)
        hashed.assert_not_called();self.assertEqual(caught.exception.diagnostic,{"phase":"stdlib-entry","reason":"byte-bound","exit_code":None})

    def test_actual_stdlib_depth_and_visibility_faults_are_bounded_redacted(self):
        """Bound synthetic deep recursion and retain unknown directory observations as failed/unverified metadata."""
        def scan(directory):
            """Create only one synthetic descendant per level to reach the exact fixed depth rejection."""
            entry=types.SimpleNamespace(path=str(directory/"deeper"),stat=mock.Mock(return_value=types.SimpleNamespace(st_uid=0,st_mode=stat.S_IFDIR|0o755,st_size=0)))
            return contextlib.nullcontext(iter([entry]))
        with mock.patch.object(wrapper.Path,"exists",return_value=True),mock.patch.object(wrapper,"root_trusted",side_effect=lambda path,directory=False:path), \
             mock.patch.object(wrapper.os,"scandir",side_effect=scan) as scanned,mock.patch.object(wrapper.time,"monotonic",return_value=0):
            with self.assertRaises(wrapper.GateError) as caught:wrapper.stdlib_inventory(["/mock/stdlib","/mock/stdlib/lib-dynload"],100)
        self.assertEqual(scanned.call_count,33);self.assertEqual(caught.exception.diagnostic,{"phase":"stdlib-entry","reason":"depth-bound","exit_code":None})
        with self.inventory_context([],scandir_fault=OSError("PRIVATE directory entry")):
            with self.assertRaises(wrapper.GateError) as caught:wrapper.stdlib_inventory(["/mock/stdlib","/mock/stdlib/lib-dynload"],100)
        self.assertEqual(caught.exception.code,"verification-input-invalid");self.assertFalse(caught.exception.incomplete)
        self.assertEqual(caught.exception.diagnostic,{"phase":"stdlib-entry","reason":"entry-observation-unverified","exit_code":None})

    def test_actual_capture_ordinary_identity_and_final_budget_gates_remain_required(self):
        """A missing ordinary identity and an expired final clock still reject otherwise successful tool observations."""
        with self.capture_context(ordinary={"cargo":None,"rustc":"1.99.0","rust_host":"x86_64-unknown-linux-gnu"}):
            with self.assertRaises(wrapper.GateError) as caught:wrapper.capture_tools(Path("/mock"),100)
        self.assertEqual(caught.exception.diagnostic,{"phase":"ordinary-tools","reason":"required-identity-missing","exit_code":None})
        with self.capture_context(hash_fault=RuntimeError("PRIVATE hash path")):
            with self.assertRaises(wrapper.GateError) as caught:wrapper.capture_tools(Path("/mock"),100)
        self.assertEqual(caught.exception.diagnostic,{"phase":"python-path","reason":"path-observation-unverified","exit_code":None})
        with mock.patch.object(wrapper.time,"monotonic",return_value=10),mock.patch.object(wrapper,"administration_tool") as paths,mock.patch.object(wrapper,"command") as command:
            with self.assertRaises(wrapper.GateError) as caught:wrapper.capture_tools(Path("/mock"),10)
        paths.assert_not_called();command.assert_not_called();self.assertEqual(caught.exception.diagnostic,{"phase":"qualification-budget","reason":"deadline-expired","exit_code":None})
        self.assertEqual((caught.exception.code,caught.exception.incomplete),("command-timeout",True))
        expired=[False]
        def ordinary_finished(*args):
            """Reach the unchanged final deadline using successful synthetic ordinary and version observations."""
            expired[0]=True
            return {"cargo":"1.99.0","rustc":"1.99.0","rust_host":"x86_64-unknown-linux-gnu"}
        with self.capture_context(clock=lambda:100 if expired[0] else 0) as calls:
            calls["ordinary"].side_effect=ordinary_finished
            with self.assertRaises(wrapper.GateError) as caught:wrapper.capture_tools(Path("/mock"),100)
        self.assertEqual(calls["command"].call_count,3)
        self.assertEqual(caught.exception.diagnostic,{"phase":"qualification-budget","reason":"deadline-expired","exit_code":None})
        with self.capture_context() as calls:
            calls["ordinary"].side_effect=RuntimeError("PRIVATE ordinary identity")
            with self.assertRaises(wrapper.GateError) as caught:wrapper.capture_tools(Path("/mock"),100)
        self.assertEqual(caught.exception.code,"verification-input-invalid");self.assertFalse(caught.exception.incomplete)
        self.assertEqual(caught.exception.diagnostic,{"phase":"ordinary-tools","reason":"identity-observation-unverified","exit_code":None})

    def test_retention_keeps_first_safe_diagnostic_and_copies_its_fields(self):
        """Later typed and unclassified failures cannot erase or mutate a retained fixed first diagnostic."""
        first=wrapper.GateError("tool-untrusted",diagnostic=wrapper.tool_diagnostic("python-path","not-root-owned"))
        second=wrapper.GateError("tool-unavailable",True,diagnostic=wrapper.tool_diagnostic("sudo-version","command-failed",-9))
        receipt={"diagnostic":None};wrapper.retain_tool_diagnostic(receipt,RuntimeError("PRIVATE"));self.assertIsNone(receipt["diagnostic"])
        wrapper.retain_tool_diagnostic(receipt,first);expected=dict(first.diagnostic)
        self.assertIsNot(receipt["diagnostic"],first.diagnostic);first.diagnostic["reason"]="PRIVATE"
        wrapper.retain_tool_diagnostic(receipt,second);wrapper.retain_tool_diagnostic(receipt,RuntimeError("PRIVATE"))
        self.assertEqual(receipt["diagnostic"],expected);self.assertNotIn(b"PRIVATE",wrapper.shared.canonical_bytes(receipt))

    def test_actual_outer_pass_and_unrelated_failure_require_null_diagnostic(self):
        """The new outer schema adds only a null slot to old pass and unrelated build-gate outcomes."""
        value,raw,capture,native_run=self.verify_faults()
        self.assertEqual(value["schema_version"],"forge.workspace-os-denial-verification/2");self.assertIsNone(value["diagnostic"])
        self.assertEqual(value["status"],"passed");native_run.assert_called_once();self.assertFalse(value["acceptance_eligible"])
        self.assertEqual(value["attempted_egress"],{"state":"unmeasured","count":None})
        self.assertEqual(wrapper.NATIVE_SCHEMA,"forge.packaged-runtime-os-denial/1")
        value,raw,capture,native_run=self.verify_faults(build="failure")
        native_run.assert_not_called();capture.assert_not_called();self.assertIsNone(value["diagnostic"]);self.assertEqual((value["status"],value["failure"]),("failed","build-not-qualified"))

    def test_actual_outer_tool_failure_never_dispatches_or_publishes_private_output(self):
        """Every fixed component reaches actual outer publication only as unqualified no-dispatch metadata."""
        for phase,reasons in self.phase_pairs().items():
            diagnostic=wrapper.tool_diagnostic(phase,reasons[0])
            code="command-timeout" if phase=="qualification-budget" else "tool-untrusted" if phase in {"stdlib-roots","stdlib-entry"} else "tool-unavailable"
            error=wrapper.GateError(code,True,diagnostic=diagnostic)
            value,raw,capture,native_run=self.verify_faults(first_fault=error)
            native_run.assert_not_called();self.assertEqual(capture.call_count,1)
            self.assertEqual(value["diagnostic"],diagnostic);self.assertEqual((value["status"],value["failure"]),("incomplete",code))
            self.assertIsNone(value["tools"]);self.assertEqual(value["tool_stability"],"unverified")
            self.assertEqual(value["producer"]["status"],"not-run");self.assertEqual(value["cleanup"],{"state":"verified-not-created","forced":False})
            self.assertEqual(value["attempted_egress"],{"state":"unmeasured","count":None});self.assertFalse(value["acceptance_eligible"])
            self.assertEqual(set(value["diagnostic"]),{"phase","reason","exit_code"});self.assertNotIn(b"/mock",wrapper.shared.canonical_bytes(value["diagnostic"]))

    def test_actual_outer_first_diagnostic_survives_later_failure_override(self):
        """Post-check errors still revoke status while the earlier safe diagnostic remains immutable and output-free."""
        first=wrapper.GateError("tool-untrusted",True,diagnostic=wrapper.tool_diagnostic("stdlib-entry","byte-bound"))
        value,raw,capture,native_run=self.verify_faults(first_fault=first,after_identity_fault=RuntimeError("PRIVATE token path"))
        native_run.assert_not_called();self.assertEqual(value["diagnostic"],{"phase":"stdlib-entry","reason":"byte-bound","exit_code":None})
        self.assertEqual((value["status"],value["failure"]),("failed","verification-input-invalid"));self.assertNotIn(b"PRIVATE",raw)
        secondary=wrapper.GateError("tool-unavailable",True,diagnostic=wrapper.tool_diagnostic("ip-version","command-failed",-9))
        value,raw,capture,native_run=self.verify_faults(after_tool_fault=secondary)
        native_run.assert_called_once();self.assertEqual(value["diagnostic"],secondary.diagnostic)
        self.assertEqual((value["status"],value["failure"]),("failed","verification-input-invalid"));self.assertEqual(value["tool_stability"],"unverified")


class FixedAdministrativePathControls(unittest.TestCase):
    """Exercise fixed administrative selection through real capture/verify code, with no tools or native experiment."""

    def test_fixed_administrative_map_rejects_item_override(self):
        """The private three-selector policy has exact defaults and cannot accept an item override."""
        expected = {"python": "/usr/bin/python3", "ip": "/usr/bin/ip", "sudo": "/usr/bin/sudo"}
        self.assertEqual(dict(wrapper._ADMINISTRATIVE_PATHS), expected)
        for name in expected:
            with self.subTest(name=name), self.assertRaises(TypeError):
                wrapper._ADMINISTRATIVE_PATHS[name] = "/PRIVATE/attacker/tool"
        self.assertEqual(dict(wrapper._ADMINISTRATIVE_PATHS), expected)

    def test_actual_capture_ignores_admin_path_and_binds_qualified_targets(self):
        """Fixed selectors ignore hostile PATH while versions/hashes use qualified leaves and ordinary identity stays separate."""
        root = Path("/mock/root")
        selectors = [Path("/usr/bin/python3"), Path("/usr/bin/ip"), Path("/usr/bin/sudo")]
        targets = {selectors[0]: Path("/usr/bin/python3.12"),
                   selectors[1]: Path("/usr/libexec/qualified-ip"),
                   selectors[2]: Path("/usr/libexec/qualified-sudo")}
        pins = {targets[path]: {"bytes": index + 4, "sha256": character * 64}
                for index, (path, character) in enumerate(zip(selectors, "abc"))}
        ordinary = {"cargo": "1.99.0", "rustc": "1.99.0", "rust_host": "x86_64-unknown-linux-gnu"}
        with ToolDiagnosticControls.capture_context(self, ordinary=ordinary) as calls, \
             mock.patch.dict(wrapper.os.environ, {"PATH": "/PRIVATE/attacker:/bin:/sbin"}), \
             mock.patch.object(wrapper.shutil, "which", return_value="/PRIVATE/attacker/tool") as discovery:
            calls["paths"].side_effect = lambda path: targets[path]
            calls["hash"].side_effect = lambda path: pins[path]
            tools, paths = wrapper.capture_tools(root, 100)
            discovery.assert_not_called()
            self.assertEqual(calls["paths"].call_args_list, [mock.call(path) for path in selectors])
            self.assertEqual(paths, {"python": targets[selectors[0]], "ip": targets[selectors[1]], "sudo": targets[selectors[2]]})
            self.assertEqual(calls["command"].call_args_list,
                             [mock.call([str(targets[selectors[0]]), "-I", "-S", "-B", "-c", wrapper.PYTHON_PROBE], root, 10),
                              mock.call([str(targets[selectors[1]]), "-V"], root, 10),
                              mock.call([str(targets[selectors[2]]), "--version"], root, 10)])
            self.assertEqual(calls["hash"].call_args_list, [mock.call(targets[path]) for path in selectors])
            calls["ordinary"].assert_called_once_with(root, 10)
            calls["inventory"].assert_called_once_with(["/mock/stdlib", "/mock/stdlib/lib-dynload"], 100)
        self.assertEqual(tools["ordinary"], ordinary)
        self.assertEqual(tools["privileged_python"], {"pin": pins[targets[selectors[0]]], "version": "3.11.9",
                                                     "root_trust": True, "stdlib_pin": plan()["release_pin"], "stdlib_entries": 10})
        self.assertEqual(tools["ip"], {"pin": pins[targets[selectors[1]]], "version": "6.17.0", "root_trust": True})
        self.assertEqual(tools["sudo"], {"pin": pins[targets[selectors[2]]], "version": "1.9.15p5", "root_trust": True})

    def test_actual_verify_missing_fixed_ip_or_sudo_never_dispatches(self):
        """Actual missing-file predicates reach incomplete/not-run publication despite an available hostile PATH alternative."""
        selectors = [Path("/usr/bin/python3"), Path("/usr/bin/ip"), Path("/usr/bin/sudo")]
        actual_administration = wrapper.administration_tool
        original_exists, original_lstat, original_resolve = wrapper.Path.exists, wrapper.Path.lstat, wrapper.Path.resolve
        for component, missing in (("ip", selectors[1]), ("sudo", selectors[2])):
            seen = []
            def exists(path):
                """Model only the three selector files and delegate private publication paths to ordinary temporary storage."""
                if path in selectors:
                    seen.append(path)
                    return path != missing
                return original_exists(path)
            def metadata(path):
                """Supply trusted synthetic executable/ancestor records without observing the host's tool tree."""
                if path in selectors:
                    return types.SimpleNamespace(st_uid=0, st_mode=stat.S_IFREG | 0o755)
                if path in (Path("/"), Path("/usr"), Path("/usr/bin")):
                    return types.SimpleNamespace(st_uid=0, st_mode=stat.S_IFDIR | 0o755)
                return original_lstat(path)
            def resolve(path, *args, **kwargs):
                """Keep synthetic selectors canonical while retaining normal private-output path behavior."""
                return path if path in selectors else original_resolve(path, *args, **kwargs)
            with self.subTest(component=component), tempfile.TemporaryDirectory() as private, \
                 ToolDiagnosticControls.capture_context(self) as calls, \
                 mock.patch.dict(wrapper.os.environ, {"PATH": "/PRIVATE/attacker:/bin:/sbin"}), \
                 mock.patch.object(wrapper.shutil, "which", return_value="/PRIVATE/attacker/" + component) as discovery, \
                 mock.patch.object(wrapper.Path, "exists", new=exists), \
                 mock.patch.object(wrapper.Path, "lstat", new=metadata), \
                 mock.patch.object(wrapper.Path, "resolve", new=resolve), \
                 mock.patch.object(wrapper.os, "access", return_value=True), \
                 mock.patch.object(wrapper, "capture_identity", side_effect=[{"tracked_source_clean": True}, {"tracked_source_clean": True}]), \
                 mock.patch.object(wrapper.shared, "checkout_binding", return_value={}), \
                 mock.patch.object(wrapper, "native_run") as dispatch:
                calls["paths"].side_effect = actual_administration
                output = Path(private) / "output"
                value = wrapper.verify(Path(private), Path(private) / "forge", output, build_outcome="success")
                raw = (output / wrapper.OUTPUT).read_bytes()
                self.assertEqual(raw, wrapper.shared.canonical_bytes(value))
                expected_order = selectors[:selectors.index(missing) + 1]
                self.assertEqual(calls["paths"].call_args_list, [mock.call(path) for path in expected_order])
                self.assertEqual(seen, expected_order)
                discovery.assert_not_called()
                dispatch.assert_not_called()
                calls["command"].assert_not_called()
                calls["inventory"].assert_not_called()
                calls["ordinary"].assert_not_called()
                calls["hash"].assert_not_called()
            self.assertEqual((value["status"], value["failure"]), ("incomplete", "tool-unavailable"))
            self.assertEqual(value["diagnostic"], {"phase": component + "-path", "reason": "missing", "exit_code": None})
            self.assertEqual(value["producer"], {"status": "not-run", "exit_code": None, "failure": None, "receipt": None, "receipt_pin": None})
            self.assertEqual(value["cleanup"], {"state": "verified-not-created", "forced": False})
            self.assertEqual(value["input_stability"], "unchanged")
            self.assertEqual(value["tool_stability"], "unverified")
            self.assertIsNone(value["tools"])
            self.assertEqual(value["attempted_egress"], {"state": "unmeasured", "count": None})
            self.assertFalse(value["acceptance_eligible"])
            self.assertNotIn(b"/PRIVATE", raw)
            self.assertNotIn(b"/usr/bin", raw)

    def test_fixed_ip_leaf_still_rejects_untrusted_ancestor_with_original_priority(self):
        """Actual fixed ip qualification follows the allowed leaf but rejects its ancestor by UID, mode, then link priority."""
        actual_administration = wrapper.administration_tool
        target = Path("/usr/libexec/qualified-ip")
        good_file = types.SimpleNamespace(st_uid=0, st_mode=stat.S_IFREG | 0o755)
        good_directory = types.SimpleNamespace(st_uid=0, st_mode=stat.S_IFDIR | 0o755)
        ip_link = types.SimpleNamespace(st_uid=0, st_mode=stat.S_IFLNK | 0o777)
        cases = ((types.SimpleNamespace(st_uid=1001, st_mode=stat.S_IFLNK | 0o777), "not-root-owned"),
                 (types.SimpleNamespace(st_uid=0, st_mode=stat.S_IFLNK | 0o777), "worker-writable"),
                 (types.SimpleNamespace(st_uid=0, st_mode=stat.S_IFLNK | 0o755), "unsupported-link"),
                 (types.SimpleNamespace(st_uid=0, st_mode=stat.S_IFDIR | 0o777), "worker-writable"))
        for bad_ancestor, reason in cases:
            def metadata(path):
                """Expose the ip-only target ancestor fault while the preceding fixed Python selector remains trusted."""
                if path == Path("/usr/libexec"):
                    return bad_ancestor
                if path == Path("/usr/bin/ip"):
                    return ip_link
                return good_file if path == Path("/usr/bin/python3") or path == target else good_directory
            with self.subTest(reason=reason), ToolDiagnosticControls.capture_context(self) as calls, \
                 mock.patch.object(wrapper.Path, "exists", return_value=True), \
                 mock.patch.object(wrapper.Path, "lstat", new=metadata), \
                 mock.patch.object(wrapper.Path, "resolve", new=lambda path, **kwargs: path), \
                 mock.patch.object(wrapper.os, "readlink", return_value=str(target)) as links, \
                 mock.patch.object(wrapper.os, "access", return_value=True), \
                 mock.patch.object(wrapper.shutil, "which", side_effect=AssertionError("administrative discovery forbidden")) as discovery:
                calls["paths"].side_effect = actual_administration
                with self.assertRaises(wrapper.GateError) as caught:
                    wrapper.capture_tools(Path("/mock/root"), 100)
                self.assertEqual(calls["paths"].call_args_list, [mock.call(Path("/usr/bin/python3")), mock.call(Path("/usr/bin/ip"))])
                links.assert_called_once_with(Path("/usr/bin/ip"))
                discovery.assert_not_called()
                calls["command"].assert_not_called()
                calls["inventory"].assert_not_called()
                calls["ordinary"].assert_not_called()
            self.assertEqual((caught.exception.code, caught.exception.incomplete), ("tool-untrusted", False))
            self.assertEqual(caught.exception.diagnostic, {"phase": "ip-path", "reason": reason, "exit_code": None})
            self.assertNotIn(b"/usr", wrapper.shared.canonical_bytes(caught.exception.diagnostic))

    def test_fixed_capture_keeps_version_and_hash_failures_closed(self):
        """Fixed names cannot qualify malformed versions or an unreadable canonical tool hash."""
        with ToolDiagnosticControls.capture_context(self) as calls, \
             mock.patch.object(wrapper.shutil, "which", side_effect=AssertionError("administrative discovery forbidden")) as discovery:
            calls["command"].side_effect = [
                {"exit_code": 0, "failure": None, "output": wrapper.shared.canonical_bytes({"version": "3.11.9", "paths": ["/mock/stdlib", "/mock/stdlib/lib-dynload"]})},
                {"exit_code": 0, "failure": None, "output": b"PRIVATE malformed ip version\n"}]
            with self.assertRaises(wrapper.GateError) as caught:
                wrapper.capture_tools(Path("/mock/root"), 100)
            discovery.assert_not_called()
            self.assertEqual(calls["command"].call_count, 2)
            self.assertEqual(calls["command"].call_args_list[-1].args[0], ["/usr/bin/ip", "-V"])
            self.assertEqual(calls["hash"].call_args_list, [mock.call(Path("/usr/bin/python3"))])
        self.assertEqual((caught.exception.code, caught.exception.incomplete), ("tool-unavailable", True))
        self.assertEqual(caught.exception.diagnostic, {"phase": "ip-version", "reason": "version-invalid", "exit_code": 0})
        def hash_tool(path):
            """Fail the actual fixed ip hashing step after successful version parsing, without returning private text."""
            if path == Path("/usr/bin/ip"):
                raise OSError("PRIVATE unreadable canonical ip hash")
            return plan()["release_pin"]
        with ToolDiagnosticControls.capture_context(self) as calls:
            calls["hash"].side_effect = hash_tool
            with self.assertRaises(wrapper.GateError) as caught:
                wrapper.capture_tools(Path("/mock/root"), 100)
            self.assertEqual(calls["hash"].call_args_list, [mock.call(Path("/usr/bin/python3")), mock.call(Path("/usr/bin/ip"))])
            self.assertEqual(calls["command"].call_count, 2)
        self.assertEqual((caught.exception.code, caught.exception.incomplete), ("verification-input-invalid", False))
        self.assertEqual(caught.exception.diagnostic, {"phase": "ip-version", "reason": "identity-observation-unverified", "exit_code": None})
        self.assertNotIn(b"PRIVATE", wrapper.shared.canonical_bytes(caught.exception.diagnostic))

    def test_fixed_capture_exact_deadline_abstains_before_all_observation(self):
        """An exact expired budget blocks selectors and PATH discovery before any tool or namespace dispatch."""
        with ToolDiagnosticControls.capture_context(self, clock=100) as calls, \
             mock.patch.object(wrapper.shutil, "which", return_value="/PRIVATE/attacker/ip") as discovery, \
             mock.patch.object(wrapper, "native_run") as dispatch:
            with self.assertRaises(wrapper.GateError) as caught:
                wrapper.capture_tools(Path("/mock/root"), 100)
            discovery.assert_not_called()
            dispatch.assert_not_called()
            for name in ("paths", "command", "inventory", "ordinary", "hash"):
                calls[name].assert_not_called()
        self.assertEqual((caught.exception.code, caught.exception.incomplete), ("command-timeout", True))
        self.assertEqual(caught.exception.diagnostic, {"phase": "qualification-budget", "reason": "deadline-expired", "exit_code": None})

    def test_actual_verify_fixed_selectors_still_revoke_changed_canonical_target(self):
        """Two real captures of fixed selectors compare canonical targets and revoke pass even when all hashes/versions match."""
        observed = []
        def qualified_path(path):
            """Keep selectors fixed but model a different qualified ip leaf in the post-experiment observation."""
            observed.append(path)
            if path == Path("/usr/bin/ip") and len(observed) > 3:
                return Path("/usr/libexec/qualified-ip-after")
            return path
        identity = {"tracked_source_clean": True, "mock": "stable"}
        producer = {"status": "passed", "failure": None, "exit_code": 0,
                    "receipt": {"cleanup": {"state": "verified", "forced": False}}, "receipt_pin": None}
        with tempfile.TemporaryDirectory() as private, ToolDiagnosticControls.capture_context(self) as calls, \
             mock.patch.object(wrapper.shutil, "which", side_effect=AssertionError("administrative discovery forbidden")) as discovery, \
             mock.patch.object(wrapper, "capture_identity", side_effect=[identity, dict(identity)]), \
             mock.patch.object(wrapper.shared, "checkout_binding", return_value={}), \
             mock.patch.object(wrapper, "native_run", return_value=producer) as dispatch:
            calls["paths"].side_effect = qualified_path
            calls["command"].side_effect = [
                {"exit_code": 0, "failure": None, "output": wrapper.shared.canonical_bytes({"version": "3.11.9", "paths": ["/mock/stdlib", "/mock/stdlib/lib-dynload"]})},
                {"exit_code": 0, "failure": None, "output": b"ip utility, iproute2-6.17.0\n"},
                {"exit_code": 0, "failure": None, "output": b"Sudo version 1.9.15p5\n"}] * 2
            output = Path(private) / "output"
            value = wrapper.verify(Path(private), Path(private) / "forge", output, build_outcome="success")
            raw = (output / wrapper.OUTPUT).read_bytes()
            self.assertEqual(raw, wrapper.shared.canonical_bytes(value))
            discovery.assert_not_called()
            dispatch.assert_called_once()
            self.assertEqual(dispatch.call_args.args[3], {"python": Path("/usr/bin/python3"), "ip": Path("/usr/bin/ip"), "sudo": Path("/usr/bin/sudo")})
            self.assertEqual(observed, [Path("/usr/bin/python3"), Path("/usr/bin/ip"), Path("/usr/bin/sudo")] * 2)
            self.assertEqual(calls["command"].call_count, 6)
            self.assertEqual(calls["ordinary"].call_count, 2)
            self.assertEqual(calls["command"].call_args_list[4].args[0], ["/usr/libexec/qualified-ip-after", "-V"])
            self.assertEqual(calls["hash"].call_args_list[4], mock.call(Path("/usr/libexec/qualified-ip-after")))
        self.assertEqual((value["status"], value["failure"]), ("failed", "tool-identity-changed"))
        self.assertEqual(value["input_stability"], "unchanged")
        self.assertEqual(value["tool_stability"], "changed")
        self.assertEqual(value["cleanup"], {"state": "verified", "forced": False})
        self.assertIsNone(value["diagnostic"])
        self.assertFalse(value["acceptance_eligible"])
        self.assertEqual(value["attempted_egress"], {"state": "unmeasured", "count": None})


if __name__ == "__main__":
    unittest.main()
