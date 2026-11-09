#!/usr/bin/env python3
"""Sum the token use of Claude Code sessions per day and model.

Usage: tokens.py [DAY] [--carry CACHE_READ,OUTPUT]   (plain integers, no separators)
  DAY as YYYY-MM-DD in Asia/Bangkok, default today. --carry adds the
  totals of earlier containers of the same day (the `carry` column of
  ledger.md) before the estimate.

Prints the totals of the day so far, not a delta. The estimate of the
used percent of the weekly limit comes from the calibration in
answers.md: PER_PERCENT holds the tokens of one percent, the larger of
the two ratios counts. Update it with each report of the user.

Reads every transcript under ~/.claude/projects, subagents included. An
assistant message is logged once per content block with the same usage,
so each message id counts once.
"""

import json
import sys
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from zoneinfo import ZoneInfo

ZONE = ZoneInfo("Asia/Bangkok")
# 2026-11-09, weekly readings of the user (whole percents): 3 % at 10:33,
# 6 % at 11:48 (180.1 million cache read, 897,769 output), 8 % at 12:30
# (273.6 million cache read, 1,279,909 output).
PER_PERCENT = {"cache_read": 32_000_000, "output": 150_000}
KEYS = ("input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens", "output_tokens")


def day_of(stamp):
    return datetime.fromisoformat(stamp.replace("Z", "+00:00")).astimezone(ZONE).date().isoformat()


def main():
    args = sys.argv[1:]
    carry = (0, 0)
    if "--carry" in args:
        i = args.index("--carry")
        parts = args[i + 1].split(",") if i + 1 < len(args) else []
        if len(parts) != 2 or not all(p.isdigit() for p in parts):
            sys.exit("--carry takes two plain integers: CACHE_READ,OUTPUT")
        carry = (int(parts[0]), int(parts[1]))
        del args[i : i + 2]
    day = args[0] if args else datetime.now(ZONE).date().isoformat()
    seen = set()
    sums = defaultdict(lambda: [0, 0, 0, 0])
    for path in Path.home().glob(".claude/projects/**/*.jsonl"):
        for line in path.open(encoding="utf-8", errors="replace"):
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            msg = event.get("message")
            if not isinstance(msg, dict) or msg.get("role") != "assistant":
                continue
            usage, mid, stamp = msg.get("usage"), msg.get("id"), event.get("timestamp")
            if not usage or not mid or not stamp or mid in seen or day_of(stamp) != day:
                continue
            seen.add(mid)
            row = sums[msg.get("model", "?")]
            for i, key in enumerate(KEYS):
                row[i] += usage.get(key) or 0
    print(f"day {day}  (input | cache write | cache read | output)")
    for model, row in sorted(sums.items()):
        print(f"{model:24} " + " | ".join(f"{n:>11,}" for n in row))
    total = [sum(col) for col in zip(*sums.values())] or [0, 0, 0, 0]
    print(f"{'total':24} " + " | ".join(f"{n:>11,}" for n in total))
    print(f"seen: {total[2]},{total[3]}")
    if carry != (0, 0):
        print(f"carry: {carry[0]:,} cache read, {carry[1]:,} output")
    cache_read, output = total[2] + carry[0], total[3] + carry[1]
    pct = max(cache_read / PER_PERCENT["cache_read"], output / PER_PERCENT["output"])
    print(f"estimate: {pct:.1f} % of the weekly limit")


if __name__ == "__main__":
    main()
