# Review: tiling v2 "Ink", pixels

Independent review of branch `claude/tiling-v2` (head 90060ba, built in
`/home/user/wt/tiling-v2/target/debug`), 2026-11-11. No code changed.
Captures: `media/2026-11-12/review-tiling-v2/` (`s1-*` scale 1 at
1280x800, `s2-*` scale 2 at 1280x800 logical = `2560x1600@2`, `crop-*`
zoomed with nearest-neighbour). Compared against `design/looks/tiling-v2.md`,
mock A, `design/decisions-taste.md` (1A islands, 2A gradient, 3C Ink when
unset) and the chora-tiling RC requirements in `design/product-briefs.md`.

## How the captures were made

- `tests/persona/session.sh tiling 1280x800` (and `2560x1600@2`) with
  `CHORA_BIN` set to the branch build and `PERSONA_KEEP` set to a scratch
  dir; `act.py look --max-side 1280` (and `--max-side 2560` for pixel work),
  `key`, `click`; `session.sh --stop` at the end. Processes were killed by
  pid only.
- **session.sh starts waybar without `--style`** (`companions` line
  `waybar --config .../config.jsonc`). That gives GTK defaults: no islands,
  a grey clock box and a blue "Connected" box
  (`s1-00-session-sh-unstyled-waybar.png`). I killed that waybar by pid and
  started `flavors/tiling/waybar/waybar.sh` the way the builder's
  `tests/gate-tiling-bar.sh` and the unit `chora-tiling-waybar.service` do,
  so every later capture shows the real bar.
- swaybg ran as session.sh and the shipped unit start it,
  `swaybg --color #2e3440`. The builder's captures have `#000000` where
  the wallpaper should be, so swaybg did not run in their session.

## First look (before reading the design)

The builder's three captures show three islands on pure black. The
terminals are foot defaults: `#242424` body with yellow and white text at
the very left edge of each window. The focused window has a lilac-to-teal
border and the unfocused ones a dim grey border. In the key list, two
entries are drawn over each other ("Fewer/More main windows"). The
"Layouts" heading appears in both columns. Keycaps and labels look like
DejaVu Sans next to the Inter bar, and the gap after each keycap is
uneven. The chip capture has a yellow square in the top-left corner, and
"Tile left" sits left of centre in its chip.

## MAJOR (13)

1. **Key list rows overlap.** "Fewer main windows" and "More main windows"
   are drawn on the same line at y 206 (logical), beside cap
   `Super+Shift+l`. The text is unreadable. The other row's cap is hidden
   under that cap. Crops: `crop-s1-keys-overlap-x3.png`,
   `crop-s2-keys-right-overlap.png`. Cause: `keys.rs::rows_of` repeats the
   group title at the top of column 2 and moves that row down by `ROW`
   (`y += ROW`). The next flow item (at = 1) is still placed at
   `88 + ROW*1`, the same y. Fix: count the repeated title as a flow slot.
   Either insert `Line::Title` into the flow at the break before placing
   items, or keep a running `y` per column instead of `at * ROW`. Then make
   `per_column` count the extra title. Add a test that no two text nodes
   share a y within a column.

2. **Wallpaper is not Ink.** The shipped unit runs `swaybg --color #2e3440`
   (Nord grey), which is what session.sh shows (`s1-03-three-windows.png`,
   pixel (5,400) = `#2e3440`). The builder's capture is `#000000` (no
   swaybg). The design and mock want `base #11131a` with two corner glows
   (mock pixel (0,400) = `#101119`, top-left glow `#28204c`). The flavor
   has no `wallpaper.png`. Verified, and slightly worse than the PM saw:
   the real session is grey-blue, not black. Fix: ship
   `flavors/tiling/wallpaper.png` (base plus glows, or at least
   `--color #11131a`) and change `chora-tiling-swaybg.service` and the
   session.sh companion to use it.

3. **Unfocused border is invisible on the shipped wallpaper.** The border
   `line #2e3344` on `#2e3440` gives 1.00:1. The right windows in
   `s1-03-three-windows.png` have no visible edge; only the body
   `#242424` against the wallpaper outlines them
   (`crop-s1-gaps-6px-unfocused-border-x4.png`). On the designed base it
   would be 1.48:1. The fix follows from 2. Also consider `#3a4054` for
   `line` so the hairline shows on base (about 1.9:1).

4. **Gaps are smaller than the design and do not line up with the bar.**
   Measured border-to-border: outer gap 10 px (x 0-9, x 1270-1279,
   y 790-799), inner gap 6 px (x 637-642, y 417-422), bar to tiles 10 px
   (y 40-49). Design and mock: outer 12, inner 10, bar to tiles 12. The
   bar island starts at x 12, but the window border starts at x 10, a 2 px
   step on both sides (`crop-s1-bar-vs-window-edge-x6.png`). Scale 2 gives
   the same values: 20 / 12 physical px. Cause: `tiling.yaml` says
   12 / 10, but the 2 px border is drawn outside the window rect and
   `wm.rs:670` does not count it. Fix: inset each tile by `border` before
   placing the window (`edge = outer - inner/2 + border`,
   `half = inner/2 + border`), or measure gaps from the border's outer
   edge. The build-list check "12 / 10 px" fails today.

5. **Terminal text touches the window border, and the corner clips it.**
   The first glyph column starts at x 13, while the border runs x 10-11
   on straight edges and up to x 12 in the corner curve. The "w" of
   "warning" is cut by the rounded corner at y 56-61. Crops:
   `crop-s1-terminal-text-at-border-x4.png`,
   `crop-s2-terminal-corner-x3.png` (2x, the clip is obvious). Verified as
   the PM saw it. Fix: ship a `flavors/tiling/foot.ini` with `pad=12x12`
   (mock about 20 px). It belongs with item 6.

6. **Terminal is unthemed foot.** Body `#242424`, foreground white, yellow
   `#e3c401`: none are on the palette (mock body `#161922`, text `#e6e8ef`).
   The font is foot's default monospace at about 7 px cap height
   (≈10 px). The design asks for JetBrains Mono and nothing under 14 px
   that must be read (P7). Verified. Fix: `foot.ini` with
   `font=JetBrains Mono:size=10.5` (14 px), `[colors] background=161922`,
   foreground `e6e8ef`, and the 16 ANSI colours mapped to the Ink tokens.
   Start foot with `-c $CHORA_FLAVOR_DIR/foot.ini` in the `MOD+Return`
   binding.

7. **No chat segment in the right island; the assistant's "Chat ..." row
   collides with the bar.** chora-assistant draws its own white
   `#fafafa` row at logical x 1194-1263, y 17-47. It sits under the
   right island: it shows through the 92 % island as a lighter patch
   (`#2c2e38` against `#1c1e29`) behind "Keys" and the power icon, and it
   sticks out 8 px below the island (y 40-47). The text "Chat ..." is
   clipped. Crops: `crop-s1-chat-row-under-bar-x4.png`,
   `crop-s2-bar-right.png`. Verified, and worse than "missing": it is
   visible as a broken white tab. Fix: as `tiling.md` says, chora-assistant
   draws the segment at height 32 and radius 10 in the Ink tokens, in
   line with the right island (x end 1256, y 8-40). Or waybar gets a
   `custom/chat` module and the assistant hides its row in the tiling
   flavor.

8. **Apps launcher (Super+space) is unthemed fuzzel.** Translucent beige
   `#e1dbca` panel with monospace grey-blue text. Text contrast is 3.22:1,
   and the selected row is `#d4cebe` on `#e1dbca`. The window border and
   the gap show through it (`crop-s2-apps.png`, `s1-08-apps.png`). Off
   palette and below 4.5:1. The design asks for "fuzzel themed in the
   palette, centred, radius 12". Fix: ship `flavors/tiling/fuzzel.ini`
   (font Inter 14, `background=1a1d27f0`, `text=e6e8efff`,
   `match=a78bfaff`, `selection=242836ff`, `selection-text=e6e8efff`,
   `border=2e3344ff`, `radius=12`, `border-width=1`) and run
   `fuzzel --config` with it from the binding and the Apps button.

9. **Empty tags fail contrast.** Digits 3-9 are `#5b5f6f` on `#1c1e29`:
   2.61:1 (`crop-s1-tags-contrast-x3.png`). `alpha(@muted, 0.5)` is in the
   design, but the design's own build check says "contrast script passes
   4.5:1 on every label", and P7 says that what must be read is readable.
   These are doors you click. Fix: empty = `@muted` at full strength
   (6.4:1), occupied = `@text` at weight 600, or mark occupied ones with a
   2 px dot. Do not dim below 4.5:1.

10. **Key list uses the wrong fonts.** Keycaps should be JetBrains Mono 13,
    but they render proportional: "Super+BTN_LEFT", "Super+Tab" and
    "Super+Ctrl+Return" have different per-character advances
    (`crop-s2-keys-left.png`). Labels, titles and the "Keys" heading render
    in DejaVu Sans, not Inter, beside an Inter bar. That is two families
    where the design has Inter plus JetBrains Mono. Cause: `keys.rs::text`
    passes `family: None` for UI text, which falls back to the default
    (DejaVu Sans), and a comma list `"JetBrains Mono, DejaVu Sans Mono,
    monospace"` for mono, which the renderer evidently does not resolve as
    a fallback list. Fix: pass `"Inter"` for UI text and `"JetBrains Mono"`
    (a single family) for the caps, or teach the renderer comma fallback.
    Add a test that renders "iiii" and "MMMM" in a cap and compares widths.

11. **Key list keycaps are sized by guess, so their padding is ragged and
    text touches the edge.** The cap width is `chars * 7.9 + 12`, not the
    measured text. Right padding inside the cap is 2 px for
    "Super+BTN_LEFT", 4 px for "Super+Ctrl+m", 11 px for "Super+Shift+c"
    and 16.5 px for "Super+Ctrl+Shift+space", against 6 px on the left.
    Label start x therefore wanders between 317 and 421 in one column
    (`crop-s1-keys-fonts-caps-x2.png`). Fix: measure the text (or use mono,
    where 7.8 px per char at 13 px is exact) with 6 px on both sides, and
    start every label of a column at one x (the widest cap of the column
    plus 10).

12. **Layout chip text is not centred.** Chip box x 558-721 (164 wide),
    y 648-703. "Tile left" ink spans x 587-655: 29 px left, 65 px right,
    36 px off centre. Vertically the cap height spans 665-679 inside
    649-702: 16 above, 23 below, 3.5 px low
    (`crop-s1-chip-offcentre-x3.png`, `crop-s2-chip.png`). Cause:
    `keys.rs::chip` estimates width `chars * 12 + 56` and places text at a
    fixed `+28, +14`. Fix: measure the word (or let the text node centre
    in a sized box) and centre on both axes, with padding 24 / 16.

13. **Persona session shows an unstyled bar.** session.sh starts
    `waybar --config` without `--style` and without waybar.sh
    (`s1-00-session-sh-unstyled-waybar.png`): GTK defaults, the grey clock
    box, the blue "Connected", digits run together "Apps123456789Tile".
    Every persona run reviews a look that ships to nobody. Fix: the
    session.sh companion line becomes
    `sh $repo/flavors/tiling/waybar/waybar.sh $repo/flavors/tiling/waybar`,
    the same as the unit.

## MINOR (9)

14. **Key list heading duplicated and groups not separated.** "Layouts"
    sits at the foot of column 1 (y 545) with five rows, and again at the
    top of column 2. Group titles sit on the same 20 px pitch as rows, so
    "Layouts" at y 545 sits right under "Window that asks" with no air.
    Fix: break columns only between groups (balance by group), and add
    8 px above each title.

15. **Key list words are not words.** "Super+BTN_LEFT" (should be
    "Super+drag" or "Super+left button"), "Super+slash" (the header says
    "Super+/"), "Super+1-9" (mixed with "1-9" spelled the same in
    "Show tag 1-9"). Fix: map `BTN_LEFT` to "drag" and `slash` to "/" in
    the key-name words.

16. **Key list panel is 880 x 620, not 880 x 560,** with 43-54 px of
    empty surface under the last row (y 666-709) against 24 px at the top.
    Fix: follows from 14 (balanced columns). Set height to content plus
    24 px.

17. **Left island jumps width with the layout word.** It is 362 px wide
    with "Tile" and 387 with "Tile left" (`s1-07-after-chip.png` against
    `s1-04-focus-moved.png`). The clock island does not move, but the
    left island's right edge does, on every layout change and on tag
    switches between tags with different layouts. Fix:
    `min-width: 64px` on `#custom-layout`, text centred.

18. **Bar icons come from a fallback font.** "▦", "⌨" and "⏻" are not in
    Inter and render from DejaVu at different weights and sizes. "⏻" is
    10 x 11 px and its centre sits 1 px above the text centre (y 22.5
    against 23.5). The mock uses drawn icons (a 2x2 grid, a keyboard,
    power). Fix: an icon font such as Symbols Nerd Font, or SVG icons via
    `custom` modules with `format-icons`, at 16 px, centred.

19. **Island separators run the full height and meet the hairline.** The
    separators beside Apps and the layout word go from y 8 to 39, which
    makes a T-junction with the 1 px island border (`crop-s2-bar-left.png`).
    The mock insets them about 8 px top and bottom. The right island has
    none between modules, though the mock has two. Fix: draw the separator
    as a 1 x 16 px element (an `#custom-sep` label with a background, or
    `border-image`), or drop the borders and use 12 px spacing.

20. **Tooltip corners are square and its text repeats the label.** The
    "Keys" tooltip has black `#000000` 4 x 4 px squares outside its 8 px
    radius, plus a white sliver of the chat row
    (`crop-s1-keys-tooltip-corners-x4.png`). The text is "⌨ Keys", not
    the configured "Every key (Super+/)": `tooltip-format` is ignored for
    a `custom` module that `exec`s plain text. Fix: `tooltip { background:
    transparent; }` with the colour on `tooltip > box` (or radius 0 to
    match), and give `custom/keys` and `apps` `"tooltip": true` with
    `return-type: json` emitting a `tooltip` field.

21. **Built steps 8 and 9 are missing (shadows, motion).** The pixels
    right outside the focused border equal the wallpaper (no
    `0 8 28 / 45 %` shadow). The tag switch, the layout change and the
    window open are instant (no slide or fade). This is expected only if
    the plan scoped them out of this branch. List them in the branch
    notes, or build them before calling the look done.

22. **Backdrop blur behind the chip and key list has no effect.** Under
    the chip, the 6 px wallpaper gap between two windows keeps
    pixel-sharp edges (`#1c1f28` to `#1c2029` in one step at x 1270 and
    1288, 2x capture). A 24 px blur would spread it over about 48 px. It
    is barely visible because the glass is 82 % opaque, but the design
    lists blur. Check that `backdrop` applies under a node in the overlay
    root, or drop it and use 92 %.

## COSMETIC (4)

23. **Tag digit off centre by 1 px.** The "2" pill is x 118-141 (centre
    129.5), the digit 126-131 (centre 128.5). Inter sidebearing. Fix:
    `font-feature-settings: "tnum"` on the tags.
24. **Keycaps 18 px tall on a 20 px pitch leave 2 px between stacked
    caps,** which reads as a striped column. Use a 24 px pitch (4 px grid)
    with 20 px caps.
25. **The core's yellow injection mark (D23) sits under the left island's
    rounded corner.** It is at x 8-17, y 8-19 (`crop-s1-yellow-flash-x6.png`,
    `crop-s2-yellow-flash-x4.png`) and appears only after `input.inject`,
    so it is not a product flaw. The builder's chip capture shows it too.
    The bar hides part of it, which weakens D23 in this look. Fix: draw
    the mark above layer surfaces, or move it off the 8 px bar margin.
26. **"Sound unavailable" is the longest word in the right island.**
    Fine for headless, but at 1280 wide the right island is 332 px. The
    design's "Volume 40 %" is shorter. Consider "No sound".

## What is right

Island geometry matches the design to the pixel: height 32 (y 8-39),
radius 10, margins 12, `surface` at 92 % with a `line` hairline, and the
clock island centred at x 639.5. Text is centred vertically in each
island (cap centre 23.5 against island centre 23.5). The current-tag pill
is 24 x 24 and centred. Clock format, Apps and Keys doors, and the layout
word all match. The focus border is crisp at 1x and 2x (2 / 4 physical
px), runs the gradient from `#a78cfa` top-left to `#5fe9d4` bottom-right,
and the outer radius is 12 around a window clipped at 10. The two columns
are equal (627 px). Esc closes the key list and the Keys button opens it.
No blur or aliasing at scale 2.

Not measured: frame-by-frame flicker on open, close and tag switch. The
capture tool takes about 0.3 s per look, too slow for a 30 fps check.
