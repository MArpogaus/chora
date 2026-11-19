# Tiling HUD

The look of the assistant in the tiling flavor, in the Paper and Ink tokens
of `chora-tiling`. It is the neutral folder without the "Chat ..." row and
the rows of the helpers: the bar of tiling has the word of the helper
(`No helper`, or the name and the state of a helper), and a click on it
opens the chat, so no row floats over the tiles. The dot at the bottom right
changes its colour with the state of the voice agent, and a click on it
opens the chat, which shows as a plain panel of 400 pixels at the right edge.
The text uses the `sans-serif` family of the system; the folder has no font
file.

| File | Content | Source | License |
|------|---------|--------|---------|
| `hud.yaml` | the figure, the bubble and the block `chat:` | drawn for chora | GPL-3.0-or-later, as chora |
| `dot.svg` | the dot, one group for each state with the ids `idle`, `listening`, `thinking`, `working`, `speaking`, `question` and `asleep`, 24 by 24 pixels | drawn for chora | GPL-3.0-or-later, as chora |

The shipped rules of `huds` pick it for the space of tiling. Copy the folder
to `~/.local/share/chora/huds/NAME`, change it, and check it with
`chora-assistant --check-hud DIR`. The format is described in `docs/hud.md`.
