# flakes r6 (rc 3): report

Lane: cloud, branch `claude/flakes-r6` from `6a067e39`, head `e3fb7d46` (pushed). Written 11-16 18:23 UTC.
Load: 8 busy shell loops on 4 cores, the page cache dropped every 10 s, and two or three gate loops at
once (load average 10 to 17). Logs are under `/home/user/logs` of the container only.

| Item | Result | Commits (red, fix) | Runs after the fix, under load |
|---|---|---|---|
| 1. F26 mc screenshot-door, volume-door | cause found, fixed | `9bff0429`, `3e7e400d` | mc cells 50 of 50; before 11 of 102 failed |
| 1. F26 tiling keys-key (as keys-alias, train 20) | cause read, gate fixed | `e3fb7d46` (gate only) | not run: stopped at the time limit; its key cells 2 of 2 |
| 1. F26 xp helper-key, mc launcher-chat | not reproduced | none | xp cells 0 of 4 failed |
| 2. F28 chat-keys | cause found, fixed | `bdc2184f`, `bb2adbd5` | chat-keys 12 of 12; red step 3 of 3 before |
| 3. F17 words alarm | not reproduced; no sighting has the r4 fix | `80ead5b5`, `4c27ea0f` (the line names the state) | 0 of 385 failed |

**F26, mc.** Cause: the chat field stays on the screen after the chat closed, at the foot of the column of
mc, over Screenshot and Volume, and takes their clicks. The field (`chora-chat`, a layer surface) takes its
surface away on Esc and tells the host `closed`; the host then only closed the view. After each open mc
answers with the box of its slot and the host places the field a second time. When that second `place` is
written before the host read `closed`, the field makes a new surface, and nothing takes it away. Evidence:
(a) a capture at the failure shows the field ("Type a message, then Enter.") in the row of the two doors
with no chat view (`f26/old/FAIL-dA-9/shot.png`, `after-none.png`); (b) the event stream has no pointer
event at all for the click on Screenshot, while Lock one row higher still answers; (c) with one log line
in the field, the failed run reads `place`, `Esc closes the surface`, `place, surface shown before: false`,
the two runs before it `place`, `place`, `Esc`; (d) the host log has no `hide`. The dump of train 22 was
read as "the click lands on the hit node": it does not, a layer surface is above it and `scene.query` does
not list layer surfaces. It needs slow I/O: 0 of 26 runs failed without the cache drops, 11 of 102 with
them, always these two cells. Red: `a_view_that_the_field_closed_hides_a_field_that_was_placed_again`
(the host sends no hide). Fix: the host tells the field to hide on every close (`close_view` is gone, the
`closed` of the field runs `hide`). Green: 231 unit tests; `tests/doors/mc.sh` 50 of 50.

**F26, tiling.** keys-key was missing in 1 of 4 loaded runs of the whole doors gate. Read, not reproduced
alone: the cell takes the key list for open when its node is `visible`; tiling makes the list with opacity
0, measures it (two queries and a frame) and only then sets opacity 1 and the keyboard in one transaction
(`toggle_keys`, main.rs 1128). An Escape in between goes to the window, the list stays, the cell says
missing after 10 s. The cell now waits for opacity 1 (no timeout changed). No red commit: gate only.

**F26, rest.** xp helper-key and helper-door (testwatch `13f86740`) and mc launcher-chat (media r1) did not
show: `tests/doors/xp.sh` 4 of 4, and the four doors gates below. Not shown, a guess from the cells only: xp
presses Esc as soon as the node `hud-chat` exists, which is before the field has the keyboard.
`train22-gate1-doors.log` of the brief is not on `claude/session-log`.

**F28.** Cause: a question of any helper told the field to hold Enter (`question_asked` sends
`{"question":true}`), in every view; opening a view sets the hold from the helper of that chat, which the
list has not. So the same question held the Enter of the list or not, by the order of the two. The fake
agent asks after every prompt, at a time that follows the load: mostly before the list opens (free field),
under load while it shows, and then `2` Enter picks nobody. Evidence: host log of a failed run
`pick_first "moin"`, then `question:true`, no pick; of a passing run `question:true` twice, then `clear`,
`question:false`. With `FAKE_ACP_DELAY=1` the gate failed 2 of 3 times with the message of F28. Red: the
unit test `a_question_while_the_list_shows_does_not_hold_the_field`, and the gate now lets the first
helper ask while the list shows (`FAKE_ACP_ASK_AFTER` of the fake agent): 3 of 3 failed before the fix.
Fix: only the field of a chat holds Enter, the list never. Green: 3 of 3, then 12 of 12 under load.

**F17.** Not reproduced: 0 of 200 alone under load, 0 of 60 with a cold cache, 0 of 25 runs of the whole
test binary (589 tests) with a cold cache, 0 of 100 at the head. Every recorded sighting ran a tree
without the r4 fix `4a5ddb40` (merged 11-15 03:20:37 UTC): train 8 gate 3 (11-14), ux r1 run 3
(`ea2947fe`), harness r2 (`cd302dd2`), ci e2e r5 (`886cbb1e`); `git merge-base --is-ancestor` says so for
the three trees. No timeout changed. The line of a cut backtrace now names the state that the alarm found
("is stuck after 10 ms in the state S"), so a next sighting names its cause. Proposal: close F17.

## Gates

No full `tests/merge-gate.sh` ran: the round ended at the time limit of the owner (PM 18:22 UTC), the
train of the PM runs it. The 2 x 5 runs of the doors gate at the head were stopped in their first run.

| Cell | Tree | Load | Result |
|---|---|---|---|
| `tests/doors/mc.sh` | `6a067e39` | as above | 11 of 102 failed (screenshot-door, volume-door); 0 of 26 without cache drops |
| `tests/doors/mc.sh` | `3e7e400d` | as above | 50 of 50 pass |
| `tests/gate-doors.sh` (all looks) | `4c27ea0f` | as above | 3 of 4 pass; 1 fail tiling keys-key; mc and xp pass in all 4 |
| `tests/doors/tiling.sh`, key cells only | `e3fb7d46` | as above | 2 of 2 pass |
| `tests/doors/xp.sh` | `3e7e400d` | as above | 4 of 4 pass |
| `tests/gate-chat-keys.sh`, new step | `bdc2184f` (red) | light | 3 of 3 fail with the message of F28 |
| `tests/gate-chat-keys.sh` | `bb2adbd5` | 8 loops | 3 of 3, then 12 of 12 pass |
| words test alone | `6a067e39`, then `4c27ea0f` | 8 loops | 0 of 200, 0 of 100 failed |
| words test, cold cache | `6a067e39` | 8 loops | 0 of 60 alone, 0 of 25 in the whole binary |
| unit tests | `e3fb7d46` | | chora-assistant 231 pass; chora `words::` 10 pass; clippy of chora clean |

Not run at the head: the whole workspace test and clippy, `pre-commit run --all-files`,
`tests/ci-local.sh`, the full gate. The hooks ran on each commit.

## Next step for the lane that continues

1. Full `tests/merge-gate.sh` at `e3fb7d46`, then 10 loaded runs of `tests/gate-doors.sh`
   (`/home/user/logs/f26/gate-loop.sh` is gone with the container: loop the gate with its own
   `CHORA_GATE_LOCK`, 8 busy loops, `echo 3 >/proc/sys/vm/drop_caches` every 10 s).
2. xp helper-key: test the guess (Esc before the field has the keyboard) with a log of the window of the
   gate (`enter`, `leave`) at the failure.
3. An independent review of the five product commits.

## Open

- A chat with helper B also holds Enter while helper A asks (host log: `question_asked` in
  `Chat("second")`, then `question:true`): `voice.confirm` does not name the helper, so the host cannot
  tell. Not fixed. Propose backlog.
- `scene.query` lists no layer surface, so a gate cannot see what covers a door. Propose backlog.
- The key list of tiling loses an Escape that follows Super+F1 within its measuring step (a person is
  slower; the gate was not). Propose MINOR.
- xp helper-key and mc launcher-chat: not reproduced. No independent review of this round yet.
- The message of commit `e3fb7d46` says "1 of 3" loaded runs; it was 1 of 4.
