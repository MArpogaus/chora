# review sh42-fix r1 (rc 1: sh-42 MAJOR): seeded review of claude/sh42-fix-r1 before its merge

Model: opus, effort high, because the diff changes the shared kit focus rule (Focus::sends) every look uses and mc's click focus; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/sh42-fix-r1 head b92de049, base c3c37c2e (5 files, +116 -3). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-sh42-fix-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the train runs it).
Read briefs/sh42-fix-r1.md (done criteria), reports/sh42-fix.md "## r1", reports/sh42-repro.md "## r2", findings sh-42 on claude/session-log; `git diff c3c37c2e..b92de049`.
Checks, each with its command and files:
1) Red first: a04b395e, 68a4ff1b, db469100 (tests only) before 7a110c34, b92de049 (`git log --oneline --stat c3c37c2e..b92de049`). A fix with no red test before it is MAJOR.
2) Kit (focus.rs:167 Focus::sends): a look that never sent a focus no longer sends focus:null on relayout. Can any look now keep a stale focus it should clear (window closed, look switched, lock)? Every caller of Focus::sends checked; `cargo test -p chora-kit`.
3) mc (manage.rs:218): a click after a null focus re-sends focus; no double send in the normal case; D119 (mc takes no free keyboard beside another look) still holds. `cargo test -p chora-mc`.
4) Deviation: Return in mc cannot recover focus (no holder); the brief asked for it. Is the deviation sound per D119 and POLS, or does a user in mc after a tiling join now type into nothing with no visible hint (then MAJOR)?
5) Gate: the lane got all 65 PASS only across one stopped run and 2 --only runs (not one full run). Not a finding by itself (the train runs the full gate); name any cell whose result depends on order.
6) gate-handover-shared.sh: new cells assert the lost-key case for xp and mc join, no sleep beyond the ratchet, no flake path.
Known, not findings: co-77, co-78, co-79, rss-11..13.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rsf1-N, severity,
file:line, text, fix in one sentence). Append to reports/sh42-fix.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
