# security a r4 (rc 6): close the injected-input class in the type, not per path

Model: sonnet, effort high (because a core event type changes across all plugins; the cause and the fix are named)
Lane: cloud, branch claude/security-a (85476db0); merge the session branch first. One round.
Stop at 250 k context. Read dev-rules.md and conventions.md (with the skills it names), then
reports/review-security-a-r3.md (FIX: 2 MAJOR, 6 MINOR) and briefs/security-a-r3.md.

Class invariant (unchanged): with test_accept_injected_keys off, an injected key, press, move,
scroll or enter changes nothing a person would see. Round 3 closed paths whose events carry the
flag; the open paths are events WITHOUT the flag. So the fix is in the type:
1. Red first: a core unit test (review point 1): injected Super + click must not reach a button
   grab as a person's input.button. A tiling unit test: an injected pointer_move with
   focus: follow_mouse must not move focus (point 2).
2. Every input event of crates/chora-scene/src/event.rs that a person can cause carries
   `injected` (InputButton, InputMoved, PointerEntered, PointerScroll, and any other without it;
   list them in the report). The compiler then names every match; each plugin drops injected
   events unless the switch accepts them, through the existing from_person / hears helpers.
3. tests/injected-optout.py: accept only `injected` or `injected: true|false` in a pattern;
   refuse `injected: _name` and the flag bound into another field; guards.sh cases (point 5).
4. Wiring tests (point 6): drive Wm::event / Tiling::event / construct input with an injected
   event and assert nothing changes; a notify test (point 7); chora-assistant ROW, CLOSE and
   helper rows refuse injected presses (point 4); gate-injected-presses.sh sends Tab once (point 8);
   add a follow_mouse cell.
Not in scope: point 9 (COSMETIC, backlog).
Done when: items 1-4 pass, clippy -D warnings, tests/merge-gate.sh once. Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines) and as reports/security-a-r4.md on
claude/session-log, with the earliest gate per item. Rules as briefs/flakes-r2.md, with push only
claude/security-a and that report; never end a turn to wait for a long background job.
