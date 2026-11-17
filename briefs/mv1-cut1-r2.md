# mv1-cut1 r2 (F4, F6): fix round after the review (FIX, one MAJOR)

Model: sonnet, effort medium (bounded fixes in two Python tools with selftests)
Lane: cloud, branch claude/mv1-cut1 (head 6d2ca7c2). One round. Stop at 250 k. Cost about 1 %.
Review (16:03 UTC, independent): MAJOR plus the MINORs below. Red test commit first (rules.md 8).

1. MAJOR tools/size-ratchet:119-120: drop the `elif listed[path] > before` branch; in --base mode
   compare against min(listed, before). A stale list entry after a split must not block later
   merges. Selftest: the stale-list-after-split case (list says 4272, base file 2000, head 2000)
   passes; growth above min still fails.
2. tools/size-ratchet.list sorted by path, and --update edits only the changed lines (three MV2
   split branches edit it in parallel; their edits must merge cleanly).
3. tests/merge-gate.sh:72: compare against `$(git merge-base "$base" HEAD)`, not the tip.
4. Selftests: each failing case greps its FAIL text (a Python traceback must not pass as a fail).
5. Document in CONTRIBUTING.md: test cut needs rustfmt form (`#[cfg(test)]` then `mod x {`);
   cut-check blocks dev-dependencies between plugins too.
Not in scope (backlog): CI --base, rename of a listed file, cut-check name boundaries.
Done when: selftests pass, tests/merge-gate.sh once on the head (the e2e apps-click step may fail
on fuzzel 1.9.2, F27: report it, do not work around it).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/mv1-cut1.md (append r2) on claude/session-log.
Rules as briefs/flakes-r2.md, push only claude/mv1-cut1 and that report; never end a turn to wait
for a long background job (poll it in the same turn). Commits by the owner identity, no
attribution lines, never --no-verify.
