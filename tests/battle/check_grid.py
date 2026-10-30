#!/usr/bin/env python3
"""check_grid.py EXPECTED -> exit 1 unless exactly EXPECTED surfaces sit on distinct places."""
import json, os, subprocess, sys
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
B = os.path.join(os.environ.get("BIN", os.path.join(REPO, "target/debug")), "choractl")
out = subprocess.run([B, "--name", "agent", "query", "app_id:chora-probe kind:surface"],
                     capture_output=True, text=True, timeout=10).stdout
nodes = json.loads(out[out.index("["):])
# the rects of T2 are already the absolute place on each output
pos = sorted((output, round(r[0]), round(r[1]))
             for n in nodes for output, r in n["rects"].items())
sizes = sorted({tuple(round(v) for v in r[2:]) for n in nodes for r in n["rects"].values()})
if len(set(pos)) != len(pos):
    print("OVERLAP"); sys.exit(1)
if len(pos) != int(sys.argv[1]):
    print(f"COUNT {len(pos)} != {sys.argv[1]}"); sys.exit(1)
print(f"grid ok: {len(pos)} windows at {pos} sizes {sizes}")
