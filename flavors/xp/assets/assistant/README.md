# XP HUD of chora-assistant

The look of the assistant in the XP flavor, a HUD folder (`docs/hud.md`).
chora-assistant shows it for windows of chora-xp, by the rules of `huds`,
and reads `hud.yaml`: the paperclip with eyes and eyebrows above the right end
of the taskbar, which blinks, and its one balloon, pale yellow with a
1-pixel black border, as every balloon of XP has, and a tail at the paperclip.
Collapsed, the balloon shows the last sentence; a click on it or on the
paperclip expands it into the chat, 340 pixels wide, in Tahoma 14, with
the transcript, the field, Close and the card of a question inside. No
row or pill stands beside it.

| File | Content | Source | License |
|------|---------|--------|---------|
| `hud.yaml` | the panels of the HUD folder and the block `chat:` | drawn for chora after XP's balloon tips | GPL-3.0-or-later, as chora |
| `paperclip.svg` | the paperclip of the voice agent, its eyes and eyebrows, one group for each state with the ids `idle`, `asleep`, `listening`, `thinking`, `working`, `speaking` and `question`, and one with shut eyes for each but `asleep` (`idle-shut`), 80 by 116 pixels | drawn for chora, an original paperclip character; no artwork of Microsoft's Office assistant was used | GPL-3.0-or-later, as chora |
| `fonts` | Tahoma of the Wine project, a copy of `../../theme/fonts/tahoma.ttf`: a HUD folder loads fonts only from files inside it; the licence text is `fonts/LICENSE.Tahoma-Wine.LGPL-2.1`, which ships with the folder | see `flavors/xp/theme/README.md` | LGPL-2.1-or-later |

The keys of `hud.yaml` are in `docs/hud.md`.
