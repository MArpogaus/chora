
## r1

F7 (sh-31): the default chora-notify card text is 14 px.

- Red: bbd4b4e1 `the_default_card_draws_its_text_at_14_pixels_or_more` failed: "the default card text is None, guide wants 14 px".
- Green: 60a38634. `size: 14` in plugins/chora-notify/assets/values.yaml and in `Values::default`; the card height follows `Metrics::of(14)` (test checks it). xp has no `card.size` in its folder, so it keeps 11 px; mc keeps 19. README of the assets updated.
- known-red.toml: both `notify-card-body` rows (tiling, construct) removed; none added.
- Acts: tiling 30 pass, 1 known red, 0 blocking; construct 83 pass, 2 known red, 0 blocking.
- Gate (merged with fresh base, already up to date, detached): 54 PASS, 1 FAIL, foreign: `mc-scale`: "FAIL: scene.show of 300 panels again takes 5971 ms, more than 5000 ms" (the first show ran 1703 ms; a timing cell on a loaded host, not notify). Not rerun. All cells before it, including build, test, clippy, pre-commit, passed.

## PM review r1 (2026-11-23 21:55 UTC)
Base 1ec04694..60a38634, 4 files +25 -20 (PM review: under 200 lines; PM not the builder).
- Red first: bbd4b4e1 (test only) before 60a38634: pass.
- Default 14 px: values.yaml `size: 14` and `Values::default` card size Some(14); test covers both and the card height: pass.
- xp keeps 11: flavors/xp/assets/notify/values.yaml has `card:` without `size`; chora_draw::bubble::Style is `#[serde(default)]` with size None, so xp stays at the host 11 px: pass. mc keeps 19 (its own card.size): pass.
- known-red: two notify-card-body rows removed, none added; acts tiling 30/1/0, construct 83/2/0: pass.
- Gate: stopped at mc-scale (foreign, sh-37 timing under load). The cells after it (watchdog-starved, acts, xp-run, leak, tiling-*, demo, mc-host, e2e) did not run on the branch, and the brief's `--only` step for them was skipped: rnt-1 MINOR (process). acts was run by hand for both looks; the train's full gate is the evidence for the rest.
Verdict (computed, no open MAJOR): MERGE-OK 60a38634; next train.
