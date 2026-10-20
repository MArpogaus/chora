#!/usr/bin/env python3
"""positions.py -> sorted id:abs-pos of all probe windows (for restart checks)."""
import sys
sys.path.insert(0, "/home/marcel/Projekte/chora/tests/battle")
import geom
nodes = geom.query("app_id:chora-probe kind:surface")
by = {n["id"]: n for n in geom.query("*")}
for n in sorted(nodes, key=lambda n: n["id"]):
    x, y = geom.abs_pos(by, n)
    print(f"{n['id']}:{int(x)},{int(y)}")
