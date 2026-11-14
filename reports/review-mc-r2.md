# Review mc r2 (claude/mc-fix 13b065c1..a39fbdb7), opus, read-only, 11-14 18:2x UTC

Verdict: FIX.
| id | sev | file:line | text |
|---|---|---|---|
| MC-1a | MAJOR | plugins/chora-mc/src/main.rs:1124,1133-1135 | Stuck focus: the skipped focus change is never applied. Chat open on mission, turn to terminal, close the chat: take_free_keyboard returns early (windows has "terminal"), the terminal gets no keyboard until a click. Reverse (chat on terminal, turn to mission, close): the hidden terminal keeps the keyboard. Fix: on chat close apply the deferred keyboard_to(current, self.focused.clone()). |
| MC-1b | MINOR | main.rs:2834 | Same gap when a started window appears while the chat is open (show(name, true)); needs the same hand-back on close. |
| MC-1c | MINOR | tests/gate-mc-paths.sh:263 | The cell ends with Escape and never checks that the terminal gets keys after the chat closes; add that, and a unit test of keyboard_to/chat_drawer with Chat::Open. |
| MC-2 | OK | tests/merge-gate.sh:10,22,160 | step wired right; red-first holds. |
Read only; no build.
