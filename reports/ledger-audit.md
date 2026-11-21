# Ledger audit

## r1
Base 598a34ff (11-21). Step 0: cargo check and build green. Cells run once: `tests/merge-gate.sh --jobs 1 --only keys,pane-keys,launcher-click,doors,xp` all PASS (run /tmp/merge-gate.VudJFC), plus tests/gate-tiling-apps-click.sh, tests/gate-xp-run.sh, `cargo test -p chora-kit --lib` (298 pass) and the input unit test. In scope: rows that were open, in flight, fix pending or fixed on a lane branch. Skipped, owned by a running lane: sh-23, sd-h1, sd-h2, ca-4, co-38, co-40, co-41, FLK-*.

- sh-10 | MAJOR | fixed on claude/ux-r3, not merged -> fixed 42d5d4b2 merged 0e303d60 | cell doors (helper-none x4 pass) | green on 598a34ff
- sh-24 | MAJOR | open, in flight (stray-grab) -> fixed cc940fec, 363276e8, 4cf1925f merged a5184639 | cells keys, pane-keys, unit an_injected_key_follows_the_keyboard_place_as_a_real_one | green on 598a34ff
- sh-29 | BLOCK | open, in flight (launcher-click) -> fixed 741d6bca merged 88258c87 | cell launcher-click | green on 598a34ff; harness fault, product was fine
- ti-1 | BLOCK | fix pending, not chora -> fixed (PKGBUILD fuzzel>=1.11, 5dd34fcd; container fuzzel 1.11.1) | gate-tiling-apps-click.sh, cell launcher-click | green on 598a34ff
- xp-36 | MAJOR | fixed on lane branch, merge next train -> fixed 3290f1d4 merged d81c8293 | gate-xp-run.sh, chora-kit launcher unit tests | green on 598a34ff; the failed-start notice is covered by unit tests only (fuzzel 1.11.1 opens the Run box, so no live failure)

Counts (open BLOCK / open MAJOR):
- before: shared 1 / 3 (sh-29; sh-10, sh-23, sh-24), tiling 1 / 0 (ti-1), xp 0 / 1 (xp-36), sd 0 / 2 (sd-h1, sd-h2), mc 0 / 0, construct 0 / 0; total 2 / 6
- after: shared 0 / 1 (sh-23), tiling 0 / 0, xp 0 / 0, sd 0 / 2 (sd-h1, sd-h2), mc 0 / 0, construct 0 / 0; total 0 / 3, all owned by running lanes
- open with no cell: none
