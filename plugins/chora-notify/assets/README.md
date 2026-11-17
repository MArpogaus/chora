# Neutral assets of chora-notify

The look of a notification card when no flavor names another asset folder
(`--assets`, `CHORA_NOTIFY_ASSETS`). A flat card with rounded corners, a
one pixel border, no tail and a round close button. The text uses the
`sans-serif` family of the system; the folder has no font file.

| File | Content | Source | License |
|------|---------|--------|---------|
| `values.yaml` | the font family, the distance of the card from its corner, the colors and the corners of the card | drawn for chora | GPL-3.0-or-later, as chora |
| `close.svg`, `close-hover.svg` | the close button of the card, normal and under the pointer, 14 by 14 pixels | drawn for chora | GPL-3.0-or-later, as chora |

## values.yaml

| Key | Meaning |
|-----|---------|
| `font` | the base font family; `sans-serif` is the family of the system |
| `margin` | the distance of the card from the corner of the output, x and y in pixels; the y of the neutral folder leaves room for the dot of chora-assistant |
| `card.fill`, `card.border`, `card.text` | red, green and blue of the inside, the border and the text |
| `card.radius` | the radius of the corners in pixels |
| `card.tail`, `card.tail_base`, `card.tail_inset` | the height of a tail below the card (0 for none), its width at the card and the distance of its tip from the edge nearer to the corner |
