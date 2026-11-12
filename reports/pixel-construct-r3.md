# Pixel review: construct v2, round 3

Independent reviewer, 2026-11-12. Inputs: `media/2026-11-12/construct-r3/r3-*.png` (6 states, each at 1x 1280x800 and 2x 2560x1600,
branch `claude/construct-v2` @ 4ba0950b), r2 report `reports/pixel-construct-r2.md`, r2 captures, brief (`design/product-briefs.md`,
"chora-construct"). All 12 PNGs viewed; crops, diffs and pixel measures with PIL.

## Worker claims, checked

| Claim | Result |
|---|---|
| Neighbour panes get labels (60 %) | NOT VISIBLE. `r3-walk-3-panes-1x/2x`: the strips under the left pane (x 0-329, y 572-612) and right pane (x 951-1280) hold only floor (min/max grey 213-241, no pill). Only `Terminal` under the centre pane (x 595-685, y 574-606) is drawn. |
| With-pane capture shows a non-overlapping pane | NO. `r3-cube-stood-back-with-pane` vs `r3-cube-stood-back` differ only at x 353-1075, y 621-788 (floor and hint-bar area, 3259 px over threshold at 1x); no pane exists anywhere in the frame. The capture does not prove a pane, its label, or hiding behaviour. |
| Rest yaw 22 deg | CONFIRMED. Side face now x 792-863 (72 px, was 15). |
| Mirror ends 60 px under the pane, 3 px feather | NOT CONFIRMED, see N4. |
| Step-in dim 0.6, copy "The other faces still work." | NOT CAPTURED (no failed-face or dim frame). |

## r2 findings

| # | Status | Evidence |
|---|---|---|
| N1 neighbour labels | OPEN | see above; the brief says every pane has a label. Raised to MAJOR (R1). |
| N2 15 px sliver | FIXED | `r3-cube-stood-back`: cube x 416-863 (centre 639.5), y 255-544; side face x 792-863 shows hills, a cloud and the taskbar and reads as a face. |
| N3 side panes cut by viewport | OPEN | `r3-walk-3-panes`: left pane x 0-329 clipped at x 0, its text off screen; right pane clipped at x 1280 (top y 229). Unchanged from r2. |
| N4 floor mirror hard slab | OPEN | Side mirrors still end in a vertical cut at x 328 and x 952 (grey 200 to 237 in 2 px, y 640-720). New: a hard-edged grey bar x 375-905, y 702-727 (grey 197 on 237 floor, no feather on left, right, top), 137 px under the pane, half hidden behind the hint capsule. |
| N5 cube mirror geometry | FIXED | front and side mirrors both start at y ~575 and read as one reflection (1x). |
| N6 capsule rim over program | OPEN | `r3-stepped-into-face-2x` crop of capsule: a straight light line across the top (x 262-862 in crop) that ignores the pill curve; at 1x the lime double outline remains. Capsule bottom y 768, XP taskbar y 770 (2 px). |
| N7 `XP 1  · face` spacing | OPEN (cosmetic) | 2x: gap before the dot ~17 px, after ~12 px. |
| N8 key cap bottom line square | OPEN (cosmetic) | 2x capsule: lighter bottom line under `Alt` and `Esc` is square-ended, inside 6 px corners. |
| N10 low composition | unchanged, not counted | |
| r2 m1-m5, m7 (failed face, disabled hint, start card, help see-through, placeholder, pull frame) | OPEN, not captured again | only F1 overlay is new: opaque, legible, search field, "Only the keys that work now are listed." (F1 legible: RC met). |

Regressed: none.

## New findings

### MAJOR
- **R1. Neighbour panes carry no label** (`r3-walk-3-panes-*`, left x 0-329, right x 951-1280, label strip y 572-612). Fix: confirm the label code runs in the capture build; draw pill (60 %) under each pane at least 40 % on screen.
- **R2. `r3-stepped-into-face-2x.png` is pure black** (0,0,0 everywhere except the Step back capsule at x 548-732, y 728-768). The 1x capture shows the XP face. At 2x the stepped-in face does not render; either a 2x texture/DPR bug in the face capture (main path: step in) or a failed face without its error text (RC d). Fix: find out which; re-capture. `r3-first-program-flown-in-2x` is fine.

### MINOR
- **R3. With-pane capture is empty** (P2 still open); no evidence for the neighbour-pane claim. Capture a cube state with a pane clear of the cube.
- **R4. Walk floor: hard grey bar under the centre pane** (see N4), reads as a rectangle half under the hint capsule at both scales.

### COSMETIC
- **R5.** Right-pane terminal text at 1x is a slanted 4 px glyph row, unreadable (x 955-1100, y 250-270). Not wrong for a distant pane.

## Checked and clean (1x and 2x)
Cube centred on the hint axis; label 9 px under the cube; hint capsule centred (x 367-913); key caps crisp at 2x; F1 overlay opaque with clean type; first program text at x 12, y 13; walk centre pane with blue edge and label.

## Verdict
**FIX.** Two MAJOR (R1 missing neighbour labels, R2 black 2x step-in), so the quality bar (nothing above COSMETIC) is not met.

Counts: r2 N1-N8,N10: 2 FIXED, 6 OPEN, 0 REGRESSED (N10 not counted); m1-m5, m7 still not re-captured. New: 2 MAJOR, 2 MINOR, 1 COSMETIC.
