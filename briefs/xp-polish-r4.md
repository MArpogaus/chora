# xp-polish r4 (rc 1, 3)
Model: sonnet, effort medium (one bounded regression in one cell)
Branch: claude/xp-polish (head edd00348). Base: session branch 6a067e39.

## Task
Train 31b (merge of edd00348 on 6a067e39) failed the gate step `xp`:
`== 16b. the chat has XP's form ...` -> `FAIL: timeout: chat_says Atlas — Working`
(log: reports/train31b-xp.log on claude/session-log). Train 31 (r2 only, 5a17aab6 era) passed `xp`,
so r3 (22ed0a03, edd00348: `chat.question_of()` sets state question early; `apply(top)` first)
is the likely cause. Find the cause, red test first (a commit that fails), then the fix.
Keep the r3 aim: card and hint in one frame (xp-question cell 10/10 at 1x and 2x).

## Done when
- tests/merge-gate.sh steps `xp` and `xp-question` pass 3 runs each locally.
- reports/xp-polish-r4.md: cause, commits, cell runs. Push branch and report, hand back.
No other scope.
