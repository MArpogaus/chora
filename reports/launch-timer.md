# launch-timer

## r1

Branch claude/launch-timer, head 629faf7b (from 1c9e64f7, the session head; origin/claude/autonomous-dev-session-3w-i33pg7 had not moved, so it was already merged). Closes DB11 F-2 and the GIVE_UP timer (plan slice 39).

| Item | Red | Fix |
|---|---|---|
| F-2: a live, windowless start is never served | dc2565e1 `a_running_program_without_a_window_is_not_served_at_the_old_limit` (launch.rs; pid of the test process, fake clock, no wait) | 629faf7b |
| GIVE_UP: no fixed failure for a program that still runs | 057063ec `a_running_program_without_a_window_gives_no_message_after_the_old_limit` (no `Change` at 30, 31, 300 s; the window still closes note 12) | 629faf7b |

One fix commit for both: the second red test needs the first fix's code gone, and a fix of the first alone left the field `watched` unused (clippy, a hook), so the pair has no passing state between them.

What 629faf7b does:

- `GIVE_UP` is gone. A start whose main process runs waits for its window or its end, however long. The Starting balloon is `expire_ms` 0 (until closed); it closes with the window, with the end, or with the drop of the start.
- A program that ended without a failure and without a window fails with `Why::Exited` (it said `Slow` before) `HAND_OVER` (30 s, a launcher that hands over to a running instance) after its end, not after its start; a window of another start meanwhile drops the start and closes its message (`Done`).
- A start with neither pid nor unit (an old `choractl`) cannot be followed; it keeps the 30 s from its start as `Why::Slow` under `HAND_OVER`. Reason: nothing else can tell its end.
- The follow thread no longer has a 31 s deadline: it ends with the program, or when the start is dropped (`Watch` guard sets a stop flag, polled once a second with the pidfd). New unit test `a_wait_for_a_process_that_runs_ends_when_it_is_told_to_stop`; tests for the hand-over wait: `a_program_that_ended_without_a_failure_or_a_window_ends_the_wait_late`, `a_hand_over_that_showed_a_window_of_another_start_closes_its_message`. The test `a_program_that_still_runs_at_the_limit_is_no_failure` (asserted the old Done at 30 s) is removed.
- Module doc, `Why` and `Change` texts, docs/configuration.md (Run box and `editor` rows) say what holds now; `plugins/chora-xp` no longer re-exports `GIVE_UP`.

StatusFile: kept. A pidfd of a non-child gives no exit status, and the main process of the unit is a child of the user manager; `choractl run` starts the unit with `--collect`, which removes a failed unit with its `Result`, so the unit has no result to read either. Reason is in the comment of `Next::status`. D104 in docs/decisions.md still says "up to 30 s" for exit 0 and "until the limit" for a pidfd that cannot hold; I left decisions.md alone (not in the brief); the PM may want a D104 note.

`git grep -n GIVE_UP` now: only `GIVE_UP_AFTER` and `GIVE_UP_AT_LAST` in crates/chora/src/words.rs, the handler watchdog, no relation.

Gate (`cargo test -p chora-kit -p chora-xp` green, `cargo clippy --workspace --all-targets -- -D warnings` clean):

- `tests/merge-gate.sh --jobs 1` (full): red-first to tiling-audit PASS (build, test, clippy, 2d-guard, xp, matrix, virtual, xp-*, hud, layout), then tiling-look FAIL at 419 s, and the gate stops there. xp-notepad (the cell with this path) PASS 55 s.
- tiling-look is foreign: `FAIL: ink-2560x1600@2 bar-font: the clock is 119.5 wide, not Inter 14 px weight 500 (100-119)` (same line for paper-2560x1600@2, then `FAIL: the bar differs from the spec of tiling-v2`). It is the tiling bar clock font at 2x; this branch touches no tiling code, and I did not rerun it.
- Because the gate stops at the first red, I ran the steps after it with `--only` (`tiling-empty` to `e2e`, 26 steps, `--jobs 1`): all PASS, no SKIP, including launcher-click (17 s) and e2e (295 s).
