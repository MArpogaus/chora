# Review ux r1: claude/ux-r1 at ea2947fe (base d349a6da, 11 commits, 20 files)

Red first: every item has a test commit before its fix (mc-30 05be0631>e1256beb, sh-10 9b27e711>4a9a7fe2,
sh-11 409c6fcb>c2e80cad, sh-7 b1a185d7>5aaf7a13, sh-5 a51a1710>ea2947fe). Re-run here: the unit reds fail
on their test commit (3 chat tests at 409c6fcb, the xp header test at 9b27e711). Gate cells were not re-run.
Unit tests pass at HEAD: chora-assistant, and the keys tests of mc, tiling and kit.
The brief's done-when is not met: merge-gate.sh did not pass as one run (builder report).

## MAJOR
M1 sh-11: an abandoned text is sent to a helper that nobody picked. plugins/chora-assistant/src/chat.rs:701-707
   sends the draft whenever `held` is set and a chat opens. `held` is cleared only in open_list (:765),
   not in hide (:866), close_view (:875, Esc/FieldEvent::Closed) or the only-helper path of open_list (:757-759).
   The field keeps its draft across hide. Scenario: two helpers, Super+a, type "rm -rf build", Enter (given
   back), Esc. Later a click on the row of helper "second" (chat/host.rs:648), a question that opens the
   balloon (voice.rs:230, no act of the person), or a chat.show RPC (main.rs:311) opens a chat. The stale
   text goes out with no Enter. Reproduced with a unit test in a scratch copy:
   clicked_row, Say("moin"), hide, show(Some("atlas")) gives sends == 1 (expected 0). This is the case that
   gate-chat-keys forbids ("a text went to a helper that nobody picked").
   Fix: keep the text inside the list, e.g. `Mode::List(picks, Option<String>)`, so it ends with the list.
   Earliest gate: compile time (that type makes it impossible). Otherwise a chat.rs unit test with the fake Link.

## MINOR
m1 chat.rs:780-781 (sh-11): with one helper and a project to start, the list still shows. The brief says
   "exactly one helper -> the chat". The builder's reason holds (else no project could start), and it is
   reported. The PM should confirm.
m2 chat.rs:983-986: after a held text, a helper name also counts as the pick. "ask" then "ask atlas"
   opens atlas and sends "ask". A number that is out of range ("hallo9") is dropped with no word.
m3 region.rs:661-664 (sh-7): with all keys shown, every content except state/keys/doors disappears from
   every side region, Left and Drawer too. The docs (configuration.md, mc-style.md) say only "the column".
   As UX, a transient help sheet that has the side to itself is acceptable (Esc restores it). Narrow the
   code or the docs.
m4 mc keys.rs:160 "Give window to a look": the keys are named, but not which digit goes to which look.
   Tiling names all four. The width reason is plausible. Acceptable as a reported deviation, but
   "1 tiling 2 xp 3 mc 4 construct" (or the tiling words in two rows) would be POLS.
m5 Process: 5aaf7a13 (the fix) brings a second behaviour (the full list hides the side) with its own new
   test, audit.rs:276, in the same commit, with no red. c2e80cad rewrites the construct helper-door
   assertion (construct.sh:178) inside the fix commit.
m6 tests/doors/mc.sh header_overlaps: it checks overlap only, not a cut. mc-30's "date cut to SUNDAY 15
   NOVEM" is not asserted, and the narrower header over an app can now ellipsize the date. With the 6 s
   retry, a one-frame overlap passes. The captures named in the report are not reachable from here.

## COSMETIC
c1 chora-kit give.rs:134 keys_row has no unit test, and its `to` parameter is always "-".
c2 sh-10 splits any header at the first ". " (chat.rs:1140). That is correct for today's words. A future
   notice with an abbreviation ("z. B.") would split wrongly.

## Checked, no finding
sh-10: Stop takes room only in a chat (view.rs:824), and the test asserts both lines fit and are whole
(EN/DE, online/offline). The German line is shorter and still honest. sh-5: the capsule hides only in
Mode::Use. Stood back (Navigate) it shows, which matches the walk (POLS ok), and the cell checks the clock,
Lock and the step in. mc-30: app_lines ends the header at the column (also under voice_hud: host) and
the footer only with the column. That is one rule, as asked. sh-7 tiling: "Give window to tiling, xp, mc or
construct", in German too. ti-4: no change.

BLOCK: none. MAJOR: M1 (earliest gate: compile time, the held text owned by Mode::List).
Verdict: FIX
