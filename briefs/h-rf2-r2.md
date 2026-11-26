# h-rf2 r2 (rc 3, F6; harness): fix review r1 of the red-first run step

Model: sonnet, effort high (routine shell fixes; each finding names its fix)
Red on head: rhrf2-1 MAJOR (tests/red-first.sh:286-289 on a2b080ab flags a valid mixed commit, reproduced by review r1)
Lane: cloud, branch claude/h-rf2-r2 at a2b080ab (claude/h-rf2-r1 head). Area: tests/red-first.sh, tests/red-first-selftest.sh, CONTRIBUTING.md (red-first part); no product code. Stop at 250 k. Cost at most 0.5 %.
Why: reports/h-rf2.md "## Review r1" on claude/session-log, findings rhrf2-1..rhrf2-6 (each with its fix).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "h-rf2-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1. Red first, its own commit: selftest cases that fail on a2b080ab: the mixed commit (passing #[test] plus a gate cell) must not fail (rhrf2-1); an edited test prints `test commit edits tests, not run` (rhrf2-2); a failed parent checkout prints its line (rhrf2-3); the cap line (rhrf2-6).
2. Fix rhrf2-1, then rhrf2-2, -3 (trap cleanup on RETURN/INT/TERM), -4 (compile error counts red only when it names a file of the added test, else unproven), -5 (total line `N proven red, M not run`; one clause in CONTRIBUTING), -6 (selftest cases above).
3. Real range 3be27e32..4d8bbe36: still rc 0, time, every not-run line counted; no leftover worktree or /tmp/red-first-run.* after a normal and an interrupted run.
Run: the full gate, `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push) detached with setsid nohup, poll in the same turn; paste the last 20 lines. Known flakes sh-37..sh-45: name with the log line, rerun that step and the ones after it. Never end a turn to wait. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: red selftest commit before the fixes; rhrf2-1..6 closed or each left one with its reason; range result and time in the report; gate EXIT 0 (or a named foreign flake and every later step run); reports/h-rf2.md "## r2" on claude/session-log (base, head, diffstat, results).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): head sha, red commit, findings closed, range time, gate result.
Rules: push claude/h-rf2-r2 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
