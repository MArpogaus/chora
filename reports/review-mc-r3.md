# Review mc r3 (claude/mc-fix 7d9fe3dc..8595d65e), opus, read-only, 11-14 18:5x UTC

Verdict: FIX. MC-1a, MC-1b, MC-1c are fixed correctly.
| id | sev | file:line | text |
|---|---|---|---|
| F1 | MAJOR | plugins/chora-mc/src/main.rs:1114-1126, 2687 | The take_free_keyboard guard is gone; chat_drawer treats "chat open on another output" (here == false) as closed, so keyboard_back takes the keyboard from the chat on the other output, and from a window the user clicked while the chat was open. Hand back only when the chat really closed (data.open != true), or skip when another client holds focus. |
| F2 | MINOR | main.rs:2684-2692 | A docked app that had the keyboard before the chat opened gets no keyboard back; it goes to the scene window (region::focus set self.focused to the dock). Regression of this branch. |
| F3 | MINOR | tests/gate-mc-paths.sh:239 | trap RETURN stays set; harmless (rmdir at start). |
| F4 | MAJOR follow-up | tests/gate-mc-paths.sh:224 (8595d65e) | Leading space works around a lost first key after hand-over; no cause found in mc or core (keys go to current_focus; focus moves in the same transaction). Probably foot or shell at enter or startup, unproven. Tracked as flake/finding F15 for the flakes lane: a dedicated check types one character after focus, reads it back, logs wl_keyboard enter and key events. |
Read only; no tests run.
