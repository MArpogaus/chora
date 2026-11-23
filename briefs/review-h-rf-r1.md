# review h-rf r1 (rc 3, process; rule 8 guard): seeded review of claude/h-rf before its merge

Model: sonnet, effort high (harness shell rule with selftests; routine but exact)
Lane: cloud, read-only on code: claude/h-rf head fff3f1ab, base bc153929 (4 files, +215 -17). One round.
Stop at 120 k context; cost at most 0.4 %; at most 8 findings. No merge gate run (keys red is the known foreign sh-34; tests/merge-gate-selftest.sh only).
A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/h-rf-r1.md (done criteria) and reports/h-rf.md "## r1" on claude/session-log; `git diff bc153929..fff3f1ab`.
PM decision 11-23 14:50 (item 3): the rule stays strict (the red names the fix's ids). 0483bbeb and 3a625261 are history below every future base; not a finding.
Checks, each with its command and files:
1) Red first: 73525d9c holds the new cases and comes before fff3f1ab; at 73525d9c the per-id cases fail, at fff3f1ab all pass (`bash tests/merge-gate-selftest.sh`, once each).
2) Rule exact (tests/red-first.sh): id regex `[a-z]{1,6}-[0-9]+[a-z]?`; only fix commits; each id needs an earlier non-merge commit in base..HEAD naming it with a test change, or a test in the fix; `No test:` passes; merge and MERGE_HEAD pass; the hook refuses only what it can prove (no-base, base-branch, unborn HEAD exits kept). Find a false red on a valid commit or a false green.
3) Gate step (tests/merge-gate.sh red-first): same rule over the range, names sha and id; same code path as the hook (no second copy).
4) In flight: for claude/co-reach, claude/xp-readable, claude/leak, claude/guard-3, claude/docs-rc-r2 (fetch each): run the new rule over origin/claude/autonomous-dev-session-3w-i33pg7..branch; list each commit it would fail. Report, not a finding.
5) Scope: no change outside tests/red-first.sh, tests/merge-gate.sh, tests/merge-gate-selftest.sh, CONTRIBUTING.md.
Known, not findings: keys cell red (sh-34, fuzzel 1.9.2); steps after keys not run in the lane.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rhr-N, severity, file:line, text, fix in one sentence). Append to reports/h-rf.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list, item 4 list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
