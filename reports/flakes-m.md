# flakes-m (F5: no flaky tests)

## r1

Lane: cloud, branch `claude/flakes-m` from `d81c8293`; head `ecfa2029`. Written 11-20.
Load: `tests/under-load.sh` with 3, 6 or 8 busy loops, a second `cargo build` loop
(clean of chora-kit, own target dir) beside it for the reproductions. Logs are under `/tmp/claude-0`
of the container only.

| Item | Result | Red, fix | Loaded runs after the fix |
|---|---|---|---|
| 1. FLAKE-CUBE-QUEUE | cause shown by stress, not seen on its own | `52bc5371`, `a6753908` | cell head 19 of 20 (the one red is FLK-CUBE-STILL, below); full cell 3 of 3 in the gate at --jobs 3 |
| 2. FJ-R2 construct, FLAKES-K-RV rv-1 | fixed | `5c5a1a79`, `ecfa2029` | construct helper cells 20 of 20 (field 6 s late, 3 busy loops); xp, tiling, mc helper cells 1 of 1 each, and doors 3 of 3 in the gate |
| 3. xp "no helper word" | not reproduced, evidence only | none | 20 of 20 (6 busy loops) |

### 1. FLAKE-CUBE-QUEUE
Not reproduced on its own: 40 loaded runs of the head of the cell (everything up to the queue check, 32 s
alone): 20 at 3 busy loops, 20 at 8, each with the build loop: 0 of 40 failed.
Cause (from the code, shown by stress). `Ctrl+Alt+Right` three times was three `choractl inject` starts.
A turn by key takes 360 ms (`TURN`, cube.rs:59) and `start_turn` queues a key only while the turn runs:
a start that takes longer than the turn lets key 2 begin a turn of its own and key 3 queue one, so the
cube ends on face 3 (or passes 2 as a turn that does not rest) and `front_is 2` fails with
`True True True`, the message of train 82. Red `52bc5371` adds `INJECT_LAG` (seconds before each
injection) to the gate: `INJECT_LAG=0.4` on the head of the cell fails at the queue check 3 of 3 ("did not
queue one turn: True True True" twice, "a third turn key during a turn turned on" once). Fix `a6753908`:
the three shortcuts go in one injection, so the construct reads them as one run of events (the turn is
ended in `step_cube`, not between two events). With `INJECT_LAG=0.4` the head passes the check 3 of 3 (one
of the three then failed on a later check, a still cube drew, which the lag delays).
Product code is unchanged.

New row FLK-CUBE-STILL. In the 20 loaded runs of the fixed head one failed (run 15, 27 s) at
gate-construct-cube.sh:271 `a still cube draws: [('HEADLESS-1', 90, 715), ('VIRTUAL-1', 8, 0), ...]`
(two windows of 2 s in which render.stats still changed), and one of the three runs with
`INJECT_LAG=0.4`. Not the queue check and not caused by the fix (the check is 40 lines earlier). No cause;
the failure line does not say what drew.

### 2. FJ-R2 construct, rv-1
Red `5c5a1a79`: `late_field 6` in tests/doors/construct.sh (the host starts the chat field 6 s late):
helper-key missing 3 of 3 (Escape at once after hud-chat exists goes to the construct; helper-door and
helper-none wait on the chat text first and pass). Fix `ecfa2029`: `chat_close` of construct calls
`wait_for_field`; `field_has_keyboard` (rv-1) is now: the chat is open, the window's last `enter`/`leave`
is not `enter`, and no node of the scene other than a surface has `focused`. Evidence for the
condition, from the session: in the walk `construct-look` has `focused` until the field takes the
keyboard, then nothing has it (t=7 s of a 6 s late field) and the window log stays empty; in a pane
the surface keeps `focused` while the field holds the keyboard and its window logs `leave`, so a
surface is not counted. The first try (any focused node, surfaces too) timed out in the pane. The old
predicate (last line is `leave`) could never be true in the walk. The note on a miss now says the state found
(`chat open, window, focused`). Not shown red separately: rv-1's "any leave" case needs a leave
without the chat; no cell makes one.
Loaded (3 busy loops): construct helper-key, helper-door, helper-none with the field 6 s late 20 of 20
(25 s each); xp, tiling, mc once each green (their doors also run in the gate).

### 3. xp "no helper word"
`tests/doors/xp.sh` helper-door: 20 loaded runs of the three helper cells of xp, 6 busy loops plus the
build loop: 20 of 20 pass (50 s each). Not reproduced, so nothing changed. The old sighting was in the
full doors gate (run 3 of 5), where helper-door runs after about 12 cells; the cells alone may be too
short. A candidate, not shown: xp learns the helper from `voice.state` events and from `Helper::ask`
(ASK_WAIT 500 ms, the machine stays without an answer); a state that arrives while xp is not yet
subscribed is only seen at the next event. No red for it.

## Gate
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,cube,doors,chat-keys`
at `ecfa2029` (PATH with ~/.local/bin): pre-commit 3 s, gate-lib 70 s, ci-check 101 s, build 30 s,
test 231 s, clippy 23 s, chat-keys 8 s, cube 122 s, doors 376 s: all PASS. Then `--only cube,doors`
twice more at --jobs 3: cube 128 s and 119 s, doors 375 s and 376 s: PASS.

## Open
- FLK-CUBE-STILL (above): log what draws in the window; next flakes round.
- FLK-XP-HW: evidence only.
- Not done: no 20 loaded runs of the full cube cell (130 s each); the head of the cell (where the
  fix is) ran 20 loaded times.
