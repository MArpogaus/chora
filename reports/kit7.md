# kit7 r1 (F2): chora-assistant the only helper presenter

Branch claude/kit7 from 4548feb3, merged with the session branch at daf83391 (merge 0f5ea027).
Head c1dbc47b. Commits: c0414145 (red), 06b9666c (feat), d86ee4ca + caff20cd (mc), 6a5cfc1d + c3c9963a
(construct), a1c59c58 (xp acts order), 0f5ea027 (merge of daf83391), 68a2d0b6 (known red kit7-a1),
c1dbc47b (construct-hud flight). Before the merge, against 4548feb3: 54 files, +916 -2571.

## Draft decision record: one helper state machine, one presenter (KIT7)
Context: four rules said the state of the helper (kit `Helpers` for state.get, the figure word of
chora-assistant, the xp tray `Helper`, `door_word` of chora-chat for the tiling panel), and mc and the
construct each had a second presenter behind `voice_hud: own` (own HUD line, own question card,
indicator claim). I8 had nothing to read in the acts, and "working" stayed after an answer in the raw words.
Decision:
1. One state machine, `chora_kit::helper::Helper` (kit, no IO): it hears `voice.state`, `voice.say`,
   `voice.remove` and the disconnect of voice; `seed` takes the `state` answer of voice for a late reader.
   States: `no_helper` (no named helper and the latest report rests, or no voice), `idle`, `listening`,
   `working`, `answered` (the latest report is spoken, a question, done or blocked). Never `working` after
   an answer event until a new report works. It also gives the words of a helper door (`No helper`, or
   name and state of the helper that reported last, with or without its task) and the microphone.
2. One presenter, chora-assistant: it feeds the machine, draws figure, card and chat, and answers
   `state.get` with `helper`, `door` and `shown` (`hud-figure`, `hud-card`, `hud-chat`).
3. A look gives only its costume: place (xp tray icon, tiling waybar module, mc chat items, construct
   door), frame (its HUD folder, as today) and words (mc labels from mc.yaml, "Voice off" in xp).
   Every look reports the machine's word in its own `state.get` (`helper`).
4. `voice_hud: own` is gone in mc and the construct; `own` (and `true`/`false` in the construct) load as
   `host` for one release with one warning line. The key stays readable in old configs.
Consequences: a door reads the presenter or the machine, never voice directly; the `state.get` word changed
from the raw voice words (`none`, `done`, `question`...) to the five machine words; with two helpers a door
names the one that reported last (xp named the first by name, tiling the first busy one).
Not done (KISS): the assistant keeps `context::agents_of` (the fold for the folder rule `agents:`) and the
chat header `Status`; both live inside the one presenter. mc keeps its agents list from `agent.set` (API).

## Old presenters gone (grep, all empty on 0f5ea027)
`struct Helpers` (kit), xp `struct Helper` and its `tip`, `fn door_word` and `fn helpers` (chora-chat),
`VoiceHud::Own`, `draws_voice`, `construct-hud-state`, `construct-ask-yes`, `mc-answers`, `mc-answer-yes`.

## Cells
- Red c0414145: `crates/chora-kit/src/helper.rs` three tests against a stub (every state with the honest
  no helper, never working after the answer, only voice counts plus the microphone): 3 FAILED at red,
  green at 06b9666c (with `a_late_reader_seeds_the_machine_from_the_state_of_voice` added).
- Red c0414145: `helper_acts` in tests/acts/looks/common.py, run by all four looks: voice with the stand-in
  helper (fake-acp) and the presenter start, a helper starts, asks (the Yes button is clicked; an injected
  key never answers), answers, the chat opens and shuts (one helper surface), the helper stops. `Expect.helper`
  now also reads the presenter (assistant in BLOCKS) and I8 reads it too. At red, run for tiling: "presenter
  starts" (no state.get), "a helper starts", "asks", "stops" FAIL. The cue of the first act waits for the
  state.get of the presenter, so it fails the same way in every look at red; I ran only tiling at red.
- Green: tiling 25 pass (1 known red I5), xp and mc acts green after the xp fix (helper acts ran twice in
  xp, the second time behind the known-red Turn Off dialog).

## Gate (tests/merge-gate.sh --only, no full gate)
- `cargo build --workspace --locked`, `cargo clippy --workspace --all-targets -D warnings`: green on 0f5ea027.
- `cargo test` of chora-kit (--features testing), chora-xp, chora-tiling, chora-mc, chora-construct,
  chora-assistant, chora-chat: green on 0f5ea027; `python3 -m pytest tests/test_acts.py -q`: 101 passed.
- construct-hud, keys, mc-handover: PASS on c1dbc47b (/tmp/merge-gate.laamap).
- acts, xp, hud, chat-keys: PASS on c1dbc47b (/tmp/merge-gate.NqcybD). So all seven listed steps pass on c1dbc47b.
- Earlier on 0f5ea027: xp, hud, chat-keys PASS; acts FAIL on mc I3 (kit7-a1, listed in 68a2d0b6), then
  acts PASS on 68a2d0b6 (104 pass, 8 known red). construct-hud failed on "Tab opened no list" (kit7-a2).

## Findings
- k7-1 (visible, door words): with two helpers the xp tooltip and the tiling panel word now name the helper
  that reported last, not the first by name or the first busy one. One helper: same words as before.
- k7-2: `indicator_clients` of voice still lists mc and construct, which no longer claim the indicator.
- k7-3: docs/construct-style.md rules 1-17 still describe the old own voice HUD (an intro note says it is gone).
- k7-4: the mc and construct worktree agents cited D93 for the one-release alias; D93 is config parity,
  the rule is in the arch plan record 13 (D93 extended). The PM may correct the citation.
- kit7-a1 (I3, listed known red, slice WM2): after `chat.show` of the helper in mc the core focuses the
  page node of mc and mc's state.get does not say it holds the focus. Found by the new chat act.
- kit7-a2 (construct, not a kit7 regression): the own-HUD cells of gate-construct-hud.sh went, and with
  them a flight into a pane and back; without that flight Tab after a new window opens no list. Reproduced
  with the session head daf83391 binaries and the new script; the old script passes there only because it
  flies first. c1dbc47b puts the flight back; the key routing after a new window is for WM2 to look at.
- k7-5: the first construct agent worked on an old base (d060dc03, dev rc.4); its commits were redone on
  06b9666c, none of the old base came in.

## Review r1
Seeded review (opus, high), read-only on claude/kit7 head c1dbc47b against base daf83391. Verdict: FIX (1 BLOCK, 2 MAJOR, 4 MINOR).
Checks: 1) `cargo test -p chora-kit --features testing helper` 7 pass at c1dbc47b; the states are complete, but the honesty fails (rw-2, rw-3).
2) red first holds: c0414145 comes before 06b9666c; at c0414145 the 3 kit cells FAIL; their bodies are unchanged at the head (door signature, asserts added).
3) the grep of the old presenters (kit Helpers, xp Helper, door_word, VoiceHud::Own, construct-hud/ask, mc-answers) is empty; chora-assistant alone answers helper/door/shown; opener.rs: the last-reporter door is a finding (rw-2).
4) helper_acts runs in all four looks (the Yes click on hud-card-yes proves the host card in each); dropped gate-construct-hud checks: rw-7.
5) kit7-a1 (mc I3, WM2) is an honest listing: the mc focus code is unchanged and the act is new; the xp reorder (a1c59c58) is fine; the construct Tab flight is a workaround (rw-5).
6) size-ratchet passes (`--base daf83391`, exit 0); `tools/cut-check` FAILS (rw-1); docs/api.md matches the five `state.get` words of all four looks and the presenter.
- rw-1, BLOCK, plugins/chora-construct/src/config.rs:239: `tools/cut-check` (also `--base daf83391`) exits 1 at c1dbc47b and 0 at daf83391: names of chora-assistant grew in chora-construct (1 to 7 lines), chora-mc (1 to 3, config.rs:322) and chora-chat (0 to 1, services/voice/chat/src/main.rs:45), so the ci-check step of the full gate blocks. Fix: say "the HUD host" in those comments and warnings (or add reasoned cut-known.toml rows) until cut-check is green.
- rw-2, MAJOR, crates/chora-kit/src/helper.rs:141: `state()` and `door()` read only the latest report of anyone, so a helper that still works shows as listening, idle or answered once the voice agent or a second helper reports (coder working, then voice listening: helper=listening; atlas working, then coder idle: door "coder · idle", tiling via services/voice/chat/src/opener.rs:66). Fix: keep `working` while any named helper's latest report works, name that helper first in the door, and add both cases to the cell.
- rw-3, MAJOR, crates/chora-kit/src/helper.rs:227: when only the voice agent thinks, one `state.get` answer says helper=working and door "No helper". That is "working" without a helper, and the red cell makes it the rule. Fix: report `no_helper` without a named helper (the figure still shows voice thinking) or name the voice agent in the door, and change step 2 of the cell to match.
- rw-4, MINOR, plugins/chora-xp/src/wm.rs:133: the machines of the looks can disagree with the presenter. Only chora-assistant seeds (voice.rs:70), so a look that starts while voice runs (a handover) says no_helper until the next voice event, and xp and tiling (main.rs:426) do not hear voice.say. Fix: seed each look's machine from voice `state` when it starts and when voice connects, and subscribe voice.say.
- rw-5, MINOR, tests/gate-construct-hud.sh:118: the restored flight hides kit7-a2 (Tab after a new window opens no list without a flight first; also on daf83391) and leaves no failing record. Fix: add a cell for Tab right after a new window, listed known red as kit7-a2 under WM2.
- rw-6, MINOR, tests/acts/looks/common.py:126: helper_acts never expects `working` in the looks and the presenter, and "the helper answers" has no Expect, so the one-state rule for working, and for answered after the click, is checked only by I8's last-word test. Fix: add Expect(helper="working") to an act where the helper works and Expect(helper="answered") after the Yes click.
- rw-7, MINOR, tests/gate-construct-hud.sh:1: the own-HUD checks went (the claim, state labels, sentence timing, the whole 6 KiB detail, a done of another id, Ja/Nein with Alt+Y/Alt+N, the restart claim and hide, the HUD place in a flight, long words in the sentence and the question, the watchdog label). Equal checks remain for the Ja click (helper_acts, every look) and for no claim and no own HUD (gate-hud.sh H10 construct-host-hud). None remains for the host card with the construct folder: the whole detail, a done of another id, long German words (the sentence at most 2 lines). Fix: move those three checks onto construct-host-hud in gate-hud.sh.
