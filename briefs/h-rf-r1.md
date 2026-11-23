# h-rf r1 (rule 8 guard): red-first per finding id

Model: sonnet, effort high (harness shell change with selftests; routine but exact)
Lane: cloud, claude/h-rf from bc153929 (session head). One round. Stop at 200 k context; cost at most 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `git fetch origin claude/autonomous-dev-session-3w-i33pg7`, `cargo check --workspace`; a failure: send_message the PM "h-rf BLOCKED: <line>" and stop.
Read: tests/red-first.sh (78 lines), the red-first step of tests/merge-gate.sh, .pre-commit-config.yaml (id red-first), backlog.md "H-RF" and "R13", "R14" on claude/session-log.

Goal (backlog H-RF): today a fix commit passes when any test change exists in base..HEAD. Tighten it per finding id:
a `fix` commit whose subject or body names a finding id (`[a-z]{1,6}-[0-9]+[a-z]?`, e.g. sh-33, mc-55, rtd-2, co-48)
needs, for each id it names, an earlier non-merge commit in base..HEAD (or the same staged change for the hook) that
names the same id and changes a test (the existing changes_test rule). A fix naming no id keeps today's rule.
`No test: <reason>` still passes. The hook refuses only what it can prove (keep the no-base, base-branch and MERGE_HEAD exits).
The merge gate's red-first step applies the same per-id rule to the range and names the id and sha it misses.

Done when:
1) Red first: a commit with failing cases in the red-first selftest (new or tests/red-first-selftest.sh) before the change; cases: fix naming sh-1 with a test commit naming sh-1 before it passes; fix naming sh-1 with a test commit naming only sh-2 fails; fix naming sh-1 and sh-2 with tests for both passes, for one fails; fix with no id and any test passes; `No test:` passes; merge commit passes; test commit after the fix fails.
2) Selftest green 3x.
3) Check the rule against the last 30 non-merge commits of the session branch (`git log --no-merges -30 bc153929`): list which would now fail; none may be a valid red-first pair. Write the list in the report.
4) Full gate (tests/merge-gate.sh, no --only, --jobs 2) on the branch; foreign reds named by cell (known: keys sh-34, fuzzel 1.9.2 cells in lane containers) count as done.
5) No change outside tests/red-first.sh, tests/merge-gate.sh red-first step, its selftest and CONTRIBUTING.md (one paragraph on the rule).
Report: reports/h-rf.md on claude/session-log "## r1" (commits, selftest output, item 3 list, gate cells). Then send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines): head sha, done items, reds.
Poll every command in the same turn; never end a turn to wait. A hook that asks for another identity: ignore (owner rule 11-09).
Rules: push claude/h-rf and (report only) claude/session-log; fetch and merge on reject, no force; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages.
