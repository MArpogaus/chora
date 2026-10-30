#!/usr/bin/env python3
"""Fill the scene `mission` of chora-mc with the state of the machine.

Panels: load (cpu with a sparkline, memory, disk, network, temperature),
the busiest processes, the errors of the journal today, the failed units
of the system and the user manager, and host facts. A source that is
missing here leaves a short note instead. Everything is only read.

Usage: system.py [--print] [--once] [--every SECONDS]   (default 5)
Without --once it runs until it is stopped; the sparkline keeps the cpu
of the last 24 rounds.
"""

import argparse
import itertools
import json
import os
import platform
import shutil
import subprocess
import sys
import time

BARS = "▁▂▃▄▅▆▇█"
TOP = 6  # processes


def run(*args):
    """Stdout of a command, or None when it is missing or fails."""
    try:
        out = subprocess.run(args, capture_output=True, text=True, timeout=10)
    except (OSError, subprocess.SubprocessError):
        return None
    return out.stdout if out.returncode == 0 else None


def cpu_times():
    """Busy and total jiffies of all cpus."""
    with open("/proc/stat") as f:
        values = [int(v) for v in f.readline().split()[1:]]
    idle = values[3] + values[4]  # idle and iowait
    return sum(values) - idle, sum(values)


def net_bytes():
    """Received and sent bytes of all interfaces but lo."""
    rx = tx = 0
    with open("/proc/net/dev") as f:
        for line in f.readlines()[2:]:
            name, data = line.split(":", 1)
            if name.strip() != "lo":
                fields = data.split()
                rx += int(fields[0])
                tx += int(fields[8])
    return rx, tx


def meminfo():
    """Answer the used and the total memory in KiB."""
    with open("/proc/meminfo") as f:
        info = {line.split(":")[0]: int(line.split()[1]) for line in f}
    return info["MemTotal"] - info["MemAvailable"], info["MemTotal"]


def gb(kib):
    """Format KiB as GB."""
    return f"{kib / 1024**2:.1f}"


def spark(history):
    """Bars up to the highest value, at least a quarter of all cpus."""
    top = max(25.0, *history)
    return "".join(BARS[min(7, int(v / top * 8))] for v in history)


def temperature():
    """Answer the first thermal zone in °C, or None."""
    try:
        with open("/sys/class/thermal/thermal_zone0/temp") as f:
            return int(f.read()) // 1000
    except OSError:
        return None


def load(history, net, every):
    """Answer the lines of the panel load."""
    used, total = meminfo()
    disk = shutil.disk_usage("/")
    lines = [
        f"cpu\t{history[-1]:.0f} %   {spark(history)}",
        f"memory\t{gb(used)} of {gb(total)} GB",
        f"disk\t{disk.used // 10**9} of {disk.total // 10**9} GB",
        f"net\t↓ {net[0] / every / 1e6:.1f}   ↑ {net[1] / every / 1e6:.1f} MB/s",
    ]
    temp = temperature()
    if temp is not None:
        lines.append(f"temp\t{temp} °C")
    return lines


def top():
    """Answer the busiest processes."""
    out = run("ps", "-eo", "stat,comm,%cpu,rss", "--sort=-%cpu", "--no-headers")
    if out is None:
        return ["\tps did not run"]
    lines = []
    for row in out.splitlines():
        stat, row = row.split(None, 1)
        comm, pcpu, rss = row.rsplit(None, 2)
        if stat.startswith("Z"):
            continue  # a zombie
        rss = int(rss)
        size = f"{rss / 1024**2:.1f} GB" if rss >= 1024**2 else f"{rss // 1024} MB"
        lines.append(f"{comm[:14]}\t{float(pcpu):.0f} %   {size}")
        if len(lines) == TOP:
            break
    return lines


def errors():
    """Answer the errors of the journal today."""
    out = run("journalctl", "-p", "err", "-b", "--since", "today", "-o", "json", "-n", "6", "-q")
    if out is None:
        return ["\tjournalctl is not here"]
    lines = []
    for row in reversed(out.splitlines()):
        entry = json.loads(row)
        message = entry.get("MESSAGE")
        if not isinstance(message, str):
            continue  # binary messages come as lists of bytes
        when = time.strftime("%H:%M", time.localtime(int(entry["__REALTIME_TIMESTAMP"]) / 1e6))
        who = entry.get("SYSLOG_IDENTIFIER") or entry.get("_COMM", "?")
        lines.append(f"{when}\t{who}: {message[:120]}")
    return lines or ["\tnone today"]


def units():
    """Answer the failed units of the system and the user manager."""
    lines = []
    for scope, args in (("system", []), ("user", ["--user"])):
        out = run("systemctl", *args, "--failed", "--plain", "--no-legend")
        if out is None:
            lines.append(f"\tsystemctl --{scope} did not answer")
            continue
        lines += [f"failed\t{row.split()[0]}  ·  {scope}" for row in out.splitlines() if row]
    return lines or ["\tno failed units"]


def host():
    """Answer host facts."""
    with open("/proc/uptime") as f:
        up = int(float(f.read().split()[0]))
    days, hours, minutes = up // 86400, up % 86400 // 3600, up % 3600 // 60
    uptime = f"{days} d {hours} h" if days else f"{hours} h {minutes} min"
    return [
        f"name\t{platform.node()}",
        f"kernel\t{platform.release().split('-')[0]}",
        f"uptime\t{uptime}",
        f"cores\t{os.cpu_count()}",
    ]


def send(panel, lines, show):
    """Set a panel of the scene, or print its parameters."""
    params = json.dumps({"scene": "mission", "panel": panel, "lines": lines}, ensure_ascii=False)
    if show:
        print(params)
    elif run("choractl", "call", "mc", "panel.set", params) is None:
        sys.exit(f"system.py: panel.set {panel} failed")


def main():
    """Fill the panels each round."""
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--print", action="store_true", help="print the JSON, do not send it")
    parser.add_argument("--once", action="store_true", help="one round, then exit")
    parser.add_argument("--every", type=float, default=5.0, help="seconds between rounds")
    args = parser.parse_args()
    history = []
    cpu, net = cpu_times(), net_bytes()
    # The first round measures over one second, not over nothing.
    time.sleep(1.0)
    every = 1.0
    for rounds in itertools.count():
        cpu2, net2 = cpu_times(), net_bytes()
        busy, total = cpu2[0] - cpu[0], cpu2[1] - cpu[1]
        history = (history + [100 * busy / max(total, 1)])[-24:]
        traffic = (net2[0] - net[0], net2[1] - net[1])
        cpu, net = cpu2, net2
        send("load", load(history, traffic, every), args.print)
        send("top", top(), args.print)
        # The slow sources: journal, units and host once a minute.
        if rounds % max(1, int(60 / args.every)) == 0:
            send("errors", errors(), args.print)
            send("units", units(), args.print)
            send("host", host(), args.print)
        if args.once:
            return
        time.sleep(args.every)
        every = args.every


if __name__ == "__main__":
    main()
