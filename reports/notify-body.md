
## r1

Result: no change needed; F7 / sh-31 is already fixed on the session head ce866b0d.

- Cause: 675ebd55 moved the notices to chora-notify; `plugins/chora-notify/assets/values.yaml` had no `card.size`, so the plain card drew the small 11 px text of the host (`card.rs` `SMALL = 11`, `Metrics::of(None)`); xp gets 11 px the same way, its `flavors/xp/assets/notify/values.yaml` names no size.
- Red: bbd4b4e1 `test(notify): the default card text is 14 px or more (red)`. Green: 60a38634 `fix(notify): the default card text is 14 px, xp keeps 11` (`values.yaml` `size: 14`, `card.rs` Default `size: Some(14)`); both are ancestors of ce866b0d.
- Now: `cargo test -p chora-notify`: `the_default_card_draws_its_text_at_14_pixels_or_more ... ok`, `the_xp_folder_loads ... ok`, 28 passed.
- `run.py --check tiling`: `chapter tiling: 28 acts, 28 green, 0 red`; no I6/sh-31 line on t21, t28. Full `--check`: `tour: check: 19 red, 0 broken` (not 14; not from notify; the findings row sh-31 can be closed).
- Gate: not run, no code changed. No `chora --headless` left.
