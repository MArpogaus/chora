# construct HUD

The HUD folder of `chora-construct` for the HUD host `chora-assistant`,
after the glass of `docs/construct-style.md` (rules 18 to 25): the voice
island at the top right, 32 from the edges of the work area, with the
state ring, the state word and the task of the busy helper (`style:
island`); it holds the sentence while the agent speaks. A click on the
island opens the chat. The chat is the glass column at the right, with
the tint, the text and the muted colour of the glass tokens as `fill`,
`text` and `name`, in both themes.

construct uses this folder with `voice_hud: host` in
`chora-construct.yaml`: construct then draws no voice HUD and no question
glass of its own, only its own notices, and claims no indicator.

| File | Content | Source | License |
|------|---------|--------|---------|
| `hud.yaml` | the island and the block `chat:` | drawn for chora | GPL-3.0-or-later, as chora |
| `ring.svg` | the state ring in `accent`, one group for each state with the ids `idle`, `asleep`, `listening`, `thinking`, `working`, `speaking` and `question`, 24 by 24 pixels | drawn for chora | GPL-3.0-or-later, as chora |

The contrast test of rule 5 in `src/config.rs` checks the colours of the
block `chat:`. The fonts are those of the system, as for construct
itself. The question card and the badge of a muted mic are the host's.
The format is in `docs/hud.md`.
