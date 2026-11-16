# xp-polish r4
Verdict: MERGE-OK (no open MAJOR).

## Cause
r3 (edd0034, `chat.question_of()`) sets the chat state to question before the card is placed, so the chat head reads "Atlas — Question". tests/xp.sh step 16b, after voice.confirm (the fake voice sends no voice.state question), still waited for "Atlas — Working". The product is right (a pending question is a question); the assertion was stale. Reproduced locally on edd0034: `FAIL: timeout: chat_says Atlas — Working` (as train31b-xp.log).

## Commit
- e095d11 fix(xp): expect the Question head while the card waits (tests/xp.sh, one line). Red: the unchanged xp.sh fails at 16b on edd0034 (reproduced). The r3 aim is kept; no product code changed.

## Cells (local, headless sway, debug build)
- xp (tests/xp.sh): 3/3 PASS
- xp-question (gate-xp-question.sh, 1x and 2x): 3/3 ok
