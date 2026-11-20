# mc-minor

## r1

Branch claude/mc-minor from 929de144, merged claude/flakes-l (cc1282db) and claude/autonomous-dev-session-3w-i33pg7 without conflicts (the commit-msg hook needed `chore:` subjects). Step 0: `git fetch --unshallow`, `cargo check --workspace` green. Pushed f7735e43. Each fix has a red commit before it.

| item | red | fix | test / cell |
|---|---|---|---|
| 1 pr2-1 + rv-2: one fade tracker | 5543d938 | 6f686942 | mc unit `a_theme_change_keeps_mc_busy_until_its_fades_end`; `mc_is_busy_while_its_regions_cross_fade` adapted to `fade:` |
| 2 rv-1: Remove clears only real children | 2785fadf | 1b2341ef | mc unit `removing_a_scene_keeps_the_fade_of_a_sibling_that_shares_its_prefix` (scenes `foot`, `foot-server`; panels `today`, `today-x`) |
| 3 rv-3: T1/T4 key list only if asked for | 51b595f8 | f7735e43 | test_acts `test_t1_a_key_list_that_opened_unasked_stays_red`, `test_t4_...` |
| 4 rv-4, rv-5 NIT | not done (backlog) | | |

Item 1: `Mc::try_apply` (build.rs) applies, and notes the writes and fades once the core accepts the transaction; the theme recolor, `apply_scenes` and the window placement use it. Region cross-fade already went through `apply` (the page turn via show), so it was tracked by `fading`; the second tracker `fading_until`/busy "fade" and `FADE_SLACK` are gone (the core ends a fade with `animation.done`, FADE_GRACE stays as the lapse of the one tracker). `note_writes` now takes colour fades too. The first render of a page also leaves `fade:` entries until their end (old test acked them).
Item 2: `is_part_of` in lifecycle.rs: below `mc-s-<name>` only the node, `-panels`, `-page`, `-wheel`, `-rule`, `-panels-*` and `-chapter-*`; above (root `mc`, region sets) every id below the prefix.
Item 3: T1/T4 allow the key list only when the persona pressed a chord that opens it (`LIST_CHORDS`: xp h/F1, tiling and mc slash, construct slash/F1; `Ctx.asked_for_list`, `Ctx.own_left`). `ctx.opened` was not usable (it is true for an unasked list too). Existing "may stay open" tests now pass the chord.

Gate (--jobs 1): pre-commit, gate-lib, ci-check, build, test, clippy, acts, mc-paths, mc-sofa, mc-column, mc-handover: all PASS. `bash tests/persona-selftest.sh` ok (not a merge-gate step name). cargo test -p chora-mc 183 ok; tests/test_acts.py 156 passed.
