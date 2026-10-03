#!/usr/bin/env python3
"""Maintained local API client; no browser bridge and no persisted capability.

Import Workspace for scripted workflows. Run this module for a read-only
summary: python3 scripts/workspace_client.py --forge ./target/debug/forge --project .
"""
import argparse
import http.client
import hashlib
import json
import queue
import re
import subprocess
import threading
import time
from calendar import monthrange
from pathlib import Path
from urllib.parse import quote, urlencode, urlsplit

MAX_RESPONSE = 4 * 1024 * 1024


class WorkspaceError(RuntimeError):
    """Expose the server's bounded typed error without retaining session credentials."""
    def __init__(self, payload):
        """Keep the received error envelope and its safe code/message display."""
        self.payload = payload
        super().__init__(f"{payload.get('code', 'request-failed')}: {payload.get('message', 'Request failed.')}")


class Workspace:
    """Own one loopback workspace process and explicitly negotiated API namespace."""
    def __init__(self, forge, project, read_only=True, *, api_major=1):
        """Launch one major; retain a matching numeric API2 descriptor and Session version."""
        if type(api_major) is not int or api_major not in (1, 2):
            raise ValueError("Select API major 1 or 2 explicitly")
        self._api_major = api_major
        self._contract_version = None
        self._api_prefix = "/api/v" + str(api_major)
        args = [str(Path(forge).resolve()), "workspace", "--project", str(Path(project).resolve()), "--machine-session"]
        if api_major == 2:
            args.extend(("--api-major", "2"))
        if read_only:
            args.append("--read-only")
        self.process = subprocess.Popen(args, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        self._capability = ""
        self._next_request_at = 0.0
        channel = queue.Queue(maxsize=1)
        def read_descriptor():
            """Read only the bounded first stdout line into the private bootstrap channel."""
            channel.put(self.process.stdout.readline(8193))
        threading.Thread(target=read_descriptor, daemon=True).start()
        try:
            raw = channel.get(timeout=20)
            if len(raw) > 8192:
                raise ValueError("Invalid machine descriptor")
            descriptor = json.loads(raw)
            url = urlsplit(descriptor["base_url"])
            if (url.scheme != "http" or url.hostname != "127.0.0.1" or not url.port or url.path not in ("", "/") or url.query or url.fragment or url.username or url.password
                or descriptor["mode"] != "machine" or descriptor["read_only"] is not read_only or str(descriptor["api_version"]).split(".")[0] != str(api_major)
                or not re.fullmatch(r"[0-9a-f]{64}", descriptor["capability"])):
                raise ValueError("Unsupported machine descriptor")
            if api_major == 2:
                required = {"base_url", "api_version", "api_major", "session_id", "capability", "mode", "read_only", "pid"}
                if (not required.issubset(descriptor) or type(descriptor["api_major"]) is not int or descriptor["api_major"] != 2
                    or not isinstance(descriptor["api_version"], str) or len(descriptor["api_version"]) > 64
                    or not re.fullmatch(r"2\.[0-9]+\.[0-9]+", descriptor["api_version"])
                    or type(descriptor["pid"]) is not int or descriptor["pid"] <= 0
                    or not isinstance(descriptor["session_id"], str) or not re.fullmatch(r"sess_[0-9a-z]{8,64}", descriptor["session_id"])):
                    raise ValueError("Unsupported machine descriptor")
            self._port = url.port
            self._capability = descriptor["capability"]
            if api_major == 2:
                session = self.request("GET", self.api_path("/session"))
                if (not isinstance(session, dict) or type(session.get("api_major")) is not int or session["api_major"] != 2
                    or session.get("contract_version") != descriptor["api_version"] or session.get("session_id") != descriptor["session_id"]
                    or session.get("mode") != "machine" or session.get("read_only") is not read_only):
                    raise ValueError("Unsupported selected API session")
                self._contract_version = session["contract_version"]
        except Exception:
            self.close()
            raise RuntimeError("The local workspace could not be started.") from None

    def api_path(self, path):
        """Build an explicitly selected route; never translate another public API-major path."""
        if (not isinstance(path, str) or not path.startswith("/") or path.startswith(("//", "/api/"))
            or "#" in path or any(ord(char) < 32 for char in path)):
            raise ValueError("Use a documented relative API operation path")
        return self._api_prefix + path

    def request(self, method, path, body=None, *, idempotency_key=None, raw=False, raw_json_body=None, expected_media_type=None):
        """Preserve negotiated transport; raw S6 import bytes and JSON downloads use narrow gates."""
        if not path.startswith(self._api_prefix + "/") or "#" in path or any(ord(char) < 32 for char in path):
            raise ValueError("Use a documented route for the selected API major")
        if raw_json_body is not None:
            if (self._api_major != 2 or self._contract_version != "2.2.0" or method != "POST"
                or path != self.api_path("/project/bundle-imports") or body is not None
                or type(raw_json_body) is not bytes or len(raw_json_body) > 1024 * 1024 or not idempotency_key):
                raise ValueError("Unsupported raw metadata import request")
        if expected_media_type is not None and (expected_media_type != "application/json" or not raw
            or method != "GET" or self._api_major != 2 or self._contract_version != "2.2.0"
            or not re.fullmatch(re.escape(self._api_prefix) + r"/project/bundle-exports/op_[0-9a-z]{12,80}/download", path)
            or body is not None or idempotency_key):
            raise ValueError("Unsupported metadata download request")
        # This synchronous client spaces calls below the documented session rate.
        delay = self._next_request_at - time.monotonic()
        if delay > 0:
            time.sleep(delay)
        self._next_request_at = time.monotonic() + 0.06
        headers = {"Authorization": "Bearer " + self._capability, "Accept": "application/json"}
        encoded = None
        if method != "GET":
            headers["Content-Type"] = "application/json"
            encoded = raw_json_body if raw_json_body is not None else json.dumps({} if body is None else body, ensure_ascii=False, allow_nan=False).encode()
            if len(encoded) > 14 * 1024 * 1024:
                raise ValueError("Request exceeds the supported bound")
        if idempotency_key:
            headers["Idempotency-Key"] = idempotency_key
        connection = http.client.HTTPConnection("127.0.0.1", self._port, timeout=30)
        try:
            connection.request(method, path, body=encoded, headers=headers)
            response = connection.getresponse()
            response_bound = 1024 * 1024 if expected_media_type else MAX_RESPONSE
            payload = response.read(response_bound + 1)
            if len(payload) > response_bound:
                raise RuntimeError("Response exceeds the supported bound")
            if not 200 <= response.status < 300:
                raise WorkspaceError(json.loads(payload))
            if expected_media_type and response.getheader("Content-Type", "").split(";", 1)[0].strip().lower() != expected_media_type:
                raise RuntimeError("The committed metadata has an unsupported media type")
            return payload if raw else json.loads(payload)
        finally:
            connection.close()

    def bundle_preview(self):
        """Inspect the negotiated major's complete version-paired index fingerprints.

        Metadata labels, keys, paths and hashes can be sensitive. This returns
        no source bytes, exported file, receipt or approval. An absent index
        raises the server's typed not-found error.
        """
        return self.request("GET", self.api_path("/project/bundle-preview"))

    def verify_bundle(self, bundle):
        """Compare version-paired metadata only with current registered captures.

        The complete encoded request must fit the server's 1 MiB bound.
        On v2, a supplied bundle1 keeps its original index1 meaning even when
        current registrations use index2; extras and index equality are separate.
        Unregistered supplied paths are never opened. Matching bytes establish
        neither current domain validity nor import readiness; retries capture
        current state again and create no effect or replay receipt.
        """
        return self.request("POST", self.api_path("/project/bundle-verifications"), {"bundle": bundle})

    def _s3_resource(self, resource_id):
        """Accept only a registered opaque resource identifier for a selected S3 read."""
        if not isinstance(resource_id, str) or not re.fullmatch(r"res_[0-9a-z]{12,80}", resource_id):
            raise ValueError("Select an exact registered resource identifier")
        return quote(resource_id, safe="")

    def _s3_read(self, path, *, required_date=False, **query):
        """Issue one bounded API2-only GET; never fall back to API1 or create an effect.

        Query values are explicit, encoded once and checked before the existing
        paced transport. Dates are supplied by the caller; no clock default is
        introduced. Each cursor belongs to its unchanged capture/filter/date.
        The server retains whole-domain validation and count authority.
        """
        if self._api_major != 2 or self._contract_version not in ("2.1.0", "2.2.0"):
            raise ValueError("Lifecycle and impact reads require an explicitly negotiated API2 2.1.0 or 2.2.0 session")
        if required_date and query.get("as_of") is None:
            raise ValueError("An explicit as_of date is required")
        values = {}
        for name, value in query.items():
            if value is None:
                continue
            if name == "page_size":
                if type(value) is not int or not 1 <= value <= 200:
                    raise ValueError("Use an integer page_size from 1 to 200")
            elif not isinstance(value, str) or not value or len(value.encode("utf-8")) > (256 if name == "cursor" else 65536):
                raise ValueError("Use nonempty bounded query strings")
            if name == "as_of":
                if not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", value):
                    raise ValueError("Use an explicit YYYY-MM-DD date")
                try:
                    year, month, day = (int(part) for part in value.split("-"))
                    if not 1 <= day <= monthrange(year, month)[1]:
                        raise ValueError("Invalid day")
                except ValueError:
                    raise ValueError("Use an explicit calendar date") from None
            values[name] = value
        suffix = "?" + urlencode(values) if values else ""
        return self.request("GET", self.api_path(path) + suffix)

    def lifecycle_records(self, *, as_of=None, owner=None, state=None, page_size=50, cursor=None):
        """Read recorded inventory; derived status requires a caller-supplied date.

        Counts distinguish registered, matching and unavailable records. A next
        cursor must retain this exact date/filter tuple and page size; a conflict
        requires a new first-page read. Returned declarations are not approval.
        """
        return self._s3_read("/lifecycle/records", as_of=as_of, owner=owner, state=state,
                             page_size=page_size, cursor=cursor)

    def lifecycle_record(self, record_id, *, as_of):
        """Read one registered record's captured status at an explicit date.

        Redacted fingerprints, declared owners and blockers do not authenticate
        actors or authorize a transition. Impact references remain unresolved.
        """
        return self._s3_read("/lifecycle/records/" + self._s3_resource(record_id),
                             required_date=True, as_of=as_of)

    def lifecycle_history(self, record_id, *, page_size=50, cursor=None):
        """Read bounded recorded transition metadata without claiming current freshness."""
        return self._s3_read("/lifecycle/records/" + self._s3_resource(record_id) + "/history",
                             page_size=page_size, cursor=cursor)

    def lifecycle_queue(self, *, as_of, owner=None, page_size=50, cursor=None):
        """Read owner placements at an explicit date, preserving distinct record/group counts."""
        return self._s3_read("/lifecycle/queue", required_date=True, as_of=as_of, owner=owner,
                             page_size=page_size, cursor=cursor)

    def framework_impact_comparisons(self, *, page_size=50, cursor=None):
        """Read declared old/new comparison metadata; inventory is not a computed analysis."""
        return self._s3_read("/framework-impact/comparisons", page_size=page_size, cursor=cursor)

    def framework_impact_comparison(self, comparison_id):
        """Read the captured comparison's complete unfiltered summary and redacted provenance."""
        return self._s3_read("/framework-impact/comparisons/" + self._s3_resource(comparison_id))

    def framework_impact_changes(self, comparison_id, *, change_class=None, page_size=50, cursor=None):
        """Read bounded change rows; matching rows do not replace the complete change count."""
        return self._s3_read("/framework-impact/comparisons/" + self._s3_resource(comparison_id) + "/changes",
                             change_class=change_class, page_size=page_size, cursor=cursor)

    def framework_impact_findings(self, comparison_id, *, group=None, decision_state=None,
                                  policy_source=None, priority=None, owner=None, page_size=50, cursor=None):
        """Read exact AND-filtered findings within one old/new pair and capture context.

        The full summary and gates remain unfiltered; matched finding and emitted
        disposition counts have distinct scopes. No mutation or approval occurs.
        """
        return self._s3_read("/framework-impact/comparisons/" + self._s3_resource(comparison_id) + "/findings",
                             group=group, decision_state=decision_state, policy_source=policy_source,
                             priority=priority, owner=owner, page_size=page_size, cursor=cursor)

    def framework_impact_prior_dispositions(self, comparison_id, *, page_size=50, cursor=None):
        """Read prior-only disposition metadata; limited admission is not current finding coverage."""
        return self._s3_read("/framework-impact/comparisons/" + self._s3_resource(comparison_id) + "/prior-dispositions",
                             page_size=page_size, cursor=cursor)

    def _s6_supported(self):
        """Gate metadata effects to2.2 while preserving numeric-major bootstrap and older reads."""
        if self._api_major != 2 or self._contract_version != "2.2.0":
            raise ValueError("Metadata bundle effects require API2 contract2.2.0")

    def prepare_bundle_export(self, target_path, *, acknowledge_sensitive_metadata, idempotency_key):
        """Prepare an explicit metadata output; returned export work is not a confirmed write."""
        self._s6_supported()
        if acknowledge_sensitive_metadata is not True or not idempotency_key:
            raise ValueError("Sensitive metadata acknowledgment and an idempotency key are required")
        return self.request("POST", self.api_path("/project/bundle-exports"), {
            "target_path": target_path, "acknowledge_sensitive_metadata": True,
        }, idempotency_key=idempotency_key)

    def prepare_bundle_import(self, bundle_bytes, *, target_index_schema_version, acknowledge_index_replacement, idempotency_key):
        """Wrap original bundle bytes for a confirmed index-only replacement, never restoring sources."""
        self._s6_supported()
        if (type(target_index_schema_version) is not int or target_index_schema_version not in (1, 2)
            or acknowledge_index_replacement is not True or not idempotency_key or type(bundle_bytes) is not bytes):
            raise ValueError("Choose a numeric index schema and acknowledge exact index replacement")
        prefix = b'{"bundle":'
        suffix = (',"target_index_schema_version":' + str(target_index_schema_version)
                  + ',"acknowledge_index_replacement":true}').encode("ascii")
        if len(bundle_bytes) > 1024 * 1024 - len(prefix) - len(suffix):
            raise ValueError("Chosen bundle and its complete import wrapper must fit 1MiB")
        return self.request("POST", self.api_path("/project/bundle-imports"),
                            idempotency_key=idempotency_key, raw_json_body=prefix + bundle_bytes + suffix)

    def download_bundle_export(self, operation_id, *, expected_sha256):
        """Return exact committed JSON bytes after typed media, size and expected-hash checks."""
        self._s6_supported()
        if (not isinstance(operation_id, str) or not re.fullmatch(r"op_[0-9a-z]{12,80}", operation_id)
            or not isinstance(expected_sha256, str) or not re.fullmatch(r"[a-f0-9]{64}", expected_sha256)):
            raise ValueError("A prepared export identity and exact SHA256 are required")
        raw = self.request("GET", self.api_path("/project/bundle-exports/" + operation_id + "/download"),
                           raw=True, expected_media_type="application/json")
        if hashlib.sha256(raw).hexdigest() != expected_sha256:
            raise RuntimeError("The committed metadata bytes do not match the prepared hash")
        return raw

    def wait(self, operation, timeout=35):
        """Poll retained IDs in the same negotiated session without restarting an effect."""
        deadline = time.monotonic() + timeout
        while operation["state"] in ("pending", "running"):
            if time.monotonic() >= deadline:
                raise TimeoutError("Query the retained operation before retrying its effect")
            time.sleep(0.1)
            operation = self.request("GET", self.api_path("/operations/" + operation["operation_id"]))
        return operation

    def commit(self, preview, *, confirmed, idempotency_key):
        """Confirm the exact session-owned preview on the selected major, with its original key."""
        if confirmed is not True:
            raise ValueError("Exact preview confirmation is required")
        return self.request("POST", self.api_path("/effects/commits"), {
            "receipt": preview["receipt"]["token"], "observed_version": preview["target_version"], "confirmed": True,
        }, idempotency_key=idempotency_key)

    def close(self):
        """Request selected-major shutdown, then retain the existing owned process cleanup."""
        if getattr(self, "_capability", "") and self.process.poll() is None:
            try:
                self.request("POST", self.api_path("/session/shutdown"), {})
            except (OSError, RuntimeError, ValueError):
                pass
        self._capability = ""
        if self.process.poll() is None:
            try:
                self.process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        if self.process.stdout:
            self.process.stdout.close()

    def __enter__(self):
        """Return this already-negotiated owned session to a context manager."""
        return self

    def __exit__(self, *_):
        """Close the owned session even when the context body fails."""
        self.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forge", required=True)
    parser.add_argument("--project", required=True)
    parser.add_argument("--api-major", type=int, choices=(1, 2), default=1)
    arguments = parser.parse_args()
    with Workspace(arguments.forge, arguments.project, api_major=arguments.api_major) as client:
        print(json.dumps(client.request("GET", client.api_path("/project/summary")), indent=2))
