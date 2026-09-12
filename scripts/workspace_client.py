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
    def __init__(self, payload):
        self.payload = payload
        super().__init__(f"{payload.get('code', 'request-failed')}: {payload.get('message', 'Request failed.')}")


class Workspace:
    def __init__(self, forge, project, read_only=True):
        args = [str(Path(forge).resolve()), "workspace", "--project", str(Path(project).resolve()), "--machine-session"]
        if read_only:
            args.append("--read-only")
        self.process = subprocess.Popen(args, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        self._capability = ""
        self._next_request_at = 0.0
        channel = queue.Queue(maxsize=1)
        def read_descriptor():
            channel.put(self.process.stdout.readline(8193))
        threading.Thread(target=read_descriptor, daemon=True).start()
        try:
            raw = channel.get(timeout=20)
            if len(raw) > 8192:
                raise ValueError("Invalid machine descriptor")
            descriptor = json.loads(raw)
            url = urlsplit(descriptor["base_url"])
            if (url.scheme != "http" or url.hostname != "127.0.0.1" or not url.port or url.path not in ("", "/") or url.query or url.fragment or url.username or url.password
                or descriptor["mode"] != "machine" or descriptor["read_only"] is not read_only or str(descriptor["api_version"]).split(".")[0] != "1"
                or not re.fullmatch(r"[0-9a-f]{64}", descriptor["capability"])):
                raise ValueError("Unsupported machine descriptor")
            self._port = url.port
            self._capability = descriptor["capability"]
        except Exception:
            self.close()
            raise RuntimeError("The local workspace could not be started.") from None

    def request(self, method, path, body=None, *, idempotency_key=None, raw=False):
        if not path.startswith("/api/v1/") or "#" in path or any(ord(char) < 32 for char in path):
            raise ValueError("Use a documented /api/v1/ route")
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

    def wait(self, operation, timeout=35):
        deadline = time.monotonic() + timeout
        while operation["state"] in ("pending", "running"):
            if time.monotonic() >= deadline:
                raise TimeoutError("Query the retained operation before retrying its effect")
            time.sleep(0.1)
            operation = self.request("GET", "/api/v1/operations/" + operation["operation_id"])
        return operation

    def commit(self, preview, *, confirmed, idempotency_key):
        if confirmed is not True:
            raise ValueError("Exact preview confirmation is required")
        return self.request("POST", "/api/v1/effects/commits", {
            "receipt": preview["receipt"]["token"], "observed_version": preview["target_version"], "confirmed": True,
        }, idempotency_key=idempotency_key)

    def close(self):
        if getattr(self, "_capability", "") and self.process.poll() is None:
            try:
                self.request("POST", "/api/v1/session/shutdown", {})
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
        return self

    def __exit__(self, *_):
        self.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forge", required=True)
    parser.add_argument("--project", required=True)
    arguments = parser.parse_args()
    with Workspace(arguments.forge, arguments.project) as client:
        print(json.dumps(client.request("GET", "/api/v1/project/summary"), indent=2))
