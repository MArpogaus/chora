# mc r2 (rc 1, 11): the open MAJOR of chora-mc after r1

Model: sonnet, effort high (host HUD schema change shared with every look, plus layout guard)
Lane: cloud, branch claude/mc, continue from c2c9c353 (r1, review MERGE-OK; merge the session
branch first). One round. Stop at 250 k context. Read reports/mc-r1.md, reports/review-mc-r1.md,
findings/mc.md mc-34, mc-35 on claude/session-log.

1. mc-34 MAJOR: the heard words and the answer are legible at a distance in mc: at least the body
   size of the page, not 11 px grey. The host draws them (plugins/chora-assistant/src/assistant.rs
   plain(), Font::Ui): add a size key to the hud.yaml schema (crates/chora-kit/src/hud.rs,
   docs/hud.md) with today's value as default, so xp, tiling and construct do not change
   (their pixel cells must stay green unchanged); mc sets a large size. Draw `said` of
   voice.state too if the schema already names it.
2. mc-35 rest: a critical notice in mc is a readable card, not the truncated footer line; the
   column never overflows (helper row plus two long notices: the Chat door stays above the
   rule of KEYS). Change the gate-mc-paths `urgent` cell first (red), then the fix.
3. The three LOW of review-mc-r1 if cheap (doc comment of `lead`, the mic.rs line).
For each: red gate cell or unit test first (its own commit), then the fix; no tolerance or retry.
Captures light and dark. Done when: the new or changed cells pass 10 times, tests/merge-gate.sh
passes once. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 8 lines) and as
reports/mc-r2.md on claude/session-log, each claim with its gate cell and capture. Rules as
briefs/flakes-r2.md; push only claude/mc and that report; never end a turn to wait for a long
background job.
