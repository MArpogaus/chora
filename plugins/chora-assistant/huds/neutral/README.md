# Neutral HUD

The look of the assistant when no setting names another folder: a dot at
the bottom right that changes its colour with the state of the voice
agent, and a flat bubble for its sentences. A click on the dot mutes the
mic. The row "Chat ..." at the top right opens the chat, which shows as a
plain panel of 400 pixels at the right edge. The text uses the `sans-serif` family of the system; the folder has
no font file. The question card and the badge of a muted mic are the
host's, not the folder's.

| File | Content | Source | License |
|------|---------|--------|---------|
| `hud.yaml` | the panels: the dot, the bubble and the row of the chat; the block `chat:` | drawn for chora | GPL-3.0-or-later, as chora |
| `dot.svg` | the dot, one group for each state with the ids `idle`, `listening`, `thinking`, `working`, `speaking`, `question` and `asleep`, 24 by 24 pixels | drawn for chora | GPL-3.0-or-later, as chora |

Copy the folder to `~/.local/share/chora/huds/NAME`, change it, check it
with `chora-assistant --check-hud DIR`, and turn it on with
`chora-assistant --use-hud NAME`. The format is described in
`docs/hud.md`.
