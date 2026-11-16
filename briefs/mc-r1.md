# mc r1 (rc 1, 11): the BLOCK and MAJOR of journey 1 in chora-mc

Model: sonnet, effort high (one BLOCK of unknown cause plus five UI fixes in one look)
Lane: cloud, branch claude/mc from the session branch (caf9c80d). One round.
Stop at 250 k context. Read reports/journey-1.md, findings/mc.md mc-32..mc-37, the stills named
there (media/2026-11-16/tour-v4/stills on claude/session-log) and demo/tour ch3 scripts on
claude/media (2ef891d) for the exact inputs.

1. mc-32 BLOCK first: remote Right/Home do not turn the page in dark mc. Reproduce with the same
   injected keys as the tour script in a gate cell. If the cell passes, the fault is the script:
   close mc-32 as not real with 10 passes and name the script line. If it fails: red cell, fix.
2. mc-35 MAJOR: a critical notice in mc is a readable card (not a truncated footer line); the
   column keeps its layout (no divider through Chat, KEYS not on top, no empty NOTES heading).
3. mc-34 MAJOR: the voice HUD (heard text, answer) is legible at a distance: at least the body
   size of the page, not grey footer text.
4. mc-36 MAJOR: the helper pill shows the helper's real state (no "No helper" while it answers)
   and never overlaps the status word.
5. mc-37 MAJOR: Super+n widens the page to the freed space.
6. mc-33 MAJOR: the chat door by mouse opens the chat.
For each: a red gate cell or unit test first (its own commit), then the fix; no tolerance or
retry. Pixel-check the light and dark themes (captures in the report). Done when: the new cells
pass 10 times, tests/merge-gate.sh passes once. Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 8 lines) and as reports/mc-r1.md on claude/session-log:
each claim with its gate cell and capture. Rules as briefs/flakes-r2.md, push only claude/mc and
that report; never end a turn to wait for a long background job.
