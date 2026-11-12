# Pixel review: construct v2, round 4

Independent reviewer, 2026-11-12. Inputs: `media/2026-11-12/construct-r4/` (8 states, 1x 1280x800 and 2x 2560x1600, branch
`claude/construct-v2` @ 15fa10e), r3 report and captures, brief (`design/product-briefs.md`, chora-construct). All 16 PNGs viewed;
crops and pixel measures with PIL (scratch: conrev4/). m1/m2/m5/m7 not re-captured.

## Worker claims, checked

| Claim | Result |
|---|---|
| Neighbour labels visible in the walk | CONFIRMED. `r4-walk-3-panes`: pills `Terminal` under left (x 8-98, y 580-612), centre (x 595-685) and right (x 1182-1272), 1x and 2x, crisp. Left/right are dimmer than the focused centre (intended hierarchy). |
| Stepped-into-face 2x no longer black | CONFIRMED. `r4-stepped-into-face-2x`: full XP face, icons, taskbar, clock all drawn, crisp. Note: the 2x tray has no "Sound unavailable" text while 1x does (x 1090-1190, y 785): two different capture moments, see C1. |
| Mirror edges feathered on panes | NOT CONFIRMED. See M2: the "mirror" under the pane is still a hard-ended grey bar. |
| Side panes clip at viewport by design | Judged below: not acceptable for the brief. |
| (new) with-pane capture shows a clear pane | PARTLY. A pane now exists (x 1040-1280, y 235-564) but is itself clipped at x 1280 (240 of ~530 px), and has no label (strip x 1030-1280, y 568-615 is plain floor, grey 232-240). |

## Side panes clipped at the viewport: acceptable?

No. The brief asks for calm, glass, minimal, and the quality bar is "nothing above COSMETIC". The walk shows left pane x 0-329
and right pane x 951-1280 cut hard by the screen edge (rows 229-570 touch both columns 0 and 1279): the left pane is a blank dark
slab (its text is off screen), the right shows only a slanted 4 px text row. They read as unexplained black wedges, the largest dark
mass in an otherwise pale scene, not as windows. In the with-pane state the pane is slammed against the right edge with 55 % of it
cut and no label, which reads as a layout bug rather than a "beside" placement. A fallback being deliberate does not make it
calm: a pane that is only partly visible should either be fully on screen (scaled/pushed in) or fade out with the edge. MINOR (M1),
repeats r2 N3 / r3 N3.

## r3 findings

| # | Status | Evidence |
|---|---|---|
| R1 neighbour labels (MAJOR) | FIXED (walk) | pills under all three panes at 1x and 2x. Still absent on the with-pane capture (M1). |
| R2 black 2x step-in (MAJOR) | FIXED | 2x frame is full colour; extrema 0-255 per channel, taskbar y 1203-1250 (displayed). |
| R3 with-pane capture empty | FIXED (capture) | pane present; but see M1 for what it shows. |
| R4 hard grey bar under centre pane | OPEN | `r4-walk-3-panes-1x`: bar x 388-897, y 709-726, grey 196 on 237 floor, flat vertical ends; blurred top/bottom but ends are square. Same bar in with-pane at x 1054-1280, y 710-725. Reads as a stray stripe, not a reflection (M2). |
| R5 right-pane tiny text | OPEN (cosmetic, unchanged) | x 955-1100, y 250-270 |

## r2 findings

| # | Status | Evidence |
|---|---|---|
| N1 labels | FIXED (walk) | see above |
| N2 sliver | FIXED | cube x 416-863, side face 72 px, carries hills and taskbar |
| N3 side panes cut | OPEN | see judgement; M1 |
| N4 floor mirror hard slab | PARTLY | side mirrors now feather: at y 680 grey 222 to 238 over ~18 px both sides (FIXED); centre bar remains (M2, OPEN) |
| N5 cube mirror geometry | FIXED | `r4-cube-stood-back-2x` crop: front and side reflections join, fade out, no hard cut |
| N6 capsule rim over program | OPEN | `r4-stepped-into-face-2x`: straight light line across the top of the Step back capsule (crop x 245-1140, y 58) ignores the pill curve; lime double outline at 1x x 549-732, y 728-768; capsule bottom 2 px above the taskbar (y 770) (M3) |
| N7 `XP 1  · face` spacing | OPEN (cosmetic) | 2x label: gap before the dot ~17 px, after ~12 px |
| N8 key cap bottom line | OPEN (cosmetic) | 2x: square-ended lighter line under `Alt` and `Esc` inside 6 px corners |
| m1 failed face | OPEN, not captured | |
| m2 disabled hint | OPEN, not captured | |
| m3 start card | FIXED | `r4-start-card`: card x 416-864, y 252-548, title, one-line help, three rows with key caps, "Click a row or press its keys.", F1 All keys; clean at 1x and 2x, glass, calm |
| m4 help see-through | FIXED (same as F1) | see C2; overlay opaque, legible |
| m5 placeholder | OPEN, not captured | |
| m7 pull frame | OPEN, not captured | |

## New findings

### MAJOR
None.

### MINOR
- **M1. Side panes hard-clipped at the viewport** (`r4-walk-3-panes-*` left x 0-329, right x 951-1280; `r4-cube-stood-back-with-pane-*` pane x 1040-1280, no label under it, y 568-615). Fix: in the "beside" placement clamp the pane so at least its full width fits (scale or move in), else fade the clipped part over 60 px; give the with-pane pane its label.
- **M2. Grey bar "mirror" under panes** (walk x 388-897, y 709-726; with-pane x 1054-1280). Hard square ends, 18 px tall, sits half under the hint capsule. Fix: draw a real reflection (same texture, 12 % alpha, fade with distance) or drop it; if kept, feather the ends like the side mirrors.
- **M3. Step back capsule rim** (`r4-stepped-into-face-*`, x 549-732, y 728-768 at 1x). Top highlight is a straight line not following the pill radius; lime outline reads doubled over the program; sits 2 px above the XP taskbar. Fix: clip the highlight to the pill (border-radius on the pseudo-element), single 1 px outline, lift capsule to bottom: 24 px above the taskbar (y <= 744).

### COSMETIC
- **C1.** Step-in 1x shows "Sound unavailable" in the tray, 2x does not: the two captures are of different XP states. Re-capture both from one run so 1x/2x pair compares.
- **C2.** `r4-help-*` is byte-identical to `r4-f1-overlay-*` (same md5); `r4-first-program-flown-in-*` is byte-identical to r3. Not new evidence; fine, but not a distinct help state.
- **C3.** N7 spacing, N8 key cap line, R5 tiny text (above).

## Checked and clean (1x and 2x)
Neighbour labels (crisp, centred in pill, 12 px gap under pane edge); stepped-in face (icons, labels, taskbar, no black); start card;
F1 overlay; hint capsule centred (walk x 273-1007, stood-back x 367-913); cube centred, label 9 px under cube; cube reflection fades cleanly;
2x type sharp everywhere; no 2x-only artefacts.

## Verdict
**FIX.** Both r3 MAJOR are fixed and nothing regressed, but three MINOR remain (clipped side panes, grey bar mirror, capsule rim), above the
COSMETIC bar. One more small round (all three are CSS/geometry) should reach MERGE-OK.

Counts: r3 R1-R5: 3 FIXED, 2 OPEN (R4, R5), 0 REGRESSED. r2 open: N1, N2, N5, m3, m4 FIXED; N3, N4 (part), N6, N7, N8, m1, m2, m5, m7 OPEN. New: 0 MAJOR, 3 MINOR, 3 COSMETIC.
