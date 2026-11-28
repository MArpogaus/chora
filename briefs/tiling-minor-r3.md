# tiling-minor r3 (rc 1b, F7: tiling MINOR and NIT from the tiling-minor r2 review)

Model: sonnet, effort high (routine: two notice bodies, two tests, two doc lines on one look)
Red on head: rtmr2-1, rtmr2-2 MINOR and rtmr2-3..5 NIT open on ad07ec70 (reports/tiling-minor.md "## Review r2"); no BLOCK or MAJOR open in tiling, so rules.md 6 allows this round.
Lane: cloud, branch claude/tiling-minor-r3 at ad07ec70743f18fe3d96fa9a60dc93dea5935dd5 (tiling-minor r2 head, MERGE-OK, in train 163). Area: the tiling look (flavors/tiling, chora-tiling, chora-kit launcher doc) and the lock notice of flavors/shared/power.sh. One look. Stop at 250 k. Cost at most 0.5 %.
Why: owner 11-17 10:59 "Be perfectionist and very critical about the met targets."; vision card: POLS, honest states.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`. A failure: send_message the PM "tiling-minor-r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1. rtmr2-1: flavors/tiling/look.sh:15 says "look" for the colour theme; "The theme stayed as it was." and `-a Theme` (De beside), the exact-notice assert of give_notice.rs changed to match.
2. rtmr2-2: flavors/shared/power.sh:29 a failed lock shows "swaylock -f" as the body; a body in words ("The screen is not locked." / "Der Bildschirm ist nicht gesperrt."), the command stays on stderr; a notice test like give_notice.rs.
3. rtmr2-3: an apps_launcher.rs case whose FakeCore refuses scene.query and still sees the three move rows (pins "a failed read counts as a window").
4. rtmr2-4: the doc of `without_moves` (crates/chora-kit/src/launcher.rs:104) says it is the Apps half; the focus rule is tiling's `window_has_focus`.
5. rtmr2-5: with no window, "move" in Apps shows one row "No window to move" as the Looks menu does (one string, one place).
Each fix: a red test commit first (failing on ad07ec70), then the fix (4 is docs only, no red); a capture before and after for 1, 2 and 5 under media/2026-11-28/tiling-minor-r3/ (each under 500 KB).
Run: the full gate, no step filter: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 --allow-skip docs` (no --push) detached with setsid nohup, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line, rerun that step alone and then the steps after it. Never end a turn to wait. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: five ids closed, each with red commit (not 4) and fix commit (and captures for 1, 2, 5); gate EXIT 0 in one run (or a named foreign flake and every later step run); reports/tiling-minor.md "## r3" on claude/session-log (base, head, diffstat, per id).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): head sha, ids closed, gate result.
Rules: push claude/tiling-minor-r3 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media folder); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
