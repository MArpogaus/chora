# Review: tiling v2 "Ink", round 3

Independent final check of `claude/tiling-v2` at 92b9224, built in
`/home/user/wt/tiling-v2/target/debug`, 2026-11-11. No code changed.
Captures: `media/2026-11-12/review-tiling-v2-r3/` (`s1-session/` holds
the raw shots and `actions.jsonl`).

## Verdict: MERGE

Both MAJOR items of round 2 are fixed. Nothing on a main path is broken.
What is left is MINOR or COSMETIC and can follow later.

## The round 2 MAJOR items, re-checked on a fresh session

The session was started with `CHORA_BIN=.../target/debug PERSONA_KEEP=<scratch>
CHORA_TEST_FLAGS=--test-injected-press-is-real tests/persona/session.sh tiling 1280x800`,
driven with `act.py` and ended with `session.sh --stop`.

1. **Power menu: fixed.**
   - `Super+Shift+e` opened the menu 3 times out of 3 (`s1-01-power-key.png`).
   - A click on the power icon opened it 3 times out of 3 (`s1-02-power-click.png`).
   - Esc closed it each time.
   - The menu now has a "Power" prompt.
2. **Screenshot: fixed.**
   - `Super+Print` wrote `~/Pictures/Screenshots/2026-11-11_21-00-00.png`,
     a full 1280x800 capture (`s1-07-super-print-output.png`).
   - The rig has no `notify-send` and no notification daemon, so no
     notice can show here (`s1-03-print-no-daemon.png`).
   - With a stub `notify-send` on PATH, `screenshot.sh` calls
     `notify-send -a Screenshot "Screenshot saved" <file>`.
   - The builder's captures (`tiling-v2-r3/s1-03`, `s2-03`) show the
     notice with a daemon running.
   - A click on Apps lists "Screenshot" first, and "Screenshot of an
     area" appears when you type (`s1-04-apps-by-click.png`). Enter on it
     writes a file.
   - Clicking the row does nothing in the rig (`s1-05-apps-click-no-launch.png`):
     no hover, no launch. The rig's fuzzel is 1.9.2 and does not react to
     the pointer, so this affects every row and every fuzzel menu. It is
     not a bug in the tiling code.

## Round 2 MINOR fixes: spot-check (3 of 3 fixed)

- **Bar spacing (5): fixed.**
  - Left island: "Apps" to separator 14 px. Separator to "Tile" 13 px.
  - Right island: 14 / 14 / 13 / 15 px between the items. The island edge
    is 12 px from the power glyph.
  - Crops: `crop-s1-left-island-x4.png` and `crop-s1-right-island-x4.png`.
- **Key list margins (6): fixed.**
  - Panel x 233-1045, ink x 257-1020: 24 px left and 25 px right
    (`s1-06-keys.png`).
- **Launcher selection contrast (8): fixed.**
  - The selected row is accent `#a78bfa` on panel `#1a1d27`: 6.2:1.
  - Its text is `#11131a` on accent: 6.8:1.
  - Paper uses `#6d4aff` with white text.

## First look at the builder's r3 captures (s1-*, s2-*)

- MINOR: the screenshot notice in Ink is a white box with black text.
  - It does not match the look. The flavor ships no mako style, so on a
    real system it will be mako's default colours instead.
  - Fix: ship a `mako` config in the theme dir with the Ink and Paper
    colours, Inter, and radius 10.
- MINOR: `notify-send` (libnotify) is not in the PKGBUILD optdepends.
  - mako does not pull it in, and `screenshot.sh` drops the notice
    silently when it is missing.
- MINOR: fuzzel needs a version that handles the pointer for the Apps and
  power menus to work by mouse alone.
  - Fix: state the minimum fuzzel version in the optdepends and the docs.
- COSMETIC: the left island has about 68 px of empty space after "Tile"
  (ink ends at x 389, island at 457).
  - The space is reserved for the longest layout word. It reads as
    unused space on the default layout.
- COSMETIC: in the key list, "Super+r / Super+Space" is one keycap. Two
  caps would match the other rows.
- COSMETIC, after a switch to Paper: foot windows that were already open
  stay dark (`s1-06`..`s1-09` Paper). New windows are Paper.
- Not rated (other lane): the white "Chat" tab still shows behind Keys
  and the power icon.
