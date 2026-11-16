# security-a r5 (branch claude/security-a)

Commits on top of 250b3716 (plus a merge of the session branch): b049f22d test (assistant, mc, kit), 148f79d9 ci (optout, guards).

1. MAJOR closed. Assistant tests drive Agent::event on a fake core: a PointerPressed on hud-row-chat with injected=false opens the chat, injected=true does not; the same for hud-figure (chats on). Mutation: replacing chat_row() in chat_pressed with a no-op fails the row test; making refuses_injected unconditional fails it too. The hud gate cannot mark a press as a person's (its core runs without --test-injected-press-is-real and every press is input.inject), so no positive hud cell was restored; the unit test is the positive proof. Backlog line left.
2. MINOR closed. injected-optout.py accepts only `injected`, `true`, `false` or an expression (contains . & : + ! < > = if or a call). `ref _x`, `x @ _`, `(_)`, `_ | true` and a plain name fail; five guards.sh cases added, the old "another name passes" case now expects fail.
3. MINOR closed. assistant (injected pointer.entered on the Close button, no hover mark) and mc (injected pointer.entered on a hit, mark stays) driven tests; deleting the hears_pointer line fails the mc test and the assistant one (assistant checked by mutation too). Kit: hears_pointer is checked against event.rs (every flagged event is listed unless a plugin judges it itself) and driven with injected events.
Checks: clippy --workspace --all-targets -D warnings clean; guards.sh green; tests/merge-gate.sh once: all steps passed (run /tmp/merge-gate.rhB6aM). First gate attempt failed at 2d-guard only because the clone was shallow (fixed by git fetch --unshallow).
Not run: r3 point 9, the two host-bound tests (out of scope).
