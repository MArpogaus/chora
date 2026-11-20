# core-l r1 (stability: core frame path and log file; F5 flake F22)

Model: sonnet, effort high (core code on the frame path; no new feature)
Lane: cloud, branch claude/core-l from 355cdbe5 (train 84 head). Stop at 250 k context; cost at most 1 %. First check each item is still open on the head; a closed one: say so, skip it.
Step 0 (no setup script): `git fetch --unshallow || true; cargo check --workspace`; red: send_message the PM "core-l BLOCKED: <line>" and stop (no apt). A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: backlog.md lines WD2, L16, F22; reports/review-core-watchdog-r1.md if present.
1) WD2: crates/chora/src/safety.rs reads /proc/self/task/*/schedstat on every frame (~0.13 ms, ~104 syscalls). Read at most once per watchdog interval, or only the renderer threads; keep the starvation verdict the same. Red first: a unit test that counts reads per frame (inject the tasks dir; the cfg(test) /nonexistent path gets a real temp dir), then the fix. Cell: gate-watchdog-starved if it exists, else name the cell that covers it.
2) L16: the logs dir chmod follows a symlink and an EPERM stops the log file: refuse a symlinked logs dir with one log line and go on without a file (POLS: the compositor never stops for a log). Red test first. Docs: voice.md rmcp default line and the glossary "at each start" as the review said.
3) F22: crates/chora/tests/log_file.rs:89-92 panic test fails about 1 in 5 under load: find the race (show it with a loop of 50 under `stress`-like load: `cargo test -p chora --test log_file -- --test-threads=8` in a loop), fix the test or the code, prove 50/50.
Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,e2e` green; merge origin/claude/autonomous-dev-session-3w-i33pg7 first if it moved; push the branch.
Report: reports/core-l.md "## r1" on claude/session-log (per item: open or closed, red sha, fix sha, cell, numbers before and after).
Send the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines with send_message. Poll every command in the same turn; never end a turn to wait.
Rules: commits as Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
red test commit before each fix; push only claude/core-l and claude/session-log (fetch and merge on reject, no force).
