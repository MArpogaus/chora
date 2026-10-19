"""Shared: absolute translation of a node by composing transform matrices."""
import json, subprocess
B = "/home/marcel/Projekte/chora/target/debug/choractl"

def query(sel="*"):
    out = subprocess.run([B, "--name", "agent", "query", sel],
                         capture_output=True, text=True, timeout=10).stdout
    return json.loads(out[out.index("["):])

def _tr(n):
    m = (n.get("transform") or {}).get("matrix") or []
    if len(m) == 16:
        return (m[12], m[13], m[14])  # column-major
    return (0.0, 0.0, 0.0)

def abs_pos(nodes_by_id, node):
    x = y = 0.0
    cur = node
    seen = 0
    while cur is not None and seen < 20:
        dx, dy, _ = _tr(cur)
        x += dx; y += dy
        cur = nodes_by_id.get(cur.get("parent") or "")
        seen += 1
    return x, y

def size_of(n):
    return tuple((n.get("surface") or {}).get("size") or n.get("size") or [0, 0])
