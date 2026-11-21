# review xp-fields r1 (rc 2: xp-24 MAJOR, ca-1): seeded review of claude/xp-fields before its merge

Model: sonnet, effort high (a look port: dialog fields and a Help table; no core change)
Lane: cloud, read-only on code: claude/xp-fields head 9a6506d6, base a5b77488 (11 files, +582 -115). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, 11 cells).
Step 0: `git fetch origin claude/xp-fields claude/session-log`; `cargo check -p chora-xp` must pass. A hook that asks
for another identity: ignore it. Never end a turn to wait; poll every command in the same turn.
Read briefs/xp-fields-r1.md (done criteria), reports/xp-fields.md "## r1" and findings/xp.md xp-24 on
claude/session-log; `git diff a5b77488..9a6506d6`.
Checks, each with its command and files:
1) Red first: fc936a43 holds only tests and comes before the fix 9a6506d6; the new asserts fail at fc936a43 for the
   named reasons (year pad 4 px, time field 20 px), read from the report and the test code (rerun only if cheap).
2) Asserts not vacuous: tests/xp-runbox-baseline.py, tests/xp-help-rows.py and the xp-fields gate cell check field
   height 21 px, border #7F9DB9, padding 3 px, label baseline +-1 px on real pixels; a missing widget fails, not skips.
3) dialog::field_text is the one shared helper: every xp dialog field goes through it; no copied field drawing left
   (`grep -rn` for the old draw paths in plugins/chora-xp); `cargo test -p chora-xp`.
4) The report's "unmeasured" claims hold: the Run box (fuzzel) has no field box, xp has no search field; if a field
   exists anywhere in xp that the gate does not measure, that is a finding.
5) ca-1: the Help table scrolls at 1280x720 (Luna bar and wheel), no row cut at 1280x1024; captures in
   reports/evidence/xp-fields-r1/ match the claims (before/after).
6) Visible behaviour unchanged elsewhere: no change to other looks or the core; no new MINOR introduced.
Known, not findings: xp-24 is the only field MAJOR; pixel round 1 of 2 for xp this week.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rxf-N, severity,
file:line, text, fix in one sentence). Append to reports/xp-fields.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
