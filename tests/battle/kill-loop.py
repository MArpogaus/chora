#!/usr/bin/env python3
"""kill-loop.py [name]: end background driver scripts by name (loop.sh,
battle.sh) without ever matching the calling shell."""
import os
import signal
import sys

name = sys.argv[1] if len(sys.argv) > 1 else "loop.sh"
me = os.getpid()
for pid in (int(p) for p in os.listdir("/proc") if p.isdigit()):
    if pid in (me, 1, os.getppid()):
        continue
    try:
        args = open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")
    except OSError:
        continue
    if len(args) >= 2 and args[0].endswith(b"bash") and args[1].endswith(name.encode()):
        print("killing", pid, args[:3])
        try:
            os.kill(pid, signal.SIGTERM)
        except OSError:
            pass
