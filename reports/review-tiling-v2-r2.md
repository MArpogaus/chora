# Review: tiling v2 "Ink", round 2

Independent second review of branch `claude/tiling-v2` at 3303979, built in
`/home/user/wt/tiling-v2/target/debug`, 2026-11-11. No code changed.
Captures and crops: `media/2026-11-12/review-tiling-v2-r2/`. `s1-*` are
scale 1 at 1280x800 and `s2-*` are scale 2 (`2560x1600@2`). `crop-*` are
zoomed with nearest-neighbour. All pixel values below are logical px
unless marked "phys".

Judged against `design/looks/tiling-v2.md`, mock A
(`design/tiling/tiling-v2-mock-a-islands-gradient-ink.png`) and the
chora-tiling RC requirements in `design/product-briefs.md`. Item 7 (the
assistant's white "Chat ..." tab) is in another lane and is not rated
again. Core items 25 and 26 are listed but not rated.

## Verdict: FIX

Two MAJOR findings remain in tiling's area. Both break the RC line "lock,
power and screenshot keys and bar entries". 21 of the first review's 24
rated tiling items are fixed, 2 are partly fixed (16, 17) and none is
open. The look now matches mock A closely: wallpaper, gaps, borders,
terminal, launcher, key list, chip and shadows. What is left is mostly
spacing polish.

Counts: **MAJOR 2, MINOR 9, COSMETIC 5.**

## How the captures were made

- `CHORA_BIN=.../target/debug PERSONA_KEEP=<scratch> CHORA_TEST_FLAGS=--test-injected-press-is-real tests/persona/session.sh tiling 1280x800`,
  then the same with `2560x1600@2`. I used `act.py key/click/move/look`
  and ended each session with `session.sh --stop`. I killed one swaybg by
  its pid only.
- **session.sh now starts the styled bar** (first review, item 13):
  `waybar --config .../chora-tiling-waybar.json --style .../chora-tiling-waybar/style.css`
  through `waybar.sh`. swaybg runs with `-i flavors/tiling/wallpaper.png`.
- Paper: I wrote `--print-config` to `$XDG_CONFIG_HOME/chora/chora-tiling.yaml`
  with `theme: paper` and ran `choractl call tiling settings.reload`. The
  bar and the wallpaper did not change (finding 11). To see the Paper bar,
  I sent `SIGUSR2` to `waybar.sh` and restarted swaybg by hand. In
  `s1-15-paper-keys.png` the wallpaper is black because swaybg was down
  at that moment. That comes from my method, not from the product.
  `s1-17-paper-desktop.png` has the Paper wallpaper.

## First look (r2 captures from the builder, before reading notes)

- s1-01: the layout word "Tile" sits alone in a wide, empty end of the
  left island. "Keys" and the power glyph look dimmer than "No sound" and
  "Connected" beside them. The white chat tab shows through behind them
  (item 7, not rated).
- s1-04 and s2-04: the chip says "Tile left" while the bar still says
  "Tile". The bar lags behind.
- s1-05 and s2-05 ("power"): **no power menu is visible**. The capture
  shows only the desktop.
- s1-04: the yellow injection mark is in the top-left corner (core item 25).
- s1-02 and s2-02: in the key list, the right half of the panel is empty
  (a wide right margin). The keycap text sits low in its cap.
- s1-03: the launcher's selected row is barely different from the panel.
  The input row is an empty field with a lone caret.

PM checks: **the gap before "Tile" is confirmed** (40 px against 14 px on
the other side of the separator, finding 3). **"Keys" in low contrast is
confirmed as a perception, not as an AA failure.** It is `muted #9aa1b4`:
6.6:1 on the island, and 5.2:1 where the chat tab shows through
(`#2c2e38`). That passes 4.5:1, but it is the only door word in `muted`.
Mock A draws it in `text` (finding 4).

## MAJOR (2)

1. **The power menu and the screenshot keys do nothing.**
   - `Super+Shift+e` failed 3 times out of 3: twice at 1x (`s1-09-power-key-nothing.png`)
     and once at 2x (`s2-07-power-key-nothing.png`). The builder's own
     `tiling-v2-r2/s1-05-power.png` and `s2-05-power.png` show the same
     empty desktop.
   - `Super+Print` wrote no file. `~/Pictures/Screenshots` was not even
     created, so `screenshot.sh` never ran.
   - A click on the power icon opened the menu once at 1x
     (`s1-08-power-click.png`) and failed twice at 2x
     (`s2-05-power-click-nothing.png`).
   - `power.sh` started by hand with the session's environment shows the
     menu (`s2-06-power-via-shell.png`).
   - Cause, as far as I traced it:
     - `theme::launch_line` looks for `power.sh` and `screenshot.sh` in
       `config_dirs()`. Those are `~/.config/chora/tiling`, the runtime
       theme dir, and then `$CHORA_FLAVOR_DIR` or `/usr/share/chora/flavors/tiling`.
     - The chora-tiling process has no `CHORA_FLAVOR_DIR` (checked in
       `/proc/<pid>/environ`). The scripts exist only in the flavor dir.
     - When `find` fails, the line becomes `vec!["true"]`. The key runs
       `true` and nothing tells anyone.
     - foot and fuzzel still work because their `.ini` files are also
       written to the runtime theme dir.
     - The failed clicks at 2x go through the same `--launch power` path.
   - On an installed system with `/usr/share/chora/flavors/tiling`, the
     keys may work. They cannot be verified in the persona rig, and the
     silent `true` hides the fault anywhere the path is missing.
   - Fix:
     - Export `CHORA_FLAVOR_DIR` to the core and plugin in `session.sh`
       (and check that `chora-flavor` exports it to `chora-tiling.service`).
     - Have `apply_theme` copy `power.sh` and `screenshot.sh` into the
       runtime theme dir as it does the `.ini` files.
     - Replace the `true` fallback with an error on stderr and a notice
       ("Power menu not installed").
     - Add a gate: `Super+Shift+e` shows a layer surface within 2 s, and
       `Super+Print` writes a file.

2. **Screenshot has no mouse door.**
   - The design's affordance table says "Apps: Screenshot", and the RC
     asks for bar entries for lock, power and screenshot.
   - There is no `.desktop` entry for screenshot (none under `flavors/`
     or `dist/`) and no bar module. The launcher lists Foot, LibreOffice
     and ImageMagick (`s1-05-apps.png`). A mouse-only user cannot take a
     screenshot.
   - Lock is reachable by mouse through the power menu, when it opens.
   - Fix: ship `chora-screenshot.desktop` (Name=Screenshot,
     Exec=`chora-tiling --launch screenshot`) and
     `chora-screenshot-area.desktop` in the flavor, or add a fifth entry,
     "Screenshot", to the power menu.

## MINOR (9)

3. **The layout word floats in a 104 px slot.**
   - Separator at x 340. "Tile" ink at x 381-403. Island edge at 444.
   - 40 px of empty space sits on each side of the word, against 14 px
     between "9" (x 325) and the separator.
   - Mock A has 16 px after the separator and 13 px to the island end.
   - This is the side effect of fixing item 17 (`min-width` with
     centred text).
   - Crops: `crop-s1-left-island-tile-gap-x4.png` and
     `crop-s2-left-island-tile-gap-x2.png` (2x is the same: 80 / 80 phys).
   - Fix: `"align": 0` on `custom/layout` with `padding-left: 12px`, and a
     `min-width` that is only the longest word of the cycle. Or drop the
     `min-width` and accept the width change. Either way the word starts
     12 px after the separator.

4. **"Keys" is the only door word in `muted`.**
   - "Apps", "No sound" and "Connected" are `text #e6e8ef` (13.8:1).
   - "Keys" and its icon are `#9aa1b4`: 6.6:1, or 5.2:1 over the chat
     patch. The power glyph is `#8a90a1` at 2x, 4.2:1, which is fine for
     a graphic but dim.
   - The design names "Apps" and "Keys" together as "the way in", and
     mock A draws Keys in `text`.
   - Crops: `crop-s1-right-island-keys-x4.png` and `crop-s2-right-island-keys-x2.png`.
   - Fix: `#custom-keys { color: @text; }`. The icons can stay `muted`.

5. **Spacing inside the islands is uneven.**
   - Left island: "Apps" to the separator is 13 px (x 83 to 97), but the
     separator to the "1" pill is 5 px (97 to 103). "9" to the separator
     is 14 px.
   - Right island: the gaps between modules are 18 / 13 / 12 / 19 px
     ("No sound" to "Connected" 1049 to 1068, "Connected" to the
     separator 1136 to 1150, the separator to the keyboard icon 1150 to
     1163, "Keys" to power 1218 to 1238).
   - Mock A spaces items evenly, about 9 px each side of a separator.
   - Fix: separators `margin: 0 8px`, modules `padding: 0 8px`, and the
     tag row `margin-left: 0` so the pill is 8 px from the separator.

6. **The key list has a 90 px right margin against 23 px on the left.**
   - Panel x 200-1079. Content ink x 224-988 and y 104-694.
   - Padding: left 23, top 23, bottom 24, **right 90**.
   - Panel height is 640 (design 560). The content needs that height, so
     the height is accepted.
   - Crop: `crop-s1-keys-panel-right-padding.png`. Scale 2 is the same
     (`s2-02-keys.png`).
   - Fix: size the panel to its content (≈ 812 wide) and centre it. Or
     put column 2 at `x = 24 + (880 - 48) / 2` and give each column's
     label column the remaining width.

7. **Keycap text sits 1 px low, and descenders cross the cap's edge.**
   - In the "Super+k" cap, the box runs y 210-225 (16 tall) and the "S"
     runs 214-223: 4 px above and 2 px below.
   - The "p" descender reaches y 226, 1 px below the cap. "Super+drag"
     shows the same.
   - The cap text baseline (223) is 1 px below the label baseline (222).
   - Crops: `crop-s1-keycap-descender-x6.png` and `crop-s2-keycap-descender-x3.png`.
   - Fix: caps 18 tall with text centred on the cap's box including
     descenders (3 px above the cap height, 3 px below the descender), on
     a 22 or 24 px pitch. Put the label on the same baseline.

8. **The selected row in the launcher and power menu is nearly invisible.**
   - The selection `#242836` on panel `#1a1d27` is 1.15:1. WCAG 1.4.11
     asks for 3:1 for a state indicator.
   - The selected text keeps `text`, so nothing else marks the row.
   - Crop: `crop-s1-apps-selection-x2.png`. In Paper, `#f2f3f7` on white
     is the same problem (`s1-16-paper-apps.png`).
   - Fix: `selection-text=a78bfaff` (accent text, 6.2:1), or a selection
     background of accent at about 24 % (`#3a3260`), or both. Paper uses
     `#6d4aff`.

9. **The bar lags up to 1 s behind the chip and the tag switch.**
   - `s1-06-chip-mid-slide.png`: the chip says "Tile left" while the bar
     still says "Tile". The builder's `s2-04-chip.png` shows the same.
   - `s1-12-tag-switch-mid.png`: the pill is still on "1" while the tiles
     slide to tag 2.
   - Cause: `tag.sh` and `layout.sh` poll with `"interval": 1`.
   - Crop: `crop-s1-bar-lags-chip-x3.png`.
   - Fix: give the modules `"signal": N`, and have chora-tiling send
     `SIGRTMIN+N` to waybar on a tag or layout change. Or use one `exec`
     that follows a tiling event stream (`"restart-interval"`) instead of
     `interval`.

10. **The power menu looks like a search box.**
    - It is an empty input with a lone caret above "Lock / Log out /
      Restart / Shut down", with no title or prompt
      (`s1-08-power-click.png`).
    - The Apps launcher has the same caret with no prompt or hint
      (`s1-05-apps.png`).
    - Fix: `fuzzel --dmenu --prompt "Power  "` (or `--placeholder
      "Power"`) in `power.sh`. For Apps, `prompt="› "` or `placeholder=Type
      to find a program`.

11. **A theme reload does not reach the bar or the wallpaper in the persona
    session.**
    - After `theme: paper` and `settings.reload`, the files in
      `$XDG_RUNTIME_DIR/chora-tiling/theme/` are Paper. The focus border
      and new foot windows are Paper.
    - The bar and the wallpaper stay Ink (`s1-14-paper-after-reload-bar-still-ink.png`,
      `crop-s1-paper-reload-bar-ink.png`).
    - `apply_theme` only runs `systemctl --user reload/restart` on the
      units, and session.sh starts the companions without units.
    - Mixed result: after `SIGUSR2` by hand the Paper bar, chip, key list
      and tooltip are right, and every label passes 4.5:1. Measured:
      muted 6.0:1, text 16.4:1, on-accent pill 5.2:1, foot "warning"
      `#8a5a00` 5.9:1 (`s1-17-paper-desktop.png`).
    - Fix: when no unit answers, have chora-tiling send `SIGUSR2` to the
      `waybar.sh` it knows (pid file in the runtime dir) and restart
      swaybg itself. Or have `session.sh` run the companions as transient
      units with the same names.

## COSMETIC (5)

12. **Tooltips do not line up with the islands.**
    - The Apps tooltip spans x 6-232, 6 px left of the island edge (12).
    - The Keys tooltip ends at x 1269, 2 px past the island (1267).
    - Crops: `crop-s1-apps-tooltip-x4.png` and `crop-s1-keys-tooltip-x4.png`.
    - Corners and text are now right (item 20 fixed).
    - Fix: `tooltip { margin: 0 6px; }`, or accept GTK placement.

13. **Key names are spelled two ways.**
    - The tooltips say "Super+Space" and "Super+Shift+E".
    - The key list says "Super+space" and "Super+Shift+e".
    - Fix: one spelling from `chora-kit::words` in the tooltips too
      ("Super+Space" follows `decisions-taste.md`).

14. **The clock's month tooltip does not mark today.**
    - In `s2-08-clock-tooltip.png` and `crop-s2-clock-tooltip-x2.png`,
      the grid is mono and plain, and 11 is not highlighted.
    - Fix: `"calendar": {"format": {"today": "<span color='#a78bfa'><b>{}</b></span>"}}`.

15. **The launcher text is 18.7 px against 14 px in the bar.**
    - `fuzzel.ini` sets `Inter:size=14`, which fuzzel reads as points
      (log: "size=28.00pt/37.33px" at 2x).
    - Fix: `size=10.5` if 14 px is meant. Or keep 14 pt on purpose and
      note it in the design.

16. **The Paper key list is two-tone where it crosses a dark and a white
    window.**
    - The glass reads 250 grey over the white window and 236 over the
      dark one (`crop-s1-paper-keys-seam-x3.png`).
    - The blur makes the change smooth (item 22 fixed), so this is the
      glass working.
    - Fix: raise the Paper glass to about 94 % if the seam bothers anyone.

## The first review's items on this build

| # | Item | State | Evidence |
|---|---|---|---|
| 1 | key list rows overlap | fixed | no shared row y; groups whole (`s1-04-keys.png`) |
| 2 | wallpaper not Ink | fixed | (0,400) = `#101119`, glows top left and bottom right (`s1-01-empty-wallpaper.png`) |
| 3 | unfocused border invisible | fixed | `#33394b` on `#10111a`, 1.6:1, visible |
| 4 | gaps | fixed | outer 12 (x 0-11, 1268-1279, y 788-799), inner 10 (x 635-644), bar to tiles 12 (y 40-51), bar island and window border both at x 12; 2x is double |
| 5 | text at border | fixed | glyphs start at x 30, 16 px inside the border |
| 6 | foot unthemed | fixed | body `#161922`, JetBrains Mono, palette ANSI |
| 7 | chat tab | not rated (other lane) | still visible behind Keys and power |
| 8 | fuzzel unthemed | fixed | `#1a1d27`, Inter, radius, centred; selection see 8 |
| 9 | empty tags contrast | fixed | empty `muted` 6.5:1, occupied `text` (`s1-11-tag2-occupied.png`) |
| 10 | key list fonts | fixed | caps mono, labels Inter |
| 11 | keycap padding | fixed | 7 / 7 px in every cap; one label x per column (410, 834) |
| 12 | chip off centre | fixed | box 580-699 x 648-703, ink 24 / 24 px, 18 / 19 px |
| 13 | session.sh unstyled bar | fixed | waybar.sh with `--style` |
| 14 | duplicated heading | fixed | no repeat; 8 px above each title |
| 15 | key words | fixed | "Super+drag", "Super+/" |
| 16 | panel height and empty foot | partly | foot 24 px, top 23 px; now 90 px empty on the right (6) |
| 17 | left island jumps | partly | width fixed at 12-445, but the word floats (3) |
| 18 | fallback-font icons | fixed | drawn SVG; grid and power y 17-30, keyboard 18-29, centred on 23.5 |
| 19 | separators full height | fixed | 1 x 16 px (y 16-31); one in the right island |
| 20 | tooltip corners and text | fixed | rounded, "Every key (Super+/)"; placement see 12 |
| 21 | no shadows or motion | fixed | shadow under tiles (y 788 `#090b0e` against the wallpaper); slides seen mid-frame on layout and tag change |
| 22 | blur had no effect | fixed | smooth 250 to 236 ramp over about 30 px under the panel |
| 23 | tag digit 1 px off | fixed | 0.5 px (digit centre 140.0 in pill centre 140.5), y centred |
| 24 | caps 18 on 20 | changed | 16 on 20, 4 px between caps; see 7 |
| 25 | yellow inject mark (core) | not rated | still under the island (`s1-06-chip-mid-slide.png` x 63-74, y 40-43) |
| 26 | long sound word (core) | fixed | "No sound" |

## Against the design and the RC

- Mock A: islands, height 32 (y 8-39), radius 10, island hairline
  `#3a4054`, clock centred (563-715, centre 639), tag pill 24 x 24, the
  gradient border `#a78cfa` to `#5fe9d4`, terminal and wallpaper all
  match. The differences are 3, 4 and 5.
- RC requirements:
  - Tag indicator: yes.
  - Focus mark: yes.
  - Key overlay on `Super+/`: yes.
  - Launcher by key and by bar: yes.
  - **Lock, power and screenshot keys and bar entries: no** (1, 2).
  - Unbound Super swallowed: not re-tested in this round.
  - Animations off with one setting: `look.animations` exists, not
    toggled here.
  - Visual parameters in one file: the `look:` section in
    `chora-tiling.yaml`.
