#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Upload files to a Logos Storage module over Qt Remote Objects (QtRO 2.0).

The storage module is not an HTTP service: it publishes a QtRO source on a Unix
socket at /tmp/logos_storage_module_<hash>. This client connects to that source
with a dynamic replica, authenticates with the module token, and calls
``uploadUrl(filePath, chunkSize, advertise)``.

Usage:
    python3 tools/qtro_storage_upload.py \\
        --token-file /tmp/proclaim_storage_token \\
        --init-json '{"data-dir":"/home/xps/proclaim-storage-data","network":"logos.test"}' \\
        --host-log /path/to/host.log \\
        --out /tmp/cids.json \\
        artifacts/registry-idl.json artifacts/proclaim-source-v5.tar.gz

Authentication: ``callRemoteMethod(authToken, methodName, args)``. The token is
the one given to logos_host via ``--token-source file:<path>`` (or stdin for a
Basecamp-managed instance, in which case you need the capability-module token).

CID capture: QtRO emits ``storageUploadDone`` with JSON containing ``cid``.
Some SDK builds drop that event when the result is serialized in the same event
loop turn as the reply; in that case pass ``--host-log`` and the tool extracts
``manifestCid=`` lines from the host's stderr instead.
"""
from __future__ import annotations

import argparse
import glob
import json
import os
import re
import sys
import time

try:
    from PySide6.QtCore import QCoreApplication, QUrl
    from PySide6.QtRemoteObjects import QRemoteObjectNode, QRemoteObjectReplica
except ImportError as exc:  # pragma: no cover
    sys.exit(
        "PySide6 with QtRemoteObjects is required: pip install PySide6\n"
        f"original error: {exc}"
    )

MANIFEST_RE = re.compile(r"manifestCid=(\S+)")
FILENAME_RE = re.compile(r'filename=some\("([^"]+)"\)')


def find_socket(explicit: str | None) -> str:
    """Return the QtRO local source name (not the full path)."""
    if explicit:
        return os.path.basename(explicit.rstrip("/"))
    candidates = sorted(
        glob.glob("/tmp/logos_storage_module_*"),
        key=lambda p: os.path.getmtime(p) if os.path.exists(p) else 0,
        reverse=True,
    )
    if not candidates:
        sys.exit("no /tmp/logos_storage_module_* socket found; start the module first")
    return os.path.basename(candidates[0])


def wait_valid(app, rep, timeout_ms: int) -> bool:
    deadline = time.time() + timeout_ms / 1000
    while time.time() < deadline:
        app.processEvents()
        if rep.state() == QRemoteObjectReplica.State.Valid and rep.isInitialized():
            return True
        time.sleep(0.02)
    return False


def pump(app, seconds: float) -> None:
    deadline = time.time() + seconds
    while time.time() < deadline:
        app.processEvents()
        time.sleep(0.02)


def log_cids(path: str, start_offset: int) -> dict[str, str]:
    """Parse newly appended manifestCid lines from the host log."""
    found: dict[str, str] = {}
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            fh.seek(start_offset)
            for line in fh:
                cid = MANIFEST_RE.search(line)
                if not cid:
                    continue
                name = FILENAME_RE.search(line)
                found[name.group(1) if name else ""] = cid.group(1)
    except OSError:
        pass
    return found


def json_from_event(payload) -> dict | None:
    text = str(payload)
    try:
        value = json.loads(text)
    except (ValueError, TypeError):
        return None
    return value if isinstance(value, dict) else None


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("files", nargs="+", help="absolute paths to upload")
    ap.add_argument("--socket", help="QtRO socket path/name (default: newest in /tmp)")
    ap.add_argument("--token", help="module auth token")
    ap.add_argument("--token-file", help="read module auth token from this file")
    ap.add_argument("--init-json", help="JSON config passed to init(); start() follows")
    ap.add_argument("--host-log", help="host stderr log, used as CID fallback")
    ap.add_argument("--chunk-size", type=int, default=65536)
    ap.add_argument("--advertise", action=argparse.BooleanOptionalAction, default=True)
    ap.add_argument("--connect-timeout-ms", type=int, default=30000)
    ap.add_argument("--event-timeout-s", type=int, default=60)
    ap.add_argument("--out", help="write result JSON here instead of stdout only")
    args = ap.parse_args()

    token = args.token
    if args.token_file:
        with open(args.token_file, "r", encoding="utf-8") as fh:
            token = fh.read().strip()
    if not token:
        sys.exit("provide --token or --token-file")

    source = find_socket(args.socket)
    app = QCoreApplication([])
    node = QRemoteObjectNode()
    if not node.connectToNode(QUrl("local:" + source)):
        sys.exit(f"failed to connect to local:{source}")
    rep = node.acquireDynamic("storage_module")
    if not wait_valid(app, rep, args.connect_timeout_ms):
        sys.exit(f"replica for storage_module at {source} never became valid")

    events: list[tuple[str, dict | None]] = []

    def on_event(name, data):
        payload = data[0] if data else None
        events.append((str(name), json_from_event(payload)))
        print(f"[event] {name}: {payload}", file=sys.stderr)

    rep.eventResponse.connect(on_event)

    crm = getattr(rep, "callRemoteMethod")

    def call(method: str, call_args: list) -> None:
        print(f"[call] {method}({call_args})", file=sys.stderr)
        try:
            crm(token, method, call_args)
        except Exception as exc:  # PySide6 may hide the pending call result
            print(f"[warn] {method}: {exc}", file=sys.stderr)

    if args.init_json:
        call("init", [args.init_json])
        call("start", [])
        deadline = time.time() + args.event_timeout_s
        while time.time() < deadline and not any(n == "storageStart" for n, _ in events):
            pump(app, 0.5)

    log_offset = 0
    if args.host_log and os.path.exists(args.host_log):
        log_offset = os.path.getsize(args.host_log)

    done: dict[str, str] = {}
    for path in args.files:
        abs_path = os.path.abspath(path)
        if not os.path.exists(abs_path):
            sys.exit(f"file not found: {abs_path}")
        before = len(events)
        call("uploadUrl", [abs_path, args.chunk_size, args.advertise])
        deadline = time.time() + args.event_timeout_s
        while time.time() < deadline:
            pump(app, 0.5)
            hit = None
            for name, payload in events[before:]:
                if name == "storageUploadDone" and payload and payload.get("success"):
                    hit = payload.get("cid")
            if hit:
                done[os.path.basename(abs_path)] = hit
                break
        if os.path.basename(abs_path) not in done and args.host_log:
            tail = log_cids(args.host_log, log_offset)
            if os.path.basename(abs_path) in tail:
                done[os.path.basename(abs_path)] = tail[os.path.basename(abs_path)]

    result = {
        "socket": source,
        "files": [
            {
                "path": os.path.abspath(p),
                "cid": done.get(os.path.basename(p)),
            }
            for p in args.files
        ],
        "events": [{"name": n, "payload": p} for n, p in events],
    }
    rendered = json.dumps(result, indent=2)
    print(rendered)
    if args.out:
        with open(args.out, "w", encoding="utf-8") as fh:
            fh.write(rendered + "\n")
    return 0 if all(f["cid"] for f in result["files"]) else 3


if __name__ == "__main__":
    # PySide6 dynamic replicas can segfault during interpreter teardown.
    code = main()
    sys.stdout.flush()
    sys.stderr.flush()
    os._exit(code)
