# Deep UX review: chora-tiling v2 (head 59df16e)

2026-11-12, headless 1280x800, Ink and Paper, about 34 looks. Judged against `design/looks/tiling-v2.md`,
`shared.md`, mock A, `product-briefs.md` (chora-tiling, shared RC), `decisions-taste.md`. No code changed.
Captures: `media/2026-11-12/ux-deep-tiling/NN-name.png` (the `s1-*` files in that folder are from an
earlier pass, not cited here). Pixels measured with PIL on the captures (1:1).

Test note: the persona harness did not start `chora-assistant` (session.sh runs `command -v` before the
binary dir is on PATH), and the assistant then loops "no HUD folder named neutral" without an installed
`huds/`. I started it by hand with `XDG_DATA_HOME` pointing at a copy of `plugins/chora-assistant/huds`, and
used `tests/fake-voice.py` for the voice agent. Findings 1 and 2 hold either way: they are what a user sees
with and without a voice agent.

## What is right (measured)

- Gaps: output edge to tile border 12, tile to tile 10, border 2, radius about 10 (07-three-windows: border
  x 12-13, next window 645-646, right edge 1266-1267). Fullscreen: no gap, no border (24-fullscreen).
- Bar: islands top 8, height 32 (y 8-39), sides 12; left x 12-459, centre 566-713 (centred on 640), right 986-1267.
- Tag pill 24 x 24, current in accent with dark number 6.8:1; muted text (layout word, empty tags, power icon)
  6.5:1; text 14.0:1. Key list text 15.9:1, hint and group heads 5.8:1 (Paper). Chat hint 7.6:1. Nothing under 4.5:1.
- Key list built from the bindings, words from the table, header line as designed (04-keys, 20-paper-keys).
- Focus mark: only the focused window has the gradient; Paper reads well (19-paper).
- Layout chip and bar word agree ("Tile left"); Super+Space and Super+Shift+Space follow the PM decision.

## MAJOR (main paths)

**M1. A first-time user has no visible way to the helper chat.** The old tab is gone and the bar has no chat
entry (`waybar/config.jsonc` modules: Apps, tags, layout, clock, volume, network, battery, tray, Keys, power).
With no voice agent connected the assistant draws nothing at all, so the desktop shows no helper, no "no helper"
state (brief: "no-helper state is shown when the agent is absent") and `Super+a` does nothing and says nothing
(13-chat). The launcher has no "Chat with a helper" row either (03-apps-chat: typing "chat" gives an empty
panel; shared.md: "the launcher of each look has Chat with a helper"). tiling-v2 promised the chat segment in
the right island (mock A shows "Listening" there).
Fix: add a bar module "Helper" (right island, before Volume) that is always present: word = state word
(Asleep, Voice off, No helper), click = `chat.open`; add the desktop entry "Chat with a helper" to the
flavor's launcher (`flavors/tiling/applications`); `Super+a` with no helper shows the chip "No helper running".

**M2. The helper surfaces do not belong to Ink: white, system font, own colours, over the content.** With a voice
agent (15, 16, 17): the "Chat ..." row is a white pill 69 x 30 at x 1194-1262, y 89-118, on top of the focused
window's text (5 px short of the bar's right edge 1267, so it aligns with nothing); the state is an 11 px word on a
white chip plus a green 24 px dot at the window's bottom-right corner (x 1239-1262, y 761-785, inside the border,
a hue outside the palette; question dot orange); the chat is a flat near-white panel x 880-1280, y 40-800: no
radius, no gap, edge-to-edge, starting 1 px under the right island (island ends y 39, panel y 40) and hiding
the right 400 px of the window instead of re-tiling; its font is a different family from Inter; links are
blue (0,80,160). The question card is white too (x 705-1263, y 583-751) and breaks shared.md: title is "Question for
you" (shared: "Question from Atlas"), no time left, buttons Yes/No 88 x 32 with 8 gap (right).
Fix: the tiling HUD folder in Ink tokens (surface, line, text, accent, radius 10/14, 12 px from edges); the chat
as an island column 12 from the edges below the bar (y 56-788) that the tiler treats as reserved space; the state
as a word in the bar module of M1; card in surface with accent buttons, same text as other looks.

**M3. Screenshot is silent.** Super+Print wrote `~/Pictures/Screenshots/2026-11-12_02-23-36.png` but nothing
appeared in 0.8 s or 3.8 s (12, 12b): `screenshot.sh` calls `notify-send`, and no notification daemon runs in the
tiling flavor (only xp starts chora-notify). tiling-v2 step 10: "notice with the path". RC (c) and honest states.
Fix: start `chora-notify` in the flavor (unit and persona companion list), or fall back to the chip
("Saved to ~/Pictures/Screenshots").

**M4. The launcher is a poor front door.** (02-apps) "Screenshot" is first and pre-selected, so Apps then Enter
takes a screenshot; the order is neither alphabetical nor by use (Screenshot, Foot, LibreOffice Writer, Foot
Client, Foot Server, ImageMagick ...). A search with no result shows an empty 509 x 411 panel without a word
(03). The prompt "Apps |" leaves a double space before the caret.
Fix: sort by name with the terminal and "Chat with a helper" first, Screenshot entries by name like the rest;
show "Nothing found" in muted text when the list is empty.

## MINOR

**m1. Tooltip stays (reproduced).** 04 and 05: after Keys is clicked the tooltip "Every key (Super+/)" (x 1111-1267,
y 49-85) stays under the open list, and 3.8 s after Esc has closed it; it vanishes only when the pointer moves
(21, pointer moved: gone). 23/24: "Run a program (Super+Space)" appears about 3 s after the click on Apps, when the
launcher is long closed, and sits on top of fullscreen content. Waybar gets no leave event when another surface
takes the pointer. Fix: set `"tooltip": false` on Apps, Keys and power (the word is visible; the key is in the key
list, where it belongs), keep tooltips only on clock and volume.

**m2. Fullscreen: the bar floats over the window** (24-fullscreen): the first terminal line (y 14-30) runs under
the islands and is unreadable; `Super+f` content does not win. Fix: hide the bar's layer on a fullscreen window
(waybar `layer: "top"` is above fullscreen; set the bar to the bottom layer while one is fullscreen, or reserve
the strip).

**m3. New windows do not take the focus** (07): Super+Return twice leaves the focus (and typing) on the first
window; the two new ones have hollow cursors. P11 allows the move after "a program they started".
Fix: `new_window: focus` default, or if on purpose, state it in the key list.

**m4. Key list size.** 813 x 692 (54-745) at 1280x800 against 880 x 560 in the note: 86 % of the height, so on a
1280x720 output it runs out (54 + 692 > 720 - 14). Fix: three columns or a max height of 560 with scroll; also
"Window that asks" (Super+u) reads as an unexplained state: "Window with a question".

**m5. Key list translucency shows the seams of what lies below** (20, 21): panel fill is (39,42,51) over a light
window and (25,28,38) over a dark one, a visible step at x about 640; and the 10 px keycap gap before "Previous
layout" (keycap ends 857, label 867) is tighter than the 12-27 px elsewhere. Fix: 100 % opaque `surface` when no
blur backing exists; label column at a fixed x past the longest keycap + 16.

**m6. Layout chip sits on the gap between two tiles** (09): 120 x 56 at x 580-700, y 648-704, on the seam at
x 634-647, so half of it covers each window border. Fix: centre it on the output but 24 px higher, or at the
bar (under the layout word it explains).

**m7. Session leftovers at a theme change.** Running programs keep the old palette (doc says so), but the Paper
and Ink windows then sit side by side with an Ink key list and bar and the Chat pill outline in dark: after
`settings.set look.theme` the assistant surfaces did not follow (19, 21). Fix: apply to the assistant at the same
time (step 11 of the note).

## COSMETIC

- C1. Shadows are not visible in Ink: pixels between tiles are (12-14, 13-15, 18-20) against wallpaper (13,14,20);
  spec `0 8 28 / 45 %`. In Paper they show (19). Fix: raise the shadow alpha in Ink or tint with accent.
- C2. Unfocused border measures (51,57,75) to (58,64,84) (left to right edge), spec `line` #2e3344 = (46,51,68);
  a slightly lighter and uneven ring: check the gradient shader for the unfocused case.
- C3. Left island has a fixed 448 px width: 45-69 px empty to the right of the layout word (x 390-459), the right
  island is tight. The layout word is muted with no button cue; tags 2-9 and "Tile" share one grey. Fix: width
  by content, or reserve the space for the Helper module of M1.
- C4. "Listening" label is 11 px (allowed next to a shape) but sits half on the window border and half on the
  desktop at 1181-1231 (15, 17); in Paper it is a grey chip on a dark window (19). Moves with M2.
- C5. Fonts in play: Inter (bar, launcher), JetBrains Mono (keys, foot), a system sans (chat, card, HUD labels),
  so three families in one frame (16).
- C6. The yellow 12 x 12 square seen at (648,507) in 08/09 and (398,38) in 22 is the pointer sprite of the headless
  capture, not a glitch (same square, same place as the pointer position).
- C7. `Super+Print` is the listed screenshot key; the note says `Print` alone (needs the grant, step 10); list and
  note should say the same.

## Not verifiable here

Motion (open, tag slide, focus cross-fade): a look takes about 100 ms and the first tag capture (10-tag2-fast)
equals the settled frame (mean grey 21.2 both), so no frame stepping was possible; use the 30 fps clip of step 9.
Hover and pressed states of bar buttons, urgent tag (red), battery and tray, power actions beyond the menu (not run),
Super+Alt give keys, a German locale.

## Consistency with the other looks

- Helper door (shared.md "A visible way in", P3): xp has the paperclip and the Start entry, mc the margin, construct
  the panel and the launcher row; tiling is the only look with no door in its own chrome (M1). The launcher row
  "Chat with a helper" is missing in tiling (M1, M4).
- Question card (shared.md P5): order and keys are right, but the title and the time left differ from the
  invariant (M2) and the card is plain white where xp, mc and construct draw it in their own idiom.
- One surface per helper conversation (U3): while the chat is open the state word and the row stay on screen
  beside it (16 shows panel plus the corner state), as the other looks fold them in.
- Cross-look keys: Super+Space = launcher and Super+Shift+Space = next layout match the PM decision; `Super+a`
  matches every look but, unlike xp and mc, gives no answer when no helper runs.
- Mute (shared.md): a visible mute and unmute is required in every look; tiling shows only the dot (click to mute),
  with no word "Mute" or "Voice off" in the bar.
- Palette: tiling is the only look whose helper has a green and an orange of its own and blue links.
- Strong points to copy elsewhere: key list header "Super+/ shows this list again. Esc closes it." (xp F1, mc
  block), the 4 px grid and measured gaps, contrast of all bar text.
