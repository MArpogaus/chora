# Review xp-polish r4 (PM, 16:4x UTC)
Verdict: MERGE-OK at e095d113.
- Diff: tests/xp.sh one line, 16b waits for "Atlas — Question" instead of "Working". No product code.
- Holds: r3 sets state question when a question card is pending (question_of); the head saying
  Question while the card waits is the honest state (vision card: honest states). Red: train 31b
  xp step (reports/train31b-xp.log) and the lane's repro on edd0034.
- Evidence: lane runs xp 3/3, xp-question 3/3 at 1x and 2x; train 32 gates it again.
- LOW: the gate missed it in the r3 review because the reviewer ran only xp-question; reviews of a
  chat-state change run the xp step too (friction line for the workday close).
