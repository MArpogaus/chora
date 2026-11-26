# h-rf2 r3 (rc 3, F6; harness): fix review r2 of the red-first run step

Model: sonnet, effort high (routine shell fix; the finding names its fix)
Red on head: rhrf2r2-1 MAJOR (tests/red-first.sh:347-352 on 59ba5c21 flags a valid commit: passing added test beside an edited test; reproduced by review r2 in a temp repo, rc 1)
Lane: cloud, branch claude/h-rf2-r3 at 59ba5c21e35ae1ecefe5b421ce9e148456208287 (claude/h-rf2-r2 head). Area: tests/red-first.sh, tests/red-first-selftest.sh; no product code. Stop at 250 k. Cost at most 0.4 %.
Why: reports/h-rf2.md "## Review r2" on claude/session-log (35183815e), findings rhrf2r2-1 MAJOR, rhrf2r2-2 MINOR (same lines, same rule).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "h-rf2-r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1. Red first, its own commit: selftest cases that fail on 59ba5c21: a passing added test beside an edited test must not fail (rhrf2r2-1); a passing crate beside an unproven crate must not fail (rhrf2r2-2).
2. Fix as one rule (KISS): a pass is excused whenever another candidate of the same fix was not run (gate cell, edited test, unproven); print its not-run line, count it not run, do not set bad. Replace the separate gate-cell special case with that rule, not add a third.
3. Real range 3be27e32..4d8bbe36: still rc 0, "2 proven red, 6 not run" (or each change named); time.
Not in scope: rhrf2r2-3 NIT (backlog), the merge-gate.sh:52-57 comment.
Run: the full gate, `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push) detached with setsid nohup, poll in the same turn; paste the last 20 lines. Known flakes sh-37..sh-45, launcher-mc mc dark: name with the log line, rerun that step and the ones after it. Never end a turn to wait. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: red selftest commit before the fix; rhrf2r2-1, -2 closed; range result in the report; gate EXIT 0 (or a named foreign flake and every later step run); reports/h-rf2.md "## r3" on claude/session-log (base, head, diffstat, results).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): head sha, red commit, findings closed, gate result.
Rules: push claude/h-rf2-r3 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
