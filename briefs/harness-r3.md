# harness r3 (rc 3, owner 11-15 02:10): the hook never blocks a valid commit

Lane: new opus worker, branch claude/harness (continue it; worktree /home/user/wt/harness3). One
round, small and fixed. Read dev-rules.md, conventions.md, then reports/review-harness-r2.md.
Start: merge origin/claude/autonomous-dev-session-3w-i33pg7 (newest) into claude/harness.
PM decision (11-15): the commit-msg red-first hook refuses only when it can prove the miss, that is
when the base ref resolves (base..HEAD has no test). In every fallback (no base ref, unborn HEAD,
shallow clone) it never refuses: it prints one note and lets the commit through; the merge gate
stays the enforcing check. KISS: fewer branches in the hook, not more.
Items, a red selftest case first for each (tests/merge-gate-selftest.sh):
1. B1': in a --depth 1 --single-branch clone without the base ref: test commit, push, docs commit,
   fix commit passes (with the note). Unborn HEAD: a first feat commit passes.
2. no-foreign-kill (tests/no-foreign-kill.sh): no false refusal of `pgrep -P"$pid"`, the
   /proc/<pid>/environ filter of session_pids, `pkill -F file`, text in echo or heredoc; catch plain
   `pgrep NAME`, `--exact`, `--full` and a `\` continued line. If a rule cannot be told apart
   reliably by grep, the hook warns for it instead of refusing (never block a valid commit).
3. xp.sh:654: two chora-xp children (a zombie and a restarted one) do not read as "died".
4. gate-0.8-plugin-runtime.sh:24-28: an allow line covers only the allowed item, not 40 lines.
Never-block proof: list every commit kind you ran (as in reports/harness-r2.md plus item 1).
Done when: tests/merge-gate.sh passes as one run (a known flake F16-F20: rerun once, name it).
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines) and reports/harness-r3.md.
Rules as briefs/harness-r1.md.
