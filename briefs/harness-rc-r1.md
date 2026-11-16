# harness rc r1 (rc 1, 3, 4): three test-side fixes and the fuzzel pin

Model: sonnet, effort medium (test scripts and one packaging line; causes are named)
Lane: cloud, new branch claude/harness-rc from the session branch
claude/autonomous-dev-session-3w-i33pg7; first merge origin/claude/fuzzel-click (tests only, f11eb6c).
One round. Stop at 250 k context. Read dev-rules.md and conventions.md (with the skills it names),
then backlog.md F16, F25, F26, reports/fuzzel-r2.md. Setup as briefs/flakes-r2.md
(setup.sh builds fuzzel 1.11.1; `git fetch --unshallow` if 2d-guard lacks its base).

Items, in order (each with its red evidence first, as a commit or a logged failing run):
1. F25: tests/agent.sh step 6 compares a capture and a grim taken at two moments. Wait for the
   bar (the condition, not a sleep) before step 6, or take both in the same frame. No tolerance.
2. F16/F26: gate-tiling-audit.sh sends waybar to /dev/null and its probe loop ends silently;
   doors step "missing" cells. Make the wait loud (log waybar's output, fail with the reason),
   then fix the cause the log shows; 10 serial runs of the audit's first config and of the doors
   step, all green, with the counts in the report.
3. ti-1: register tests/gate-tiling-apps-click.sh (and gate-tiling-apps.sh if unlisted) in
   tests/merge-gate.sh and the CI e2e list; require fuzzel >= 1.11 in the rpm package like
   dist/arch/PKGBUILD does.
Done when: items 1-3 pass and tests/merge-gate.sh passes once. Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines) and as reports/harness-rc-r1.md on
claude/session-log, with the earliest gate per item. Rules as briefs/flakes-r2.md, with push only
claude/harness-rc and that report; never end a turn to wait for a long background job.
