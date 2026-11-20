# mc-fix

## r1

Lane: cloud, branch claude/mc-fix from dac835dd (train 85), merged with origin/claude/autonomous-dev-session-3w-i33pg7 (already contained). Step 0: `git fetch --unshallow` and `cargo check --workspace` green. Pushed: claude/mc-fix at b4104cae. Captures: reports/mc-fix/.

| item | verdict | red | fix | cell or test | capture |
|---|---|---|---|---|---|
| 1 mc-47 click on a launcher row | not a bug of mc (candidate reproduces, cause is fuzzel) | cell written, dropped (see below) | none | replay in reports/mc-fix/mc-47-* | mc-47-fuzzel-events.log, mc-47-fuzzel-rows.png, mc-47-mc-after-click.png |
| 2 mc-49 xp+mc never quiet | fixed | c3432638 | 55f75baa | mc unit test a_build_in_the_midst_of_a_write_leaves_mc_quiet | check.py start xp+mc 3 of 3 quiet |
| 3 mc-48 I5 at Super+a | checker misread of a cross-fade, fixed at its source | 137437c5 | bb3de644 | mc unit test a_cross_fade_keeps_mc_busy_until_its_end | mc-48-busy-after-fix.txt |
| 4 P2-CHECK | done, checker only | tests with the change | b4104cae | tests/test_acts.py (6 tests), tests/persona-selftest.sh | |

### 1. mc-47
`mc-sofa launcher-click-row` was written and was red: real fuzzel on mc (a Foot entry, the list Chat with a helper, Foot, Screenshot), a pointer move and a click on the row Foot starts nothing; the control with typed `Foot` and Return passes in the same cell (the window of the program gets its page). The cause is not in mc or the core:
- `WAYLAND_DEBUG=1 fuzzel --dmenu` under chora-tiling (mc-47-replay-tiling.sh, mc-47-fuzzel-events.log): fuzzel gets `wl_pointer.enter` at the row, `motion`, and `button` press and release (272) at the row beta, and prints nothing, stays open. Held press (hold) and a plain tap give the same.
- The installed fuzzel is 1.9.2; its binary has no handling for clicks (its strings and the events agree). The same click on tiling does nothing as well, so no flavor starts a program by a click on a row.
- So the click of r2 sofa (and r1: it fell on the page below) cannot start a program with this fuzzel. The cell can never go green here and a red cell in mc-sofa blocks every merge, so it is not on the branch (patch kept: mc-47-cell.patch, sha ddd43ea3 before the reset).
- For the PM: T1 by mouse needs a launcher that takes clicks (a newer fuzzel if one does, not checked; or a list that mc draws itself). That is a design item, not a fix of this lane. `launcher-click` is no step of tests/merge-gate.sh (--list), the gate ran without it.

### 2. mc-49
Reproduced on the first start: `round.sh start xp+mc` gives `NotQuiet: still busy after 10 s: mc:write:mc-s-agenda-panels-today`, and state.get stays `["write:mc-s-agenda-panels-today"]` for 30 s and more (no rewrite loop; the entry is never removed). Cause: `Mc::build` (a build after the second look joined and the output changed) removes the node `mc` with the panel whose write animation runs; the core ends no animation of a removed node, `animation.done` never comes, and the `writing` entry stays. Fix: `note_writes` forgets the entries of every node that a Remove takes away (the build passes its removal through it). Red test c3432638 fails with the busy word of the capture. After: `round.sh start xp+mc` 3 of 3 with `check.py start` quiet (binaries of the fix, 55f75baa).

### 3. mc-48
Replay of the sofa actions 1 to 18 on mc (replay-sofa: look, Right, Super+space, click, type Foot, Return, XF86Back x2, XF86HomePage, Super+a), nodes read every 150 ms after Super+a:
- before the fix: the group of the old set `mc-regions-1` goes 1.0 -> 0.2 -> hidden between 0.2 s and 0.6 s while `mc-regions-0` fades in (0 -> 1); both are shown for 0.2 to 0.6 s; after that the old set is hidden, not stale, and there is no second draw. The old set is meant to stay under the root hidden until the next switch.
- A check at +0.55 s (the sofa breach: 0.548 s after the key, on a loaded machine with two sessions) reads both sets: I5 17 times with equal rects. mc did not say busy, so `wait_quiet` returned at once. That is a misread of a transient state that mc should have called busy.
- Fix: every fade of opacity that mc starts is a busy word `fade:<node>` in state.get until `animation.done` of the node, at most its time and 2 s (a removed node forgets it too), so the harness waits. After the fix, same replay: busy `['fade:mc-r0-right', 'fade:mc-regions-1']` from 0.4 to 0.56 s, `[]` from 0.71 s, no breaches.jsonl (mc-48-busy-after-fix.txt). Not changed in I5: a set that stays visible after the fade would still breach.

### 4. P2-CHECK (tests/persona, b4104cae)
- M1 passes when the page differed from the start at the end or at any look of the task (the observation now holds the page: the shown scene groups of mc with opacity of at least 0.5); the windows of mc count as before.
- T6 passes on the launcher row `Nothing found for ...`: session.sh puts a `fuzzel` stand-in first in the PATH that logs `fuzzel-first-row <row>` in calls.log and runs the real fuzzel with the same rows.
- T1 and T4 read `open` besides the key list of the look (`expected open []` for anything else).
- The wpctl stand-in keeps its volume in a file: get-volume answers the value after set-volume `N%+`, `N%-` or a number (clamped to 0 to 1).
- Tests: test_acts.py (t1 and t4 key list, t6 row, m1 turned and came back, m1 fade); persona-selftest.sh (wpctl 0.55 after 5%+, 0.45 after 10%-, the first row logged). `python3 -I tests/test_acts.py` 140 ok, `tests/persona-selftest.sh` ok.

### Gate
`tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,keys,chat-keys,doors,mc-paths,mc-sofa,mc-handover,layout`: all 12 PASS (build 111 s, test 242 s, clippy 46 s, ci-check 134 s, layout 201 s, doors 357 s, mc-paths 80 s, mc-sofa 43 s, mc-handover 12 s, keys 20 s, chat-keys 9 s, pre-commit 6 s). The step `launcher-click` of the brief does not exist and was left out. It is a partial run: nothing was pushed by it.

Not touched: mc-50, the M2 ruling.
