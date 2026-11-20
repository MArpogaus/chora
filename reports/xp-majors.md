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
