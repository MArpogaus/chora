# tiling-minor

## r1

Branch claude/tiling-minor-r1, head 31af60c5, on 84f66d43. Red commit before
each fix (the gate step for rth-5; the unit test for the others).

| finding | red sha | fix sha | test |
|---------|---------|---------|------|
| rth-1 | 8e2052a7 | 6b199a70 | input.rs `a_press_on_a_bar_outside_the_claims_goes_to_the_look_of_its_output`, `..._where_no_claim_lies_goes_to_every_look` (two managers; the far one connected first). The act goes to the claim at the point, else the managers whose claim overlaps the output of the layer surface, else every live manager. |
| rth-2 | docs only | 6a801c0c | api.md, architecture.md, state.rs `Act`/`note_act` name a real press on a layer surface |
| rth-3 | ab0738de | (tests only; they pass at once) | input.rs `a_real_press_on_a_bar_is_an_act_of_the_look_of_the_claim`, `an_injected_press_on_a_bar_is_no_act`, `a_press_on_a_bar_while_locked_is_no_act` |
| rth-4 | 9fe4649a | bcc8cb4f | main.rs `a_failed_reload_without_a_notice_server_shows_the_reason_on_the_chip`; reload.rs shows "title: reason" (70 chars) on the chip for 5 s when `show_or_log` gives None |
| rth-5 | b8b2d3b5 | 87aa0d1e | tests/gate-tiling-reload-notice.sh last step: a good reload leaves no notice within 8 s (the card expires after 12 s); red run showed "a good reload after a failed one leaves the card up" |
| rth-6 | open (backlog) | | |
| rth-7 | docs only | 6a801c0c | configuration.md `show_helper` row: "as `MOD+a` (`run: chora-chat`) does" |
| rth-8 | b8b2d3b5 (same commit as the rth-5 step) | b8b2d3b5 | the gate takes the pixel before the reload (`$base`) and reads it at the card centre; no separate red, the check was hardened |
| rtd1-2 | 2c928a8b | 47465ea4 | theme.rs `the_titles_of_every_missing_script_are_pinned_in_both_languages`; German "Design-Umschalter nicht installiert" |
| rtd1-3 | f8a15eb7 | ea5ab55e | main.rs `the_hint_names_no_help_key_when_none_is_bound_rtd1_3` (red) and `the_hint_names_a_rebound_help_key_rtd1_3` (passed at once); new word `KeyUnboundBare` in chora-kit (en, de) |
| rtd1-4 | 6a20aba7 | 58270886 | looks.rs `a_door_that_cannot_list_the_looks_says_so_in_both_languages` (red on a stub that answers empty strings); `launch_looks` shows the notice |
| rtd1-5 | docs only | 6a801c0c | configuration.md "The theme rows set `look.theme`" |

Not in the brief and not touched: rtd1-1 (the sleep 0.5 in gate-tiling-apps.sh).
The sleep list was lowered by one (31af60c5, `tools/sleep-ratchet --update`:
tree 778, list was 779); no sleep line was added.

### Ratchets (on head 31af60c5)

```
ok: sleep-ratchet (sleep 778, sleep-all 929)
ok: size-ratchet (0 files over 1500)
ok: lint-ratchet (allow 360, no-reason 0, eprintln 46)
```

The lint ratchet was at 47 after rth-4 (one `eprintln` for the chip); the last
commit removed it, so the count is the 46 of the base.

### Gate

`tests/merge-gate.sh` (no --only), one run, on head 31af60c5: all steps PASS,
no SKIP ("all steps passed"). Run directory /tmp/merge-gate.rTgTF5 (cloud
lane; red-first, identity, pre-commit, ci-check, build, test, clippy, layout,
acts, doors, matrix, e2e and the tiling-* steps among them).

One slip, corrected before the push: the first red commit of rth-1 failed the
subject-length hook, and the fix commit then held the tests; I split them
again (8e2052a7, then 6b199a70), so red-first holds in the history.

## Review r1

Reviewer lane, read-only on code: claude/tiling-minor-r1 head 31af60c5 on 84f66d43
(13 files, +363 -50, 15 commits). Step 0 green (`git fetch --unshallow`,
`cargo check --workspace`). No merge gate run (lane gate /tmp/merge-gate.rTgTF5).

Checks:
1) rth-1 core press_layer: PASS. Claim at the point (claimant_at, one owner),
   else live claim owners overlapping the shown rect of the layer's output, else
   every live claim owner. One claim at the point gives one look; with one look
   the result equals the old `first_manager` (which also needs a claim; the new
   code adds the `live` filter). Injected press: no `last_real_press`, no act;
   locked: no act. Escape hatch, grab path and `note_act` lock guard untouched.
   `cargo test -p chora input::` 65 passed; red confirmed at 8e2052a7 (both rth-1
   tests FAILED, lock test ok).
2) Red first: PASS. Every fix sha follows its test sha (ab0738de; 8e2052a7 ->
   6b199a70; 9fe4649a -> bcc8cb4f; b8b2d3b5 -> 87aa0d1e; 2c928a8b -> 47465ea4;
   6a20aba7 -> 58270886; f8a15eb7 -> ea5ab55e). rth-8 hardened in b8b2d3b5 with
   the rth-5 red run pasted: accepted.
3) rth-4: PASS. The chip shows "title: reason" (70 chars, 5 s) only when
   `show_or_log` gives None; the only `eprintln` in the diff is the moved line of
   main.rs; `tools/lint-ratchet`: eprintln 46.
4) rth-5 gate step: PASS with rtm1-1. A poll (`until_ok no_notice 16`), no new
   sleep; `tools/sleep-ratchet`: ok, sleep 778, sleep-all 929 (list 779 -> 778 true).
5) rtd1-2..4: PASS with rtm1-2. en and de for KeyUnboundBare, the theme titles and
   the Looks notice; chora-tiling 208 unit tests and chora-kit words tests green.
6) Overlap with claude/tiling-doors-r2 (still at 721be592, nothing pushed): no
   behaviour conflict. HINT_TIME only feeds `hint_unbound`; the reload chip has its
   own CHIP_MS. If r2 injects a clock into `show_words`/`chip_until` (its test b),
   the new call in reload.rs:81 fails to compile on merge, a visible conflict, not
   a silent one. Word overlap is text only (rtm1-5).

Findings:
- rtm1-1 MINOR tests/gate-tiling-reload-notice.sh:67: the card expires 12 s after the failed reload and the step waits 8 s after `card_shows`, so when the earlier waits take more than about 4 s the step passes without the fix; fix: wait 2 s at most (`until_ok no_notice 4`), since the dismiss is synchronous.
- rtm1-2 MINOR plugins/chora-tiling/src/main.rs:375: the rtd1-4 test pins only the words of `unreachable_notice`; nothing checks that a failed clients.list in `launch_looks` shows the notice; fix: a unit or doors cell that runs the Looks launch against a core whose clients.list errors and sees one notice.
- rtm1-3 NIT crates/chora/src/input.rs:3117: the lock test still passes with the lock guard of `note_act` removed (the lock hides the bar from `surface_under`), so it pins the hit test, not the guard; fix: say so in its doc comment or add a `note_act` unit test under a lock.
- rtm1-4 NIT crates/chora-kit/src/runtime.rs:568: a good reload through the method `settings.reload` (not the key) leaves the failure card up until it expires; fix: dismiss in a plugin hook on Ok of the method too, or note it in rth-5.
- rtm1-5 NIT plugins/chora-tiling/src/looks.rs:60: the notice says "Other desktops" / "Andere Desktops" while ti-55 (TILING-DOORS r2) makes "look" the word on screen; fix: align the title with the bar word when r2 lands.

Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 2 MINOR, 3 NIT).

## r2 (lane tiling-minor-r2)

Head 7987897c0 on claude/tiling-minor-r2 (from e39448ad). Step 0 green.

- rtm1-1: tests/gate-tiling-reload-notice.sh waits `until_ok no_notice 4` (2 s). Red with the dismiss in reload.rs:53 removed: `FAIL: a good reload after a failed one leaves the card up: Settings not applied chora-tiling.yaml: unknown variant`.
- rtm1-2: plugins/chora-tiling/tests/launch_looks.rs runs `chora-tiling --launch looks` against a FakeCore that refuses clients.list and counts one notify.show (new FakeCore::socket()). Red with tell_launch removed: `notices shown: ["methods.call {...hide_keys...}", "clients.list null"]` (left 0, right 1). The first version set LC_ALL and tripped cut-check copies; the locale is gone.
- rtm1-3: a note_act unit test under a lock already exists (state::tests::a_press_while_locked_is_no_act); red with the `.filter(|_| self.lock.is_none())` removed: FAILED at state.rs:1975. The input.rs doc comment now says what each test pins.
- rtd3-1: no test of two real connects: the pid is the only input of the name and one test process has one pid; the pure test guards a constant name (doc comment). rtd3-2: GAP_LINE closes the quote it opened (backreference); red `AssertionError: '20' != '12'`. rtd3-3: the name test sits below `use super::*;`.

Gate: full `tests/merge-gate.sh --jobs 2`, EXIT 1 with one foreign red: cell keylist of mc-sofa (`FAIL: a rule of the key list strikes a text (rpx1-9)`, tests/gate-mc-sofa.sh:472; no file of mine near it). The same step on e39448ad alone also fails (cells door empty-head answer answer-long alarm keylist pill-clock remote-back), so it is not mine. All other cells PASS (ci-check, clippy, test, tiling-*, layout, doors, pixels).

## PM review r2 (lane tiling-minor-r2, 7987897c on e39448ad)

+51 -6 over 7 files, test and doc only except the GAP_LINE regex. rtm1-1: the wait is now 4 ticks (2 s), red pasted with the dismiss removed. rtm1-2: new launch_looks test plus FakeCore::socket(); red by mutation (behaviour was right). rtm1-3: existing unit test, red by mutation, doc comment. rtd3-1: named reason accepted (one pid per process; the pure name test is the guard). rtd3-2: test a54a8581 before fix be637d5c. rtd3-3: done.
Gate: EXIT 1 on one foreign cell (mc-sofa keylist, rpx1-9) that the lane reproduced on e39448ad alone; train140 shows mc-sofa PASS on the PM host, so it is a lane-container red, logged for FLAKE-SERIES r10. The train gate reruns everything.
Findings: none MAJOR. rtm2-1 NIT: 4 ticks is tight on a loaded runner; watch it in FLAKE-SERIES r10.
Verdict (computed): MERGE-OK 7987897c, next train141.

## r1 (rc 1b, F7: the rpt ids of the pass)

Branch claude/tiling-minor-r1, base 7822a66c (session head with kit-swallow r2, train 159), head bd641acb
(15 commits, 18 files). Red commit before each fix; every red failed on 7822a66c by an assertion (the
rpt3-2 test first failed to start for want of a field program and was moved to the xp balloon chat with the open
mode kept, then failed on `No helper` against a header that said `working`). Step 0: `git fetch --unshallow`,
`cargo check --workspace`, `cargo build --workspace` green. Captures: media/2026-11-27/tiling-minor-r1/ (each
under 500 KB; before from the binaries of 7822a66c, after from those of 587aa6f7, 1280x800, fresh compositors
with `round.sh start`).

| id | red | fix | test | capture |
|----|-----|-----|------|---------|
| rpt3-1 | 0b558a6a | a1d8f4aa | kit `tests/keysheet.rs` `a_chord_has_one_order_of_modifiers_and_the_names_of_the_keys_on_the_keyboard`; tiling `the_hint_spells_the_chord_as_the_key_list_does_rpt3_1` | rpt3-1-before/after-chip-* (Alt+Super+X to Super+Alt+X, Bracketleft to `[`, Ctrl+Shift+Super+Next to Super+Ctrl+Shift+Page Down) |
| rpt3-2 | 9fa77480, 6f97804d | 6d7e92da, bd641acb | assistant `the_door_says_the_state_that_the_chat_header_says_rpt3_2`, `the_door_word_is_told_once_for_each_change_rpt3_2`; tiling `the_helper_word_follows_the_door_at_once_rpt3_2` | rpt3-2-before/after-helper-* (see the note) |
| rpt2-2 | 5ba7ff37 | 6e1365d1 | tiling `tests/looks_picker.rs` `the_picker_offers_no_move_without_a_window_rpt2_2`, `a_refused_give_names_no_tool_in_its_notice_rpt2_2` (and `the_picker_offers_the_moves_with_a_window`, green at once) | rpt2-2-before/after-looks-picker-empty-tag |
| rpt3-3 | 01b30501 | 7d318133 | tiling `panel::the_volume_has_a_tooltip_only_for_a_reason_rpt3_3` and the changed rows of `the_volume_always_has_honest_words` | rpt3-3-before/after-volume-sh.txt (see the note) |
| rpt3-4 | 170d1d38 | 587aa6f7 | tiling `tests/looks_picker.rs` `the_picker_box_fits_its_rows_rpt3_4` | rpt3-4-before/after-looks-picker-* |

### What each fix does

- **rpt3-1.** One formatter, the one the key list uses: `keysheet::caps` (kit) now puts the modifiers in the order
  Super, Ctrl, Alt, Shift whatever order a binding names them in (the core spells them Ctrl, Alt, Shift, Super), and
  `glass::cap` shows Prior and Next as Page Up and Page Down and comma, semicolon, apostrophe, grave, minus, equal,
  backslash and the two brackets as the mark on the key, BackSpace as Backspace. The chip calls `keysheet::chord`
  as before, so no second table; every key list of the four looks gets the same spelling. A chord of the file with
  these keys now shows the same way in the list (before it said "Bracketleft" there too). Not added to
  tests/keysheet/words.table: the acts (tests/acts/invariants.py I10) match its key names as words in any key line,
  and "Next" and "Prior" are words of "Next desk" and "Next layout".
- **rpt3-2.** Two sources said the state: the bar read the presenter's helper machine (heard events, polled every 2 s
  through `chora-chat --door`), the header read the chat session's own status (it says `working` at once on a send and
  `idle` at once on Stop). Now `presented()` of the assistant takes the name and the state of the open chat's header
  (`Session::shown_state`) for `door` and `helper` and keeps the machine only without an open chat; the assistant
  tells `assistant.door {door}` on each change of that word (`announce_door`, once per change, in `event` and
  `chat_step`); tiling hears it and sends SIGRTMIN+2 to waybar (`push_helper`); the helper module has `"signal": 2`
  and still polls every 2 s in case a signal is lost; waybar.sh ignores SIGRTMIN+2 until waybar has its handlers, as
  for SIGRTMIN+1.
- **rpt2-2.** `--launch looks` asks `scene.query` for the focused window (a read that fails counts as a window, so
  the rows stay); with none it lists the one row "No window to move" ("Kein Fenster zum Verschieben", new word
  `NoWindowToMove`) instead of the moves. give.sh no longer puts `choractl: no focused window` (tool name, English
  only) in the notice: its body says "The window stayed where it was." / "Das Fenster ist geblieben, wo es war."
  and the reason goes to stderr.
- **rpt3-3.** volume.sh prints no tooltip for a level or "Muted" (it said the word again), only for "No sound" with
  its reason ("wpctl not found", "No output device", "No answer"). With no tooltip there is nothing to stand under a
  resting pointer.
- **rpt3-4.** `--launch looks` passes `--lines N` with the number of its rows, as power.sh passes `--lines 5`.

### Notes

- **Captures that do not show the fault before.** rpt3-3: the waybar tooltip did not draw in the persona session
  before the fix either (the injected pointer does not hover a GTK popup in five tries, click, move, 4 s rest, with
  and without the key list), so the capture is the output of volume.sh, `{"text": "Volume 55%", "tooltip": "Volume
  55%"}` before and `"tooltip": ""` after, with the red test. rpt3-2: the disagreement is a phase of the 2 s poll
  against the header; 7 frames at 0.3 s after a question show bar and header equal in both runs (the before run did
  not catch the 1 to 2 s window), so the proof is the red test (header `working`, door `No helper`) and the second
  test (one `assistant.door` per change); the after frames show the bar and the header change in the same frame.
- The chip is shown in a tiling-only session, as in the pass: in tiling+xp the swallow of Super+Alt chords did not
  show the chip (not looked into, not a finding of this brief).
- The Apps launcher of tiling still lists the "Move window to ..." rows with no window (only the Looks picker was in
  the brief); its refusal now says the new notice words, as it runs the same give.sh.
- The brief names the report section "## r1"; the file already had an "## r1" of an earlier round (rth ids on
  84f66d43), so this one carries the suffix.
- **A regression of my own, caught by the gate.** The first full gate stopped at `acts` (2162 s): `xp act:the helper
  answers` read `helper: idle`, not `answered`. 6d7e92da had made the `helper` word of `state.get` follow the header
  of the open chat, which says `idle` for a done report. Red test (`state.get` keeps the word of the machine; test
  commit before bd641acb), fix bd641acb: only `door` follows the header. cut-check (ci-check) had also failed once on
  my first test file (notify-send and LC_ALL counted in plugins/chora-tiling, a pid-named temp folder): the give.sh
  notice test moved to crates/chora-kit/tests/give_notice.rs (387bac39).
- The default base (origin/claude/autonomous-dev-session-3w-i33pg7) is 0e18f14c here, far behind 7822a66c, so the
  `docs` step failed ("mkdocs is not installed and docs changed") on the first run: a host and base gap, not mine.
  Every later run passes `--base 7822a66c8f6a78080fd416f03ef0420b65d172e7`; docs is then `SKIP: mkdocs is not
  installed (docs unchanged)`, allowed by `--allow-skip docs`.

### Gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 --allow-skip docs --base 7822a66c...` (no --push), all on head
bd641acb, in three runs because the usage limit and a worker restart ended the process of the first two between
steps (not a red): run 1 (/tmp/merge-gate.jchBeV) red-first to cube, 33 steps, all PASS, docs SKIP; run 2
(/tmp/merge-gate.HbpXH2, `--only` the rest) 29 steps PASS including acts (237 pass, 1 known red, 0 blocking) and
leak, cut off after leak by the restart; run 3 (/tmp/merge-gate.C9ghTI, `--only tiling-disc,mc-host,e2e`) PASS.
Every one of the 68 steps passed on bd641acb in one of the runs; no step was red there; the last lines:

```
mc-host    PASS    26s /tmp/merge-gate.C9ghTI/mc-host.log
tiling-disc PASS    86s /tmp/merge-gate.C9ghTI/tiling-disc.log
e2e        PASS   288s /tmp/merge-gate.C9ghTI/e2e.log
partial run: nothing is pushed
EXIT 0
```

An unfiltered single run to EXIT 0 on bd641acb was not made (about 2 h each; the earlier unfiltered run on 387bac39
passed 51 steps and stopped only at the acts regression above). `pgrep -fa 'chora --headless'` is empty.

## Review r1 (rc 1b tiling-minor r1; rpt2-2, rpt3-1..4)

Reviewer lane, read-only on code: claude/tiling-minor-r1 head bd641acb on 7822a66c (17 files, +502 -57, 15 commits).
Step 0 green (`git fetch --unshallow`, `cargo check --workspace`). No merge gate (the lane ran it, check 7).

Checks:
1) Red first: PASS. Run at each test commit: 0b558a6a kit `a_chord_has_one_order_of_modifiers_...` and tiling
   `the_hint_spells_...rpt3_1` FAILED; 9fa77480 `the_door_says_the_state_...` FAILED; 6f97804d the door-told test and the
   tiling helper test FAILED; 5ba7ff37 `the_picker_offers_no_move_without_a_window` and `a_refused_give_names_no_tool`
   FAILED (`the_picker_offers_the_moves_with_a_window` ok, as claimed); 01b30501 both volume tests FAILED; 170d1d38
   `the_picker_box_fits_its_rows_rpt3_4` FAILED; ee2a56c9 `the_door_says_the_state_...` FAILED (the told-once test ok). All
   pass on head (`cargo test -p chora-tiling -p chora-kit -p chora-assistant`: 959 passed, 0 failed; clippy clean).
   387bac39 removes no assert: the give notice test moved to kit `give_notice.rs` and gained two (exact notice, German body).
2) rpt3-1: PASS. The chip calls `keysheet::chord` as the key list does; `caps` orders modifiers (Super, Ctrl, Alt, Shift) and
   `glass::cap` spells the keys; one formatter, no second table. Side effect: every key list of the four looks now spells
   `Page Up`, `[` and orders modifiers the same way (intended, gates tiling-keys, tiling-keycaps, chat-keys green).
3) rpt3-2: PASS with rtmr1-2. `presented()` takes `door` from the header of the open chat (`shown_state`, same `words::row`
   as the header) and keeps `helper` as the machine word; the only readers of `state.get` `door` are the chat opener
   (`chat-door`, the bar word) and `announce_door`; `helper` is read by the acts as before. `announce_door` tells once per
   change and does not fail on a core refusal. No 1-2 s path remains except the 2 s poll kept as fallback. The before frames did not
   show the window; the proof is the red tests, which fail on the header/door pair (judged enough for a state read; the
   push to waybar is untested, rtmr1-2).
4) rpt2-2: PASS with rtmr1-1. The picker lists "No window to move" without a focused window (a failed read keeps the rows);
   `chosen` returns none for that row; give.sh says "The window stayed where it was." / German, the reason goes to stderr.
5) rpt3-3: PASS: volume.sh gives an empty tooltip for a level or "Muted" and a reason only for "No sound"; the capture is the
   script output (the tooltip did not draw before either, said in the report). rpt3-4: PASS: `--lines <rows>` as power.sh; the
   box height does not depend on the resolution (captured at 1280x800 with one row, both pictures fit; 1280x720 and
   1920x1080 not captured, none needed); the width (36) holds the longest row on screen.
6) POLS: no other tiling path loses a word, key or notice; other looks keep their helper word; gates tiling-chip,
   tiling-notice, tiling-bar (all ok), chat-keys, tiling-keys, tiling-keycaps, tiling-empty-output, 0.8-tiling-look ok on head.
7) The lane's three-part gate: not a finding.

Findings:
- rtmr1-1 MINOR flavors/tiling/look.sh:20: the notice of a failed look change shows the body "choractl call tiling settings.set", a tool name in a notice a person sees (rpt2-2 fixed the same class only in give.sh); fix: body "The look stayed as it was." / "Das Design ist geblieben, wie es war." with the command line to stderr, and a test like `a_refused_give_names_no_tool_in_its_notice_rpt2_2`.
- rtmr1-2 MINOR plugins/chora-tiling/src/main.rs:2986: `the_helper_word_follows_the_door_at_once_rpt3_2` reads the text of config.jsonc, waybar.sh and EVENTS only; nothing checks that an `assistant.door` of the assistant makes tiling send SIGRTMIN+2 (`push_helper`, main.rs:885), so a handler change leaves it green; fix: a unit test that an Emitted `assistant.door` from `assistant` signals the waybar pid of a fake companion with RTMIN+2, and one from another client does not.
- rtmr1-3 NIT flavors/tiling/applications/chora-give-xp.desktop (and mc, construct): the Apps launcher still lists "Move window to ..." with no window (named in the report); the notice after the refusal is now honest, so it is a word of the menu only; fix: leave, or list the entries only with a window as the Looks picker does.

Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 2 MINOR, 1 NIT).

## r2 (rc 1b, F7: rtmr1-1..3 of the tiling-minor r1 review)

Branch claude/tiling-minor-r2, base bd641acb (tiling-minor r1 head), head ad07ec70 (8 commits, 6 files, +367 -32).
Step 0 (`git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`) green. Captures:
media/2026-11-28/tiling-minor-r2/ (each under 500 KB; 1280x800 fresh persona sessions of `round.sh start`, before from the
binaries and flavor of bd641acb, after from the head).

| id | red | fix | test | capture |
|----|-----|-----|------|---------|
| rtmr1-1 | 433fc761 | ab985149 | kit `tests/give_notice.rs` `a_refused_look_change_names_no_tool_in_its_notice_rtmr1_1` (exact en notice, German words, no `choractl`, no `settings.set`) | rtmr1-1-before/after-look-notice.txt (the arguments of notify-send with a refusing stand-in of choractl, C and de_DE) |
| rtmr1-2 | none: the code held; test by mutation (see below) | none | tiling `tests::the_assistant_door_signals_the_bar_rtmr1_2` (fb05f028, f5cc8bc2) | no capture asked |
| rtmr1-3 | 3fce00cd | cda21a03 | tiling `tests/apps_launcher.rs` `the_apps_list_offers_no_move_without_a_window_rtmr1_3` (and `..._offers_the_moves_with_a_window`, green at once); kit `launcher::tests::the_rows_that_move_a_window_are_the_give_rows_only`; gate `tests/gate-tiling-apps.sh` (5e45a47a, ad07ec70) | rtmr1-3-before-apps-move-no-window.png, rtmr1-3-after-apps-move-no-window.png, rtmr1-3-after-apps-move-with-window.png (Super+Space, then the text "move") |

### Per id

- **rtmr1-1.** look.sh says "Theme not changed" and now the body "The look stayed as it was." / "Design nicht geändert",
  "Das Design ist geblieben, wie es war."; the reason of choractl goes to stderr (`look.sh: ...`), as give.sh does since
  rpt2-2. Red on bd641acb: the test failed with `C: -a Look Theme not changed choractl call tiling settings.set`. Every other
  notice a person sees in flavors/tiling was grepped for `choractl`: give.sh (already clean) and look.sh were the only
  `notify-send` in the flavor; the rest is a command in config.jsonc or a comment. Not in the scope, named: flavors/shared/power.sh:29
  shows the locker command line ("swaylock -f") as the body of its notice when the lock fails; that is a tool name in a notice
  of the shared folder, left for the PM.
- **rtmr1-2.** The handler was right (main.rs `Event::Emitted assistant.door` from `assistant` calls `push_helper`), so the test
  is the deliverable. A stand-in of waybar (a bash script named `waybar` with the pid file `waybar.pid` in a private runtime
  directory) writes `door` to a fifo for each SIGRTMIN+2 and `end` for SIGRTMIN+3; a child process of the test binary is
  tiling and hears `assistant.door` from `other` (the fifo says only `end`) and from `assistant` (`door`, `end`); the parent
  ends each round with SIGRTMIN+3, which a pending SIGRTMIN+2 precedes, so no sleep and no timing. Red by mutation, run twice:
  `Self::push_helper();` removed from the handler: `the door of the assistant did not signal the bar, left: ["end"] right:
  ["door", "end"]`; the `client == "assistant"` test removed: `another client signalled the bar, left: ["door", "end"] right:
  ["end"]`. Five runs green on head. The first version set `XDG_RUNTIME_DIR` and tripped `cut-check copies` (KIT1);
  it names `chora_scene::paths::RUNTIME_ENV` now (f5cc8bc2).
- **rtmr1-3.** One rule, one place: `window_has_focus` (main.rs) reads the focus for the Looks picker and the Apps list (a
  failed read counts as a window, so the rows stay), and `chora_kit::launcher::without_moves` drops the `chora-tiling --launch
  give ...` rows; `run_ordered` takes `Option<Tiling { helper, window }>` (a sixth argument tripped clippy's too_many_arguments).
  Red on bd641acb: `the_apps_list_offers_no_move_without_a_window_rtmr1_3` failed with the three "Move window to ..." rows.
  The capture: with no window, the text "move" in Apps shows three rows before and an empty list after; with a window (foot) after the
  three rows are back. The Apps gate listed the move rows on an empty desktop (the e2e step failed there, `no launcher row "Move
  window to the xp desktop"`, my change): it checks now that an empty desktop has none and, with a window, all three.

### Gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 --allow-skip docs --base bd641acb` (no --push), detached, EXIT 0:
all steps PASS, docs SKIP (mkdocs not installed, docs unchanged in the range). Why `--base bd641acb`: the default base
(autonomous-dev-session-3w-i33pg7) has docs/decisions.md changed against the merge-base, so the docs step failed without mkdocs
before any of my code; the range of r2 changes no doc.
Earlier runs of this lane, in order, so that nothing is hidden:
1. docs FAIL (the base above); cut-check copies XDG_RUNTIME_DIR (fixed, f5cc8bc2).
2. doors FAIL twice in the gate and once alone: cell helper-none of tiling, `tiling: the chat does not say No helper whole: A
   question is waiting.` (the chat of the cell before). Not reproduced on bd641acb (one run, pass), on head the same cell passed
   in two later runs and in the final run; I cannot name the cause, it is intermittent and nothing in the range touches the chat
   or the voice cells. Watch it in FLAKE-SERIES.
3. pull FAIL once with `--jobs 2` (`one output: a title drag to the screen edge shows no pull hint`), pass alone and in the final run.
4. e2e FAIL: `gate-tiling-apps` (my change, above); fixed in the gate script, then e2e PASS alone and in the final run.
5. ratchets FAIL: my first gate edit added one `sleep 600`; the window is a `cat` now (ad07ec70).
A note for the next lane: the red-first step builds the parents of the fix commits into the shared target/debug, so a gate script run by
hand right after it sees the old binary; the gate's own build step rebuilds.

Last lines of the final run:

```
tiling-bar PASS   189s
leak       PASS   474s
tiling-theme PASS    18s
mc-host    PASS    23s
tiling-disc PASS    73s
e2e        PASS   286s
skipped: docs (mkdocs is not installed (docs unchanged))
all steps passed; no --push, nothing is pushed
```
