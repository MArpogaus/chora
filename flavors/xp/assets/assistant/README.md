# XP assets of chora-assistant

The look of the assistant in the XP flavor: the paperclip with eyes and
eyebrows above the right end of the taskbar, a pale yellow bubble with a
black border and a tail for its sentences, and flat buttons in the colors of
XP for a question. The XP flavor passes this folder to chora-assistant
with `--assets`. The paperclip is drawn in code, in
`plugins/chora-assistant/src/paperclip.rs`, until it becomes SVG files with
the XP step X7; this folder has no picture.

| File | Content | Source | License |
|------|---------|--------|---------|
| `values.yaml` | the font family, the paperclip as the figure, the distance from the corner, the look of the bubble and of the buttons | drawn for chora after XP's balloon tips | GPL-3.0-or-later, as chora |
| `fonts` | a link to `../../theme/fonts`: Tahoma of the Wine project and Libre Franklin | see `flavors/xp/theme/README.md` | LGPL-2.1-or-later and OFL-1.1 |
| none, drawn in code | the paperclip of the voice agent, its eyes and eyebrows | drawn for chora, an original paperclip character; no artwork of Microsoft's Office assistant was used | GPL-3.0-or-later, as chora |

The keys of `values.yaml` are in the section chora-assistant of `docs/configuration.md`.
