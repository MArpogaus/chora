# harness r2 (rc 3, owner 11-15 02:10): close the review of harness r1

Lane: same worker (opus) if under 250 k context, else a new opus worker; branch claude/harness.
One round, fixed. Read reports/review-harness-r1.md first. Start: merge
origin/claude/autonomous-dev-session-3w-i33pg7 (c2d0a116 or newer: the tiling-audit fix).
Items, a red selftest case first for each (tests/merge-gate-selftest.sh):
1. B1 (BLOCK): the commit-msg red-first hook never blocks a valid commit. When the base ref does not
   resolve, fall back to `git rev-list HEAD --not --remotes`, and if that is empty too, let the commit
   through with a one-line note (the merge gate stays the backstop). Selftest cases without
   RED_FIRST_BASE and without the remote ref: test commit then fix commit passes; amend and rebase
   reword of such a fix pass.
2. M1 (MAJOR): with --jobs, no step judges a process by name machine-wide. xp.sh checks the PIDs it
   started (kill -0 or pgrep -P); sweep the other steps for pgrep/pkill by name and fix the same way,
   or list them in serial_steps with a reason. A selftest or a grep check keeps it so.
3. m1: the hook returns 0 when MERGE_HEAD exists. m2: disallow Conn::connect_at and Runtime::with in
   plugins/clippy.toml (an #[allow] with a reason at the two test sites) and in the grep backstop.
   m3: clippy-config-sync checks both directions. m4: 2d-guard runs before the parallel steps.
   Cosmetic: CONTRIBUTING.md:239 width and the measured minutes.
Never-block proof (routines.md): list in the report each commit kind you ran through the hook
(feat, fix, test, ci, docs, chore, revert, fixup, merge, amend, reword, shallow clone, no remote).
Done when: tests/merge-gate.sh passes as one run (also once with --jobs 3; a known flake F16-F20:
rerun once, name it). Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines) and as
reports/harness-r2.md on claude/session-log. Rules as briefs/harness-r1.md.
