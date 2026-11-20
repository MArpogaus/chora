# mc notify

The slot of the notices of `chora-mc` for `chora-notify`, the only
notification server beside mc: `notify.push`, the notices of mc itself
and `notify-send` all show as a card of chora-notify. Nothing else of mc
draws a notice.

The card stands at the bottom left of the first output, 160 pixels from
the left edge, which is the left edge of the text block of a page, and 56
pixels above the bottom: in the two lines between the text of the page
and its footer line. It never covers the margin column, where the chat,
the key list and the doors stand. A card of a title and one line of text
fits the two lines; a longer text grows into the last lines of the page
until the person closes the card or its time of 10 to 30 seconds ends.
The colors are those of the dark theme of mc; the font is Inter at 19 pixels, the size of the body of a page, so that the card reads from the sofa.

chora-notify starts with this folder as
`chora-notify --assets /usr/share/chora/mc/notify --config /usr/share/chora/mc/notify/notify.yaml`.

| File | Content | Source | License |
|------|---------|--------|---------|
| `values.yaml` | the font and its size, the distance of the card from its corner, the colors and the corners of the card | drawn for chora after `docs/mc-style.md` | GPL-3.0-or-later, as chora |
| `notify.yaml` | the option `position` and the times of chora-notify | drawn for chora | GPL-3.0-or-later, as chora |
| `close.svg`, `close-hover.svg` | links to the close button of the neutral folder of chora-notify | drawn for chora | GPL-3.0-or-later, as chora |

The keys of `values.yaml` are in `plugins/chora-notify/assets/README.md`.
