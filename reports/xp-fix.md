# xp-fix (RC 2, shared bar): xp-11, xp-35, the r2 fix check and the persona harness gaps

## r1
Branch claude/xp-fix from 79c8d2b5, head a767790d. Gate (merge of origin/claude/autonomous-dev-session-3w-i33pg7 was a no-op): `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,acts,xp,xp-faces,xp-paperclip,xp-question,xp-fields,xp-notepad,doors,keys`: all PASS (acts 72 pass, 0 known red). Step 0: `cargo check --workspace` and `cargo build --workspace` green.

| Item | Red | Fix | Cell |
|---|---|---|---|
| xp-11 Help window | guard only, no red: Esc, the filled pane and the whole key table at 1280x800 already held (72b0c2e2 fits the table, b778b05a Escape order) | none | acts xp: Help and Support of the Start menu opens Help on a filled pane; the topic Keyboard shortcuts lists every key inside the window; Escape shuts Help (f72943f2) |
| xp-35 Starting balloon | f72943f2: I5 red (balloon over the 25% row), act:the balloon goes when the window of Notepad shows red, kit test `a_window_without_the_token_ends_the_message_of_a_start_that_shows_one` | 027a8218 (kit::launch and xp, not xp only) | acts xp I5 and the two acts pass, 4 known-red rows gone |
| xp-26 tooltip and tick | none, already guarded | none | acts xp: the tooltip of the speaker says the level that the key set / the popup set pass |
| xp-27 question card | none | none | cell gate-xp-question PASS |
| xp-33 Super+F1 | none | none | acts xp: the keys key opens the key list pass |

The xp-11 faults of the ledger text are gone on 79c8d2b5: the r2 capture (murat shot 005) already shows the whole key table, and the pane of the first topic is filled; the persona left Help open, so Esc was never tried by it. Cause of xp-35: kit::launch dismissed the Starting message only when the window carried the activation token; a window without one (the stand-in editor, many real programs) left it up for 30 s, and nothing took it away when a menu opened. Fix: `check` ends the message of a shown start once any window opened (Change::Done, the start still reports a failure), and `take_notes` hands the messages to xp, which dismisses them when a menu, popup, dialog or the switcher opens. The change is in kit, so tiling and mc get the first part too.

Harness (tests/persona, tests/acts; red 8d430a70, fixes 8f61c32b and a767790d):
- a. wpctl stand-in keeps the level and mute (selftest red at set-volume 80%).
- b. Notepad stand-in is a foot window that shows the typed text and appends it to notepad.txt (probe window when foot is missing).
- c. check.py `left_open`: a key list left open by the persona is no red of T1, T2, T4, X2, M2 (a menu or dialog stays red); unit tests in tests/test_acts.py.
- d. persona-probe reports to PERSONA_PROBE with the look's title bar; T5 uses it, not a bare chora-probe window.
- e. swaylock stand-in draws a window `Screen locked` for 8 s.
tests/persona-selftest.sh passes with all five. The acts session now has a `lowriter` stand-in (the first editor Notepad tries) that shows its window when an act lets it, and `notify_up` in tests/acts/looks/common.py.
