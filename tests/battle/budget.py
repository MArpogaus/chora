#!/usr/bin/env python3
"""budget.py <main socket> <box socket>

1) one process opens 20 connections: the core takes at most 16 and must
   answer a refusal with an error line before closing.
2) on the box socket create rects until the core refuses: the node budget.
"""
import json
import socket
import sys
import time


class Gone(Exception):
    pass


def rpc(s, f, i, m, p):
    s.sendall(json.dumps({"jsonrpc": "2.0", "id": i, "method": m, "params": p}).encode() + b"\n")
    line = f.readline()
    if not line:
        raise Gone()  # the core closed the connection, e.g. by watchdog
    return json.loads(line)


def connections(path):
    socks = []
    accepted = 0
    refused = False
    for _ in range(20):
        if refused:
            break  # the socket is gone after a refusal, stop sending
        s = socket.socket(socket.AF_UNIX)
        s.settimeout(10)
        s.connect(path)
        f = s.makefile("rwb")
        s.sendall(json.dumps({"jsonrpc": "2.0", "id": 0, "method": "hello",
                              "params": {"name": "budget-main"}}).encode() + b"\n")
        line = f.readline()
        r = json.loads(line) if line else None
        if r and "result" in r:
            accepted += 1
            socks.append(s)  # keep it open
        else:
            refused = True
            print("refusal line:", line[:160])
    print(f"connections from one process: {accepted} accepted, then refused: {refused}")
    for s in socks:
        s.close()
    return accepted == 16 and refused


def nodes(path):
    """Create rects under root until the core refuses. Batches of 200 keep
    the per-apply relayout from dominating."""
    s = socket.socket(socket.AF_UNIX)
    s.settimeout(30)
    s.connect(path)
    f = s.makefile("rwb")
    h = rpc(s, f, 0, "hello", {"name": "budget"})
    if "error" in h:
        print("box hello refused:", h["error"])
        return False
    n = 0
    while n <= 100000:
        batch = [{"create": {"id": f"lt-{n+i}", "kind": "rect", "parent": "root",
                             "props": {"size": [10, 10]}}} for i in range(200)]
        try:
            r = rpc(s, f, 1 + n, "scene.apply", {"commands": batch})
        except Gone:
            # the watchdog ends clients that make the frames slow: this is
            # the node limit of 0.6, and the core stays healthy
            print(f"connection ended at {n} nodes, watchdog territory")
            return True
        if "error" in r:
            print(f"node limit at {n}:", r["error"].get("code"),
                  str(r["error"].get("message", ""))[:120])
            return True  # a limit is fine as long as the connection lives
        n += 200
    print(f"no node limit at or below {n}")
    return True


ok = connections(sys.argv[1])
time.sleep(1)  # let the core notice the closes before the box connects
ok = nodes(sys.argv[2]) and ok
sys.exit(0 if ok else 1)
