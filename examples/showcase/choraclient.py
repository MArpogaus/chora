"""Minimal JSON-RPC client for the chora IPC socket (see docs/api.md)."""
import base64, json, os, queue, socket, subprocess, threading, time


class Chora:
    def __init__(self, name, **hello):
        self.sock = socket.socket(socket.AF_UNIX)
        self.sock.connect(os.environ["CHORA_SOCKET"])
        self.file = self.sock.makefile("rw")
        self.id = 0
        self.events = queue.Queue()
        self.result = {}
        self.reader = threading.Thread(target=self._read, daemon=True)
        self.reader.start()
        self.name = name
        self.call("hello", {"name": name, **hello})
        # `screen.capture` wants a grant for the name; with CHORACTL set
        # to the admin binary, the demos hand themselves one.
        ctl = os.environ.get("CHORACTL")
        if ctl:
            subprocess.run([ctl, "grant", name, "screen.capture"],
                           capture_output=True)

    def _read(self):
        for line in self.file:
            msg = json.loads(line)
            if "id" in msg and msg["id"] in self.result:
                self.result[msg["id"]] = msg
            elif "params" in msg and "event" in msg["params"]:
                self.events.put(msg["params"])
        for rid in list(self.result):
            self.result[rid] = {"error": {"message": "the core closed the socket"}}

    def call(self, method, params=None, timeout=30):
        self.id += 1
        rid = self.id
        self.result[rid] = None
        self.file.write(json.dumps({"jsonrpc": "2.0", "id": rid, "method": method, "params": params}) + "\n")
        self.file.flush()
        end = time.monotonic() + timeout
        while self.result.get(rid) is None:
            if time.monotonic() > end:
                raise TimeoutError(method)
            time.sleep(0.001)
        msg = self.result.pop(rid)
        if "error" in msg:
            raise RuntimeError(f"{method}: {msg['error']}")
        return msg.get("result")

    def apply(self, commands):
        return self.call("scene.apply", {"commands": commands})

    def put(self, name, data):
        return self.call("assets.put", {"name": name, "data": base64.b64encode(data).decode()})["path"]

    def capture(self, path, output=None, node=None, max_side=None):
        params = {k: v for k, v in (("output", output), ("node", node), ("max_side", max_side)) if v}
        r = self.call("screen.capture", params)
        with open(path, "wb") as f:
            f.write(base64.b64decode(r["png"]))
        return r
