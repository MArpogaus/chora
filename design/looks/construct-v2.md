# Construct look v2 (proposal)

Design lane, 2026-11-11, for the PM (taste decisions are marked **D** with the option left out).
Builds on the HUD rework (`design/construct-hud-spec.md`: glass, no buttons, F1/`Super+/` overlay)
and keeps it. Base: `construct.md`, `shared.md`, `combinations.md`, `review-checklist.md`,
`docs/design-principles.md` (P), `docs/construct-style.md` (rule numbers), `docs/cube-modes.md`.
Inputs: tour stills `media/2026-11-12/tour/stills/04-*, 05-*, 06-*` (old HUD), the built HUD
`media/2026-11-12/construct-hud-built/*` (new HUD; `tour-v2/` does not exist yet), swarm 2 to 4.
Mocks (1280x800, 1x, labelled "mock"): `media/2026-11-12/design/construct/mock-first-floor.png`,
`mock-cube-drag.png`, `mock-cube-failed.png`; sources `design/mockups/construct-v2/`.

## 1. Philosophy: a quiet gallery

The construct is a gallery at night or at noon: marble, sky, and the person's windows hung in the
air. Content is the only colour; the frame is one material (blurred glass), one accent (#2680B4)
and one family (Inter, DejaVu Sans Mono for data). Space is memory: a thing stays where it was
put, appears where the person looks, and the HUD speaks only where attention already is (ahead,
under the pane, at the bottom edge). Nothing moves unless the person moved it; nothing is drawn on
the bare space except glass. If a frame shows nothing useful, it is wrong: no black, no grey slab.

## 2. First time: empty floor to first window

Today the empty floor shows nothing to click and a hint bar without a way to start a program
(swarm 2 #19, swarm 3 #2, swarm 4); see `mock-first-floor.png` for the fix.

**The start card.** While no pane exists, one glass card (strong tint, w 448, radius 24) stands
where the first pane will appear: ahead, centred on the horizon, drawn flat (rule 10), with the
floor shadow of a pane. Head `Nothing is open yet` (`text`), `Start a program. It opens here, in
front of you.` (`small`, muted). Three rows (h 48, r 16): `Terminal  Super T`,
`All programs  Super Space`, `A cube of desktops  Super C`; footer `Click a row or press its
keys.` and `F1 All keys`. No hint bar while it shows (one place per key).
- **D1** the card lives in the space at the first pane's place, and the first pane materialises
  in exactly its rect (160 ms). *Left out:* a screen-fixed dialog (feels modal), a right-click
  root menu (invisible, P3), a dock (permanent chrome against "minimal").
- **Keyboard:** `Super T` starts the terminal; the card fades (120 ms) as the pane appears in its
  place; the camera is already at the 1:1 distance, the construct flies in (use mode) and the
  hint switches at once to `Alt Esc Step back` for 3 s. Typed keys reach the terminal, as the
  person expects after starting it, and the hint says how to get out (swarm 2 #5, swarm 3 #3).
- **D2** a program the person starts opens flown in. *Left out:* open it in walk mode with
  walk keys still live; that is the "keys typed into a terminal" surprise turned around (the
  person types and nothing gets the text).
- **Mouse:** hover fills the row (0 ms in), a click starts it, same result as the keys.
- **Voice:** "open a terminal" does the same; the card does not take the keyboard (P11).
- **Second program:** `Super T` again from use mode steps back, places the new pane right of the
  first at the same distance and flies into it (glide 480 ms). A pane whose first buffer has not
  come shows a glass placeholder with its name and the working ring, never a grey slab
  (`swarm-2/c-novice/shots/020.png`); a label is always the window title ("Terminal" from the
  desktop entry until the title arrives), never the app id (`foot`).

## 3. Pixel polish list (seen in the stills)

| # | Still | Off | Fix |
|---|---|---|---|
| 1 | `tour/stills/04-01-start.png` | OPEN CUBE chip x 1140-1263, y 680-711: 16 from the right, 9 from the bottom (rule 8: 32); 16 px caps without tracking beside the 11 px tracked SUNSET label: two caps styles | the chip is gone in v2 (`Super C` lives in the start card and F1); no screen-fixed chip in any state |
| 2 | `04-07-fly-in.png` | flown-in pane 760 x 480 on a 1280 output (not 1:1); frame larger than buffer: a #1A1B26 strip 4 px right (x 1016-1019) and 16 px at the bottom (y 584-599) | the frame takes the buffer size; the fly-in ends at the 1:1 distance; no strip in any frame of the clip |
| 3 | `04-15-notice.png` | SUNSET label centred at x 592.5, the pane's bottom edge at about 590: 2 to 3 px off for a turned pane | centre on the midpoint of the projected bottom edge, round the left once |
| 4 | `04-13-help.png`; `construct-hud-built/construct-help-dark.png` | help is see-through: the sunset and the SUNSET pill read through it (old); in the built one the sunset glow still blooms behind "Fly into the pane ahead" (x 420-820, y 150-330) | command surfaces (help, window list) tint 0.92 over a scrim 0.48 light / 0.56 dark; pane labels hide while a command surface shows |
| 5 | `05-01-start.png` | tiling pane a flat #404040 slab (x 842-1090) with no label; XP label 50 x 31 with letter-spaced caps, tight | an empty manager pane shows its wallpaper and its label; labels in `small` 14, display names |
| 6 | `05-10-both.png` | two terminals at about 0.3 scale, text about 4 px | a pane given to the construct lands ahead at the 1:1 distance |
| 7 | `06-01-use-mode.png` | STEP BACK chip (x 1077-1263, y 650-681) covers the Recycle Bin label and the chora window corner; 16 from the edge | no chip in use mode: the v2 hint `Alt Esc Step back` bottom centre, 3 s after the fly-in and on bottom-edge dwell |
| 8 | `06-02-stepped-back.png` | three chips right-aligned 16 from the edge; the cube seen square-on reads as a flat picture | hint bar bottom centre; rest pose of section 4 |
| 9 | `06-04-turn-key-mid.png`, `06-06b-drag-turn-mid.png` | "mid" frames show a face at rest: the turn is drawn only at its end (or after the release) | live turn, section 4 |
| 10 | `06-09-hover-edge.png` | pointer on the face, edge stays a 1 px line | lit edge 2 px accent + glow 16, section 4 |
| 11 | `06-14-stood-off.png`, `06-16-pane.png` | cube has no floor shadow and no mirror image while every pane has both | the cube gets the pane shadow and mirror passes |
| 12 | `construct-hud-built/construct-walk-light.png` | pane label (x 600-680, y 434-466) carries the panel shadow 0 24 64: a grey smudge 3x its height on the white floor | capsules up to 40 high take a short shadow: 0 1 2 /0.06, 0 4 12 /0.08 |
| 13 | same | walk hints omit `Super T` (swarm 3 #2) | hints show only what applies, at most 5: `W A S D Walk`, `Return Fly in` (a pane ahead), `Tab Windows` (2+ windows), `Super T Terminal`, `F1 All keys` |
| 14 | `construct-hud-built/construct-cube-dark.png` | face label `xp` lower case, help says "Give it to xp" | display names from the word table: XP, Tiling, Media center, Construct |
| 15 | `construct-hud-built/construct-help-dark.png` | `Show these keys F1` under EVERYWHERE · EVERY LOOK; the every-look key is `Super /` | `Super /` in the shared group, `F1` in Walk |
| 16 | `swarm-2/c-cube2/shots/034.png` | a `foot` pill floats over an xp face in use mode, after its window closed | a label fades with its pane (120 ms); no pane label in use mode (spec 6) |
| 17 | `swarm-2/c-novice/shots/020.png` | toast `tiling did not take the window.`: lower case, no reason, no next step | `Tiling did not take the window: <reason>.` from the word table (P4) |

## 4. The cube, readable

- **It looks like a cube.** **D3** At rest, stood back, the cube turns 14 degrees off square
  toward the next face, so its depth and the next face show (`mock-cube-failed.png`); the camera
  stays at the cube's mid height. *Left out:* square-on (today, reads as a flat picture, finding
  "faces look alike"), a view from above (needs caps; shows the hollow inside). Step in glides
  the 14 degrees away (480 ms) so use mode is exactly 1:1.
- **Faces differ.** **D4** The default cube is four looks: XP, Tiling, Media center, and XP with a
  second wallpaper, instead of four identical XP faces. *Left out:* a tint or a number painted on
  the face (text on a pane, rule 11). The face label under the cube (spec 5) names the face ahead:
  `Media center  face 3 of 4  ● ● ● ●`; the dot of the face ahead is accent.
- **Hover.** Pointer over a face: its edge lights, 2 px accent plus a 16 px glow at 0.35, 0 ms in,
  120 ms out; the matching hint (`Return Step in`) shows the hover fill. Not during a turn.
- **Live turn while dragging.** **D5** The cube follows the hand: yaw = drag dx / view width x 90
  degrees, no threshold, no lag (`mock-cube-drag.png`). On release it settles to the nearest face
  (past 45 degrees: the next) over 320 ms ease out; a flick faster than 1.5 widths/s goes on to the
  next face. The face label changes at 45 degrees; the next dot fills with the turn in between;
  `Step in` dims to 0.45. *Left out:* the stepped turn of cube-modes (a third of the width per
  face, applied after the fact), which testers read as "turns only after release".
- **Turn keys** (`Ctrl Alt ←`, `Ctrl Alt →`) turn over 360 ms ease out; a second press during a
  turn queues one.
- **Starting and failed faces.** A face whose manager is starting shows, within 300 ms, its
  plain face fill with a flat glass card: working ring, `Starting Media center`. If it is not up
  within 5 s, or exits, the card says what failed and the next step (`mock-cube-failed.png`):
  `Media center did not start.`, the reason in a `data` well (`chora-mc stopped after 2 s: it
  needs foot 1.26 or later.`), `The other faces still work.`; the face label says `did not
  start`, its dot becomes a hollow ring; the middle hint reads `Return Try again`, and a click on
  the face tries again. Never a black frame (swarm 2 #4: 10 s black, no message).
  **D6** the error lives on the face only, not also as a notice. *Left out:* a notice under the
  island (would say the same thing twice, U3); voice says it once if voice is on.

## 5. Palette, glass, type, motion

**Palette.** Light: sky #E4E8ED to #ECEEF1, horizon glow white, floor #EEEFF0 with veins at
0.08; dark: void #000, horizon glow white 0.10, floor #070707 with veins at 0.06. Text #14181E /
#ECEEF2, muted #4A5260 / #A3AAB5. Accent #2680B4 only for: ring, focus ring, selection,
caret, focused pane edge, lit face edge, the face-ahead dot, a waiting question. **D7** no
second hue for errors (no red): the word carries the failure, the hollow dot marks it.
*Left out:* a red dot or edge (breaks "one accent", reads as alarm, colour-only state).

**Glass** as spec 2, plus: command surfaces tint 0.92 (polish 4); capsules up to 40 high take the
short shadow (polish 12); every surface rect is rounded to whole logical px once, after layout.

**Type.** Spec 3 sizes unchanged: label 11 (state words, group heads), small 14 (keys, pane and
face labels), data 14 mono, text 16, display 24 (search lines only). Pane and face labels in
`small` 500, sentence case, never caps (the old caps labels go).

**Motion** (all ease out (0.2, 0.8, 0.2, 1), nothing bounces or overshoots):

| Change | Duration |
|---|---|
| surface appears / disappears | 160 ms (blur 200 ms) / 120 ms |
| start card to first pane (cross-fade in one rect) | 160 ms |
| fly in, fly out, step in, step back | 480 ms |
| walk: tap a key | glide one pane step, 320 ms |
| walk: hold a key | accelerate over 160 ms to one pane step per 400 ms; stop over 160 ms |
| cube turn by key or chip | 360 ms; release of a drag settles in 320 ms |
| lit face edge, row hover | 0 ms in, 120 ms out |
| tint adapt | 240 ms |
| hints after a fly-in | 3 s, then fade 300 ms |

**D8** walking moves in pane steps: a tap glides to the next pane, holding walks at one step
per 400 ms (swarm: "slow walk"). *Left out:* a faster constant speed (overshoots, makes the
floor swim at 1280 px).

## 6. Build list (ranked, small steps; each with its check)

1. **Start card** on the empty floor (section 2). Check: fresh session, no instructions, mouse
   only: one click starts a terminal; keyboard only: `Super T`; capture shows the card centred
   to the px and no hint bar.
2. **New program opens flown in**, hint `Alt Esc Step back` 3 s. Check: `Super T` then `d`:
   the terminal shows `d`, the hint is up in the next frame; `Alt Esc` steps back.
3. **Hints only what applies, `Super T` included** (polish 13). Check: walk captures with 0, 1
   and 3 panes list the right hints, at most 5, bar centred, 32 from the bottom.
4. **No grey slab, no app-id label** (section 2, polish 5, 16, 17). Check: second `Super T`
   capture shows a placeholder or content in every frame; label reads `Terminal`; closing it
   leaves no pill within 120 ms.
5. **Face starting and failed states** (section 4). Check: break `chora-mc`: within 300 ms the
   face shows `Starting`, by 5 s `did not start` with reason; no black frame in the clip;
   `Return` retries.
6. **Live drag turn** (D5). Check: a 60 fps clip of a slow drag shows the yaw change in every
   frame; release at 30 degrees returns, at 50 goes on, both within 320 ms.
7. **Rest pose 14 degrees, cube shadow and mirror** (D3, polish 11). Check: stood-back capture
   shows the next face's depth, a shadow and a mirror; step in ends at 1:1 (a 1 px line of
   the face is 1 px on screen).
8. **Lit edge 2 px + hover hint** (polish 10). Check: pointer on a face, edge 2 px accent;
   off, gone within 120 ms; never during a turn.
9. **Default cube of four looks, display names** (D4, polish 14). Check: stood back, turn
   through all faces: four distinct labels, words from the table in en and de.
10. **Fly-in frame equals buffer, 1:1** (polish 2). Check: last frame of a fly-in has no strip
    at any edge; pane width equals the window's width in px.
11. **Label centring and short capsule shadow** (polish 3, 12). Check: label centre within
    0.5 px of the projected bottom-edge midpoint for a turned pane; no smudge under the label.
12. **Command surfaces 0.92 + scrim** (polish 4). Check: help over the sunset pane: no image
    reads through, contrast of muted text 4.5:1 or more measured on the capture.
13. **Help groups** (polish 15). Check: `Super /` in EVERYWHERE, `F1` in Walk; no row twice.
14. **Walk in pane steps** (D8). Check: tap `w` glides one step in 320 ms; hold reaches the
    third pane in 1.2 s; no overshoot.
15. **Reshoot `tour-v2`** at 1280x720 and 1920x1080, light and dark, 1x and 2x, and run
    `review-checklist.md` plus this list on it.

## 7. Open for the PM

- D2 makes `Super T` fly in; a power user may prefer to stay in walk mode. A setting
  (`open_flown: false`) or not at all?
- D4 needs the cube to host three managers at once; if that costs too much, fall back to four XP
  faces with four wallpapers and keep the labels.
- Pass to the xp designer: desktop icon labels cut at the face's left edge
  (`06-01-use-mode.png`, "My Documen" at x 0-66).
