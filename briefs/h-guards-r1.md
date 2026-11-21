# h-guards r1 (rules 9-10 guards; F4 ratchet; harness hygiene)

Model: sonnet, effort low (three small checks with selftests, one shellcheck fix)
Lane: cloud, claude/h-guards at a84effda for the code part; claude/session-log for tools/ there. Cost at most 0.82 %; stop at 150 k context. One area: guards (tools/lint-ratchet, tools/size-ratchet, tests/gate-injected-presses.sh on the code branch; tools/lanes.py and tools/hooks on claude/session-log).
Why: plans/week-3.md row 44 and backlog.md H-ID (line 331), the ratchet item (line 367), the SC2178 item (deps r1) on claude/session-log.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "h-guards BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Each item: red selftest commit first, then the fix.
1) Code branch, ratchet: a commit that raises a count in tools/lint-ratchet.list or tools/size-ratchet.list fails pre-commit (commit-msg stage) unless the message has a line `ratchet-raise: <reason>`. Falling counts need nothing. Selftest: a raised count without the line is refused, with it accepted, a lower count accepted.
2) Code branch, shellcheck SC2178 in tests/gate-injected-presses.sh:54 under `pre-commit run --all-files`: fix the code (no disable comment unless the warning is wrong; then name why in one comment).
3) claude/session-log, H-ID: tools/hooks pre-commit (installed by tools/install_hooks.sh) refuses a commit on claude/session-log whose author or committer is not `Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>`, and one with a `Co-Authored-By:` or `Claude-Session:` line. Selftest in tools/ (plain python3 or bash, no new package).
4) claude/session-log, tools/lanes.py: warn when the sum of the % on Queue lines in state.md exceeds (100 - week used - 5); read week used from the budget line ("week N %"). Selftest with a fixture state.md.
Show for each new check that a valid commit of today's log passes (run it on HEAD of claude/session-log).
Gate: full `tests/merge-gate.sh --jobs 1` (no --only) green on claude/h-guards merged with origin/claude/autonomous-dev-session-3w-i33pg7. A red cell you did not cause: name it with its log line; do not rerun to green. Start runs in the background and poll in the same turn; never end a turn to wait.
Done when: four items each with a red and green commit, the gate green, session-log hooks pass on HEAD.
Report: reports/h-guards.md "## r1" on claude/session-log (items, shas, selftest output lines, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with both head shas.
Rules: push claude/h-guards and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only tools/ and your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
