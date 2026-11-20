
## r1
Branch claude/xp-minor, head 22884a79 (base fee9c695). Gate (pre-commit, ci-check, build, test, clippy, xp, xp-faces, xp-question, xp-notepad, doors, acts): all PASS. Captures 1280x800 in media/2026-11-20-xp-minor/.

| # | item | red | fix | cell | capture |
|---|------|-----|-----|------|---------|
| 1 | xp-36 rv-1 Run... warning then Escape quiet; exit 1 = Closed, 2+ = Failed | d14a8c84 | 908b26d8 | kit test a_warning_then_escape_stays_silent (the fail cell now exits 2) | none: no fuzzel box opens on fuzzel 1.9.2 here |
| 2 | Screenshot balloon | c9643e03 | a4010f03 | tests/screenshot-notice.sh (a notify-send that answers must not hide the balloon) | screenshot-balloon.png: XP balloon "Screenshot saved" + file, above the tray |
| 3 | Volume popup level slider | cdcf9569 | 9c6d97a8 | xp tests the_volume_popup_checks_the_level_or_mute, a_click_on_the_slider_sets_the_level_of_its_place | volume.png (40 %, thumb at 40 %) |
| 4 | Super+E notice in plain words | bb35e050 | 22884a79 | xp test the_notice_without_a_file_manager_says_what_to_do | none: Super+E not driven |
| 5 | small yellow square | none | none | none | injection-mark.png |

Notes:
- 2: the notice was made, but the persona session's notify-send is a stand-in that answers, so no balloon showed. screenshot.sh now tries choractl notify.show first (expire 6 s), notify-send second; chora-notify places it as xp does (above the tray, tail to it). Not run with the real Start click (no grim in the act session); gate-tiling-power and gate-tiling-print-notice (fake notify-send, notify-send path) not run here, they are not in this gate: check them in the train.
- 3: the slider is the last row (after Mute), so the act row ids stay; a click on the track sets the level by x. XP has it on top: one move if wanted.
- 5: the yellow square is the injection mark of the core (D23, crates/chora/src/inject.rs): shown 500 ms after every input.inject, only for agent-driven input, never for a person's mouse. Not xp, not the balloon anchor; the core is not mine, left as is.
