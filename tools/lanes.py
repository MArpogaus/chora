#!/usr/bin/env python3
"""Print the live lanes of the PM session (read-only): subagents, chora processes, merged worktrees, disk, PM context.

Usage: python3 tools/lanes.py [--all]   (--all lists finished subagents too)
Exit status 1 when any flag is raised. Flags: fewer than two lanes (running local subagents plus the
cloud workers under "Lanes:" of state.md Now), subagent context above 250k, opus or fable builders, chora process older than 1 h,
worktree merged into the session branch, PM context above 400k ("compact now").
"""
import argparse
import json
import os
import shutil
import sys
import subprocess
import time
from pathlib import Path

SESSION = Path("/root/.claude/projects/-home-user/8a28ab73-2182-54dd-8448-69a0fbb58e8a")
MAIN = SESSION.with_suffix(".jsonl")
WT = Path("/home/user/wt")
CODE = Path("/home/user/chora")
BASE = "origin/claude/autonomous-dev-session-3w-i33pg7"
AGENT_CONTEXT_LIMIT = 250_000
STATE = Path("/home/user/wt/log/state.md")
MIN_LANES = 2
PM_CONTEXT_LIMIT = 400_000
PROCESS_AGE_LIMIT = 3600
ACTIVE_WINDOW = 1800
HEAVY = ("opus", "fable")


def ago(seconds):
    if seconds < 3600:
        return f"{int(seconds // 60)}m"
    return f"{seconds / 3600:.1f}h"


def last_turn(path):
    model, context = "?", 0
    with open(path, "rb") as f:
        f.seek(max(0, os.path.getsize(path) - 400_000))
        for raw in f.read().splitlines()[1:]:
            try:
                d = json.loads(raw)
            except ValueError:
                continue
            msg = d.get("message")
            if d.get("type") == "assistant" and isinstance(msg, dict) and msg.get("usage"):
                u = msg["usage"]
                context = u.get("input_tokens", 0) + u.get("cache_read_input_tokens", 0) + u.get("cache_creation_input_tokens", 0)
                model = msg.get("model", model)
    return model, context


def subagents(show_all):
    rows, now = [], time.time()
    for p in SESSION.glob("subagents/*.jsonl"):
        meta = {}
        try:
            meta = json.loads(p.with_suffix(".meta.json").read_text())
        except (OSError, ValueError):
            pass
        idle = max(0, now - p.stat().st_mtime)
        if meta.get("finished") and not show_all:
            continue
        if idle > ACTIVE_WINDOW and not show_all:
            continue
        model, ctx = last_turn(p)
        desc = (meta.get("description") or p.stem)[:38]
        flags = []
        if ctx > AGENT_CONTEXT_LIMIT:
            flags.append("CONTEXT>250k")
        if any(h in model for h in HEAVY) and "build" in (meta.get("description") or "").lower():
            flags.append("HEAVY BUILDER")
        rows.append((idle, desc, model.replace("claude-", ""), ago(idle), f"{ctx // 1000}k", " ".join(flags)))
    rows.sort()
    return [r[1:] for r in rows]


def chora_processes():
    out = subprocess.run(["ps", "-eo", "pid,etimes,args"], capture_output=True, text=True).stdout
    rows = []
    for line in out.splitlines()[1:]:
        pid, age, args = line.split(None, 2)
        exe = args.split()[0]
        if os.path.basename(exe).startswith("chora") and "flock" not in args:
            flag = "OLDER THAN 1h" if int(age) > PROCESS_AGE_LIMIT else ""
            rows.append((pid, ago(int(age)), os.path.basename(exe), flag))
    return rows


def merged_worktrees():
    out = subprocess.run(["git", "worktree", "list", "--porcelain"], cwd=CODE, capture_output=True, text=True).stdout
    rows = []
    for block in out.split("\n\n"):
        fields = dict(l.split(" ", 1) for l in block.splitlines() if " " in l)
        path = fields.get("worktree", "")
        if not path.startswith(str(WT)) or "branch" not in fields:
            continue
        head = fields["HEAD"]
        branch = fields["branch"].replace("refs/heads/", "")
        created = subprocess.run(
            ["git", "reflog", "show", "--format=%H", fields["branch"]], cwd=CODE, capture_output=True, text=True
        ).stdout.split()
        merged = subprocess.run(["git", "merge-base", "--is-ancestor", head, BASE], cwd=CODE).returncode == 0
        if created and created[-1] == head:
            flag = "fresh lane"
        else:
            flag = "MERGED, remove" if merged else ""
        rows.append((os.path.basename(path), branch, flag))
    return rows


def cloud_lanes():
    try:
        text = STATE.read_text()
    except OSError:
        return 0
    count, on = 0, False
    for line in text.splitlines():
        if line.startswith("Lanes:"):
            on = True
        elif on and line.startswith("- "):
            lower = line.lower()
            count += "cloud" in lower and "done" not in lower
        elif on and not line.strip():
            break
    return count


def pm_context():
    model, ctx = last_turn(MAIN)
    return model.replace("claude-", ""), ctx, "compact now" if ctx > PM_CONTEXT_LIMIT else ""


def table(title, header, rows):
    print(f"\n{title}")
    if not rows:
        print("  none")
        return 0
    widths = [max(len(str(x)) for x in col) for col in zip(header, *rows)]
    for r in (header, *rows):
        print("  " + "  ".join(str(c).ljust(w) for c, w in zip(r, widths)).rstrip())
    return sum(1 for r in rows if r[-1] and r[-1] != "fresh lane")


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--all", action="store_true")
    args = ap.parse_args()
    flags = 0
    agents = subagents(args.all)
    flags += table("subagents (unfinished or active in 30 min)", ("agent", "model", "idle", "context", "flag"), agents)
    flags += table("chora processes", ("pid", "age", "name", "flag"), chora_processes())
    flags += table("worktrees", ("dir", "branch", "flag"), merged_worktrees())
    free = shutil.disk_usage("/home").free / 2**30
    print(f"\ndisk free: {free:.1f} GB" + ("  LOW" if free < 10 else ""))
    flags += free < 10
    model, ctx, flag = pm_context()
    print(f"PM context: {ctx // 1000}k ({model}) {flag}".rstrip())
    flags += bool(flag)
    lanes = len(agents) + cloud_lanes()
    print(f"lanes: {lanes} running (local subagents plus cloud workers in state.md)" + ("  FEWER THAN TWO LANES" if lanes < MIN_LANES else ""))
    flags += lanes < MIN_LANES
    return 1 if flags else 0


if __name__ == "__main__":
    sys.exit(main())
