#!/usr/bin/env python3
"""A fake voice agent for the gates.

It connects to CHORA_SOCKET as the client `voice`, exports
`indicator.claim`, and emits the events of stdin.

Each line of stdin is `EVENT JSON`, for example
`voice.state {"state": "listening"}`; the event goes out with
`events.emit`. The count of the calls of `indicator.claim` goes to the
file of the first argument after each call.
"""

import json
import os
import socket
import sys
import threading

claims_file = sys.argv[1] if len(sys.argv) > 1 else os.devnull
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect(os.environ["CHORA_SOCKET"])
lock = threading.Lock()
next_id = 0


def send(method, params):
    """Send one request; the answers come to serve."""
    global next_id
    with lock:
        next_id += 1
        line = {"jsonrpc": "2.0", "id": next_id, "method": method, "params": params}
        sock.sendall((json.dumps(line) + "\n").encode())


def serve():
    """Answer the calls of indicator.claim and count them."""
    claims = 0
    for line in sock.makefile("r", encoding="utf-8"):
        message = json.loads(line)
        if "error" in message:
            print(f"fake-voice: {message['error']}", file=sys.stderr)
        params = message.get("params") or {}
        if message.get("method") == "event" and params.get("event") == "method.called":
            if params.get("method") == "indicator.claim":
                claims += 1
                with open(claims_file, "w", encoding="utf-8") as f:
                    f.write(f"{claims}\n")
            send("methods.return", {"call": params["call"], "result": {}})
    os._exit(0)


send("hello", {"name": "voice"})
send("methods.export", {"names": ["indicator.claim"]})
threading.Thread(target=serve, daemon=True).start()
for line in sys.stdin:
    event, _, data = line.strip().partition(" ")
    if event:
        send("events.emit", {"event": event, "data": json.loads(data or "null")})
