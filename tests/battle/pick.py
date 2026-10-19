#!/usr/bin/env python3
"""pick.py <selector> [center|<field>] -> `X=.. Y=..` of the first match."""
import json, sys
sys.path.insert(0, "/home/marcel/Projekte/chora/tests/battle")
import geom
nodes = geom.query("*")
by = {n["id"]: n for n in nodes}
for n in geom.query(sys.argv[1]):
    if len(sys.argv) > 2 and sys.argv[2] not in ("center", "title"):
        print(json.dumps(n.get(sys.argv[2]))); sys.exit(0)
    x, y = geom.abs_pos(by, n)
    w, h = geom.size_of(n)
    if len(sys.argv) > 2 and sys.argv[2] == "title":
        print(f"X={int(x+w/2)} Y={int(y+12)}")  # the title bar of an xp frame
    else:
        print(f"X={int(x+w/2)} Y={int(y+h/2)}")
    sys.exit(0)
sys.exit(1)
