# lint-down-r3 (F4 guard: the lint ratchet list matches the head it merges onto)

Model: sonnet, effort medium (a recount and one help line; the PM reviews)
Lane: cloud, branch claude/lint-down, continue at 2254d32a. One round, stop at 250 k. Cost about 0.3 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true` and `cargo check --workspace`; a failure: send_message the PM "lint-down BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/lint-down.md "## PM review r2" (rv-4, rv-5).
Scope: tools/lint-ratchet, tools/lint-ratchet.list, tests/lint-ratchet-selftest.sh. No Rust changes.
Items:
1. rv-4: merge origin/claude/autonomous-dev-session-3w-i33pg7 into claude/lint-down. Then recount in a scratch merge of that branch with these queued shas, in this order: fee9c695 e0c3a266 b0997b71 (fetch each by sha from origin; a conflict: stop and report it). Commit the list of the scratch merge on claude/lint-down with `tools/lint-ratchet --update`. Report three counts: session head alone, scratch merge, and the delta per sha.
2. A count over the list prints which counter, the list value, the tree value and the files that add the most (top 5), so a train red names its cause in one line. Red first in tests/lint-ratchet-selftest.sh (the fixture over the list must show the file name), then the change.
3. rv-5: the help says a comment on the line above or next to the attribute counts as a reason.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy`; poll it in the same turn; never end a turn to wait. Show a valid commit is not blocked.
Done when: items 1-3 with shas, the selftest red then green, gate cells pass, reports/lint-down.md "## r3" on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/lint-down and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages; red test commit before each fix.
