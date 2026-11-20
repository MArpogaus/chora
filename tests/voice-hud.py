#!/usr/bin/env python3
"""A HUD stand-in for tests/voice-acp.sh (docs/voice.md sections 5 and 18).

A question of a sub-agent that edits a file takes a key or a HUD button;
a spoken yes or `voice confirm` counts as nothing. This client connects to
CHORA_SOCKET as `assistant` (a name of `indicator_clients`), claims the
indicator, waits for the event `voice.confirm` of a question, shows it
for more than the 400 ms that a button needs, and answers it with
`confirm.answer {id, yes}` on the same connection that claimed.

A button needs the question to show for 400 ms, and a question may be
replaced meanwhile, so the client answers what shows after a pause.

Usage: voice-hud.py yes|no GOFILE   (starts before the question, answers once GOFILE
exists, prints `answered ID` and exits)
"""

import json
import os
import queue
import socket
import sys
import threading
import time

yes = sys.argv[1] == "yes"
go = sys.argv[2]
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect(os.environ["CHORA_SOCKET"])
inbox = queue.Queue()
threading.Thread(
    target=lambda: [inbox.put(json.loads(line)) for line in sock.makefile("r", encoding="utf-8")],
    daemon=True,
).start()
next_id = 0
shown = {}


def note(message):
    """Remember the open question of a voice.confirm event."""
    params = message.get("params") or {}
    if message.get("method") == "event" and params.get("event") == "voice.confirm":
        data = params.get("data") or {}
        if data.get("done"):
            shown.pop("id", None)
        elif "id" in data:
            shown["id"] = data["id"]


def next_message(timeout):
    """The next message, or None after the timeout."""
    try:
        return inbox.get(timeout=timeout)
    except queue.Empty:
        return None


def call(method, params):
    """Send one request and return its result; events that come first are kept."""
    global next_id
    next_id += 1
    wanted = next_id
    sock.sendall((json.dumps({"jsonrpc": "2.0", "id": wanted, "method": method, "params": params}) + "\n").encode())
    while True:
        message = next_message(30)
        if message is None:
            sys.exit(f"voice-hud: {method}: no answer")
        if message.get("id") == wanted:
            if "error" in message:
                sys.exit(f"voice-hud: {method}: {message['error']}")
            return message.get("result")
        note(message)


call("hello", {"name": "assistant"})
call("events.subscribe", {"events": ["voice.confirm"]})
call("methods.call", {"client": "voice", "method": "indicator.claim", "params": {}})
deadline = time.time() + 90
while time.time() < deadline:
    note(next_message(0.1) or {})
    if "id" in shown and os.path.exists(go):
        time.sleep(1.2)
        while (message := next_message(0.05)) is not None:
            note(message)
        asked = shown["id"]
        call("methods.call", {"client": "voice", "method": "confirm.answer", "params": {"id": str(asked), "yes": yes}})
        print(f"answered {asked}")
        sys.exit(0)
sys.exit("voice-hud: no question")
