# Review: claude/swarm5-fixes (4ca41a98) against claude/autonomous-dev-session-3w-i33pg7

Verdict: FIX (small; one packaging gap, one wrong notice). Items 1, 2, 4, 5 are sound.

## Part A, code
HIGH
- dist/arch/PKGBUILD (package(), lines 78-80) installs only flavors/tiling/*.service, *.target and waybar/*.
  It never installs look.sh, power.sh, screenshot.sh, wallpaper.sh, foot.ini, fuzzel.ini, swaylock.conf,
  the wallpapers or applications/*.desktop. On Arch the three look entries do not exist, and `--launch look`
  and `--launch power` fail "not installed". The builder's suspicion is confirmed; the gap is older than this
  branch but this branch adds a feature that needs it. Fix: install -Dm644 flavors/tiling/*.sh *.ini *.conf
  *.png -t .../flavors/tiling/ and flavors/tiling/applications/* -t .../flavors/tiling/applications/.
- crates/chora/Cargo.toml (RPM assets, lines 82-83): `flavors/tiling/*` plus `waybar/*`. No entry lists
  `applications/`, so the .desktop files (old Screenshot ones too) are probably not in the RPM; `*` matches
  files directly under tiling/ and does not recurse. Add `../../flavors/tiling/applications/*` (not built here,
  so check the rpm with rpm -qpl). No debian packaging exists in the repo.
MEDIUM
- plugins/chora-tiling/src/theme.rs LaunchError::notice(): `look` falls into the `(_, _)` arm and shows
  "Screenshot not installed / screenshot.sh not found" (EN and DE). Add a `look` arm. The test only checks
  that the error names look.sh, not the notice, so it misses this.
- look.sh: no injection risk (theme comes from a fixed case, JSON is built from that). Dark look while
  chora-tiling is not running fails with a notice only if notify-send exists; otherwise silent exit 1. Acceptable.
- Mouse line (keys.rs header): fixed x 24/84 and one unwrapped line at 14 px. The gate checks only the
  English right edge against the panel. The German line ("Ein/Aus-Symbol: Sitzung") is longer and has no
  fit test. Check whether the panel width counts the line; if not it can clip or overrun in DE.
LOW
- tests/gate-tiling-programs.sh: the fuzzel 1.11 check only greps three files for text; it will pass with
  any wording that holds "fuzzel" and "1.11" on a line (e.g. the old "1.11" in an unrelated line). A
  documentation grep, not a behaviour test. Harmless, but do not count it as coverage.
- mc: prev/next share page_line(), clean (rem_euclid wrap). prev is shown on the first page too (wraps to
  the last page); consistent with next, fine. head_hit is rebuilt in align_head, one more place that
  must stay in step with the title; covered by the Rust test and the click gate.
- docs/configuration.md: the fuzzel sentence is long and sits in the middle of a bullet; split it.
- The power-icon words in the Mouse line mention only "lock, log out", though the menu also has Restart
  and Shut down.

## Part B, pixels (1280x800)
- mc-footer.png: left "← 1  Agenda" at (160-246, 772) mirrors "→ 3  Agents" at right (751-831, 772),
  same size, colour and baseline, aligned to the text column edge x=160. No flaw on the main path.
  MINOR: the arrows look a size small beside the numbers at (160-172, 772) and (751-763, 772); font fallback.
- tiling-keys.png (Ink): Mouse line at y=118, label x=257 and text x=317, readable, 24 px margins kept,
  nothing overlaps the Windows/Layouts headings (y=152). MINOR: the hint "Super+/ shows this list..."
  now floats at (353-637, 88) with a 48 px gap after "Keys", a baseline about 2 px below the title;
  the gap is odd against the left-aligned Mouse line below. MINOR: label "Mouse" (semi-bold muted) and the
  text (regular, full ink) are only weakly separated; "·" separators are fine.
- tiling-light-look.png (Paper): same layout, contrast fine (panel #ebebeb on black). MINOR: this shot shows
  the key list in the light look, not the Apps list with the three look entries, so item 3 has no
  visual proof here; a shot of fuzzel with "Light look / Dark look / System look" is missing.
- No MAJOR pixel flaw found. Design documents mc-v2.md and tiling-v2.md were not re-read line by line;
  the footer follows the existing next-line pattern.

## Summary
Fix the PKGBUILD (and check the RPM assets) before merge, add the `look` notice arm and a DE width test.
