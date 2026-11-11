# Construct HUD rework (user, 11-11 ~20:45)

"Yes, go with the construct HUD rework. Make it look modern with an
expensive blurred-glass look like modern macOS or similar. Minimal yet
complete. Avoid buttons. Keyboard / mouse focus. Add an overlay to show
all bindings, shared as a plugin all other window managers may also use.
Work hand in hand with design agents and programmers. Make this stunning."

## Lanes
1. Design (opus designer): construct HUD spec and mockups (rendered
   images), the bindings overlay design for every look, a reference
   sheet; user sees the mockups before the build (rule of reflection 2).
2. Core (opus programmer): real backdrop blur for glass nodes (a node
   prop `backdrop: {blur, saturation, tint}` that samples what lies
   behind it, dual Kawase or similar), cheap and damage-aware, with the
   2D guard and render.stats unchanged when unused; the watchdog may
   drop it to the plain tint.
3. Construct (opus programmer, after 1 is approved): the HUD built on
   the spec: glass surfaces with blur, no buttons, contextual hints that
   appear on focus or hover, keyboard and pointer focus with a visible
   focus ring, the voice ring as the chat entry.
4. Shared bindings overlay (chora-kit + a small plugin): each window
   manager lists its grabs with words; one key (Super+/ or F1) shows a
   glass overlay of all bindings of the focused look plus the shared
   ones; works in every look, themed per look.

## Mockups (11-11 ~21:10): design/construct-hud-spec.md, media/2026-11-11/construct-hud/
Sent to the user. PM recommendations on the designer's questions, the
default unless the user decides otherwise:
1. Hints and the voice island also respond to a click (mouse path P3),
   with a hover fill so they read as live.
2. Help key: Super+/ everywhere, plus F1 where the window manager has
   the keyboard.
3. Ctrl+. stops the helper; the Stop button goes.
4. Four type sizes (11, 14, 16, 24) replace the three.
5. The shared chat key Super+A wins; mc's Agenda moves to its own scene
   key (number keys or Super+G).
6. The help shows only for now; Return on a row does nothing (KISS);
   revisit after the rc.
