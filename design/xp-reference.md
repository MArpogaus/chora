# xp reference sources (user 11-12 04:27 UTC; do not lose)

- **Sister project:** github.com/MArpogaus/kde-plasma-winxp-launcher (pixel-perfect
  XP on KDE Plasma 6; read at 868b5b9). Use it as the behaviour and pixel
  reference for chora-xp (plasmoids winxptaskbar and winxptray, Kvantum,
  look-and-feel layouts, docs/preview-*.png).
- **Asset import:** its `get-microsoft-files.sh` (82 lines, GPL-2.0-or-later) is
  the model for chora's optional import. It fetches, never redistributes:
  - github.com/rozniak/xfce-winxp-tc (sparse: fonts/ttf, icons/luna,
    wallpapers/common, themes/luna/blue/Resources/start_flag.png);
  - github.com/wonordel/Windows-XP-Sound-Theme-KDE (sounds).
  It installs to `~/.local/share/{fonts,icons,sounds,wallpapers}/WindowsXP`
  and `winxp-kde/start_flag.png`. It links icons from xfce-winxp-tc's
  `icons/luna/mappings` ("context/name-->image") into each size, sets
  Tahoma 8 (7 smallest) and Trebuchet MS 10 bold for titles, and Bliss.
- **chora port (week-2 lane):** a `chora-xp-import` with the same sources and
  layout, minus kwriteconfig6/plasma calls; chora-xp reads those paths when
  present and falls back to our own art. Audit cells run with and without.
- Tahoma comes from xfce-winxp-tc's fonts/ttf through the import, so the
  "Wine Tahoma" fallback is only the no-import case.
