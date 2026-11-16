#!/usr/bin/env python3
"""Sum the token use of Claude Code sessions per day and model.

Usage: tokens.py [DAY] [--carry CACHE_READ,OUTPUT]   (plain integers, no separators)
       tokens.py --forecast   (pace and projected use at the reset from READINGS)
  DAY as YYYY-MM-DD in Asia/Bangkok, default today. --carry adds the
  totals of earlier containers of the same day (the `carry` column of
  archive/ledger.md) before the estimate.

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
# Weekly readings of the user on 2026-11-09 (whole percents): 3 % at
# 10:33, 6 % at 11:48, 8 % at 12:30, 9 % at 13:11. Tokens count in
# opus-5-5 equivalents; the model weights are an assumption from the
# price ratios and are refined by later readings.
MODEL_WEIGHT = {"opus": 1.0, "sonnet": 0.6, "haiku": 0.2}
PER_PERCENT = {"cache_read": 33_000_000, "output": 132_000}
KEYS = ("input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens", "output_tokens")


def day_of(stamp):
    return datetime.fromisoformat(stamp.replace("Z", "+00:00")).astimezone(ZONE).date().isoformat()


# Weekly readings of the owner (UTC, percent of the week, all models).
# Append each new reading; --forecast fits the pace since the week reset.
RESET = "2026-11-21T03:59"  # next reset, Saturday 03:59 UTC
READINGS = [
    ("2026-11-15T03:32", 20),
    ("2026-11-15T09:20", 33),
    ("2026-11-16T01:17", 38),
    ("2026-11-16T07:00", 39),
    ("2026-11-16T08:49", 40),
]


def forecast():
    """Print the pace (percent a day) and the projected use at the reset.

    Uses the owner's readings, which include the cloud lanes that the
    local transcript count cannot see. Pace: last reading against the
    reading about a day earlier (or the first one).
    """
    utc = ZoneInfo("UTC")
    pts = [(datetime.fromisoformat(t).replace(tzinfo=utc), v) for t, v in READINGS]
    now = datetime.now(utc)
    last_t, last_v = pts[-1]
    base = next((p for p in reversed(pts[:-1]) if (last_t - p[0]).total_seconds() >= 20 * 3600), pts[0])
    days = (last_t - base[0]).total_seconds() / 86400
    pace = (last_v - base[1]) / days if days > 0 else 0.0
    reset = datetime.fromisoformat(RESET).replace(tzinfo=utc)
    left_days = (reset - last_t).total_seconds() / 86400
    print(f"last reading: {last_v} % at {last_t:%a %H:%M} UTC ({(now - last_t).total_seconds() / 3600:.1f} h ago)")
    print(f"pace: {pace:.1f} %/day since {base[0]:%a %H:%M} UTC")
    print(f"forecast at reset {reset:%a %H:%M} UTC: {last_v + pace * left_days:.0f} %")
    print(f"room: {(100 - last_v) / left_days:.1f} %/day over {left_days:.1f} days")


def main():
    args = sys.argv[1:]
    if args[:1] == ["--forecast"]:
        forecast()
        return
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
    weighted = [0.0, 0.0]
    for model, row in sums.items():
        weight = next((w for name, w in MODEL_WEIGHT.items() if name in model), 1.0)
        weighted[0] += row[2] * weight
        weighted[1] += row[3] * weight
    cache_read, output = weighted[0] + carry[0], weighted[1] + carry[1]
    pct = max(cache_read / PER_PERCENT["cache_read"], output / PER_PERCENT["output"])
    print(f"estimate: {pct:.1f} % of the weekly limit")


if __name__ == "__main__":
    main()
