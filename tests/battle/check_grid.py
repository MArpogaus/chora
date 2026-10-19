#!/usr/bin/env python3
"""check_grid.py -> exit 1 if two surfaces share a position."""
import sys
sys.path.insert(0, "/home/marcel/Projekte/chora/tests/battle")
import geom
nodes = geom.query("app_id:chora-probe kind:surface")
by = {n["id"]: n for n in geom.query("*")}
pos = sorted(tuple(round(v) for v in geom.abs_pos(by, n)) for n in nodes)
sizes = sorted({geom.size_of(n) for n in nodes})
if len(set(pos)) != len(pos):
    print("OVERLAP"); sys.exit(1)
print(f"grid ok: {len(pos)} windows at {pos} sizes {sizes}")
