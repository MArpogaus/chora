#!/usr/bin/python3 -I
"""usage.py: token totals of this session and its local agents after a time (UTC ISO), weighted
like the bill: input + 1.25*cache_write + 0.1*cache_read + 5*output, per model family
(haiku weighted 1/5 of opus, sonnet 3/5).  Usage: usage.py SINCE [UNTIL]"""
import json, sys, glob, os
since = sys.argv[1]; until = sys.argv[2] if len(sys.argv) > 2 else "9999"
base = "/root/.claude/projects/-home-user"
files = glob.glob(base + "/8a28ab73-2182-54dd-8448-69a0fbb58e8a.jsonl") + glob.glob(base + "/8a28ab73-2182-54dd-8448-69a0fbb58e8a/subagents/*.jsonl")
fam = {"haiku": 0.2, "sonnet": 0.6, "opus": 1.0, "fable": 1.0}
tot = {}
seen = set()
for f in files:
    for line in open(f, errors="ignore"):
        try: e = json.loads(line)
        except Exception: continue
        ts = e.get("timestamp", "")
        if not (since <= ts < until): continue
        m = e.get("message") or {}
        u = m.get("usage")
        if not u: continue
        key = (m.get("id"), f)
        if key in seen: continue
        seen.add(key)
        model = m.get("model", "?")
        w = next((v for k, v in fam.items() if k in model), 1.0)
        units = (u.get("input_tokens", 0) + 1.25 * u.get("cache_creation_input_tokens", 0)
                 + 0.1 * u.get("cache_read_input_tokens", 0) + 5 * u.get("output_tokens", 0)) * w
        tot[model] = tot.get(model, 0) + units
for k, v in sorted(tot.items(), key=lambda x: -x[1]): print(f"{k:28s} {v/1e6:8.2f} M units")
print(f"{'all':28s} {sum(tot.values())/1e6:8.2f} M units")
