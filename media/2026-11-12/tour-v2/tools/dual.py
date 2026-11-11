"""A recorder like `choractl record` for more than one output: the same
requests on the admin socket (record.step, clock.step of every plugin,
record.frame), but one frame for each output. Used for the two-screen
chapter, where `choractl record` takes one output only."""

import base64
import json
import os
import socket


class Admin:
    def __init__(self, main_socket):
        path = os.path.splitext(os.path.realpath(main_socket))[0] + ".admin.sock"
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(path)
        self.file = self.sock.makefile("rwb", buffering=0)
        self.next = 0
        self.call("hello", {"name": "tourrec", "keep_nodes": True, "protocol": 1})

    def call(self, method, params=None):
        self.next += 1
        want = self.next
        line = {"jsonrpc": "2.0", "id": want, "method": method, "params": params if params is not None else None}
        self.file.write((json.dumps(line) + "\n").encode())
        while True:
            raw = self.file.readline()
            if not raw:
                raise RuntimeError("admin socket closed")
            msg = json.loads(raw)
            if msg.get("id") == want:
                if "error" in msg and msg["error"]:
                    raise RuntimeError(f"{method}: {msg['error']}")
                return msg.get("result")

    def close(self):
        try:
            self.sock.close()
        except OSError:
            pass


def record(main_socket, fps, count, outputs, out_dirs, cues, pointer="arrow"):
    """cues: list of (frame, method, params), frame counted from 1."""
    admin = Admin(main_socket)
    try:
        methods = admin.call("methods.list", None) or []
        clocks = [m["client"] for m in methods if m.get("name") == "clock.step"]

        def step_clocks(ms):
            for name in clocks:
                admin.call("methods.call", {"client": name, "method": "clock.step", "params": {"ms": ms}})

        def step():
            s = admin.call("record.step", {"fps": fps})
            step_clocks(s.get("time_ms"))

        pending = sorted(cues, key=lambda c: c[0])
        index = 0
        step()
        try:
            for frame in range(1, count + 1):
                while index < len(pending) and pending[index][0] <= frame:
                    _, method, params = pending[index]
                    admin.call(method, params)
                    index += 1
                step()
                for output, directory in zip(outputs, out_dirs):
                    picture = admin.call("record.frame", {"output": output, "pointer": pointer})
                    with open(os.path.join(directory, f"frame-{frame:05d}.png"), "wb") as f:
                        f.write(base64.b64decode(picture["png"]))
        finally:
            step_clocks(None)
            admin.call("record.stop", None)
    finally:
        admin.close()
