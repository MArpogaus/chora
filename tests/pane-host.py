#!/usr/bin/env python3
"""A pane host for the gates of virtual outputs (D69).

It does what a construct does with window managers in panes. It adds one
virtual output per pane, shows each in a shader node with `forward` at a
place of the first screen, in the overlay of that screen so that no window
covers it, and holds its connection until it gets a signal: the virtual
outputs live as long as their owner.

Each PANE is `WxH+X+Y`: the size of the output and of the pane, and the
place of the pane on the screen. The panes show their outputs at 1:1, so
a point of the output at (x, y) is on the screen at (X + x, Y + y).

It prints one line per pane, `NAME X Y W H`, then `ready`.

Usage: pane-host.py SCREEN PANE...   (CHORA_SOCKET, XDG_RUNTIME_DIR)
"""

import json
import os
import re
import signal
import socket
import sys

SHADER = (
    "precision mediump float;\n"
    "uniform sampler2D face;\n"
    "varying vec2 v_uv;\n"
    "void main() { gl_FragColor = texture2D(face, v_uv); }\n"
)


class Core:
    """A connection to the core that calls methods and skips events."""

    def __init__(self, path, name):
        self.sock = socket.socket(socket.AF_UNIX)
        self.sock.connect(path)
        self.lines = self.sock.makefile("r")
        self.next = 0
        self.call("hello", {"name": name})

    def call(self, method, params):
        """Call a method and return its result, or exit with its error."""
        self.next += 1
        request = {"jsonrpc": "2.0", "id": self.next, "method": method}
        request["params"] = params
        self.sock.sendall((json.dumps(request) + "\n").encode())
        while True:
            answer = json.loads(self.lines.readline())
            if answer.get("id") != self.next:
                continue
            if "error" in answer:
                sys.exit(f"{method}: {answer['error']}")
            return answer["result"]


def pane(core, screen, shader, spec):
    """Add a virtual output and show it in a pane; return its line."""
    width, height, x, y = map(int, re.fullmatch(r"(\d+)x(\d+)\+(\d+)\+(\d+)", spec).groups())
    output = core.call("output.add", {"spec": f"{width}x{height}", "virtual": True})
    name = output["name"]
    props = {
        "size": [width, height],
        "shader": shader,
        "forward": True,
        "uniforms": {"face": {"output": name}},
        "transform": {"translate": [x, y]},
    }
    node = {"id": f"pane-{name}", "kind": "shader", "parent": f"screen:{screen}"}
    node["props"] = props
    core.call("scene.apply", {"commands": [{"create": node}]})
    return f"{name} {x} {y} {width} {height}"


def main():
    """Add the outputs, show the panes and wait for a signal."""
    screen, specs = sys.argv[1], sys.argv[2:]
    core = Core(os.environ["CHORA_SOCKET"], "pane-host")
    shader = os.path.join(os.environ["XDG_RUNTIME_DIR"], "pane-host.frag")
    with open(shader, "w", encoding="utf-8") as out:
        out.write(SHADER)
    for spec in specs:
        print(pane(core, screen, shader, spec), flush=True)
    print("ready", flush=True)
    signal.sigwait({signal.SIGTERM, signal.SIGINT})


if __name__ == "__main__":
    main()
