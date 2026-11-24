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

## r1

Lane: cloud, claude/ledger-audit at ce866b0d. Step 0 green (unshallow, fetch, `cargo check --workspace`, `cargo build --workspace`). No product change, no test written. Each guard run once on ce866b0d, foreground, `pgrep -fa 'chora --headless'` empty after. Ancestor = `git merge-base --is-ancestor <sha> ce866b0d`.

| id | fix sha | in head | guard | line | proposed state |
|---|---|---|---|---|---|
| co-48 | 866382dd (red 9b0466ae); report: contract fix, not product (C3/T1 expectation in persona check.py) | yes | tests/persona/check.py C3/T1 via persona-selftest; tests/gate-construct-focus.sh | persona-selftest rc 0 "ok: persona session"; gate-construct-focus rc 0 "ok: the keys stay with the front face after its focused window closed" | fixed 866382dd (contract fix; guard persona check.py C3/T1, ledger-audit r1). Super+Return on a face is a UX item (co-55), not this row |
| co-49 | 866382dd (same cause as co-48) | yes | check.py T3 (persona-selftest) | persona-selftest rc 0 | fixed 866382dd (contract fix; guard check.py T3, ledger-audit r1) |
| co-50 | 866382dd, 8a599917, 3a625261 (xp hears the turn of its host) | yes | tests/gate-construct-focus.sh "a turn takes the keys from the window of a face that does not show" | rc 0 "ok: a turn takes the keys from the window of a face that does not show (core: xp-icons xp: xp - construct: {'holder': None})" | fixed 866382dd (guard gate-construct-focus turn cell, ledger-audit r1) |
| co-51 | 866382dd (window close path); co-reach r1: harness NIT for the I2 row | yes | gate-construct-focus "the keys stay with the front face after its focused window closed" | rc 0 ok line as above | fixed 866382dd for the key half (guard above); the I2/I3 demo rows are "not product, harness NIT" (reports/co-reach.md r1), not reproduced 0/5 in co-focus r1: no open product bug. Note rcf-3/rcf-8 (xp icons on faces 1..3) are separate rows |
| co-52 | d9b2c4f1 (red 1b3abdfc, gates fe369799); b2fa7792 for the way out | yes | tests/gate-construct-cube.sh "stood back with a pane open ..." | rc 0 "ok: stood back with a pane open, the pane goes beside the cube and shows, its label hides, the face label stays under the cube" | fixed d9b2c4f1 (guard gate-construct-cube, ledger-audit r1) |
| co-54 | d9b2c4f1 / fe369799 | yes | tests/gate-construct-placed.sh "a pane nobody placed goes beside the cube that stands back and stays drawn"; gate-construct-cube same cell | rc 0 both | fixed d9b2c4f1 (guard gate-construct-placed, ledger-audit r1) |
| mc-45 | 17d9a798 (guard named in the row) | yes | tests/gate-mc-paths.sh chat-focus | rc 0 "ok: chat-focus", "ok: every path of the mc gate" | "not a bug" holds: deciding line is the row text (nurse 011-013: chat open at the run's end); guard chat-focus passes. State: closed, not a bug, guard 17d9a798 gate-mc-paths chat-focus (ledger-audit r1) |
| ti-20 | none (no fix; row says verified 5/5 465-531 ms on 1c599c74) | yes (1c599c74) | tests/gate-launcher-click.sh; tests/gate-tiling-apps-click.sh | launcher-click rc 0 "ok: tiling: a click by act.py on the Mid row of the launcher starts mid..."; apps-click rc 0 "ok: a click on a row of the Apps list starts the program of that row" | closed (not reproduced; guards gate-launcher-click + gate-tiling-apps-click green, ledger-audit r1). Timing of the click to launcher (<1.5 s) rests on verify-maj r3, not re-measured here |
| ti-32 | e86b3cfa (red 6f300250); keys-route e16d558e | yes | tests/persona-selftest.sh "tiling+xp: Super+Return opens a tile of tiling, the first look" | rc 0 "ok: tiling+xp: Super+Return opens a tile of tiling, the first look" | "setup, not product" holds; fixed e86b3cfa (guard persona-selftest tiling+xp, ledger-audit r1). The product half is ti-38 |
| ti-38 | 4441f1d4 (red 27b5476d, ac366c40; merge a1f3c8d7). The row names d613b525 "merges in the next train": d613b525 is NOT an ancestor of ce866b0d, but the fix commits 4441f1d4 and a1f3c8d7 are | yes (4441f1d4, a1f3c8d7); d613b525 no (a review note commit, not code) | tests/gate-join-keys.sh both directions | rc 0 "ok: gate-join-keys: a joining look leaves the keyboard with the look that holds it (7s)"; "ok: xp-first: tiling joined and left the keyboard with xp; Super+Return opened a window of xp" | fixed 4441f1d4 (guard gate-join-keys, ledger-audit r1) |
| ti-47 | none | n/a | none (cell proposed in the row: waybar count = 1) | not run (state only) | open: no guard (MAJOR stays open; TILING-REACH r1 runs it) |
| ti-48 | none | n/a | none | not run (state only) | open: no guard (MAJOR stays open; TILING-REACH r1 runs it) |
| sh-23 | 60314da1 (red 938f9382, green ccf7ebab); key half "not red" on a5b77488 | yes (all four) | tests/gate-tiling-empty-output.sh (render half), tests/gate-start-tap.sh cell return (key half) | tiling-empty-output rc 0 "ok: a bare chora-tiling shows the bar within 5 s and its start door starts the launcher"; start-tap rc 0 "ok: gate-start-tap: tap cube drag return give pass" | fixed 60314da1 (guards gate-tiling-empty-output, gate-start-tap return, ledger-audit r1) |
| sh-38 | none | n/a | none (the leak step RSS cell itself is the flake) | not run (state only; flake-series runs it) | open: no guard (MAJOR stays open) |
| co-67 (asked extra) | none (co-reach r4 pending) | n/a | none | not run (state only) | open: verify, no guard (MAJOR candidate stays open) |

Also checked: sd-h1 (shared-doors.md:30) names the same fix as sh-23, r2 60314da1, guard gate-tiling-empty-output.sh rc 0 above: proposed fixed 60314da1 (ledger-audit r1).

Count of open BLOCK/MAJOR(+candidate) among the 15 rows listed: before 15 (co-67 and the 14 of the brief, counting mc-45/ti-20/ti-32/sh-23/ti-38 whose state text is not "fixed"), after 4 open: ti-47, ti-48, sh-38, co-67. 11 proposed fixed or closed: co-48, co-49, co-50, co-51, co-52, co-54, mc-45, ti-20, ti-32, ti-38, sh-23 (plus sd-h1). Other ledger rows were not audited.

## Seen
- ti-38 row cites d613b525 as the review commit "merges in the next train"; the fix commits are in the head, so the row text should say merged (t-number not checked).
- tests/gate-tiling-empty-output.sh prints "waits for /tmp/chora-gate.lock" mid-run (a gate lock wait, harmless alone).
- co-51 row's rcf-3/rcf-8 follow-ups (xp icons per face) belong to their own rows; not checked here.
