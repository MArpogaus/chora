# Design per look: combinations (draft)

Draft for the PM and the user, 2026-11-11. Where two looks meet, the person must not notice a seam:
a window, a chat, a notice or the voice state behaves the same on both sides (P1, P3, P8).
Base: `shared.md`, `docs/cube-modes.md`, `design/window-handover.md`.

## Rules at the seams

1. **A window moving between looks** keeps its content, its title and the keyboard if it had it.
   It lands where it was dropped, at the point that was grabbed, not its corner. The receiving
   look may then place it its own way (tiling tiles it, mc gives it a scene page), and shows where.
   Every path works: Super+drag, the give keys, `choractl give`, voice.
2. **xp and tiling as panes in construct.** A pane is a window like any other: its label is its
   title, its keys go to it in use mode, and Alt+Escape steps back. The look inside the pane draws
   its own chrome; the construct draws only glass around it, never over it.
3. **The xp cube.** Four faces, each an xp on a virtual output. Use mode at 1:1 feels like xp;
   navigate mode feels like construct. Pulling a window out and putting it back take the same hand
   (plain drag both ways in navigate mode). A window pulled out is always visible and in the list.
4. **Two screens, two looks.** Each output shows only its own look: its own bar, taskbar or page,
   and its own helper surface. Nothing of one look draws on the other's output.
5. **One chat follows the person.** There is one conversation (it lives in voice) and one expanded
   surface at a time. It shows on the output where the person opened it, in that output's look.
   When the person moves focus to the other output by their own act, the chat redraws there in that
   look, with the same lines and the same draft, within 0.5 s. It never moves while the field has
   the keyboard or a question waits.
6. **Notices** show on the output where the act happened (the key press, the drop), in that look's
   notice form, one line. A notice never shows twice on two outputs.
7. **Voice state** is one state. Every output's collapsed helper surface shows the same state word
   at the same time; the muted state shows on every output. One indicator per output, never two.
8. **A question** shows once, on the output where the person is (the focused output), and stays
   there until it ends, even if the person moves on.

## Rough edges known today, with the target

| # | Seen | Source | Target |
|---|---|---|---|
| 1 | The chat looks the same plain panel in every look | ux-loop-1 4, persona-3 M3 | each look's own surface (`xp.md` ...); until then one neutral style that follows the theme |
| 2 | mc light shows a black chat box, construct dark a light slab | ux-loop-1 4, U2 | the look emits its theme; the host picks the folder's `dark:` block |
| 3 | The xp chat field sits over the taskbar | U2 | every helper surface stays inside the work area of its output |
| 4 | "Chat ..." pill and OPEN CUBE overlap; OPEN CUBE clipped at x<0 | ux-loop-1 5, U3 | one corner per element, 32 from the edges |
| 5 | Chips stay over a full-screen pane | ux-loop-1 6 | in use mode only the helper pill (dimmed) and STEP BACK show |
| 6 | The host pill covers the xp Start menu | ux-loop-1 7 | no pill in xp, also on a cube face; the paperclip is the door |
| 7 | Construct sentence balloon apart from the open chat; mc repeats the sentence | U3 | one surface; the sentence lives in the expanded surface while open |
| 8 | Two voice indicators in construct (LISTENING card and a green dot) | persona-3 minor | one indicator per output |
| 9 | Construct notices draw nowhere with `voice_hud: host` | ux-loop-1 1 | notices always draw, one line, no label |
| 10 | Give keys refuse in silence in construct | ux-loop-1 2 | give acts on the pane ahead; a refusal is a notice |
| 11 | Pull out of the cube by plain drag, put back only by Super+drag | ux-loop-1 8 | the same drag both ways |
| 12 | A window pulled out of the cube is lost to view | persona-3 B5 | it shows as a pane ahead and in the window list |
| 13 | A dropped window lands with its corner at the pointer | ux-loop-1 17 | the grabbed point stays under the pointer |
| 14 | After `cube.close`, a face window takes about 3 s to show | ux-loop-1 21 | it shows within one second, with no empty frame |
| 15 | Two screens: waybar draws on the xp output; the HUD jumps between screens | persona-3 M4 | rules 4 and 5 |
| 16 | A new window opens on the screen clicked last, not the one looked at | persona-3 M4 | open question below |
| 17 | The chat field never gets the keyboard after `Super+a` | persona-3 B4 | the field gets the token of the act, at once, on any output |
| 18 | mc drag and the key path in tiling, xp and mc are gaps in the handover matrix | state.md | every cell of `gate-handover-matrix.sh` passes |
| 19 | The window list says "construct · no pane" and hides windows | ux-loop-1 9 | every window listed; hidden ones "not in view"; no owner names |
| 20 | In the cube, the STEP BACK chip covers the Recycle Bin label and moves | ux-loop-1 15 | it stays left of the icons' row, at one fixed place |

## Open questions

- **The helper in the xp cube.** In use mode the person sees xp at 1:1: does the face's own
  paperclip show (it turns with the cube) or the construct's screen-fixed pill? Showing both breaks
  "one indicator per output".
- **Which output gets a new window** when two screens show two looks: the one under the pointer,
  the focused one, or the one the launcher was opened on?
- **The chat across outputs:** does it follow the focus at all (rule 5), or stay where it was
  opened until the person moves it? The sprint chose "follow"; persona runs saw it as jumping.
- **Theme across outputs:** if one output is xp (one theme) and the other construct dark, does the
  chat take each look's theme on redraw (this draft), or keep one theme?
- **Notices with no clear output** (a helper finished while nobody acted): focused output, or all?
