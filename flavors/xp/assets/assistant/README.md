# XP assets of chora-assistant

The look of the assistant in the XP flavor: the paperclip with eyes and
eyebrows above the right end of the taskbar, a pale yellow bubble with a
black border and a tail for its sentences, and flat buttons in the colors of
XP for a question. The XP flavor passes this folder to chora-assistant
with `--assets`, which reads `values.yaml`. The same folder is also a HUD
folder (`docs/hud.md`): `hud.yaml` holds the sentence balloon, a tray
balloon of helper rows while a helper works and no chat is open (the
host does not draw helper rows yet), and the chat balloon, 340 pixels wide at the bottom right, in Tahoma 14
with the 1-pixel border of an XP edit box (#7F9DB9). A click on a row or
on the sentence balloon opens the chat. The paperclip is drawn in code, in
`plugins/chora-assistant/src/paperclip.rs`, until it becomes SVG files with
the XP step X7; this folder has no picture.

| File | Content | Source | License |
|------|---------|--------|---------|
| `hud.yaml` | the panels of the HUD folder and the block `chat:` | drawn for chora after XP's balloon tips | GPL-3.0-or-later, as chora |
| `values.yaml` | the font family, the paperclip as the figure, the distance from the corner, the look of the bubble and of the buttons | drawn for chora after XP's balloon tips | GPL-3.0-or-later, as chora |
| `fonts` | Tahoma of the Wine project, a copy of `../../theme/fonts/tahoma.ttf`: a HUD folder loads fonts only from files inside it; the licence text is `../../theme/fonts/LICENSE.Tahoma-Wine.LGPL-2.1` | see `flavors/xp/theme/README.md` | LGPL-2.1-or-later |
| none, drawn in code | the paperclip of the voice agent, its eyes and eyebrows | drawn for chora, an original paperclip character; no artwork of Microsoft's Office assistant was used | GPL-3.0-or-later, as chora |

The keys of `values.yaml` are in the section chora-assistant of `docs/configuration.md`.
