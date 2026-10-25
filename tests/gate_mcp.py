"""The pieces of an MCP client that the gate scripts of plan 0.8 share.

A script imports Mcp, or calls its main(command, out) through bash, which
starts `choractl --name agent mcp` and hands the client to the command.
"""

import base64
import json
import os
import subprocess
import sys


class Mcp:
    def __init__(self, choractl=None, stderr=None):
        ctl = choractl or os.path.join(
            os.environ.get("BIN") or os.path.join(
                os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                "target/debug"), "choractl")
        self.server = subprocess.Popen(
            [ctl, "--name", "agent", "mcp"],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=stderr, text=True)
        self.n = 0
        self.request("initialize", {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "gate-0.8", "version": "1"}})
        self.notify("notifications/initialized")

    def send(self, message):
        self.server.stdin.write(json.dumps(message) + "\n")
        self.server.stdin.flush()

    def notify(self, method, params=None):
        message = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            message["params"] = params
        self.send(message)

    def request(self, method, params=None):
        self.n += 1
        self.send({"jsonrpc": "2.0", "id": self.n,
                   "method": method, "params": params or {}})
        line = self.server.stdout.readline()
        if not line:
            sys.exit(f"the server closed during {method}")
        answer = json.loads(line)
        if answer.get("id") != self.n or "error" in answer:
            sys.exit(f"{method}: {answer}")
        return answer["result"]

    def call(self, name, arguments=None):
        """The raw tools/call result, so a script can read isError."""
        return self.request("tools/call", {"name": name, "arguments": arguments or {}})

    def tool(self, name, arguments=None):
        result = self.call(name, arguments)
        if result.get("isError"):
            sys.exit(f"{name}: {result['content']}")
        return result["content"]

    def text(self, name, arguments=None):
        return [c["text"] for c in self.tool(name, arguments) if c["type"] == "text"]

    def json(self, name, arguments=None):
        return json.loads(self.text(name, arguments)[0])

    def capture_png(self, name, arguments, path):
        images = [c for c in self.tool(name, arguments) if c["type"] == "image"]
        if len(images) != 1:
            sys.exit(f"{name} gave {len(images)} images")
        with open(path, "wb") as f:
            f.write(base64.b64decode(images[0]["data"]))

    def close(self):
        self.server.stdin.close()
        self.server.wait(timeout=10)
