#!/bin/bash
# Gate check 6 of 0.7: a client with `input.grab-any` grabs F13 and a
# letter key and gets their events; a client without it gets the error
# code for a missing capability. Injected keys run over the grabs (D23),
# so the check needs no real keyboard. The stock pc105 keymap binds no
# key between F1 and F12; the fkeys option adds F13 to F24.
#
# Usage: tests/gate-0.7-6.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
need python3
export XKB_DEFAULT_OPTIONS=fkeys:basic_13-24
out=$(mktemp /tmp/gate6.XXXXXX.py)
trap 'rm -f "$out"' EXIT
cat >"$out" <<'PY'
import json, os, socket, sys, time

PATH = os.environ["CHORA_SOCKET"]

class Conn:
    """One newline-delimited JSON-RPC connection. It keeps the event
    notifications that arrive while it waits for an answer."""

    def __init__(self, name):
        self.sock = socket.socket(socket.AF_UNIX)
        self.sock.connect(PATH)
        self.sock.settimeout(5)
        self.file = self.sock.makefile("rw")
        self.n = 0
        self.events = []
        answer = self.call("hello", {"name": name})
        if "error" in answer:
            sys.exit(f"hello {name}: {answer['error']}")

    def _line(self):
        try:
            line = self.file.readline()
        except (TimeoutError, socket.timeout):
            return None
        if not line:
            sys.exit("the core closed the socket")
        event = (json.loads(line).get("params")) or {}
        if event.get("event"):
            self.events.append(event)
            return None
        return json.loads(line)

    def call(self, method, params=None):
        self.n += 1
        self.file.write(json.dumps({"jsonrpc": "2.0", "id": self.n,
                                    "method": method, "params": params or {}}) + "\n")
        self.file.flush()
        while True:
            msg = self._line()
            if msg is not None:
                if msg.get("id") == self.n:
                    return msg

    def collect(self, seconds):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            self._line()

# A client without the capability gets code 4 for a plain F13.
guest = Conn("guest")
refusal = guest.call("input.grab", {"binding": "F13"})
error = refusal.get("error") or {}
if error.get("code") != 4:
    sys.exit(f"guest grabbing F13: expected code 4, got {refusal}")
if "input.grab-any" not in error.get("message", ""):
    sys.exit(f"code 4 without a word on the capability: {error}")

# The client with the capability grabs F13 and the letter a. Its
# injected presses arrive as input.key events on its own connection.
agent = Conn("agent")
agent.call("events.subscribe", {"events": ["input.key"]})
for binding in ("F13", "a"):
    grabbed = agent.call("input.grab", {"binding": binding})
    if "error" in grabbed:
        sys.exit(f"agent grabbing {binding}: {grabbed['error']}")
events = [{"key": {"key": k, "state": s}}
          for k in ("F13", "a") for s in ("pressed", "released")]
injected = agent.call("input.inject", {"events": events})
if "error" in injected:
    sys.exit(f"the injection: {injected['error']}")
agent.collect(3)
pressed = {e.get("binding") for e in agent.events
           if e["event"] == "input.key" and e.get("state") == "pressed"}
if pressed != {"F13", "a"}:
    sys.exit(f"the presses of F13 and a arrived as: {sorted(pressed)}")
print("guest got code 4; agent grabbed F13 and a and saw both presses")
PY
session_start 1280x800
"$bin/choractl" grant agent input.grab-any input.inject || fail "the grants"
res=$(python3 -I "$out" 2>&1) || fail "$res"
echo "$res"
ok "input.grab-any grabs F13 and a letter; without it the code is 4"
