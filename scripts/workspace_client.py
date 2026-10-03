#!/usr/bin/env python3
"""Maintained local API client; no browser bridge and no persisted capability.

Import Workspace for scripted workflows. Run this module for a read-only
summary: python3 scripts/workspace_client.py --forge ./target/debug/forge --project .
"""
import argparse
import http.client
import json
import queue
import re
import subprocess
import threading
import time
from pathlib import Path
from urllib.parse import urlsplit

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
        """Launch the requested single API major; default v1 arguments and behavior remain unchanged."""
        if type(api_major) is not int or api_major not in (1, 2):
            raise ValueError("Select API major 1 or 2 explicitly")
        self._api_major = api_major
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
        except Exception:
            self.close()
            raise RuntimeError("The local workspace could not be started.") from None

    def api_path(self, path):
        """Build an explicitly selected route; never translate another public API-major path."""
        if (not isinstance(path, str) or not path.startswith("/") or path.startswith(("//", "/api/"))
            or "#" in path or any(ord(char) < 32 for char in path)):
            raise ValueError("Use a documented relative API operation path")
        return self._api_prefix + path

    def request(self, method, path, body=None, *, idempotency_key=None, raw=False):
        """Send only the negotiated prefix, preserving bounds, pacing and typed errors."""
        if not path.startswith(self._api_prefix + "/") or "#" in path or any(ord(char) < 32 for char in path):
            raise ValueError("Use a documented route for the selected API major")
        # This synchronous client spaces calls below the documented session rate.
        delay = self._next_request_at - time.monotonic()
        if delay > 0:
            time.sleep(delay)
        self._next_request_at = time.monotonic() + 0.06
        headers = {"Authorization": "Bearer " + self._capability, "Accept": "application/json"}
        encoded = None
        if method != "GET":
            headers["Content-Type"] = "application/json"
            encoded = json.dumps({} if body is None else body, ensure_ascii=False, allow_nan=False).encode()
            if len(encoded) > 14 * 1024 * 1024:
                raise ValueError("Request exceeds the supported bound")
        if idempotency_key:
            headers["Idempotency-Key"] = idempotency_key
        connection = http.client.HTTPConnection("127.0.0.1", self._port, timeout=30)
        try:
            connection.request(method, path, body=encoded, headers=headers)
            response = connection.getresponse()
            payload = response.read(MAX_RESPONSE + 1)
            if len(payload) > MAX_RESPONSE:
                raise RuntimeError("Response exceeds the supported bound")
            if not 200 <= response.status < 300:
                raise WorkspaceError(json.loads(payload))
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
