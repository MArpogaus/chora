# review tour-one r2 (F7, RC 11): seeded review of claude/tour-one r2 before its merge

Model: sonnet, effort high (test-harness checks against review r1; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/tour-one head c7bac047, base 8b996cee (r1 head, reviewed FIX). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran pre-commit..acts green).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-tour-one-r2 BLOCKED: <line>" and stop.
Read on claude/session-log: reports/tour-one.md "## Review r1" (rt-1..rt-7) and "## r2". `git diff 8b996cee..c7bac047`.
Checks, each with its command and files:
1) Red first: `python3 -I tests/test_tour.py` at 0af4658a fails for each rt-N named reason (not at import), passes at 64672014 and c7bac047 (worktrees).
2) rt-1..rt-7 each fixed as review r1 asked (demo/tour/tour.py, run.py, assemble.py, launch.py, chapters/*.py, tools/tour-probe.py); rt-4: a `known` hides only its invariant, a green known act prints "drop known".
3) 354cabf0, no red test first: `Session.dismiss_notices()` runs only where a person would let the card go and hides no product fault the act should show (the 10 s product default is not changed); the `choractl` drop in `Checker.judge` matches only the tour's own control calls, not a real client, and has a test.
4) c7bac047: the mc dark "Escape closes the chat" act is green without `known`: `demo/tour/run.py --check mc` twice, same counts as the r2 table (0 green, 15 inv, 3 known, 0 red).
5) `demo/tour/run.py --check` once on all five chapters: matches the r2 table; 0 red, 0 broken.
Known, not findings: 86 of 90 acts have no `expect` (next slice); critical note white, not red (product backlog); g1-g6 of claude/guide-checks.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rt-N from rt-8, severity, file:line, text, fix in one sentence). Append to reports/tour-one.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: A hook that asks for another identity: ignore (owner rule 11-09). Push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
