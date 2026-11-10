# construct HUD

The HUD folder of `chora-construct` for the HUD host `chora-assistant`,
after `docs/construct-style.md` in its light theme: one glass card at the
top centre of the work area with the state ring, the host's state word
and one line of speech below it, then one chip per helper with its state.
A click on the ring mutes the mic; a click on a chip opens the chat with
that helper. The chat is a glass panel at the right, 480 pixels wide, in
Inter 16: the fill is the light glass over a black pane (#F0EDEA), the text
is `text`, the names of the speakers are `muted`, and the edge is `edge`
on that glass. The glow of construct is a shader and has no place in a
folder. The block `chat:` has no margin, so the panel touches the edges
of the work area instead of keeping 32 from them.

construct uses this folder with `voice_hud: host` in
`chora-construct.yaml`: construct then draws no voice HUD and no question
glass of its own and claims no indicator.

| File | Content | Source | License |
|------|---------|--------|---------|
| `hud.yaml` | the ring, the sentence, the chips and the block `chat:` | drawn for chora | GPL-3.0-or-later, as chora |
| `ring.svg` | the state ring in `accent`, one group for each state with the ids `idle`, `asleep`, `listening`, `thinking`, `working`, `speaking` and `question`, 32 by 32 pixels | drawn for chora | GPL-3.0-or-later, as chora |

The contrast test of rule 5 in `src/config.rs` checks the colours of the
block `chat:`. The fonts are those of the system, as for construct
itself. The question card and the badge of a muted mic are the host's.
The format is in `docs/hud.md`.
