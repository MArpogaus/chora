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
