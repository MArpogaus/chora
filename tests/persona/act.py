#!/usr/bin/python3 -I
"""The hands and eyes of a simulated user: screen and input, nothing else.

Usage: PERSONA_ENV=FILE python3 -I tests/persona/act.py ACTION ...
  look [--max-side N]        save a capture as PNG, print its path
  click X Y [--button left|right] [--double]
  move X Y
  drag X1 Y1 X2 Y2
  type TEXT
  key COMBO                  for example Super+Return, Return, Ctrl+c
  scroll DX DY               logical pixels; positive DY scrolls down

PERSONA_ENV is the session.env that tests/persona/session.sh wrote. Each
action appends one JSON line to its PERSONA_LOG: time, action,
arguments, ok, result. Exit status 0 on success, 1 on a failed action,
2 on a bad command. Coordinates are logical pixels of the first output.
"""

import argparse
import json
import os
import subprocess
import sys
import time

BUTTONS = {"left": 272, "right": 273}
TIMEOUT = 20


def load_env(path):
    """Read the KEY=VALUE lines of the session file into a dict."""
    env = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            key, sep, value = line.rstrip("\n").partition("=")
            if sep:
                env[key] = value
    return env


def choractl(env, *args, stdin=None):
    """Run choractl as the client persona; return (ok, output)."""
    run_env = {
        "PATH": env["PATH"],
        "CHORA_SOCKET": env["CHORA_SOCKET"],
        "XDG_RUNTIME_DIR": env["XDG_RUNTIME_DIR"],
        "HOME": os.environ.get("HOME", "/"),
    }
    try:
        p = subprocess.run(
            ["choractl", "--name", "persona", *args],
            input=stdin,
            capture_output=True,
            text=True,
            timeout=TIMEOUT,
            env=run_env,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as e:
        return False, str(e)
    text = (p.stdout + p.stderr).strip()
    return p.returncode == 0, text


def inject(env, events):
    """Send input events through choractl inject."""
    return choractl(env, "inject", "-", stdin=json.dumps(events))


def look(env, args):
    """Capture the screen to a numbered PNG file."""
    shots = env["PERSONA_SHOTS"]
    n = len([f for f in os.listdir(shots) if f.endswith(".png")]) + 1
    path = os.path.join(shots, f"{n:03d}.png")
    cmd = ["capture", path]
    if args.max_side:
        cmd += ["--max-side", str(args.max_side)]
    ok, text = choractl(env, *cmd)
    return ok, {"path": path, "info": text} if ok else text


def do_click(env, args):
    """Move, then click once or twice."""
    b = BUTTONS[args.button]
    events = [{"pointer_move": {"position": [args.x, args.y]}}, {"click": {"button": b}}]
    if args.double:
        events.append({"click": {"button": b}})
    return inject(env, events)


def do_drag(env, args):
    """Press at the start, move to the end, release."""
    return inject(
        env,
        [
            {"pointer_move": {"position": [args.x1, args.y1]}},
            {"button": {"state": "pressed", "hold": True}},
            {"pointer_move": {"position": [(args.x1 + args.x2) / 2, (args.y1 + args.y2) / 2]}},
            {"pointer_move": {"position": [args.x2, args.y2]}},
            {"button": {"state": "released"}},
        ],
    )


def parser():
    """The command line: exactly the allowed actions."""
    p = argparse.ArgumentParser(prog="act.py", description=__doc__.split("\n")[0])
    sub = p.add_subparsers(dest="action", required=True)
    s = sub.add_parser("look")
    s.add_argument("--max-side", type=int, default=800)
    s = sub.add_parser("click")
    s.add_argument("x", type=float)
    s.add_argument("y", type=float)
    s.add_argument("--button", choices=sorted(BUTTONS), default="left")
    s.add_argument("--double", action="store_true")
    s = sub.add_parser("move")
    s.add_argument("x", type=float)
    s.add_argument("y", type=float)
    s = sub.add_parser("drag")
    for name in ("x1", "y1", "x2", "y2"):
        s.add_argument(name, type=float)
    s = sub.add_parser("type")
    s.add_argument("text")
    s = sub.add_parser("key")
    s.add_argument("combo")
    s = sub.add_parser("scroll")
    s.add_argument("dx", type=float)
    s.add_argument("dy", type=float)
    return p


def run(env, args):
    """Do the action; return (ok, result)."""
    a = args.action
    if a == "look":
        return look(env, args)
    if a == "click":
        ok, text = do_click(env, args)
    elif a == "move":
        ok, text = inject(env, [{"pointer_move": {"position": [args.x, args.y]}}])
    elif a == "drag":
        ok, text = do_drag(env, args)
    elif a == "type":
        ok, text = inject(env, [{"text": {"text": args.text}}])
    elif a == "key":
        ok, text = inject(env, [{"shortcut": {"keys": args.combo}}])
    else:
        ok, text = inject(env, [{"scroll": {"dx": args.dx, "dy": args.dy}}])
    return ok, text or "ok"


def main():
    """Parse, run, log, report."""
    args = parser().parse_args()
    path = os.environ.get("PERSONA_ENV")
    if not path:
        sys.exit("act.py: PERSONA_ENV is not set")
    env = load_env(path)
    arguments = {k: v for k, v in vars(args).items() if k != "action"}
    started = time.time()
    try:
        ok, result = run(env, args)
    except Exception as e:  # a log line for every attempt, also a crash
        ok, result = False, f"{type(e).__name__}: {e}"
    entry = {
        "time": round(started, 3),
        "action": args.action,
        "args": arguments,
        "ok": ok,
        "seconds": round(time.time() - started, 3),
        "result": result,
    }
    with open(env["PERSONA_LOG"], "a", encoding="utf-8") as f:
        f.write(json.dumps(entry) + "\n")
    if not ok:
        print(f"failed: {result}", file=sys.stderr)
        sys.exit(1)
    if args.action == "look":
        print(result["path"])
    else:
        print("ok")


if __name__ == "__main__":
    main()
