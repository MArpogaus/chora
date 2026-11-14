# Review mc r5 (claude/mc-fix 265d697a..39ceb8cb, whole branch vs 498d9dea), opus, read-only, 11-14 19:5x UTC

Verdict: MERGE-OK. One field (core_focus), every r1-r4 case covered; main.rs down 16 lines net; red tests real; the gate needing chora-tiling for a second output is sound (built by the workspace build, pid cleaned up).
| id | sev | file:line | text |
|---|---|---|---|
| R5-1 | MINOR | plugins/chora-mc/src/main.rs:2703 | dock had the keyboard, chat opens, user clicks the scene window under it: on close the keyboard goes back to the dock (265d697a kept the clicked window). Hand to the dock only if core_focus is not the current scene's window. |
| R5-2 | MINOR | main.rs:2703 | the dock check matches any dock, not one on the current scene; same as without the chat. |
| R5-3 | MINOR | tests/gate-mc-paths.sh:342 | the click lands before tiling's window maps; wording or click after map. |
| R5-4 | MINOR | gate-mc-paths.sh:318 | chat-click-turn is a guard against rule B, not red on 265d697a; say so. |
MINORs go to backlog (rules.md 6: mc has open MAJORs elsewhere).
