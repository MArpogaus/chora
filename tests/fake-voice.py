#!/usr/bin/env python3
"""A fake voice agent for the gates.

It connects to CHORA_SOCKET as the client `voice`, exports
`indicator.claim` and `confirm.answer`, and emits the events of stdin.

Each line of stdin is `EVENT JSON`, for example
`voice.state {"state": "listening"}`; the event goes out with
`events.emit`. The count of the calls of `indicator.claim` goes to the
file of the first argument after each call; the parameters of each
call of `confirm.answer` go as a line of JSON to that file plus
`.answers`.

FAKE_VOICE_ENTRIES names a JSON file with the transcript of the helper
`atlas`; then the agent also exports `agent`, `agent.read`,
`agent.cancel`, `chat.open` and `chat.close`. `agent.read` answers the
entries after `after`, `agent` answers `agent_list` with one helper, and
every call of these goes as a line of JSON to the file of the first
argument plus `.chat`.

The agent exports `state` too: it answers the data of the last
`voice.state` without a name that it emitted, or FAKE_VOICE_STATE (a
JSON object) before the first, or `{}`.
"""

import json
import os
import socket
import sys
import threading

claims_file = sys.argv[1] if len(sys.argv) > 1 else os.devnull
entries_file = os.environ.get("FAKE_VOICE_ENTRIES")
CHAT_METHODS = ["agent", "agent.read", "agent.cancel", "chat.open", "chat.close"]
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect(os.environ["CHORA_SOCKET"])
lock = threading.Lock()
next_id = 0
state = json.loads(os.environ.get("FAKE_VOICE_STATE", "{}"))


def send(method, params):
    """Send one request; the answers come to serve."""
    global next_id
    with lock:
        next_id += 1
        line = {"jsonrpc": "2.0", "id": next_id, "method": method, "params": params}
        sock.sendall((json.dumps(line) + "\n").encode())


def chat_answer(method, params):
    """The answer of a chat method, and a line of its call in the log."""
    with open(claims_file + ".chat", "a", encoding="utf-8") as f:
        f.write(json.dumps({"method": method, "params": params}, sort_keys=True) + "\n")
    if method == "agent" and params.get("tool") == "agent_list":
        return "projects: demo; agents: atlas (demo, working)"
    if method == "agent":
        return "ok"
    if method == "agent.read":
        with open(entries_file, encoding="utf-8") as f:
            entries = [e for e in json.load(f) if e["seq"] > params.get("after", 0)]
        return {"entries": entries, "seq": entries[-1]["seq"] if entries else params.get("after", 0), "epoch": 1}
    return {}


def serve():
    """Answer the calls of indicator.claim and count them."""
    claims = 0
    for line in sock.makefile("r", encoding="utf-8"):
        message = json.loads(line)
        if "error" in message:
            print(f"fake-voice: {message['error']}", file=sys.stderr)
        params = message.get("params") or {}
        if message.get("method") == "event" and params.get("event") == "input.key" and claims_file != os.devnull:
            # FAKE_VOICE_KEYS=1: the grabbed Alt+y and Alt+n of a physical question
            with open(claims_file + ".keys", "a", encoding="utf-8") as f:
                f.write(json.dumps(params, sort_keys=True) + "\n")
        if message.get("method") == "event" and params.get("event") == "method.called":
            if params.get("method") == "indicator.claim":
                claims += 1
                with open(claims_file, "w", encoding="utf-8") as f:
                    f.write(f"{claims}\n")
            if params.get("method") == "confirm.answer" and claims_file != os.devnull:
                with open(claims_file + ".answers", "a", encoding="utf-8") as f:
                    f.write(json.dumps(params.get("params"), sort_keys=True) + "\n")
            result = {}
            if params.get("method") == "state":
                result = state
            if entries_file and params.get("method") in CHAT_METHODS:
                result = chat_answer(params["method"], params.get("params") or {})
            send("methods.return", {"call": params["call"], "result": result})
    os._exit(0)


# FAKE_VOICE_NAME: another client name, for example `voice.evil` (W15);
# FAKE_VOICE_PRIVILEGED=1: both methods are privileged, as in the agent.
send("hello", {"name": os.environ.get("FAKE_VOICE_NAME", "voice")})
privileged = os.environ.get("FAKE_VOICE_PRIVILEGED") == "1"
send(
    "methods.export",
    {"names": [{"name": n, "privileged": True} if privileged else n for n in ["indicator.claim", "confirm.answer", "state"] + (CHAT_METHODS if entries_file else [])]},
)
if os.environ.get("FAKE_VOICE_KEYS") == "1":
    for binding in ("Alt+y", "Alt+n"):
        send("input.grab", {"binding": binding})
threading.Thread(target=serve, daemon=True).start()
for line in sys.stdin:
    event, _, data = line.strip().partition(" ")
    if event == "voice.state" and "name" not in json.loads(data or "{}"):
        state = json.loads(data)
    if event:
        send("events.emit", {"event": event, "data": json.loads(data or "null")})
