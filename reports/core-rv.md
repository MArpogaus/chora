# core-rv

## r1

Lane: claude/core-rv, base 1c599c74, head 1db1a50f (merged state: the base is the head of claude/autonomous-dev-session-3w-i33pg7, so no merge was needed). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` green.

| Finding | Red test commit | Fix commit |
|---|---|---|
| rv-1 safety.rs: unmeasured first slow frame set `run.last` | e6e80a7e `an_unmeasured_slow_frame_is_no_sign_that_a_suspect_stays_guilty` (failed: suspect still set after 32 s) | 4768ca68: `note` sets `run.last` only when the frame was measured (the frame before was over budget, which is when `clock_in` reads the threads) |
| rv-5 safety.rs: removed outputs never forgotten | 09320703 `an_output_that_went_starts_a_new_run_when_it_returns` (failed: stale run) | 9a9351ef: `Watchdog::forget`, called from `leave_layout` (output.rs) |
| rv-2 logs.rs: symlinked folder said "Not a directory" | 3639bb39 `a_symlinked_folder_gets_no_file_and_no_chmod` now asserts "is a symlink" (failed on the ENOTDIR message) | 1db1a50f: ENOTDIR and ELOOP map to the symlink message when `symlink_metadata` says symlink |
| CM-R1 api.md "kept nodes stay shown" and ipc.rs pin | none: already on the base (ff69a833 test `kept_nodes_stay_visible_during_the_resume_grace`, the api.md line "The kept nodes stay shown during the grace time"); the test passes at the head | none needed |

No new lint allow. Commit hooks (fmt, clippy, no inline comments) passed on every commit.

Gate: `tests/merge-gate.sh --jobs 1` (no --only): all steps passed (49 PASS, no FAIL, no SKIP), exit 0. Opt-in `tests/gate-watchdog-starved.sh`: both lines ok ("a starved host makes the watchdog blame nobody"; "frames that cost too much by themselves still blame their client"), exit 0. `cargo test -p chora --lib safety` 27 passed; `cargo test -p chora-scene logs` 6 passed.

Note: CM-R1 had no red step (pinning test of documented behaviour, rule 8 needs red only for a fix).
