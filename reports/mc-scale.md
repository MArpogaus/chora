# mc-scale

## r1

Branch claude/mc-scale. Commits: red 4380c958, green 75623a87, acts known-red follow-up ac538e82. Base 6b01c976 (origin session head was already merged, "Already up to date").

### Step 1: timings before (ms, order: scene.save, scene.show, settings.set theme, small show, show again, settings.set write)
Deviation: loops ran `merge-gate.sh --only mc-scale --jobs 2` (16 s) not `ci-check,mc-scale` (84 s each); ci-check does not touch the cell. One ci-check,mc-scale run first: mc-scale PASS, write 3720 ms.

Idle, 5 runs: write 3270, 3419, 3523, 3095, 2850 (all PASS).
```
2161 1344 1876 404 1370 3270
2281 1305 1909 451 1284 3419
2115 1297 2158 395 1283 3523
2256 1334 2069 454 1306 3095
2282 1300 1998 419 1283 2850
```
Under `cargo build --release` (other target dir), 5 runs: write 3451, 4102, 3689, 3502, 3664 (all PASS, under 5000 ms by 900 ms at the worst).
```
2717 1535 2157 399 1397 3451
2438 1447 2417 576 1836 4102
2464 1453 3055 525 1461 3689
2913 1456 2044 452 1556 3502
2311 1500 2146 412 1667 3664
```
Note: the load build had finished part-way; the six-step cell was loaded for the whole 5.

### Step 2: the split (Instant logs, dev build, later removed)
settings.set `{"write":{"duration_ms":300}}` beside 300 panels, in `Mc::settings_changed` (plugins/chora-mc/src/main.rs, ~line 844):

| part | ms |
|---|---|
| grabs (grab_all, end_grabs) | 0 |
| `scene_commands` of 300 panels (layout of text, build.rs build()) | 500-520 |
| `apply_scenes` (one transaction, ~1300 commands, core) | 1210-1430 |
| `align` (query and place) | 440-750 |
| render | 8-22 |
| total | 2.5-3.2 s plus IPC |

Cause: `shape_of()` (build.rs ~185) removed only `theme`, so any other change, including `write.*`, fell to `self.build()` (unbuild + create all nodes). `write.enabled` and `write.duration_ms` are read only by `Mc::write` (show.rs:15-62) when text writes; no node that stands depends on them. The cost is the core transaction, as the cell header says, plus mc's own scene_commands and align.

### Step 3: red then green
- Red 4380c958: `tests::a_write_setting_that_changes_no_look_rebuilds_nothing` (main.rs): `scene.query` count 18 vs 6 expected after write.duration_ms and write.enabled changes (each ran a build, whose `align` queries). Failed on the cost, no sleep, no bar raised.
- Green 75623a87: `shape_of` leaves out `write` too; new `themed` field keeps the theme of the nodes that stand; `settings_changed` returns after the grabs when the shape and the theme are unchanged (`no_look`); a theme change still recolors (falls to build when refused).
- Follow-up ac538e82: acts cell mc I10 (known-red, "a word of the builders") named captures with the cross-fade set of the regions (`mc-r{set}-...`). A settings write that changes nothing no longer recolors at startup, so the set parity flipped: mc-r1-footer-next -> mc-r0-footer-next and mc-r0-header-head -> mc-r1-header-head in tests/acts/known-red.toml. The red is the same finding, only the node name moved. Checked: same cell on base code (fix reverted in the tree) known red with the old names; on the fix it was blocking until renamed; after: 41 pass, 2 known red, 0 blocking.

### Step 4: after, 10 runs under load (write column)
```
3152 1601 2488 416 1959 559
2809 1627 2328 536 1893 796
2880 1549 2078 396 1511 398
2410 1437 2153 500 1524 384
2425 1545 2021 345 1602 382
2855 1606 2097 507 1475 364
2546 1797 2819 516 1424 388
3833 1438 2231 592 1672 681
3699 2376 3709 572 1605 412
2592 2178 2462 448 1579 406
```
settings.set of the write time: 364-796 ms, 10/10 under 2500 ms, 0 red (the load build was running through all 10).
The other cells (scene.save 2.4-3.8 s, theme settings.set 2.0-3.7 s) are not the subject of this brief and stay under the 5000 ms bar; the theme change is unchanged (one recolor).

### Gate
Full `tests/merge-gate.sh --jobs 2` (no --only) on claude/mc-scale ac538e82 (merged with origin/claude/autonomous-dev-session-3w-i33pg7, already up to date): 57 steps PASS, EXIT 0, no SKIP, no foreign red.
A first full run on 75623a87 failed at `acts` only: mc I10 blocking (capture rename above; my cause, fixed in ac538e82). Every other step of that run had passed up to it.

## PM review r1 (2026-11-22 19:40 UTC; PM not the builder; 43 lines)
Range 6b01c976..ac538e82 (+43/-6): red 4380c958, fix 75623a87, known-red rename ac538e82.
- Cause checked: shape_of() kept `write`, so settings.set of write.* took the build path (300 panels, about 3 s). Now `write` leaves the shape, and `themed` holds the theme of the nodes that stand: same shape and same theme = no build and no recolor (main.rs:863-876). Theme change still recolors; recolor refused still builds, and build() sets `themed` (build.rs:158). OK.
- Unit test a_write_setting_that_changes_no_look_rebuilds_nothing: duration and enabled change, query count unchanged. OK, red first (4380c958).
- known-red.toml I10: capture names r0/r1 swap, because the acts no longer rebuild (region numbers stay from the first build). Still red, same match, same finding (mc-55). Accepted; LOW: the swap rests on build order, mc-55 should key by text, not region index.
- Lane: 10/10 settings.set under load 364-796 ms; full gate 57 PASS on ac538e82.
Findings: 0 MAJOR, 1 LOW (above). Verdict: MERGE-OK. Joins t110.
