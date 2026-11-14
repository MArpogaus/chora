# Review mc r4 (claude/mc-fix e9288bc9..265d697a), opus, read-only, 11-14 19:0x UTC

Verdict: FIX.
| id | sev | file:line | text |
|---|---|---|---|
| A | MAJOR | plugins/chora-mc/src/main.rs:856-861 | focused_under_chat is only set, cleared only when the chat reopens; focus.changed(null) is dropped. Click s9 under the chat, s9 closes: no hand-back, keyboard with nobody. Click s9 of scene A, turn to page B with the chat open, close: the hidden window of A keeps the keyboard. |
| B | MINOR | main.rs:857 | Simpler rule: keep core_focus from every focus.changed (null included); on close hand back only if core_focus is None or equals what mc had when the chat opened. No second field. |
| C | MINOR | main.rs:1131 | A chat that moved to another output and closes there: mc never hands back on its own output; say so in a comment or test. |
| D | MINOR | tests | F1/F2 only with a fake core; no cell for click-then-close or click-then-turn. |
Fine: no loops; the chat field's node focus emits no focus.changed; mc may subscribe; the trap removal.
PM note: four rounds of FIX on one finding; fixes add state. r5 takes rule B (replace, not add) and real gate cells, on opus.
