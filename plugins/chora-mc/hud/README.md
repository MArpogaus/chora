# mc HUD

The HUD folder of `chora-mc` for the HUD host `chora-assistant`, after
`docs/mc-style.md`: no box, Inter for the frame, the accent of the dark
theme on the mark of the state and on the state word of a helper. At the
top right of the work area a
small mark shows the state of the voice agent, the host writes the state
word beside it, the sentence follows as plain text, and below it one row
per helper, `Atlas · working · Write file (2/2)`, the name in `text`, the
state word in `accent` and the task in `muted`. A click on the mark mutes
the mic; a click on a row opens the chat with that helper. The chat is a
plain page in the colours of the dark theme of mc, set in TeX Gyre Pagella
18 with hairlines in `faint`; the names of the speakers are in `muted`.

mc uses this folder with `voice_hud: host` in `chora-mc.yaml`: mc then
draws no voice state, sentence or question of its own and claims no
indicator. While the host shows the chat on the output of mc, mc opens its
drawer and emits its box as `mc.chat.slot {output, rect}`. The block
`chat:` names that box with `slot: mc`, so the host draws the chat inside
the drawer; without a box the chat stands at the right edge, 400 pixels
wide.

| File | Content | Source | License |
|------|---------|--------|---------|
| `hud.yaml` | the mark, the sentence, the helper rows and the block `chat:` | drawn for chora | GPL-3.0-or-later, as chora |
| `state.svg` | the mark of the state, one group for each state with the ids `idle`, `asleep`, `listening`, `thinking`, `working`, `speaking` and `question`, 16 by 16 pixels | drawn for chora | GPL-3.0-or-later, as chora |

The fonts are those of the system, as for mc itself. The question card and
the badge of a muted mic are the host's. The format is in `docs/hud.md`.
