# Pixel review: construct v2, round 5

Independent reviewer, 2026-11-12. Inputs: `media/2026-11-12/construct-r5/` (23 PNGs, 1x 1280x800 and 2x 2560x1600, `claude/construct-v2` @ d3843e0),
r4 report and captures, brief. All 23 PNGs viewed; PIL measures in scratchpad conrev5/. Captures arrived on `origin/claude/session-log` d86698a5.
Not capturable or 1x only (per worker): m5 placeholder (not seen), m7 pull frame (1x only).

## Worker claims, checked
| Claim | Result |
|---|---|
| Beside panes whole on screen, clear of others | CONFIRMED. Walk 1x: left pane x 7-312, right x 967-1272 (7-8 px gutters), centre x 375-905, y 234-565; no pane touches an edge or another. With-pane: pane x 968-1272, y 311-512, fully in. |
| Neighbour labels with cube stood back | CONFIRMED. `Terminal` pill under the side pane at x 1076-1166, y 492-524 (1x, 2x crisp). Left pane label in walk too. |
| Mirror reaches 240 px, feathered 60 px | CONFIRMED in effect. Hard grey bar is gone: under the centre pane only the soft contact shadow (y 620-645) and the hint capsule's own shadow (grey 237 to 210 over y 700-727, no square ends). Side panes show blurred smudges (y 560-610, see C3). |
| Step back capsule hairline only, lifted 24 px | PARTLY. Lift CONFIRMED (1x pill x 548-732, y 703-744, taskbar y 770: 26 px). Not a hairline: it is a solid dark pill with key caps (legible, one outline, no lime, no straight rim). Fine visually. At 2x the capsule is absent (0 dark px in y 1360-1560), see N1. |

## r4 findings
| # | Status | Evidence |
|---|---|---|
| M1 side panes clipped | FIXED | measures above; with-pane pane now labelled and whole, 1x and 2x. |
| M2 grey bar mirror | FIXED | no square-ended bar; only shadows with soft falloff. |
| M3 capsule rim | FIXED at 1x | single pill, curved, no lime double line, 26 px above taskbar. 2x unverifiable (N1). |
| C1 tray text differs 1x/2x | OPEN (cosmetic) | stepped 1x tray "Sound unavailable" x 1090-1190, 2x has speaker icon and clock only; clocks 8:56 vs 8:57, so still two runs. |
| C2 duplicate captures | OPEN (cosmetic) | r5 has two md5-identical pairs again (help = f1-overlay style pairs); not a distinct help state. |
| C3 XP 1 spacing / key cap line / right-pane tiny text | OPEN (cosmetic) | 2x label `XP 1  · face 1 of 4` gap before the dot still wider than after; tiny terminal text in side panes unchanged. |
| m2 disabled hint | FIXED | `r5-cube-turning-1x`: `Return Step in` greyed while turning, other keys full. |
| m1 failed face | FIXED at 1x, see N2/N3 | honest card "did not start", reason, "The other faces still work.", `Try again` hint. |
| m3/m4 start card, help | still FIXED | start card x 416-864 clean glass at 2x; F1 overlay legible, opaque panel. |
| m5, m7 | OPEN, not evidenced | m5 not captured; m7 only a blank blue slab (N4). |

## New findings
### MAJOR
None.

### MINOR
- **N1. Step back capsule missing at 2x** (`r5-stepped-into-face-2x`, expected x ~1096-1464, y ~1406-1488). 0 dark pixels; 1x has it. Worker says it hides before the face paints; a 2x user sees no way out hint. Fix: show the capsule on a fixed timer after step-in (not on first paint) and re-capture 2x after 1 s.
- **N2. Failed face stood back at 2x has no scene** (`r5-failed-face-stood-back-2x`): flat bg (233,237,242), no horizon, floor or reflection; 1x has them. Fix: find why the floor does not render in this state at dpr 2 (capture race or lost scene), re-capture.
- **N3. Failed-face card not fitted to the face** (`r5-failed-face-stood-back-1x`: card x 404-804, y 306-494 vs front face x 416-792). Card is flat, spills 12 px past the left face edge and across the side face; face is pale 247 on bg 233, so the cube reads as a ghost with the card floating. Fix: size the card inside the face with a 16 px margin and apply the face transform; give the failed face a visible edge or tint. Reflection slab under it (x 416-864, y 575-695) reads as a grey plate.

### COSMETIC
- **C4/N4. Pull frame** (`r5-pull-1x`, 1x only): a blank blue slab x 11-293, y 268-530 and a yellow 12 px square at x 908-920, y 428-440 (looks like a debug marker). If the square is not intended, remove. Needs 2x.
- **C5.** Mid-turn frame (`r5-cube-turning-1x`) has sawtooth edges on face top and bottom (x 449-831, y 275-285, 510-525), transient only.
- **C6.** Side pane reflections are blurred grey smudges (y 560-610 walk 1x, detached from the pane, 40+ px below its label); softer than a reflection, fine but not clearly one. Fix: shorten gap or drop.

## Checked and clean (1x and 2x)
Walk layout and labels; hint capsules centred (walk x 273-1007); cube centred, label 9 px under; cube reflection fades; stepped-in XP face crisp, taskbar and icons; start card; F1 overlay; terminal flown in; 2x type sharp.

## Verdict
**MERGE-OK** (no MAJOR; PASS needs N1-N3 resolved).
Counts: r4 M1, M2, M3 FIXED (M3 1x only); C1-C3 OPEN cosmetic; REGRESSED 0. New: 0 MAJOR, 3 MINOR, 3 COSMETIC (C4-C6).
