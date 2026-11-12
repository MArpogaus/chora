# Review: construct look v2, pixels (round 1)

Independent UI reviewer, 2026-11-12. Inputs: the 23 PNGs and `cv2-step6-drag.mp4` in
`media/2026-11-12/construct-v2/` (first look before any note), then `design/looks/construct-v2.md`,
`design/construct-hud-spec.md` (state table), the three mocks in `media/2026-11-12/design/construct/`,
`design/decisions-taste.md` (construct v2, launcher rule) and `design/product-briefs.md`
(chora-construct). Fresh captures of the branch head `claude/construct-v2` 3b0b91d
(`session.sh construct 1280x800`, `CHORA_TEST_FLAGS=--test-injected-press-is-real`, act.py) are in
`media/2026-11-12/review-construct-v2/` (`rv-*`). The PM already knows: step 9 (a different
manager per face) is not done, faces are now "XP 1" to "XP 4", the rest-pose cube is about 33 % of
the view width against 46 % in the mock, and the lit-edge capture has no pointer. These are not
counted again.

Rating: a flaw you can see at 1x on a main path (first floor, first program, walk, cube stood back
and stepped in, failed face, help) is MAJOR, whichever lane caused it (rules.md 8). MINOR: a
spec or mock deviation off the main path, or one you only see when zoomed. COSMETIC: you can
measure it but hardly see it.

## MAJOR

**M1. With a pane open, the cube stood back is hidden behind the pane, and two labels pile up.**
`rv-10-cube-stood-back.png`, `rv-11-cube-hover.png`, `rv-12/13-stood-back-face-*.png`.
Path: `Super T`, `Alt Esc`, `Super C`, `Alt Esc`. The terminal pane (x 375-905, y 235-565) is
drawn in front of the cube. You only see the cube as its mirror image on the floor (x 400-820,
y 625-700). The face label `XP 1 · face 1 of 4` (x 505-698, y 541-573) sits 24 px over the
pane's bottom edge (565). Over the dark pane its glass turns grey and gets a white rim line (see
M5). The pane label `Terminal` (y 574-606) shows right under it, although the spec state table
("stood back from the cube") hides pane labels. Turning the cube changes only the label text,
so the cube itself never shows. The hover changes only the hint fill (diff bbox 571-700,
732-764); there is no lit edge to see.
*Fix:* while you stand back from the cube, fade out the walk panes (or put the cube in front of
them on the camera axis), hide the pane labels, and put the face label 8 px below the cube's
lowest projected point. Add a step 7 check: stood-back capture with one terminal open.

**M2. In use mode the old corner chip is back, and it covers the face.** `rv-04-super-c-use-mode.png`,
`rv-07-use-mode-face-2.png`, `rv-08-use-mode-face-3.png`, `cv2-step5-starting.png`,
`cv2-step5-failed-face-use.png`, `rv-09-help.png`. A screen-fixed chip `Step back  Alt+Esc`
sits at x 1107-1263 (17 px from the right edge), y 721-751. It covers the XP Recycle Bin label
(x ~1205-1268, y ~728). On face 2 it moves to y 731-761, a 10 px jump between faces. It writes
`Alt+Esc` as text, not as key caps, and it stays visible under the help scrim. This is polish
item 7 of the look, which removes it: "no chip in use mode".
*Fix:* remove the chip. Use the v2 bottom-centre hint `Alt Esc Step back` (caps) for 3 s after
step-in and on bottom-edge dwell, as for flown-in panes (`rv-02`).

**M3. The pane label floats over the face in use mode.** `rv-04`, `rv-07`, `rv-08`: the `Terminal`
pill sits at x 595-685, y 574-606 over the XP wallpaper while you are stepped into face 1, 2
and 3. This is polish 16 and spec 6: "no pane label in use mode".
*Fix:* hide pane labels while flown in or stepped into a face (fade 120 ms).

**M4. The cube at rest is off centre, and its label touches it** (no pane open).
`cv2-step7-rest-pose.png`, `cv2-step8-lit-edge.png`, `cv2-step9-faces-1..4.png`.
- Cube bbox x 461-888 (centre 674.5, +34.5 px) in step 7, and x 398-818 (centre 608, -32 px)
  in step 9. The rest pose flips side between the two: in step 7 the left side face shows, in
  step 9 the right one.
- The face label follows the cube (centres 681 and 600), so it is 41 and 40 px off the axis of
  the hint bar (x 367-913, centre 640): two stacked capsules that do not line up.
- The top edge of the label is at y 488. The cube's lowest corner is at y 487 (step 7 left
  corner x 476; step 9 right corner x 803), a 1 px gap where spec 5 asks for 8.

*Fix:* aim the camera so that the projected bbox of the cube is centred on x 640. Turn the same
way at rest every time (D3: toward the next face). Place the label 8 px below the lowest
projected vertex, centred on the view axis.

**M5. The glass capsule over dark content: grey fill and a stray rim line.**
`cv2-step2-flown-in.png`, `rv-02-flown-in.png`, `rv-10`. Over the dark terminal the
`Alt Esc Step back` pill fills with #ACACAE. That is not a palette or glass token, and it reads
muddy. A 1 px #F5F5F5 line runs straight along its top (x 563-717 at y 648 in the 720 capture,
y 728 in the 800 one). It does not follow the rounded ends (pill x 549-732), so it reads as a
loose white stroke. The same line shows on the face label in `rv-10` (y 541). This is the first
frame after the first program starts.
*Fix:* clip the rim highlight to the capsule's rounded rect, inset 1 px. Over dark content use
the dark glass tint (text #ECEEF2 on a dark tint), not a grey mix.

**M6. The side panes in walk mode are grey slabs.** `cv2-step3-walk-hints.png`,
`cv2-step4-label.png`. The neighbouring terminals (x 115-351 and 928-1164) are flat #ACACAC
with faint unreadable text, and the floor mirrors them as grey slabs too. The look's philosophy
says "no black, no grey slab", and a dark terminal should read dark at an angle. This is not
re-checked on head (`rv-03` has only one pane).
*Fix:* draw panes off the axis with their content (plus a light fog at most). No veil that turns
dark content into #ACACAC.

**M7. The text of the first program touches the screen edge.** `cv2-step2-flown-in.png`,
`rv-02-flown-in.png`, `cv2-flown.png`. foot text starts at x 0, y 0 (`warning:` at 0,0; in
`cv2-flown` at the pane edge x 260). The construct's own demo pane (`cv2-step10-flown-in.png`)
pads 14 px. This is the foot/flavor lane, but it sits on the main path.
*Fix:* ship `pad=12x12` in the flavor's foot config (or the construct's default terminal
arguments).

## MINOR

- **m1 Failed face differs from the spec and the mock** (`cv2-step5-failed-face.png`).
  - The card says `Click or press Return to try again.`; the look says `The other faces still
    work.` (the hint already carries Return).
  - The face label lacks `face 1 of 4`. All four dots are hollow rings, where only the failed
    face's dot should be.
  - The face fill #F8F9FB equals the card fill #F8F9FC, so only the shadow separates them. The
    mock uses a grey face (~#D5D9E0).
  - The face is square-on, not at the 14° rest pose.

  *Fix:* use the spec copy, the label and the dots. Fill the face with the plain face colour
  one step darker than the glass.
- **m2 Disabled hint at 2.9:1.** `cv2-step6-turning.png`: `Return Step in` at 0.45 measures
  #8B8E91 on #EBECEE, 2.9:1 (`Return` key cap 2.92:1). WCAG exempts inactive controls, but the
  brief asks for 4.5:1. *Fix:* 0.6 opacity gives about 4.6:1. Or accept it as an exemption and
  write it down.
- **m3 The start card's rows are inset 8 px against the head and footer.** `rv-01-first-floor.png`,
  `cv2-step1-first-floor.png`: row labels at x 448 and key caps ending at x 832, while the head,
  the footer and the separator run x 440-840. In the mock the row labels align with the head
  (436). No floor shadow shows under the card (the mock has one). *Fix:* put the row text on the
  440/840 grid and let only the hover fill bleed 8 px.
- **m4 The help is see-through.** `cv2-step12-help.png`: a rectangle 8 levels darker (#EEF0F2
  against #F6F8FA) reads through the card at x ~380-900, y ~210-500 (the pane behind). Polish 4
  says "no image reads through". *Fix:* tint 0.92 plus the scrim as specified; check the
  delta is ≤ 2.
- **m5 Placeholder shows a stale buffer.** `cv2-step4-placeholder.png`: the previous terminal's
  first line shows through the glass placeholder at x 380-520, y 212. *Fix:* do not sample the
  old buffer under the placeholder.
- **m6 Label words.** `cv2-step11-label.png` shows `foot` (foot's default title equals its app
  id; the look says "never `foot`"). `cv2-step3-walk-hints.png` shows `Extra 1`. *Fix:* when the
  title equals the app id, use the desktop entry name (`Terminal`).
- **m7 The pull frame loses the HUD.** `cv2-pull-debug.png` (a debug capture): the hint bar has
  no glass and no key caps, only the bare words `Turn left  Step in  Turn right` at y 748. The
  face label is missing. The pulled window is a flat #2679C8 square with no content. If this
  frame ships on the pull path (a USP), it is MAJOR. *Fix:* re-capture the pull on head.

## COSMETIC

- Uneven space around the `·`: in `Media center⎵⎵· did not start` the gap is 9.5 px before
  the dot and 6.5 px after (`cv2-step5-failed-face.png`). The same holds for `XP 1 · face 1 of 4`
  and the help heads `HELPER AND CHAT  · EVERY LOOK` (`rv-09-help.png`). Use one space token on
  both sides.
- The face label width jitters by 1-2 px from face to face (step 9: x 509-689, 508-693,
  507-693). Use tabular figures.
- On a face at rest (`cv2-step6-rest.png`) the XP desktop is scaled by 0.625: icon labels are
  about 6 px and soft. That is expected stood back; just noting it.
- In the drag clip, the hint state changes are hard cuts (frames 2, 60, 71, 110) where the spec
  asks for 120 ms out. **No frame flashes:** the largest mean frame-to-frame change is
  4.8/255, there are no single-frame spikes, and the settle takes 9 frames (300 ms, within
  320 ms).
- `cv2-step8-lit-edge.png` matches step 7 except the clock (diff bbox 869-872, 466-570). Both
  show the 2 px #2680B4 edge plus glow. There is no pointer-off capture of the same pose, so
  "gone within 120 ms" is not shown.

Checked and clean:
- Contrast of all text but m2 is 6.7:1 or more (the muted #4A5260 on glass measures 6.7 to
  7.4).
- One family (Inter; DejaVu Sans Mono in the data well and the terminals).
- The accent appears only on the dot, the ring and the pane or face edge.
- The hint bar is centred (x 367-913) and sits 32 px from the bottom.
- The start card is centred to the px (x 416-864, y 252-548 at 800).
- The help groups are fixed on head (`F1` in Walk, `Super /` in Everywhere, display names
  Tiling, XP, Media center, Construct).

## Main-path flaws, any lane

| # | Path | Flaw | Capture | Lane |
|---|---|---|---|---|
| M1 | cube stood back with a pane open | cube hidden behind the pane; face label 24 px over the pane; pane label shown | rv-10..13 | construct (scene/HUD) |
| M2 | stepped into a face / starting / failed | corner chip `Step back Alt+Esc` covers Recycle Bin, jumps 10 px | rv-04, rv-07, cv2-step5-* | construct HUD |
| M3 | stepped into a face | `Terminal` pane label over the face | rv-04, rv-07, rv-08 | construct HUD |
| M4 | cube stood back (no pane) | cube ±33 px off centre; label 40 px off the hint axis; 1 px gap instead of 8 | cv2-step7, cv2-step9-* | construct camera/HUD |
| M5 | first program flown in | grey #ACACAE glass, rim line not following the capsule | cv2-step2, rv-02, rv-10 | construct glass |
| M6 | walk with 3 panes | side panes are #ACACAC slabs | cv2-step3, cv2-step4-label | construct render |
| M7 | first program flown in | terminal text at x 0, y 0 | cv2-step2, rv-02 | flavor (foot config) |

## Verdict

**FIX.** Seven main-path flaws. M1 to M3 are on head and break the "stood back" and "use mode"
states of the HUD spec. M4 to M7 are visible at 1x. Round 2 should confirm with a re-capture:
the cube stood back with and without an open pane, stepped into a face, and the first program.
