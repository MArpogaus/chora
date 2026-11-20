# flakes-j (F5: no flaky tests)

## r1

Lane: cloud, branch `claude/flakes-j` from `79c8d2b5` (the session branch; the merge of
`origin/claude/autonomous-dev-session-3w-i33pg7` was "Already up to date"), head `6192f696`
(pushed). Written 11-20 about 05:00 UTC. Logs are under `/home/user/logs` of the container only.
Load: busy loops on the 4 cores (`tests/under-load.sh`), the page cache dropped every 10 s in the
first runs, up to three gate cells at once (own `CHORA_GATE_LOCK` each), load average 6 to 11.

| Item | Result | Red, fix | Runs after the fix, loaded |
|---|---|---|---|
| 1. FLAKE-DOORS xp helper-door | cause found, fixed | `7641a6f3`, `6192f696` | xp cell (all 20 cells) 10 of 10; before: 2 of ~20 runs failed in the cell loop, 1 of 5 in the doors gate |
| 2. hud 73b | not reproduced | none | before the fix: 0 of 20 failed (see below); no change |
| 3. FLAKE-SIG | cause found, fixed | `d13da066`, `eef3b496` | whole `cargo test -p chora --lib` binary 10 of 10; the red test 6 of 6 red before |
| 4. tiling-look ink | not reproduced, cause only read | none | 2 loaded runs (8 rounds) passed; no change |
| 5. F22 log_file | not reproduced; fixed before | none | 20 of 20 at the head, 20 of 20 with the test before its fix |

### 1. FLAKE-DOORS

Cause. The cells helper-key and helper-door (and `settle`, helper-none) press Escape as soon as
the node `hud-chat` exists. The host draws that node when the chat opens; the chat field
(`chora-chat`, a layer surface) is a process of its own and has the keyboard later. An Escape in
between goes to the window of the gate, and the chat stays: `chat_close` waits 20 s and the cell is
`missing`. This is the guess of flakes-r6 ("xp presses Esc before the field has the keyboard"), now shown.

Evidence. (a) `chat_close` now counts the Escapes that the probe window got while the chat stayed
open: loaded runs of the cell alone, stream D, runs 1 and 3 of the first ~20, print "the chat stays
open; Escapes that the probe window got meanwhile: 1" with helper-key `missing` (run 3 also
helper-door `missing`, "the chat did not close"); the doors gate under load, run 2 of 5, xp column
helper-key and helper-door `missing`, the same note. (b) A field that starts 2 s late
(`late_field 2` in `tests/doors/lib.sh`: the host runs from a tree of its own, where `chora-chat`
sleeps first) gives the same two cells `missing` every time, 3 of 3, with the same note. Red:
`7641a6f3`.

Fix `6192f696`: the probe window (`chora-probe window --report`) prints `enter` and `leave` when the
keyboard comes and goes, and `chat_close` waits until the last of them is `leave`, which is when the
field holds the keyboard, then presses Escape. No timeout changed, no retry. With the field 2 s late
(now the setting of the xp cell, so the gate keeps guarding it) helper-key, helper-door and
helper-none pass, 3 of 3, then the whole xp cell 10 of 10 under load, then the doors gate in the
merge gate (below).

Click count (FLAKE-H MINOR-1): the cell logs `note: helper-door clicks N`. N was 1 in every run
that printed it, 66 runs (20 of 20 loaded runs before any change, 10 of 10 after the fix, the
rest in the loops): no lost first click, so no product check of input inject under load.

Not explained. (1) The `jq` "null has no keys" of the construct part did not show in about 30 runs
of the doors gate cells (`center` and `text_at` use `to_entries` on a node that is not there yet;
that is the likely source). (2) Once in the loaded doors gate (run 3 of 5, before any change to the
cells) xp helper-door said "no helper word shows": the tooltip of the helper door never left
`No helper` in 40 s. Not reproduced again, no cause. Kept for the next lane.

The same pattern (Escape at once after `chat_open`, no wait for the keyboard of the field) is in
`tests/doors/tiling.sh`, `mc.sh` and `construct.sh`. They did not fail here, so they are unchanged;
`chat_close` of each would wait for the same `leave` line (their probe window is the same).

### 2. hud 73b

Not reproduced. `tests/gate-hud.sh` with `ONLY='construct-host-hud|construct-chat-panel|chat-long-task'`
(the three cells in the order of the gate), 20 runs, with the doors gate and the bar-palette gate
running beside it and the cache dropped every 10 s: 20 of 20 pass (18 to 20 s each). The full hud
step of the merge gate passed (78 s).

A first try with `ONLY='construct-host-hud|chat-long-task'` failed 20 of 20 with the line of 73b,
`timeout 5s: header_capped atlas · working · Read this first.`, and the meta `Working\natlas · Read
this first. Please tidy the …`. That is the text of the glass chat of the construct folder: the cell
construct-host-hud ends without `use_folder neutral` and without `stop_construct`; the cell
construct-chat-panel sets the neutral folder again at its end, and with `ONLY` it did not run. So
a chat-long-task `missing` is a consequence of a folder left at construct. In 73b that can only
follow a construct-chat-panel that ended early (its `|| return 1` lines skip `use_folder neutral`)
or did not run. The 73b log is not on `claude/session-log`, so this is a reading and not a
finding: nothing changed. Proposal: `run_cell` calls `use_folder neutral` and `stop_construct`
itself after each cell, so no cell can leave them (MINOR, 0.2 %).

### 3. FLAKE-SIG

Cause. `the_worker_thread_blocks_the_emergency_signal` scans `/proc/self/task` of the whole test
process. The tests of the `chora` binary run as threads of one process, so the list holds the
`chora-images` workers of other tests, and one of them ends between the read of its `comm` and the
read of its `status`: `read_to_string(...).unwrap()` panics with `Os { code: 3, "No such process"
}` (images.rs, the line of the status read). It needs a worker of another test that ends during the
first scan, hence once in a full `cargo test -p chora`.

Evidence. 20 loaded runs of the whole lib binary (623 tests) did not fail. The new test
`the_scan_of_the_workers_survives_a_worker_that_ends` runs the same scan (`some_worker_blocks_usr1`)
while four threads start and end threads named `chora-images`: red in 0.2 s, 6 of 6 runs, with the
panic above (`d13da066`). Fix `eef3b496`: a status that cannot be read is a worker that is gone and
does not count. Green: the four `images::tests::the_` tests 3 of 3, the whole lib binary 10 of 10
under load.

### 4. tiling-look ink-1280x800 tag colour 1 1 1 255

Not reproduced: `tests/gate-tiling-bar-palette.sh` ran 2 times under load (8 rounds, one beside two
other gates at about 600 s) and in the merge gate, all pass; short of the 20 runs, because a loaded
run takes 600 s.

Read: the wait before the measure stops when the pixel of the bar row differs from the pixel of
the window and the focus border has the accent colour. It does not read the pill itself (113, 16), so
a bar that waybar has drawn without the current tag module yet passes the wait and the measure reads
the colour behind the pill, near black: `1 1 1 255` is what the failed run printed. That fits
"12 s" (the loop ends at the first capture where both hold). A waybar that starts 3 s late did not
show it (1 run, pass), so the guess is not shown. No change. Next lane: log the capture of the first
passing wait and wait for the pill pixel (`near "$(px 113 16)" "$accent" 2`) in the loop; or run 20
loaded rounds with a hook that delays the tag module.

### 5. F22 log_file.rs panic test

Not reproduced, and fixed before this round. The line numbers 89 to 92 of the backlog are those of an
older file. The last sighting (11-17 06:01, CI coverage on `aa8aa8c0`) is before `e5e44719`
(11-19 21:03, "wait for both log lines in the panic test"), which `git merge-base --is-ancestor`
says is not in `aa8aa8c0`. At the head: 20 of 20 loaded runs pass. The test of before that fix
(`git show e5e44719^:crates/chora/tests/log_file.rs`): 20 of 20 pass too, so the load here is not
enough to show the old race either. Proposal: close F22.

## Gates

`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,doors,hud,chat-keys,tiling-look,gate-lib`
at `6192f696`: pre-commit 4 s, gate-lib 67 s, build 32 s, test 248 s, clippy 22 s, ci-check 46 s,
chat-keys 9 s, hud 78 s, doors 372 s, tiling-look 618 s, all PASS. The gate says "partial run:
nothing is pushed" (it was a subset); the branch was pushed by hand.

## Open

- The Escape race in the doors of tiling, mc and construct (item 1), and the helper word that did not
  show once. Propose backlog, MINOR.
- hud: cells that leave the construct folder (item 2). Propose backlog, MINOR.
- tiling-look ink: not shown (item 4). Keep FLAKE open with the next step above.
- Not reviewed independently: the four commits.
- The merge gate cells ran with a subset; no full gate.

## PM review r1 (2026-11-20, PM is not the builder)
Range: 79c8d2b5..6192f696, 4 files +94 -23, tests only (images.rs `mod tests`, tests/probe, tests/doors).
Checks: reds first (d13da066, 7641a6f3); the thread scan returns false on a missing status (ESRCH) instead of unwrap; chat_close waits for `leave` on the probe window before Escape; probe `--report` now prints enter/leave: every other consumer greps `^press`/`^key` (3d, cube, out, pull, wm-pane, winit, persona check.py verify_probe), so none breaks.
- rv-1 MINOR: the churn test runs 4 busy threads for 5 s inside `cargo test` beside the other lib tests; on a loaded train machine that is new load (FLK pattern). 1 s suffices for the race. Backlog FJ-R1.
- rv-2 NOTE: xp doors runs with `late_field 2` always: +2 s per xp doors run, by design (stress kept on).
- rv-3 NOTE: same Esc-before-field pattern in tiling, mc, construct doors (lane's Open): backlog FJ-R2; hud 73b, tiling ink, F22 not reproduced: stay open as FLK items, F22 close proposal accepted (e5e44719).
Verdict (computed): no open MAJOR: MERGE-OK 6192f696; harness train t84 (no conflict with debt-f4 or harness-k).
