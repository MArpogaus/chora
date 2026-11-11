# Construct HUD: blurred glass, no buttons (spec)

Design lane of `construct-hud-rework.md`, 2026-11-11. Mockups:
`media/2026-11-11/construct-hud/` (1x and `-2x`, light and dark); sources: `design/mockups/`
(`construct.html?state=…&theme=…`, `keys-xp.html`, `keys-mc.html`, `render.js`, `clean.py`).
Base: `docs/design-principles.md` (P), `looks/shared.md`, `docs/construct-style.md`, `docs/cube-modes.md`.

## 1. The idea

One material, three families of surfaces, nothing that looks like a button.
- **Ambient** (screen corners and the pane): the voice island top right, the pane label, the
  context hints bottom centre, notices under the island. Small capsules; they fade when unused.
- **Conversation** (right column): the chat, the question card. The island *is* the chat's
  collapsed form; it expands down into the column in place (U3: one surface).
- **Command** (centre, over a scrim): the window list and the bindings overlay. Typing filters.
Actions come from keys, pointer focus and hover. Every hint names its key. A hint, a row and
a face respond to the pointer (hover, click), but none has button chrome: no bevel, no border box,
no fill until hovered. The two exceptions are fixed by `shared.md`: Yes and No of the question
card stay click targets (P5), drawn as key segments ("Alt Y  Yes").

## 2. Material tokens (logical px)

| Token | Light | Dark |
|---|---|---|
| backdrop blur | 40 (Gaussian equivalent; dual Kawase 4 passes, offset 3) | 40 |
| saturation | 1.8 | 1.6 |
| tint, base | `#F8F9FB` at 0.64 | `#181A20` at 0.56 |
| tint, strong | `#F8F9FB` at 0.82 | `#181A20` at 0.84 |
| inner highlight | top 1 px `#FFFFFF` 0.90; inner ring 1 px `#FFFFFF` 0.55 | top 1 px `#FFFFFF` 0.16; ring `#FFFFFF` 0.10 |
| outer hairline | 1 px `#10141C` 0.10 | 1 px `#000000` 0.60 |
| shadow | 0 1 2 /0.06, 0 8 24 /0.08, 0 24 64 /0.12 of `#10141C` | same offsets, black 0.30, 0.30, 0.50 |
| `text` / `muted` | `#14181E` / `#4A5260` | `#ECEEF2` / `#A3AAB5` |
| hover fill | `#14181E` 0.05 | `#FFFFFF` 0.07 |
| selection fill | accent 0.16 | accent 0.30 |
| well (field, detail, segment) | `#14181E` 0.05 | `#000000` 0.22 |
| keycap | `#FFFFFF` 0.72, ring `#10141C` 0.12, bottom 1 px 0.10 | `#FFFFFF` 0.10, ring 0.12, bottom black 0.40 |
| scrim (command surfaces) | `#ECEFF4` 0.40 | `#000000` 0.36 |

- **Adaptive tint.** The core measures the mean luminance of the blurred backdrop under each
  surface (it has it from the blur). Light: below 0.45 the tint goes linearly to *strong*; dark:
  above 0.30. It eases over 240 ms, never per frame. The question card, the chat, the window list
  and the help are always *strong* (text-heavy, security).
- **Fallback.** Without blur (watchdog, `backdrop` unavailable): strong tint, alpha +0.08, same
  hairline and shadow. Contrast holds; it only looks flatter.
- **Accent** `#2680B4` stays the one hue: ring, focus ring, selection, caret, edge of the focused
  pane and of a lit cube face, a waiting question's hairline (0.55) and glow (24 px, 0.18).
  Never text.

## 3. Shape, type, space

- Radii: panel and card 24; inner rows, field, answer segments 16 (concentric: panel inset 8);
  detail well 12; keycap 6; capsules full (height / 2).
- Unit 8. Safe margin 32 from every output edge (24 below 1024 px wide). Panel padding 16;
  command surfaces inset rows by 8.
- Type, four sizes (adds `small` 14 and `display` 24, drops the 12 `data` size):

| Role | Spec | Use |
|---|---|---|
| `label` | Inter 500 11/16, caps, +0.08 em | state words, group heads, scope; only beside a shape or as a head |
| `small` | Inter 500 or 400 14/20; keycaps 14/24 | key names, pane label, notes, key lines |
| `data` | DejaVu Sans Mono 14/20 | paths, question detail, tool steps, row notes |
| `text` | Inter 400 16/24 | sentences, messages, prompts, row names |
| `display` | Inter 300 24/32, -0.01 em | the search line of the window list and the help only |

- Keycaps: min 24 x 24, padding 0 7, 4 apart; a chord is caps side by side, no "+" (`Ctrl Alt →`).
  A pointer gesture is a plain muted word after the cap (`Super drag`).
- **Focus ring** (keyboard focus): 2 px accent + 4 px halo accent 0.22, outside, radius follows.
  Only the surface or row that has the keys shows it; the pointer never draws it.
- **Hover** (pointer): hover fill; a capsule raises its tint by 0.06. 0 ms in, 120 ms out.
- **Selection** (list choice): selection fill, text stays `text`.

## 4. Motion

| Change | Timing |
|---|---|
| appear | opacity 0→1 160 ms, blur 0→40 over 200 ms ("materialise"), ease out (0.2, 0.8, 0.2, 1) |
| disappear | 120 ms, ease out |
| island → panel (chat open), panel → island | size morph 240 ms ease out; content cross-fades 120 ms after 80 ms |
| hint after hover dwell | 400 ms dwell, then 200 ms in; fades 300 ms after 3 s idle |
| tint adapt | 240 ms |
| ring | thinking 1.2 s/turn, working 2.4 s/turn, question pulse 0.6→1.0 over 2 s (only loop) |
No bounce, no slide, no scale of content. Screen-fixed surfaces never move with the camera.

## 5. Surfaces and layout (1280 x 720; positions scale with the output, sizes do not)

| Surface | Place | Size | Contents |
|---|---|---|---|
| island | top 32, right 32 | h 40, hugs content, ring 24 inset 8 | ring, state word (`label`), task (`small` muted) after a 1 x 16 separator |
| notice | under the island, 8 gap, right 32 | w ≤ 360, h 40 or 64 | one line `text`, optional `small` muted; 4 s, max 2 stacked |
| question card | top 80, right 32 | w 400 | head `QUESTION FROM ATLAS` + time left (`data`); prompt `text`; detail well `data`, scrolls, never cut; physical note `small`; Yes / No segments h 40, 8 apart |
| chat | top 32, right 32, bottom 32 | w 400 (clamp 400–480 = 28 vw) | header 56 (ring, state, helper · task); transcript; field h 48 inset 8; key line h 40 |
| pane label | 8 below the projected pane bottom, centred | h 32, w ≤ 320 | window title `small` 500 |
| cube face label | 8 below the cube, centred | h 32 | look name, "face 1 of 4" muted, 4 dots (current in accent) |
| hints | bottom 32, centred | h 40 | 2–4 hints: keycaps + `small` word, 1 x 16 separators |
| window list | top 104, centred, over scrim | w 560, rows 56 | search (`display`), count; rows: thumbnail 64 x 40 r 8, name `text`, place `data` muted; footer keys |
| bindings overlay | top 64, centred, over scrim | w 880 (2 cols) / 560 (search) | search, scope `CONSTRUCT · WALKING`; groups; footer |
In the chat the user's message sits right in an accent bubble (0.16 / 0.34), r 20 20 6 20; helper
messages are plain `text` under a `label` name; tool steps are `data` muted with a check or arc.
With a question waiting and the chat open, the card keeps its place (top 80) inside the column;
the transcript flows under it.

## 6. What shows in each state

| State | Shows | Hidden |
|---|---|---|
| walking | island; pane label of the pane ahead or under the pointer; hints `W A S D Walk · Return Fly in · Tab Windows · Super / All keys` for 6 s after entering and while the pointer rests in the bottom 64 px | everything else |
| a pane moving (walk, drag) | island | pane label (rule 17), hints |
| flown into a pane | island at 0.6 opacity; hint `Alt Esc Step back` for 3 s after the fly-in and on bottom-edge dwell | pane label, walk hints |
| stood back from the cube | island; face label; hints `Ctrl Alt ← Turn left · Return Step in · Turn right Ctrl Alt →`; the face under the pointer lit (2 px accent) and its hint `Step in` hovered | pane label |
| cube turning | the same; `Step in` at 0.45; the face label changes at the end of the turn | lit edge |
| in a cube face (use mode) | as flown into a pane | — |
| helper speaking | island widens left to the sentence (`text`, wraps at 480, ≤ 2 lines, end kept), bars ring; 2 s after the end it shrinks back | — |
| helper working | island: arc ring, `WORKING`, task and step `1 of 2` | — |
| question waiting | island `QUESTION FOR YOU` with pulse; card under it; hints hidden | — |
| chat open | the column; keyboard in the field (focus ring) only after the person's act (P11) | island (it is the header), hints |
| window list / help open | scrim, the command surface, island | hints, pane label |
| muted | island ring struck, `MICROPHONE OFF`; hover hint `Super M Turn on` | — |

## 7. Keyboard maps

**Construct, walking** (from `construct.yaml`): W A S D or ↑ ↓ walk, Q E or ← → turn, R F look,
H home, Return fly in, Tab windows, Super T terminal, Super Space launcher, Super X close the
pane ahead, Super drag move, Super wheel resize, F1 or Super / keys. **In a pane:** Alt Esc out.
**Cube:** Ctrl Alt ← → turn (both modes), Return step in, Alt Esc step back, Esc steps in again.
**Chat** (field has the keys): Return send, Shift Return new line, Ctrl . stop the helper,
Esc close (draft stays), ↑ in an empty field edits the last message. **Question:** Alt Y yes,
Alt N no, from any window; typing never answers. **Window list:** type to filter, ↑ ↓ choose,
Return fly to it, Super Alt 1–4 give the chosen window, Esc clears the filter, then closes.
**Help:** type to search, ↑ ↓ move, Esc clears, then closes. **Shared, every look:** Super A chat,
Alt Y / Alt N, Esc back one step, Super Alt 1–4 give, Super drag hand over, Super / keys.
**Pointer:** click the island opens the chat; hover a hint shows its key, a click does its act
(P3 mouse path); click a face steps in; drag a face turns; wheel in a list scrolls.

## 8. The shared bindings overlay (`chora-keys`)

- **Key:** `Super+/` in every look, a grab, so it works while any window has the keyboard.
  `F1` also opens it where the window manager itself has the keyboard (construct walk mode, xp
  desktop, mc); inside a window F1 stays the program's. The same key or Esc closes it.
- **Source of truth:** a window manager declares each grab with words through `chora-kit`:
  `keys::bind(group, "Super+t", words::OPEN_TERMINAL, when)`; the call *is* the grab, so the list
  cannot drift from what is bound. The host keeps the table per output and serves it (`keys.list`)
  with the shared set (chat, Alt Y / N, Esc, give keys, the overlay key). `when` (walk, pane,
  cube, always) hides rows that do not apply now (construct checklist 7).
- **Contents:** groups of the focused look first, then the shared groups marked "every look".
  A shared key that the look shadows (mc's `Super+a` = Agenda) shows once, in the look's group,
  with the shared act listed under its fallback key; `--check-keys` warns about the clash.
- **Search:** typing filters by words and key names (en and de), highlights the match (accent
  0.24 behind the letters), groups stay; ↑ ↓ move a selection; the count shows in the footer.
- **Themes** (a `keys` style in each look's yaml):
  construct: glass, centred, 880 / 560 wide (mockups `construct-help-*`);
  xp: a Luna window "Keyboard shortcuts", task pane with Search and Groups, Tahoma 14, blue
  group bars, a classic scroll bar, matches on top and the rest under "All other keys" (`keys-xp`);
  mc: a full page like a scene, chapter `F1 | KEYS`, keys in the margin in Inter 20, acts in
  Pagella 26 (step 1.5), rows 40 (1.5 baselines), search on the right (`keys-mc-*`);
  tiling: dense, a bar-height list in the bar font, 3 columns, 20 px rows, no scrim.

## 9. Mockups (each also at `-2x`, 2560 x 1440)

`construct-{walk,list,question,chat,cube,help,help-search}-{light,dark}.png`,
`keys-mc-{light,dark}.png`, `keys-xp.png` (xp has one theme). Backgrounds are real captures of
today (`bg/`, HUD removed by `clean.py`); the dark cube is the real face placed in the real void.

## 10. Open questions

1. Hints and the island answer a click (P3 mouse path) though they look like text. Enough, or
   should the mouse path in a pane be a hover at the top edge only?
2. Help key: `Super+/` everywhere plus `F1` where the manager has the keyboard. Agree?
3. Stop key `Ctrl+.` (macOS "Cmd+.") instead of a Stop button. Agree?
4. Four sizes instead of three (adds 14 and 24, drops 12). Agree?
5. mc binds `Super+a` to Agenda, against the shared chat key. Which one moves?
6. Should Return on a help row run the act (a command palette), or does the help only show?
