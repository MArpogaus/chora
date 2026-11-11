# Taste decisions by the PM (user away until Saturday, 11-11)

The user gave taste decisions to the PM until Saturday ("You work
autonomously until sat"). Each is reversible; the user may override any.

## Tiling v2 (design/looks/tiling-v2.md, mocks media/2026-11-12/design/tiling/)

- Q1 bar shape: **A, three floating islands.** It is the Hyprland
  signature the user asked for and keeps the windows' gaps visible
  around the bar. Rejected B (flush bar: plainer, sway-like) and C
  (one floating bar: heavier block).
- Q2 focus mark: **A, gradient border plus a deeper shadow.** It is the
  one place colour moves on screen, which makes focus unmistakable;
  the gradient uses only the accent and its second hue, so the "one
  accent" rule bends but holds. Rejected B (dimming hurts reading of
  side windows) and C (too plain for "exciting").
- Q3 default theme: **C, follow the system setting, Ink when unset.**
  POLS: a light-mode user gets Paper without config. Rejected A and B as
  fixed defaults.
- Seen on mock A: tag 5 in red needs a meaning in the doc (urgent) or
  goes; the Keys and power entries are right.

## mc v2 (design/looks/mc-v2.md, mocks media/2026-11-12/design/mc/)

- Q1 column separation: **A, a hairline only.** The printed-page idea
  wants one rule, not a second paper; KISS. Rejected B (tint reads as a
  panel, the thing the user asked to remove).
- Q2 key list: **A, a block at the column foot.** Every swarm tester
  failed to find keys in mc; it must be visible without a key. Rejected B
  (one footer line is easy to miss) and C (hidden after first start).
- Q3 chat type: **A, the book serif with the speaker on its own line.**
  It keeps one voice across page and margin; labels and the card stay in
  the sans. Rejected B (denser but breaks the page aesthetic).
- Accepted: agenda moves from Super+a to Super+Home, because Super+a is
  the chat key in every look (shared invariant).
- Watch: mc is read from the sofa; nothing in the column below 14 px at
  1280, and the card keys stay as large as on the mock.

## construct v2 (design/looks/construct-v2.md, mocks media/2026-11-12/design/construct/)

- D1 to D8 accepted as written: glass start card on the empty floor;
  programs open flown in with the "Alt Esc Step back" hint; cube rests
  14 degrees off square; default faces XP, Tiling, Media center, XP 2,
  each labelled; live turn while dragging, settles in 320 ms; failing
  face says "Starting", then "did not start" with the reason and Return
  to retry; no red, word plus hollow dot; walking pane by pane.
- Open point "fly in always or a setting": **always**, no setting. One
  behaviour is the least surprise and the simplest; the hint names the
  way back. Rejected: a setting nobody finds.
- Three managers on one cube: try; if the idle or turn frame cost breaks
  the gate budgets, fall back to four XP faces with distinct wallpapers.
- Cross-look decision: **Super+Space opens the program launcher in every
  look** (construct, mc already; tiling moves its layout cycle to
  Super+Shift+Space). The product promise is four desktops in one
  session; one launcher key across them beats tiling custom. Rejected:
  per-look launcher keys (swarm finding "keys differ per look").

## xp v2 (design/looks/xp-v2.md, mocks media/2026-11-12/design/xp/)

- D1 to D15 accepted as written (Large Fonts sizing, F1 help window built
  from live bindings, "Super" naming, Stop greyed not hidden, no default
  button on the question card, icon-only tray volume, desktops 1 to 4,
  Recycle Bin above the paperclip). The mock reads as XP at first glance
  without its art; that is the bar.
- Open "Super+E": **opens the file manager** (Win+E means files in XP);
  when none is installed, a balloon says so and names the setting. It no
  longer opens a terminal. Rejected: keep terminal (surprises every XP
  user).
- Cross-look rule applied: Super+Space opens the program launcher (in xp
  the Start menu's Run box); Super alone still opens Start.
