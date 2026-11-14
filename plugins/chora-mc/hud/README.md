# mc HUD

The HUD folder of `chora-mc` for the HUD host `chora-assistant`, after
`docs/mc-style.md`: no box, Inter for the frame, the accent of mc
on the state word. At the top right of the work area, on the first line of
the margin column of mc (19 pixels from the top and 56 from the right edge),
a small mark shows the state of the voice agent with its state word at 14
pixels in `accent`, and the sentence stands beside them as plain text
without a box. The helpers are mc's own rows in its column, in its colours
of state, and the door of the chat is its line `Chat with a helper ...`:
the host draws no row and no pill there. A click on the mark mutes the mic,
and so does a click on the word `Mute`, which mc draws at the left of that
line; the badge of the host turns the mic on again.
The chat is a plain page in the colours of mc, set in TeX Gyre Pagella 18
(a serif of the system where the host does not have it), the speakers in
`muted` on lines of their own; the question card takes its fill and its
rule, its title and key line in Inter 500, its prompt in the serif of the
chat and its path in a mono face.

mc uses this folder with `voice_hud: host` in `chora-mc.yaml`: mc then
draws no voice state, sentence or question of its own and claims no
indicator; it keeps the first line of its column free for the state word.
mc also gives the host its theme, so that the chat and the card are dark
in the dark theme even where no window manager claims the screen. While the host shows the chat on the output of mc, mc folds its
notifications, hides its key list and emits the box of its margin column
under the head `Chat` as `mc.chat.slot {output, rect}`. The block
`chat:` names that box with `slot: mc`, so the host draws the chat inside
the column; without a box the chat stands at the right edge, 400 pixels
wide.

| File | Content | Source | License |
|------|---------|--------|---------|
| `hud.yaml` | the mark of each theme, the sentence and the block `chat:` | drawn for chora | GPL-3.0-or-later, as chora |
| `state.svg` | the mark of the state in the light theme, one group for each state with the ids `idle`, `asleep`, `listening`, `thinking`, `working`, `speaking` and `question`, 16 by 16 pixels, in the accent and the `muted` of the light page | drawn for chora | GPL-3.0-or-later, as chora |
| `state-dark.svg` | the same marks in the colours of the dark page; the panels `state` and `state-dark` of `hud.yaml` take each its theme by `when: {theme: ...}` | drawn for chora | GPL-3.0-or-later, as chora |

The fonts are those of the system, as for mc itself. The question card and
the badge of a muted mic are the host's; the card takes the page of this
folder. The format is in `docs/hud.md`.
