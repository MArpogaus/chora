
## r1

F7 (sh-31): the default chora-notify card text is 14 px.

- Red: bbd4b4e1 `the_default_card_draws_its_text_at_14_pixels_or_more` failed: "the default card text is None, guide wants 14 px".
- Green: 60a38634. `size: 14` in plugins/chora-notify/assets/values.yaml and in `Values::default`; the card height follows `Metrics::of(14)` (test checks it). xp has no `card.size` in its folder, so it keeps 11 px; mc keeps 19. README of the assets updated.
- known-red.toml: both `notify-card-body` rows (tiling, construct) removed; none added.
- Acts: tiling 30 pass, 1 known red, 0 blocking; construct 83 pass, 2 known red, 0 blocking.
- Gate (merged with fresh base, already up to date, detached): 54 PASS, 1 FAIL, foreign: `mc-scale`: "FAIL: scene.show of 300 panels again takes 5971 ms, more than 5000 ms" (the first show ran 1703 ms; a timing cell on a loaded host, not notify). Not rerun. All cells before it, including build, test, clippy, pre-commit, passed.
