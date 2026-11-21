# xp-rd r1 (rc 6, RC bar honest UI): the review-delta xp findings rd-2 and rd-3

Model: sonnet, effort medium (two small input fixes in one plugin, each with a red test)
Lane: cloud, claude/xp-rd at a84effda. Cost at most 0.6 %; stop at 150 k context. One area: plugins/chora-xp (context.rs, help.rs and their tests).
Why: reports/review-delta.md "## r1" (claude/session-log), verdict MERGE-OK with one MEDIUM and three MINOR; xp has no open BLOCK or MAJOR, so rd-3 (MINOR) may ride along (rules.md 6).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "xp-rd BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) rd-2 MEDIUM, plugins/chora-xp/src/context.rs:943: a release anywhere on the Volume slider row sets the level from the clamped place (a click on the word "Volume" sets 0 %, one in the right padding 100 %). Red test first (own commit, test: prefix): a press and release on the label leaves the level unchanged. Fix: act only when the pointer is over the track (left - TRACK_END .. left + TRACK + TRACK_END), else no act.
2) rd-3 MINOR, plugins/chora-xp/src/help.rs:341: `dy.signum()` is 1.0 for dy 0.0, so a horizontal-only wheel scrolls the key table down. Red test first: a wheel with dy 0.0 does not scroll. Fix: return false for dy 0.0; accumulate sub-row deltas (or use `discrete`) only if it stays under 20 lines.
3) Checks: `cargo test -p chora-xp`, `cargo clippy -p chora-xp -- -D warnings`, `tools/lint-ratchet`.
4) Gate: the full gate `tests/merge-gate.sh --jobs 1` (no --only) green on the branch merged with origin/claude/autonomous-dev-session-3w-i33pg7. A red cell you did not cause: name it with its log line; do not rerun to green. Start runs in the background and poll in the same turn; never end a turn to wait.
Done when: both red tests are green after their fix, full gate green.
Report: reports/xp-rd.md "## r1" on claude/session-log (each finding: red sha, fix sha, test name; gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/xp-rd and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
