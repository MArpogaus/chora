# xp-majors

Lane: cloud, branch claude/xp-majors from 30727c82 (train 77), head 81a29fe8e098ab9bd75552447313c5b2b4083d82. Step 0: unshallow, cargo check and build green.

## r1
Gate (merge of origin/claude/autonomous-dev-session-3w-i33pg7 was already up to date): `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,acts,xp,xp-faces,xp-paperclip,xp-question,xp-fields,doors,keys`: all PASS.

| Finding | State | Guard or red | Fix |
|---|---|---|---|
| xp-23 paperclip | gone before | cell gate-xp-paperclip, 65f4510b, green on the head, capture shows it 8 px from the right edge | none |
| xp-27 question card | gone before | cell gate-xp-question, cf78bef2, green at 1x and 2x: card inside the balloon under the transcript, Yes and No 28 px, hint once | none |
| xp-31 Super+D | gone before | acts "Super+d puts the windows away", "a second Super+d brings them back", "the task button brings the window back", 5ac45053, green on the head | none |
| xp-33 Super+F1 | gone before | acts "the keys key opens the key list" and "the key list of xp speaks the keysheet", 5da2712c, green on the head | none |
| xp-26 level shown | gone before for the key and the Control Panel row | act "the tooltip of the speaker says the level that the key set", 5ac45053; live probe with a wpctl that keeps its volume: tooltip "Volume 75%" and tick on 75% after Control Panel > Sound > 75% | none |
| xp-32 speaker click | fixed | red 24640558 (act "a click on the speaker opens the volume popup" and four more) | 81a29fe8 |

What was wrong with xp-32: a click on the speaker toggled Mute at once and showed nothing. Now it opens a popup (xp plugin only, context.rs and wm/panels.rs) left of the helper and above the tray: 100, 75, 50, 25 percent with the current level checked, a separator, Mute checked while muted; a row sets the level through the mixer. It stands left of the helper because the paperclip sits right above the tray and took the clicks on a popup at the speaker (found by the acts, which run the assistant). Door cell volume-door (tests/doors/xp.sh) now clicks Mute in the popup.

Deviation to note: the popup is a stepped list in the context-menu look (four levels and Mute), not a draggable slider; the mouse reaches every level and Mute, and the level shows in the tooltip and the checks. A drag slider is a later polish.

The persona round could not see xp-26, xp-31 and xp-33 for harness reasons: its `wpctl` stand-in always answers 0.50 (so a set never shows), Help was left open when it pressed Super+F1, and it pressed Super+D once and did not press it again or click the task button. The new acts hold all three.

Harness change (tests/acts): a look's module may name `STAND_INS` (programs of the system, here a wpctl that keeps its volume and mute); the session puts them first on PATH. Fallback art only; no Microsoft files.

Commits: 5ac45053 (guards), 24640558 (red), 81a29fe8 (fix).

## Review r1
Reviewer lane (cloud, read-only on code): claude/xp-majors head 81a29fe8 against base 30727c82. Step 0: unshallow, cargo check and build green.

Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 4 MINOR).

Checks:
1. Red first: acts xp with the same binaries (full workspace build per sha): 24640558 fails the speaker acts (a click on the speaker opens the volume popup, 75 percent, Mute twice, the tooltip after the popup, Escape; I4 xp-ctx declared open and is not); 81a29fe8 passes 56 of 56. The red act file against head binaries passes 56 of 56, so the red comes from the product, not the act.
2. Gone before, by one-line revert in a scratch worktree: xp-31 (`self.hidden.clear()` at the head of toggle_desktop) fails act "a second Super+d brings them back"; xp-33 (line `Super+F1: keys` out of xp.yaml) fails act "the keys key opens the key list"; the menu tick of xp-26 (`.checked(false && ..)`) fails unit test the_volume_popup_checks_the_level_or_mute (not an act; the acts do not read the tick). The tooltip act compares the text with the level the stand-in holds (55, then 75), so a stale or missing read fails it; no revert run. xp-23 and xp-27 cells (gate-xp-paperclip, gate-xp-question) pass on the head; the capture of the question card shows it in the balloon under the transcript, in the chat font and frame, Yes and No, the hint once. Their asserts are weaker than the claim: rv-2, rv-3.
3. Stand-in: wpctl of STAND_INS keeps volume and mute in a file, answers get-volume in the real format (`Volume: 0.55 [MUTED]`), takes set-mute 1|0|toggle and set-volume N%, N%+ and N%-; it hides no product fault (the tooltip and the popup read it back after each change).
4. xp-32 live (scratch probe on the harness, chora-assistant running): popup union 1103,641 to 1190,770 at 1280x800 (paperclip at 1192) and 847,609 to 934,738 at 1024x768 (paperclip at 936): on screen, left of the helper, rows 100, 75, 50, 25, separator, Mute; a click at 400,300 closes it; text "qz" and Up while it is open reach no client (the probe log stays empty; no control run with the popup closed). Stepped list instead of a slider: the popup is plain, in the context-menu look, every level and Mute one click away, reachable by the mouse and found by the tooltip and Help words: MINOR (rv-1), not MAJOR for Erika or Murat.
5. tests/doors/xp.sh volume-door passes (clicks the speaker, then Mute in the popup, then wpctl set-mute is marked), log-off check passes.
6. Unit test for the popup in tests_moved.rs plus context.rs; tools/size-ratchet: 0 files over 1500.
7. cargo test -p chora-xp: 306 passed; cargo clippy --workspace --all-targets -D warnings: clean; acts xp: 56 pass, 0 blocking.
8. No binary in the diff (8 files, all text).

Findings:
- rv-1 | MINOR | plugins/chora-xp/src/context.rs:284 | the speaker popup is a stepped list of four levels with Mute, not the XP slider; the 55% a key sets has no row and no tick | known issue for the polish round: a vertical slider with Mute below, or a fifth row for the current level.
- rv-2 | MINOR | tests/gate-xp-question.sh:64 | the cell says the card stands under the transcript, but only asserts the card inside the balloon, never that its top is below the transcript's last line, nor the font; moving the card over the transcript would pass | add the transcript rect (or the node above the card) and assert card.y >= its bottom.
- rv-3 | MINOR | tests/gate-xp-paperclip.sh:29 | the header says "its art drawn", the cell asserts position and visible only | assert the figure's picture size or a pixel of the capture, or drop the words.
- rv-4 | MINOR | tests/acts/looks/xp.py:216 | the popup is guarded at 1280x800 only, with no act for a click outside or for keys not leaking (Escape only), and the unit test a_click_on_the_speaker_opens_the_volume_popup (tests_moved.rs:420) says "does not mute" but asserts only that the popup opens | add a Session size 1024x768 act with the popup inside the screen and left of hud-figure, a click outside that closes it, and an assert that the mixer is unchanged after the first click.
