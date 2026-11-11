# Design per look: tiling v2, "Ink" (proposal)

Proposal for the PM and the user, 2026-11-11, on build 06044cf. Extends `tiling.md` (people, chat
segment, voice stay) and obeys `shared.md`; where they differ, this page replaces `tiling.md`
(bar height, motion). Mocks: `media/2026-11-12/design/tiling/` (stills, labelled "mock").

## Philosophy

**Lit glass on ink.** The desk is dark and quiet; windows float on it as cards with room to
breathe, and exactly one thing glows: the window that has the keys. Light means focus, nothing
else. Every pixel of chrome is a door (a tag, a button, a key), never decoration; the eye goes to
content first, then to the glow, then to the bar. Precision over ornament: one grid (4 px), one
radius family, one accent, motion that shows where a thing came from and stops.

## Visual system

**Grid.** Everything on a 4 px unit. Outer gap 12 (output edge and bar to the tiles), inner gap 10
(between tiles). Fullscreen and `max` layout: no gap, no radius, no border (content wins, POLS).

**Windows.** Radius 10. Border 2 px inside the 10 px radius family (border rect radius 12).
- Focused: border in the accent; option A of Q2 a 135 degree gradient accent to accent-2.
  Shadow `0 8 28 / 45 %` of `#000`.
- Unfocused: border `line`, shadow `0 4 12 / 25 %`. Option B of Q2 also dims them to 88 %
  opacity (Hyprland `dim_inactive`); A does not.
- Urgent: border `urgent`, no blink (P6).
- Floating windows: the same, plus `on_top` keeps the focused shadow.

**Blur (optional).** Only behind surfaces chora draws itself: the key list and the layout chip use
a backdrop blur of 24 at saturation 1.2. Blur behind the bar (a waybar layer surface) needs new
core work (build step 12); without it the bar islands are 92 % opaque, which reads the same.

**Palette.** One accent (P6); accent-2 exists only as the far end of the focus gradient.

| Token | Ink (dark, default) | Paper (light) | Use |
|---|---|---|---|
| `base` | `#11131a` | `#e9ebf1` | wallpaper, gaps |
| `glow` | `#2a2150` / `#0f2a2a` | `#dcd6ff` / `#d3f1ec` | wallpaper corner glows |
| `surface` | `#1a1d27` | `#ffffff` | bar islands, overlays |
| `surface-2` | `#242836` | `#f2f3f7` | hover, keycaps, wells |
| `line` | `#2e3344` | `#d5d8e1` | unfocused border, hairlines |
| `text` | `#e6e8ef` | `#1b1e27` | text, 13.7:1 / 16.7:1 |
| `muted` | `#9aa1b4` | `#5b6274` | secondary text, 6.5:1 / 6.1:1 |
| `accent` | `#a78bfa` | `#6d4aff` | focus, current tag, 6.2:1 / 5.2:1 |
| `on-accent` | `#11131a` | `#ffffff` | text on accent, 6.8:1 / 5.2:1 |
| `accent-2` | `#5eead4` | `#0e9f8e` | gradient end only (graphics, 3:1+) |
| `urgent` | `#f87171` | `#d33f3f` | urgent tag and border |

Contrast measured with WCAG relative luminance on `surface`. Wallpaper: a PNG of `base` with two
soft glows (top left, bottom right), shipped in the flavor for swaybg; a ricer swaps one file.

**Type.** Inter for the UI, JetBrains Mono for keys and the key list. Bar 14 px, weight 500 for
tags and clock, 400 elsewhere. Key list 14/20, keycaps mono 13 on `surface-2` (beside the label,
not read alone). Layout chip 20 px weight 600. Nothing under 14 px that must be read (P7).

**Bar.** Height 32, 8 from the top edge, 12 from the sides, radius 10, `surface` at 92 %, 1 px
`line` hairline. Q1 decides the shape; the modules are the same:
- Left: **Apps** (grid icon and the word; runs the launcher) | tags **1 to 9** as 24 x 24 buttons:
  current = `accent` pill with `on-accent` number, occupied = `text`, empty = `muted` 50 %, urgent =
  `urgent`; a click views the tag | the **layout word** ("Tile", "Fair"); a click goes to the next.
- Centre: clock "Wed 11 Nov 15:57"; the tooltip shows the month.
- Right: the chat segment (`tiling.md`, drawn by chora-assistant at height 32 and radius 10) |
  Volume 40 % | Connected | Battery 80 % | tray | **Keys** (opens the key list) | **power** icon.
- No IP, cpu or memory (ux-loop-1, 12). Words from the word table; no "1[tile]".

**Motion.** Through the core's `animate` (cubic curves); all ease out, within the shared 120-240 ms.

| Change | What moves | Duration |
|---|---|---|
| window opens | opacity 0 to 1, scale 0.94 to 1 about its centre | 180 ms |
| window closes | opacity 1 to 0, scale 1 to 0.96 (needs core work, step 12) | 140 ms |
| layout changes, swap, master width | position slides to the new tile; size snaps at the client's commit | 200 ms |
| tag switch | the tiles slide one output width in the tag's direction | 200 ms |
| focus moves | border colour and shadow cross-fade | 120 ms |
| key list, layout chip | fade and scale 0.98 to 1; the chip leaves after 700 ms | 140 ms |

No bounce, no overshoot, no continuous motion (the gradient never rotates). `animations: false`
in the config makes every change instant (and the reduced-motion persona uses it).

## Key affordances and how a first-timer finds them

| Task | Mouse door (P3) | Key | Notes |
|---|---|---|---|
| see every key | **Keys** in the bar | `Super+/` | list built from the bindings, so a ricer's new key shows |
| start a program | **Apps** in the bar | `Super+r` | fuzzel themed in the palette, centred, radius 12 |
| switch tag | click a tag number | `Super+1`..`9` | current tag always lit, also when empty |
| change layout | click the layout word | `Super+space` | the chip names the new layout, so the key no longer looks dead |
| lock, log out, restart, shut down | **power** icon in the bar | `Super+Shift+Escape` locks, `Super+Shift+e` the menu | menu of four words; swaylock in the palette |
| screenshot | Apps: "Screenshot" | `Print` whole output, `Super+Shift+s` area | file in `~/Pictures/Screenshots`, copy in the clipboard, notice with the path |
| chat | chat segment | `Super+a` | unchanged from `tiling.md` |

**Key list.** Centred panel 880 x 560, radius 14, `surface` with blur, two columns of groups:
Windows, Layouts, Tags, Programs, System. Each row: keycaps (mono) then the word ("Next window").
The words come from `chora-kit::words` per action; a `run` binding shows its `label` or else its
command. `Super+/`, Esc or a click outside closes it; it never takes the keys from a window
without that act and gives them back on close (P11).

**Unbound Super chords** type nothing (M5): tiling swallows every `MOD+key` it does not bind.

**First run.** No popup or tour (P2, Calm). The two words "Apps" and "Keys" in the bar are the way
in; the key list's header says "Super+/ shows this list again. Esc closes it."

## Hackability: what to change where

| To change | File (user copy under `~/.config/chora/`) |
|---|---|
| gaps, radius, border width, shadow, animations on/off and durations, palette | `chora-tiling.yaml`, new section `look:` (values only, checked by `settings.set`) |
| a key, a program on a key, its label in the key list | `chora-tiling.yaml`, `bindings:` and `labels:` |
| bar modules and their order | `tiling/waybar/config.jsonc` |
| bar look (sizes, fonts) | `tiling/waybar/style.css`; colours come from `colors.css` |
| launcher, lock, power menu | `tiling/fuzzel.ini`, `tiling/swaylock.conf`, `tiling/power.sh` |
| wallpaper | `tiling/wallpaper.png` |
| the palette everywhere at once | `look.palette` in the yaml, then `chora-tiling --apply-theme` writes `colors.css`, the fuzzel and swaylock colours |

A theme is a folder of these files: copy, share, drop in (P8). The yaml never runs code (shared:
security); `style.css` is waybar's own and stays as hackable as any waybar rice.

## What the renderer can do today (06044cf) and what needs new work

- **Today:** rect `radius`, `border` (solid), `backdrop` blur 2-128 with saturation, `clip` with a
  radius on any node with a size, image nodes with 9-slice (`slice`), shader `effects` on a node,
  `animate` of transform, opacity, size, colour with linear or cubic ease in/out/in-out.
- **Tiling today:** one `gap` (4) around each window, `style.opacity` and `effects`, wobbly.
  No border, no corners, no shadow, no animation; positions jump. Waybar has no `style.css`, so it
  draws GTK defaults (the grey clock box and blue "Connected" in the tour stills).
- **Buildable in the plugin:** per window a group with the shadow image, the border rect
  (gradient by a small fragment shader in `effects`) and the surface clipped at radius 10. The
  shadow PNG comes from the code chora-assistant already uses (`glass.rs`), moved to chora-kit.
  Roadmap gate 2 (four windows, blur, rounded corners, 300 ms transition, 99 % on time) is the
  performance bar; a clipped surface renders as a layer.
- **New core work:** (a) keep the last frame of a closed surface for the close animation; (b) a
  backdrop blur behind a layer surface (bar); (c) a grab of "MOD plus any key" for the swallow, or
  a grab list built from all letters; (d) `Print` without a modifier needs the grant
  `input.grab-any` for tiling. Springs (roadmap) are not needed.

## Build list (ranked; each step small and shippable alone)

1. **Bar style.** `waybar/style.css` + `colors.css` in Ink: islands, sizes, fonts as above; tags.jq
   emits classes, not markup. *Check:* a 1280x720 capture matches mock A within 1 px on height,
   margins and radius; contrast script passes 4.5:1 on every label.
2. **Bar doors.** Apps button, tags 1 to 9 as buttons (`choractl call tiling view_tag`), layout
   word (`set_layout`), Keys, power. *Check:* the mouse-only persona starts foot, views tag 2 and
   locks without a key.
3. **Swallow unbound Super.** *Check:* Super+q, Super+z and Super+F5 in foot type nothing
   (t-power/019 again); bound keys still work.
4. **Gaps and corners.** `look.gap_outer` 12, `look.gap_inner` 10, `look.radius` 10; none in
   fullscreen and `max`. *Check:* pixel test on a headless capture: 12 / 10 px, corner at radius 10.
5. **Focus mark.** Border rects, focused / unfocused / urgent colours (Q2 picks gradient or solid
   and dim). *Check:* after Super+j only the new window carries the accent; a capture over a white
   and a black window shows the border unbroken at 1x and 2x.
6. **Key list.** `Super+/` and the Keys button; words per action, `labels:` for `run`. *Check:* a
   binding added to the yaml shows after reload; Esc gives the keys back to the window of before.
7. **Layout chip.** Super+space shows "Fair" for 700 ms. *Check:* finding "Super+space looks dead"
   cannot be reproduced.
8. **Shadows.** 9-slice image per window, focus cross-fade. *Check:* idle scene renders 0 frames
   in 10 s (core counter); shadow never covers the bar.
9. **Motion.** Open, move, tag switch, focus as in the table; `animations: false`. *Check:* a
   30 fps clip stepped frame by frame: no jump, no flash frame, durations within one frame.
10. **Screenshot, lock, power.** grim + slurp script, `Print` grant, power.sh, swaylock and fuzzel
    themed, chora-notify in the flavor for the notice. *Check:* Print writes a file and the notice
    names it; the power menu's four entries each work in the VM.
11. **Palette apply and Paper.** `--apply-theme`, the light palette, chat panel of chora-assistant
    in the same tokens (today a white panel on dark). *Check:* one change of `accent` in the yaml
    recolours border, tags, launcher, lock and chat after one command.
12. **Core work (optional, last).** Close animation, blur behind the bar. *Check:* closing a
    window fades in 140 ms with no frame of a black box; idle stays at 0 frames.

## Taste questions for the user

1. **Bar shape.** A: three floating islands (left, centre, right) on the wallpaper (mock A).
   B: one slim bar flush with the top edge, full width, height 28 (mock B). C: one floating bar,
   full width with 12 px margins and radius 10 (between A and B).
2. **Focus mark.** A: 2 px gradient border accent to accent-2 and a deeper shadow; unfocused stay
   at full brightness (mock A). B: 2 px solid accent border, unfocused windows dimmed to 88 %
   (mock B). C: solid border, no dimming (the calmest).
3. **Default theme.** A: Ink, dark (mock A, B). B: Paper, light (mock C, with the key list open).
   C: follow the system colour scheme (`org.freedesktop.appearance`), Ink when unset.

Recommendation: 1A, 2A, 3C. The gradient bends P6 ("one accent colour") a little: accent-2 only
ever appears inside the focus border, so the eye still reads one signal.
