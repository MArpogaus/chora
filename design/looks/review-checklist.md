# UI review checklist (draft)

Draft for the PM and the user, 2026-11-11. Every UI review uses this list (charter, "UI reviews":
hunt for perfection). Check the look's own page first (`xp.md`, `tiling.md`, `mc.md`,
`construct.md`), then `shared.md`, then this list. Any visual surprise, inaccuracy or unaesthetic
detail is a finding; on a main path it blocks a release.

## Inputs

- Captures at 1x and 2x output scale, at 1280x720 and 1920x1080, light and dark where the look
  has both, and over a white and a black window where a surface floats over content.
- Clips (at least 30 fps) of every motion: open, close, expand, collapse, state change, fly-in,
  turn, drag, give. Step through them frame by frame.
- The look's style guide values (`mc.yaml`, `construct.yaml`, the HUD folder) for measuring.
- Name each finding with the capture file, the pixel position and the rule it breaks.

## 1. Aspect ratio and scale

- [ ] Every picture (figure, icon, logo) has its exact aspect ratio: measure width / height.
- [ ] Pixel art scales by whole numbers only; no blur, no smear, no uneven pixels at 2x.
- [ ] Nothing scales during a pose or state change unless the style guide says so.

## 2. Clipping and edges

- [ ] No element is cut by the output edge, the work area, the taskbar, the bar or another surface.
- [ ] No text touches an edge of its panel; padding is the style guide's value on all four sides.
- [ ] Screen-fixed elements keep the look's margin (construct 32) from every output edge.
- [ ] Wrapped text shows whole lines only; an ellipsis is where the guide puts it.
- [ ] Borders and hairlines are unbroken, 1 px, crisp at 1x and 2x.

## 3. One surface per thing

- [ ] One helper surface per output; the sentence, a reply or a question shows in one place only.
- [ ] One voice indicator per output.
- [ ] One door per corner; no two chips, pills or badges overlap.
- [ ] A question card appears once and does not move until it ends.

## 4. Alignment

- [ ] Left edges line up (text column, titles, rows); measure in the 1x capture, to the pixel.
- [ ] Baselines line up in a row (bar modules, chips, label and value).
- [ ] Icons and dots are centred on their text's x-height or cap height, as the guide says.

## 5. Spacing

- [ ] Every gap and padding is a whole number of the look's unit (construct 8; mc baseline lines).
- [ ] Equal things have equal gaps: rows, buttons (Yes and No 8 apart), chips.

## 6. Type and contrast

- [ ] Only the look's families and sizes; no bold or italic where the guide forbids it.
- [ ] Text a person must read is at least 14 px (mc: its TV minimum); labels at 11 only beside a shape.
- [ ] Contrast measured on the real background: 4.5:1 text, 7:1 key line and buttons, 3:1 graphics.
- [ ] Words come from the word table; no internal terms, no lowercase names where display names exist.

## 7. Motion

- [ ] Fades and grows use the guide's timings and ease out; no bounce, no overshoot.
- [ ] No jitter: screen-fixed elements stay on the same pixel through a camera move or a fly-in.
- [ ] No frame flashes: no empty, black, white or wrongly themed frame at the start or end of a change.
- [ ] Updates show in the next frame after a click or key; nothing lags one capture behind.

## 8. Theme

- [ ] Every surface follows the look's theme: no light panel on dark, no dark box on light.
- [ ] Accent only where the guide puts it; state colours only for states.

## 9. Behaviour seen on screen

- [ ] Every state shows a word (idle, busy, done, failed, refused, offline, question, muted).
- [ ] Escape backs out one step; the keyboard returns where it was.
- [ ] Two screens: each output shows only its own look (see `combinations.md`).

## 10. The look's own rules

- [ ] Run the checklist of the look's page in full.

## Ranking

A finding on a main path (start a program, find a window, the chat, a question, volume, give) is
high. Elsewhere it is medium, or low when only a review at 2x can see it.

## Open questions

- Who captures the 2x set: the persona harness or a separate script?
- Is a quarter-size view of a 1080 capture a fair stand-in for mc at 3 m?
