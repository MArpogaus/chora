# XP assets of chora-notify

The look of the balloon tips of XP: a pale yellow card with a black border
and a tail that points at the notification area of the taskbar of
chora-xp. The XP flavor passes this folder to chora-notify with
`--assets`. The notification area is at the bottom right, which is the
default `position` of chora-notify; `margin` puts the tail tip at the
tray.

| File | Content | Source | License |
|------|---------|--------|---------|
| `values.yaml` | the font family, the distance of the balloon from the corner, the colors, the corners and the tail | drawn for chora after XP's balloon tips | GPL-3.0-or-later, as chora |
| `close.svg`, `close-hover.svg` | the close button of a balloon tip, normal and under the pointer | drawn for chora after XP's balloon tips | GPL-3.0-or-later, as chora |
| `fonts` | a link to `../../theme/fonts`: Tahoma of the Wine project and Libre Franklin | see `flavors/xp/theme/README.md` | LGPL-2.1-or-later and OFL-1.1 |

The keys of `values.yaml` are in `plugins/chora-notify/assets/README.md`.
