# Neutral assets of chora-assistant

The look of the assistant when no flavor names another asset folder
(`--assets`, `CHORA_ASSISTANT_ASSETS`): a dot that changes its color with
the state of the voice agent, a flat bubble for its sentences, and flat
buttons for a question. The text uses the `sans-serif` family of the
system; the folder has no font file.

| File | Content | Source | License |
|------|---------|--------|---------|
| `values.yaml` | the font family, the figure, the distance from the corner, the look of the bubble and of the buttons | drawn for chora | GPL-3.0-or-later, as chora |
| `dot.svg` | the dot, one group per pose with the ids `idle`, `listening`, `thinking`, `speaking`, `asking` and `sleeping`, 24 by 24 pixels | drawn for chora | GPL-3.0-or-later, as chora |

## values.yaml

| Key | Meaning |
|-----|---------|
| `font` | the base font family; `sans-serif` is the family of the system |
| `figure` | `dot`, which draws the groups of `dot.svg`, or `paperclip`, the character of the XP flavor that the code draws until it becomes SVG files; a `paperclip` folder has no `dot.svg` |
| `margin` | the distance of the figure from the corner of the output, x and y in pixels |
| `speech.fill`, `speech.border`, `speech.text` | red, green and blue of the inside, the border and the text of the bubble |
| `speech.radius`, `speech.tail`, `speech.tail_base`, `speech.tail_inset` | the radius of the corners, the height of the tail (0 for none), its width at the bubble and the distance of its tip from the edge of the bubble that is nearer to the corner |
| `buttons` | the same keys for the buttons Ja and Nein of a question; the tail is ignored |
