#!/usr/bin/python3
"""The exit test of milestone M4: an MCP client drives chora.

It starts `choractl --name agent mcp`, asks for "describe the screen",
calls `tiling.set_layout`, and asks again. It saves both screenshots as
OUT_DIR/before.png and OUT_DIR/after.png and both descriptions as JSON, for
the checks of tests/agent.sh. Then it subscribes to events, clicks a rect
of its own and presses a key that it grabbed, and waits for both (#7). It
puts a shader as an asset, draws it as a shader node and saves a capture
of the node as OUT_DIR/asset.png and the path as OUT_DIR/asset-path.txt
(#6).

Usage: tests/mcp-client.py CHORACTL OUT_DIR LAYOUT
"""

import base64
import json
import subprocess
import sys


class Mcp:
    """A client of an MCP server over stdio."""

    def __init__(self, command):
        self.server = subprocess.Popen(
            command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True
        )
        self.next = 1

    def send(self, message):
        self.server.stdin.write(json.dumps(message) + "\n")
        self.server.stdin.flush()

    def request(self, method, params=None):
        number = self.next
        self.next += 1
        self.send({"jsonrpc": "2.0", "id": number, "method": method, "params": params or {}})
        line = self.server.stdout.readline()
        if not line:
            sys.exit(f"the server closed during {method}")
        answer = json.loads(line)
        if answer.get("id") != number or "error" in answer:
            sys.exit(f"{method}: {answer}")
        return answer["result"]

    def tool(self, name, arguments):
        result = self.request("tools/call", {"name": name, "arguments": arguments})
        if result.get("isError"):
            sys.exit(f"{name}: {result['content']}")
        return result["content"]

    def close(self):
        self.server.stdin.close()
        self.server.wait(timeout=10)


def describe(mcp, out, name):
    """Saves the screenshot and the description of describe_screen."""
    content = mcp.tool("describe_screen", {"max_side": 1280})
    images = [c for c in content if c["type"] == "image"]
    texts = [c for c in content if c["type"] == "text"]
    if len(images) != 1 or images[0]["mimeType"] != "image/png" or len(texts) != 1:
        sys.exit(f"describe_screen gave {[c['type'] for c in content]}")
    with open(f"{out}/{name}.png", "wb") as f:
        f.write(base64.b64decode(images[0]["data"]))
    with open(f"{out}/{name}.json", "w") as f:
        f.write(texts[0]["text"])
    return json.loads(texts[0]["text"])


def wait_for(mcp, wanted):
    """Waits for events until one of them matches; returns all of them."""
    seen = []
    for _ in range(10):
        content = mcp.tool("wait_events", {"timeout_ms": 1000})
        batch = json.loads(content[0]["text"])
        if batch["ended"]:
            sys.exit(f"wait_events: {batch}")
        seen += batch["events"]
        if any(wanted(e) for e in seen):
            return seen
    sys.exit(f"no such event in {seen}")


def events(mcp):
    """A click on a rect and a grabbed key arrive through wait_events."""
    try:
        mcp.tool("wait_events", {})
        sys.exit("wait_events worked before subscribe_events")
    except SystemExit as err:
        if "subscribe_events first" not in str(err):
            raise
    content = mcp.tool(
        "subscribe_events",
        {"events": ["pointer.pressed", "input.key"], "keys": ["Super+F9"]},
    )
    subscribed = json.loads(content[0]["text"])
    if subscribed["keys"] != ["Super+F9"]:
        sys.exit(f"subscribe_events: {subscribed}")
    mcp.tool(
        "apply_commands",
        {
            "commands": [
                {
                    "create": {
                        "id": "mcp-button",
                        "kind": "rect",
                        "parent": "root",
                        "props": {
                            "size": [80, 40],
                            "color": [0, 1, 0, 1],
                            "input": "accept",
                            "transform": {"translate": [1100, 700]},
                        },
                    }
                }
            ]
        },
    )
    mcp.tool(
        "inject_input",
        {"events": [{"pointer_move": {"position": [1140, 720]}}, {"click": {}}]},
    )
    seen = wait_for(
        mcp, lambda e: e["event"] == "pointer.pressed" and e["node"] == "mcp-button"
    )
    if any(e["event"] not in ("pointer.pressed", "input.key") for e in seen):
        sys.exit(f"unsubscribed events: {seen}")
    mcp.tool("inject_input", {"events": [{"shortcut": {"keys": "Super+F9"}}]})
    wait_for(mcp, lambda e: e == {"event": "input.key", "binding": "Super+F9"})
    mcp.tool("apply_commands", {"commands": [{"remove": {"id": "mcp-button"}}]})


MAGENTA = """precision mediump float;
uniform float alpha;
varying vec2 v_uv;
void main() { gl_FragColor = vec4(1.0, 0.0, 1.0, 1.0) * alpha; }
"""


def asset(mcp, out):
    """A shader of the client, which chora cannot read, draws a node."""
    content = mcp.tool("put_asset", {"name": "magenta.frag", "text": MAGENTA})
    path = json.loads(content[0]["text"])["path"]
    mcp.tool(
        "apply_commands",
        {
            "commands": [
                {
                    "create": {
                        "id": "mcp-shader",
                        "kind": "shader",
                        "parent": "root",
                        "props": {"size": [100, 100], "shader": path},
                    }
                }
            ]
        },
    )
    content = mcp.tool("capture", {"node": "mcp-shader"})
    images = [c for c in content if c["type"] == "image"]
    with open(f"{out}/asset.png", "wb") as f:
        f.write(base64.b64decode(images[0]["data"]))
    with open(f"{out}/asset-path.txt", "w") as f:
        f.write(path)
    mcp.tool("apply_commands", {"commands": [{"remove": {"id": "mcp-shader"}}]})


def main():
    choractl, out, layout = sys.argv[1:4]
    mcp = Mcp([choractl, "--name", "agent", "mcp"])
    info = mcp.request(
        "initialize",
        {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "chora-e2e", "version": "1"},
        },
    )
    if info["protocolVersion"] != "2025-06-18":
        sys.exit(f"initialize: {info}")
    mcp.send({"jsonrpc": "2.0", "method": "notifications/initialized"})
    names = [t["name"] for t in mcp.request("tools/list")["tools"]]
    for wanted in ["describe_screen", "call_method"]:
        if wanted not in names:
            sys.exit(f"no tool {wanted} in {names}")
    before = describe(mcp, out, "before")
    tiling = [p for p in before["plugins"] if p["name"] == "tiling"]
    if not tiling or "set_layout" not in tiling[0]["methods"]:
        sys.exit(f"tiling exports no set_layout: {before['plugins']}")
    result = mcp.tool(
        "call_method",
        {"client": "tiling", "method": "set_layout", "params": {"layout": layout}},
    )
    if json.loads(result[0]["text"])["layout"] != layout:
        sys.exit(f"set_layout: {result}")
    describe(mcp, out, "after")
    events(mcp)
    asset(mcp, out)
    mcp.close()
    print("the MCP client saw the screen before and after set_layout")
    print("the MCP client saw its click and its key")
    print("the MCP client drew its own shader")


main()
