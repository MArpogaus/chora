# ux r3 (rc 1): sh-10 and ti-1

Lane: opus, cloud, branch claude/ux-r3 from be205f89 (pushed, not merged). Full `tests/merge-gate.sh` on
42d5d4b2: 28 steps PASS, exit 0 (e2e: the five listed skips of tests/e2e-skips.txt: fkeys option,
thunar, pipewire, desktop-file-validate, xwayland-satellite).

| Item | Red commit | Fix commit | Cell | State |
|---|---|---|---|---|
| sh-10 MAJOR | ff9e68d7 | 42d5d4b2 | gate-doors helper-none x4; units `the_plain_header_says_no_helper_and_a_waiting_question_whole_in_its_lines`, `the_glass_header_..._whole_in_its_lines` | fixed on claude/ux-r3 |
| ti-1 BLOCK | - | - | - | blocked: no meson, fcft or tllist in the container (reported to the PM at the start) |

## sh-10
- Cause: the plain header (tiling) drew `No helper is set up. Ask the person who set up this computer.` in one
  line box (417.5 px in 372, ellipsis); the glass header without a state word (construct) drew it as one SVG
  line (406.9 px in 368, clipped at the panel edge). The meta of `hud-chat` held the whole text, so the doors
  cells could not see the cut.
- Fix (wrapping, same type): the plain header wraps at its words and the buttons and transcript move down
  under it; the glass header without a state word wraps into its 56 px band (two lines of 20). The meta of
  `hud-chat` holds each drawn line, a line that does not fit ends in `…`. docs/hud.md says both.
- Red: units fail on be205f89 with the widths above. Gate: the helper-none cells of tiling, mc and construct
  read both sentences whole (`says_no_helper`, lines joined by spaces; xp already read both lines). On a local
  build of the fix with the wrap switched off, `ONLY=helper-none tests/gate-doors.sh` fails tiling
  (`...who set up this c…`) and construct; with the fix all four pass.
- Sizes: probes of the four cell files at 1280x800, 1024x768, 2560x1600@2 and 2048x1536@2 (logical 1280x800
  and 1024x768 at 2x): 16 of 16 pass; meta of tiling, construct, mc: `No helper is set up. Ask the person who
  set up this` / `computer.`; xp: its two sentences on two lines. (At 1280x800@2, 640x400 logical, mc's
  column does not open: not a sh-10 cut, not entered.)
- Waiting word: `A question is waiting.` / `Eine Frage wartet.` fits whole in the plain and glass headers
  (in both units); xp and mc show it in their own two-line header and margin hint, unchanged.
- The doors gate runs the helper-none cells at 1280x800 1x only; 1024x768 and 2x are covered by the units
  (the plain chat is 400 wide and the glass chat at least 400 at both sizes) and the probes above.

## ti-1
Not run. `fuzzel --version` 1.9.2; fuzzel 1.11 needs meson, fcft and tllist, none of them here; the brief
and conventions override 9 make that a blocker. Asked the PM: allow a $HOME-only build (pip meson, fcft,
tllist, fuzzel tarballs) in this lane, or add it to setup.sh. No gate cell added yet.

## Captures at 1x (media/2026-11-15/ux-r3/)
xp-, tiling-, mc-, construct-nohelper-1280x800.png and -1024x768.png. Seen in passing (MINOR, not entered):
in tiling-nohelper-1280x800.png the bar still says `coder · idle` while the chat says No helper (at 2x the
bar says No helper); the cell waits for the bar word first, so it is a refresh lag of the capture moment.
