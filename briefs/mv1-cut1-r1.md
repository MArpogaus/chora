# mv1-cut1 r1 (F4, F6): size ratchet and the cut gate dependency check

Model: sonnet, effort medium (two bounded tools with selftests; design fixed in the plan)
Lane: cloud, branch claude/mv1-cut1 from the session branch c78eca98. One round. Stop at 250 k. Cost about 2 %.
Read reports/arch-plan-fable.md (sections 2.4, 4: MV1, CUT1; section 6), reports/review-arch-plan-risk.md (R3), reports/debt-rc.md (the line count command), dev-rules.md.

1. MV1: a size ratchet on production lines (tests and test modules excluded, the debt-rc command).
   A list of files over 1,500 lines with their count; a merge may grow a listed file only if it also
   extracts from it (net drop of the file); a new file over 1,500 fails. Wired into ci-check and
   tests/merge-gate.sh. Selftest: a planted growth fails, an extraction passes.
2. CUT1: tools/cut-check: plugin-to-plugin dependencies block (cargo metadata); assets, names,
   packaging and copies (copies.toml rows) only report. Wired into ci-check and the merge gate.
   Selftest per check (a planted breach fails or is reported).
3. Both run in under 10 s; neither blocks a valid commit of the head (show it).
Done when: selftests pass, tests/merge-gate.sh once on the head (one run, no wrapper).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/mv1-cut1.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only claude/mv1-cut1 and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
