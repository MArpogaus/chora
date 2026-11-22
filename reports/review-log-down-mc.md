# PM review: log-down-mc r1

Base 70cbf9bd, own d64ea5c0, head b843bf5f (merged with the session branch). 14 files, +65 -30; builder session_016iS3kQGuYZia3PrtJKkqgJ, reviewer the PM (small diff, owner 11-19 06:12).

## Checks (brief "Done when")
- mc eprintln 27 to 1: `git grep -n eprintln d64ea5c0 -- plugins/chora-mc` gives only main.rs:455, the usage line of `--launch` (CLI text, no log). Met.
- Ratchet: tools/lint-ratchet.list eprintln 91 to 65; allow 360 unchanged. Met.
- lc-2: config.rs test shows `WARN chora_mc::config: voice_hud: own is gone...` on stderr (level and target). Met.
- Levels: all 26 sites are `tracing::warn!`. Each one is a failed call with a fallback or a skipped item ("the scenes go one by one", "stays", "left out", "voice gone", a failed method call), so warn fits the text; no state note or per-frame line was among them. The "chora-mc: " prefix is gone. Met.
- Gate: 26 PASS, then acts FAIL (3 blocking in one construct act, "the hint Keys of a face opens the key list by a click": construct-help not open); later cells did not run. Not mc and not logging. Train 109 runs the full gate.

## Findings
- ldm-1 MINOR: `#![allow(clippy::cognitive_complexity, reason)]` at crate level in plugins/chora-mc/src/main.rs silences the lint for all of mc, not only the handlers that the tracing macros push over. Fix: one fn-level allow per handler (ALLOW-DOWN backlog). Not a merge block (rules.md 7: no open MAJOR).
- ldm-2 FLAKE (F5): the construct act "the hint Keys of a face opens the key list by a click" depends on the 3 s hint (co-keys.md: hint_click after the hint timed out). Red in this lane gate on a foreign change; looks1-mc r1 also saw construct acts red. Goes to a flake lane.

## Verdict
0 BLOCK, 0 MAJOR: MERGE-OK. LOG-DOWN-MC b843bf5f joins train 109.
