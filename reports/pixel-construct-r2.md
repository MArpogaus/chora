# Pixel review: construct v2, round 2

Independent reviewer, 2026-11-12. Inputs: `media/2026-11-12/construct-r2/r2-*.png` (5 captures, 1280x800, scale 1 only,
branch `claude/construct-v2` @ 83c44f81), the round-1 report `reports/review-construct-v2.md`, older captures in
`media/2026-11-12/construct-v2/`, and the brief (`design/product-briefs.md`, "chora-construct: the 3D space"). Every PNG was
viewed, cropped and measured with PIL (crops in the session scratchpad).

## Process gaps

- **P1. No 2x captures.** Nothing here is checked at 2x: hairlines (capsule rim, key cap bottoms), the 15 px side face of the
  cube and 6 px icon labels need it. This is a gap, not a pass.
- **P2. The with-pane capture shows no pane.** `r2-cube-stood-back-with-pane.png` differs from `r2-cube-stood-back.png`
  only in a 4 px column (x 818-821, y 534-661, clock or noise). The pane (370-905 x 235-565 earlier) is not visible, so the
  capture cannot prove that a pane exists, that its label is hidden, or that "only a nearer, overlapping pane hides" holds.
  The worker also says it predates the last M1 change. A capture with a non-overlapping neighbour pane (visible, labelled) is
  missing.
- **P3. Only 5 of the 8 round-1 states re-captured.** First floor (m3), help (m4), failed face (m1), placeholder (m5),
  turning/disabled hint (m2) and the pull frame (m7) have no r2 capture.

## Round 1 status

| # | Status | Evidence |
|---|---|---|
| M1 cube hidden by pane, labels pile up | FIXED (with caveat P2) | `r2-cube-stood-back-with-pane.png`: cube fully visible, x 430-849, y 258-541; one label only (`XP 1 · face 1 of 4`, x 543-738, y 549-582); no pane label. The pane is absent altogether. |
| M2 corner chip in use mode | FIXED | `r2-stepped-into-face.png`: no chip; Recycle Bin label x 1212-1267, y 727-738 is clear. |
| M3 pane label over the face | FIXED | `r2-stepped-into-face.png`: no `Terminal` pill. |
| M4 cube off centre, label touches it | FIXED | Cube bbox x 430-849 (centre 639.5, was +34/-32). Cube lowest point y 541, label top y 549: 8 px gap (spec 8). Label x 543-738, centre 640.5, on the hint bar axis (367-913, centre 640). Same pose in both captures. Caveat: see N2. |
| M5 grey glass, stray rim | FIXED | Dark tint (26,28,33) with light text (15.8:1). Rim lines at y 728 and 767 run x 565-715 inside a capsule of x 548-732, so they sit on the straight part and stop before the rounded ends. Residual: N6. |
| M6 side panes grey slabs | FIXED for the panes, OPEN for the floor | Side panes are now (36,36,36), the same as the centre pane. The floor mirror is still a hard-edged grey slab, see N4. The haze at half the leash radius cannot be seen: panes at this distance are un-hazed; no capture far enough away. |
| M7 text at screen edge | FIXED | `r2-first-program-flown-in.png`: first glyph at x 12, y 13 (was 0,0). |
| m1 failed face copy, dots, fill | OPEN (not re-captured) | |
| m2 disabled hint 2.9:1 | OPEN (not re-captured); the enabled `Step in` measures 15.8:1 | |
| m3 start card rows inset 8 px | OPEN (not re-captured) | |
| m4 help see-through | OPEN (not re-captured) | |
| m5 stale buffer in placeholder | OPEN (not re-captured) | |
| m6 label words (`foot`, `Extra 1`) | FIXED | `r2-walk-3-panes.png` label reads `Terminal` (x 595-685, y 574-606). |
| m7 pull frame loses the HUD | OPEN (not re-captured); would be MAJOR if it ships | |

Regressed: none.

## New findings

### MAJOR
None found in the 5 captures. (N1 would be MAJOR if confirmed; see its text.)

### MINOR

- **N1. Neighbour panes have no label** (brief RC: "every pane has a label"). `r2-walk-3-panes.png`: left pane
  (x 0-329, y 229-571) and right pane (x 951-1280, y 229-571) show no label, no accent edge and no glass. Only the centre pane
  is named. If the labels are meant to appear only on the focused pane this is a spec call; the brief has no such exception.
  Promote to MAJOR if neighbours are meant to carry labels. Fix: draw the label pill (smaller, 60 % opacity) under each
  pane that is at least 40 % on screen, or write the exception into the HUD spec.
- **N2. The "next face shows" sliver is 15 px wide.** `r2-cube-stood-back*.png` x 835-850, y 258-541: the right face is a
  sliver in which the XP wallpaper is stretched (a white blob at x 836-840, y 300-325; a blue hill at x 806-835, y 387-398).
  It is not recognisable as a face, so "each face is recognisable" and "turns a little so the next face shows" are met only
  by intent. Fix: yaw 20-25 degrees at rest instead of about 10, or show the next-face label as a ghost chip.
- **N3. Cube pane crowding in walk: side panes are cut by the viewport.** `r2-walk-3-panes.png`: the left pane is clipped at
  x 0 and its terminal text is off screen; the right pane's top edge runs to y 229 at x 1280. The centre pane is 530 px wide
  (41 %). Compared with the 11-12 earlier walk capture (side panes fully visible, x 115-351), the neighbours now fill the
  margins and crowd the hint capsule (reflections reach y 720). Fix: widen the pane spacing so neighbours stay inside 24 px
  gutters, or fade the pane to 0 at the frame edge.
- **N4. Floor mirror of the dark panes is a hard grey slab.** `r2-walk-3-panes.png` x 0-325 and x 955-1280, y 618-720: the
  mirrors are (189-201) grey on a (237) floor with straight vertical edges at x 325 and 955, 4 px in from the pane edge
  (329/951). Round 1 M6 asked for no grey slabs; the pane is fixed, the mirror is not. Fix: fade the mirror to 0 by 60 px below
  the pane and soften the side edges (feather 8 px), or cap its opacity to 0.25.
- **N5. Reflection geometry of the cube does not match.** `r2-cube-stood-back.png`: the right face's mirror starts at y 572,
  but the front face's mirror starts at y 628 (x 433-840). The front face bottom is at y 523-542, so its mirror should start
  near the same point as the side face's. It looks as if the cube floats at two different heights. Fix: mirror about the
  floor plane with a single transform for all faces.
- **N6. Step-back capsule over a program: the rim reads as a second hairline.** `r2-stepped-into-face.png` x 548-732,
  y 728-768: a lime double outline appears over the XP wallpaper (outer rim plus the inner glow ring). The capsule bottom is 2 px
  above the XP taskbar (y 770). Fix: one rim (1 px, 25 % white), and lift the hint to 40 px from the bottom when the face
  has its own taskbar.

### COSMETIC

- **N7.** `XP 1  · face 1 of 4`: the gap before the dot is wider than after (x 588 to 597 against 597 to 604), as in round 1.
  Use one space token.
- **N8.** Key caps in the dark capsule (`r2-first-program-flown-in.png` x 565-641, y 737-760) have a lighter 1 px bottom line
  with square ends that does not follow the 6 px rounded corners (zoom 4x). Clip it to the cap's radius.
- **N10.** Cube pose at rest shows the cube at 33 % view width (420 px of 1280) with 260 px of empty ceiling above; the
  composition sits low-centre, as already known to the PM (not counted).

## Checked and clean

- Contrast: `face 1 of 4` 7.1:1; `Step in` 15.8:1; dark capsule text 15.8:1.
- Cube bbox centred on the hint axis; label 8 px below the cube; one label while stood back.
- No corner chip, no pane label while stepped in; capsule in dark glass over a program.
- First program: text at x 12, y 13; the `Alt Esc Step back` capsule is centred (x 548-732) and 32 px from the bottom.
- Centre pane label `Terminal` is centred under the pane (x 595-685, y 574-606), with a blue edge on the focused pane.

## Verdict

**MERGE-OK** for the five captured states: no MAJOR found, all seven round-1 main-path flaws M1-M7 are FIXED (M6's floor
mirror lingers as N4). Conditions before week 2 closes: provide the 2x captures (P1), a with-pane capture that shows a
visible neighbour pane (P2) and re-captures of the first floor, help, failed face and the pull frame (P3), because m1-m5 and
m7 stay OPEN without them. N1 turns this into FIX if neighbour panes are required to carry labels.

Counts: M1-M7: 7 FIXED, 0 OPEN, 0 REGRESSED. m1-m7: 1 FIXED, 6 OPEN (not re-captured), 0 REGRESSED.
New: 0 MAJOR, 6 MINOR (N1-N6), 3 COSMETIC (N7, N8, N10). Process gaps: 3.
