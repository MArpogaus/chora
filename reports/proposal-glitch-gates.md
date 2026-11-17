# Proposal: glitches caught by tools, not by watching (PM, 2026-11-17 14:2x)

Owner 11-17 14:20: "Current screen cast is horrible. Lots of unexpected glitches still exist. No more time
wasting on it. New demo has to become smoother and more "natural". We need stronger review guidelines to
catch all issues automatically. Note and discuss sugfestion"

## Why the reviews missed them
Eight tours (v1-v8) were judged by people and reviewers watching video. Each found some issues and missed
others; the same classes came back (menu left open, overprint, black screen, stale chat state, notice not
drawn, raw launcher entries). Watching does not scale and is not repeatable. The tour scripts asserted
almost nothing after an act, so a glitch passed as long as the script did not crash.

## Suggestions (cost in % of the week)
1. Act contracts (3 %). Every scripted act declares its expected end state (`act(..., expect=...)`:
   owner of the window, focused window, menus closed, notice shown, chat idle) and the run fails on the
   first mismatch, read from choractl state, not from pixels. Media r6 proposed this; it becomes the rule.
2. Invariants after every act (2 %). One checker run after each act, for every look: no menu or popup left
   open that the act did not open; no window lost or off-screen; no text overlapping text (layout boxes
   from the scene); no output fully black; chat not "Working" after the answer; a requested notice drawn.
   Each invariant comes from a class seen in v3-v8 (catalogue in the same file, one line per class).
3. Video checker (1 %). On every recording: black or frozen stretches, flicker (a frame that differs and
   reverts), frame-time jitter. Fails the recording; output is a list of timestamps.
4. Review guideline (0.5 %). A review starts from the checker output; the human or opus reviewer reads
   only flagged timestamps plus one fresh-eyes pass, and every finding found by eye becomes a new
   invariant or contract before it is fixed (same as rules.md 7, extended: no eye-only finding twice).
5. Natural demo (in P5, 2 %). The playbook beats run with human pacing (eased pointer paths, typing
   delays, dwell before a click), one take, no retakes; shown only after the checkers are clean 3 times.

## Placement
- Stop now: tour rounds and screencast fixes (no tour v9, no v8 fresh-eyes review).
- Foundation F7 changes from "tour v9 matches v8" to "contracts and invariants on every playbook beat,
  video checker green" (1-3, about 6 %, replaces tour v9 3 % and the v8 review 1 %: net +2 %).
- 4 at the next merge boundary (rules.md 10); 5 stays in P5.
