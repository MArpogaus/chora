# flakes-o (F5: no flaky tests)

## r1

Lane: cloud, branch `claude/flakes-o` from `ecfa2029`; head `16da7bc7`. Written 11-20.
Load: `tests/under-load.sh` with 2 to 6 busy loops and a second `cargo build` loop (clean of chora-kit,
own target dir) beside it; the head of the cube cell (to the still check, about 25 s) in one to three
parallel streams. Logs are under `/tmp/claude-0` of the container only.

| Item | Result | Commits | Loaded runs after the fix |
|---|---|---|---|
| 1. FLK-CUBE-STILL | cause shown, fixed | diagnostic `dd7d463a`, red `f17cae94`, fix `38ca5b26` | head of the cell with `CLOCK_EDGE=2` 20 of 20 (red 7 of 15); full cell 3 of 3 in the gate at --jobs 3 |
| 2. FLK-XP-HW | not reproduced, evidence only | diagnostic `16da7bc7` | 20 of 20 (flakes-m: 20 of 20) |
| 3. pr1-1 spaced keys | not done | none | the inject API has no pause |

### 1. FLK-CUBE-STILL
Diagnostic `dd7d463a`: `still()` dumps the scene (`choractl query '*'`) before and after each window of
2 s; a red names per window its time, the render.stats counters and the node properties that changed.
Reproduction: 120 loaded runs of the head of the cell (20 at 3 busy loops, 60 at 6 with `INJECT_LAG=0.4`
including 40 with the new diagnostic, 40 in three streams at 2 busy loops each): 2 red. The one with
both windows named (run 14 of stream b, 16:28:58):
window 16:28:58 drew `construct-door-clock-text.text "16:28" -> "16:29"` (HEADLESS-1 +1 frame);
window 16:29:00 drew `xp-tray-VIRTUAL-1..4` `meta.text "4:28 PM" -> "4:29 PM"` and the tray image
(each face +1 frame, HEADLESS-1 +1). The first red (stream with 6 busy loops) printed the second window
only, the same tray tick.
Cause: a still cube draws once a minute. The construct door clock and the xp tray clock (xp checks its
clock every second, `TICK`) redraw at the minute change, and under load not in the same 2 s window, so
the second window of `still()` does not absorb the draw (it takes one clock, there are two). The 90 and
715 of flakes-m's failure line were the absolute counters `damaged_frames` and `passes` of HEADLESS-1,
not a rate; it was the same case. A check across a minute change fails about 4 s in 60 under load, which
is the 1 in 20 seen.
Red `f17cae94`: `CLOCK_EDGE=2` starts the first window 2 s before a minute change (default unset): 7 of 15
red on the head of the cell (3 streams of 5, 2 busy loops each and the build loop), every red a draw by
the two clocks.
Fix `38ca5b26`: `clock_clear` (poll of the wall clock, `until_ok`) lets a window of `still()` start only at
second 6 to 52 of a minute, so no window holds a change or its late tray tick; a draw in a clear window is
still a red. Product code is unchanged. Loaded after the fix, with `CLOCK_EDGE=2` (the stress now waits
for second 6): 20 of 20 (7, 7 and 6 in three streams). The same `still()` serves the floating check
(line 596 of the cell).
Not done: no 20 loaded runs of the full cell (130 s alone, 180 s loaded); the head of the cell, which holds
the check, ran 20 times, the full cell 3 times in the gate at --jobs 3.

### 2. FLK-XP-HW
Not reproduced in 20 loaded runs of the xp cells helper-key, helper-door and helper-none (3 streams,
2 busy loops each and the build loop, 147 s each; one full xp look ran green on top). Nothing changed in the
product. `16da7bc7`: the failure of helper-door now says the tooltip text, the helper node of the tray
(meta, visible) and the answer of voice `state`, so the next sighting tells a helper that voice does not
have (voice side) from a tray that did not hear it (xp side). Candidate, not shown: `Helper::ask` has
`ASK_WAIT` 500 ms and the machine stays without an answer; `call_within` queues the events that come in
between (ipc.rs), so a late answer is not an event loss. The row stays open on evidence only.

### 3. pr1-1
Not done, as the brief says: `InjectEvent` has PointerMove, Button, Click, Scroll, Key, Shortcut, Text and
no pause, so keys spaced in real time during a turn are separate `choractl` starts, whose start time is the
cause of the flake `a6753908` removed. A cell that wants them needs a pause event in the inject API
(core), or a gate-side sleep between two injections that again depends on the start time.

## Gate
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,cube,doors` at
`16da7bc7` (PATH with ~/.local/bin): pre-commit 4 s, gate-lib 70 s, ci-check 95 s, build 28 s, test 218 s,
clippy 17 s, cube 144 s, doors 370 s: all PASS. Then `--only cube,doors` twice more at --jobs 3: cube 120 s
and 119 s, doors 372 s and 372 s: PASS.

## Open
- FLK-XP-HW: evidence only, next sighting has the state.
- pr1-1: needs a pause in the inject API; a core item.

## PM review r1 (PM not the builder)
MERGE-OK 16da7bc7 (tests only); diag dd7d463a, red f17cae94 (CLOCK_EDGE=2: 7 of 15 loaded fail), fix 38ca5b26 (20 of 20 loaded); cube and doors PASS at --jobs 1 and twice at --jobs 3. Checks: the still check now runs only in seconds 6-52, so the minute redraw of the construct door clock and the xp tray clock cannot fall in its 2 s windows; the failure line names the nodes that changed. No product change; the clocks redraw once a minute as they should (POLS). No open MAJOR.
- pr1-1 NIT: still() can wait up to ~10 s for a clear second; fine for one gate, note it if more cells copy it (a kit helper then).
- FLK-XP-HW stays open (0 of 20 loaded); 16da7bc7 names tooltip, tray node and voice state on the next failure. F5 not met yet.
Contains flakes-m ecfa2029: in t92 it replaces flakes-m.
