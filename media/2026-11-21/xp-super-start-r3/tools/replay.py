#!/usr/bin/env python3
"""Replay the ricer's actions.jsonl through act.py. usage: replay.py ENV LOG KEEP(comma list or 'all') OUTDIR"""
import json, subprocess, sys, time, os
env_path, log, keep, out = sys.argv[1:5]
env = {**os.environ, "PERSONA_ENV": env_path, "PERSONA_CHECK": "1"}
act = "/home/user/chora/tests/persona/act.py"
rows = [json.loads(l) for l in open(log)]
keepset = None if keep == "all" else {int(x) for x in keep.split(",")}
os.makedirs(out, exist_ok=True)
vals = dict(l.strip().split("=", 1) for l in open(env_path) if "=" in l)
def menu():
    r = subprocess.run(["choractl", "--name", "persona", "query", "id:xp-item-0"], capture_output=True, text=True,
        env={"PATH": vals["PATH"], "CHORA_SOCKET": vals.get("CHORA_SOCKET", ""), "XDG_RUNTIME_DIR": vals.get("XDG_RUNTIME_DIR", "")})
    try:
        n = json.loads(r.stdout); return bool(n and n[0]["rects"]), (n[0]["rects"] if n else None)
    except Exception: return None, r.stdout + r.stderr
prev = None
for i, a in enumerate(rows, 1):
    if prev is not None:
        time.sleep(min(6.0, max(0.0, a["time"] - prev)))
    prev = a["time"]
    if a["action"] != "look" and keepset is not None and i not in keepset:
        continue
    args = a["args"]; cmd = ["python3", "-I", act, a["action"]]
    if a["action"] == "look": cmd += ["--max-side", "800"]
    elif a["action"] == "click": cmd += [str(args["x"]), str(args["y"])]
    elif a["action"] == "type": cmd += ["--", args["text"]]
    elif a["action"] == "key": cmd += [args["combo"]]
    elif a["action"] == "scroll": cmd += [str(args["dx"]), str(args["dy"])]
    r = subprocess.run(cmd, env=env, capture_output=True, text=True)
    if a["action"] == "look":
        print(i, "look", r.stdout.strip()[:200].replace("\n"," "), flush=True)
    if i in (28, 29, 30):
        print(i, "xp-item-0 open:", menu(), flush=True)
    if i == 29:
        time.sleep(1.5); print("29+1.5s menu:", menu(), flush=True)
