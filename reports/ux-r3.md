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
Not run. `fuzzel --version` 1.9.2. The PM chose a $HOME-only build (05:33); this lane's permission
check refused the download and build (06:35: no user approval in this session), so nothing was
built and no gate cell was added. Missing here: meson, tllist, fcft and wayland-protocols (fuzzel 1.11 needs
>= 1.32); harfbuzz, utf8proc, librsvg and scdoc are absent too but optional. Present: freetype2 26.1,
fontconfig 2.15, pixman 0.42.2, wayland-client 1.22, xkbcommon 1.6, cairo 1.18, libpng 1.6.43,
wayland-scanner, ninja, gcc.
Commands for setup.sh or a lane with the owner's approval (not run):
```
P=$HOME/opt/fuzzel; python3 -m venv $HOME/opt/meson-venv; $HOME/opt/meson-venv/bin/pip install meson
export PATH=$HOME/opt/meson-venv/bin:$PATH PKG_CONFIG_PATH=$P/lib/pkgconfig:$P/lib/x86_64-linux-gnu/pkgconfig:$P/share/pkgconfig
curl -fLO https://gitlab.freedesktop.org/wayland/wayland-protocols/-/releases/1.38/downloads/wayland-protocols-1.38.tar.xz
curl -fLO https://codeberg.org/dnkl/tllist/archive/1.1.0.tar.gz
curl -fLO https://codeberg.org/dnkl/fcft/archive/3.1.8.tar.gz
curl -fLO https://codeberg.org/dnkl/fuzzel/archive/1.11.1.tar.gz
# each unpacked: meson setup build --prefix=$P --buildtype=release [opts]; ninja -C build install
#   wayland-protocols: -Dtests=false
#   fcft: -Dgrapheme-shaping=disabled -Drun-shaping=disabled -Ddocs=disabled -Dtest-text-shaping=false
#   fuzzel: -Denable-cairo=enabled -Dpng-backend=libpng -Dsvg-backend=nanosvg
# then PATH=$P/bin:$PATH for the tiling launcher cell
```
Next for ti-1 once fuzzel >= 1.11 is there: click a program in the tiling launcher at 1x and 2x; a gate
cell that skips with its reason when fuzzel < 1.11 and fails when CI=true.

## Captures at 1x (media/2026-11-15/ux-r3/)
xp-, tiling-, mc-, construct-nohelper-1280x800.png and -1024x768.png. Seen in passing (MINOR, not entered):
in tiling-nohelper-1280x800.png the bar still says `coder · idle` while the chat says No helper (at 2x the
bar says No helper); the cell waits for the bar word first, so it is a refresh lag of the capture moment.
