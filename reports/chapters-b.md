# chapters-b

## r1 (lane 5 of design/tour-v9.md section 8, 2026-11-21)

Branch `claude/chapters-b` on f4f49783. Red `2a54ec55` (tests only), green `a76476be` (pushed). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` ok.

**Red first.** `tests/test_tour.py` `ChaptersB`: `test_every_row_has_an_expect` (AST of the `t.act` calls of `chapters/mc.py` and `construct.py`: a row without `expect` fails, so the `inv` count of these chapters is 0), `test_rows_name_their_contract`, and one planted wrong state per chapter (`test_mc_expects_turn_red_on_a_planted_state`: wrong page, other HUD line, no notice, no clock, chat not shown; `test_construct_expects_turn_red_on_a_planted_state`: cube on face 0 where face 1 is expected, mode `use` expected on a stood-back cube, s1 still a pane). At `2a54ec55`: 2 failures (bare rows, missing contracts) and 2 errors (the checks do not exist). At `a76476be`: 24 ok.

**Rows with expect (all of them).** mc 22 rows, construct 18 rows, each with an `Expect` on the snapshot (checks in the chapter files, no new gate cell, no new dependency; tour.py untouched).
- mc: page (`scene.list` current, as acts/looks/mc.py), HUD line (`mc-rN-footer-hud`), notice shown in chora-notify (`blocks.notify.shown`), clock with a time (doors clock-door), column count after Super+n (hidden, back), launcher process (`chora-mc --launch fuzzel`), chat shown (`blocks.assistant.shown` has `hud-chat`) and helper word, key list `open` (`mc-keys`), stand-in wpctl state moved by `XF86AudioRaiseVolume`. Contracts on rows: M1 (voice, page turns, Right/Left/Home), T3 (answer, chat), T2 (key list), T1 (launcher), T4:volume. New rows: Home, Super+F1 and Escape (m6), volume up (m10).
- construct: `Expect(cube=...)` (faces, front, mode, turning) on every cube row, `window_is_pane` and `window_back_on_face` on the pull rows, `open` on the key sheet rows, drag and key-turn rows compare the front face with the one read before the act. Contracts: C1 (step back, step out), C2 (pane out, pane back), T2 (sheet).
- Not in the chapters, said here: m11 / T6 (the launcher list is a fuzzel layer surface the scene does not list, so no expect could be non-vacuous; the stand-in rows live in gate-mc-sofa launcher-none), m9 question and answer (the chat row checks chat and helper word only), c1 T1 and c7 T3 (the tour's construct is the cube scene: no walk room, no strip helper), c6 (doors in the strip). T1/T3 of construct and T6 of mc are therefore not in `test_rows_name_their_contract`.
- Script slips the new expects found and fixed in the chapters: Home goes to the agenda (not to the first of the cook's pages); Return and Escape as plain press/release (as tests/acts/looks/construct.py); the cube drag 780 to 80 (the old 260 px did not turn it); Super+F1 for the key sheet (F1 opened nothing); `opens`/`closes` declared for the two key lists.

**`demo/tour/run.py --check mc construct`, twice** (code `a76476be`; both runs print the same)

| chapter | acts | green | inv | known | red |
|---|---|---|---|---|---|
| mc | 22 | 18 | 0 | 4 | 0 |
| construct | 18 | 9 | 0 | 9 | 0 |

`tour: check: 0 red, 0 broken` both times. At the start (f4f49783) mc was 0 green / 15 inv / 3 known and construct 1 green / 16 inv, and construct already had 1 red (I3, twice).

**Known rows (findings; none is a weakened expect)**
- mc m3-m5 timer card (3 rows): I5, the Power glyph lies under the card body (v8 open row, unchanged).
- cb-4, mc "Super+n brings the column back": after the second Super+n the column has 24 visible texts, 72 before the first (hidden: fewer, green). Either the column does not come back in full or a stale region of the cross-fade keeps nodes (my count reads both regions); needs a product look. `known="expect: cb-4 ..."`.
- cb-1, construct "Alt+Escape: step back": I3, construct and xp each say they hold the focus (red on f4f49783 in two runs; tour-one r2 saw this chapter without a red).
- cb-2, construct Super+F1 sheet in the stood-back cube: I5, the sheet row 28 "Volume down" overlaps the chips "Turn right" and "Step in" (717,658 and 664,658).
- cb-3, construct: Escape (shortcut and plain press) does not shut the sheet, it stays open to the end of the chapter (so the 6 rows after it keep the cb-2 overlap as `known`), and Return does not step in while it is open (`expect` known on that row). Hypothesis, not tested: the Escape goes to xp, which cb-1 says holds the focus too.

**Gate** `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,gate-lib,build,test,acts`: pre-commit PASS (1 s); gate-lib FAIL (88 s, `merge-gate-selftest`: "an unknown job leaves no log directory", "--help leaves no log directory", "a run without a failure removes its log directory": got 4, 4, 6, want 0); the gate stops there. The same three checks fail on the untouched base f4f49783 in this container (`tests/merge-gate-selftest.sh` run in a worktree of it), so it is not this change. The rest as `--only ci-check,build,test,acts`: ci-check PASS 85 s, build PASS 23 s, test PASS 208 s, acts PASS 284 s (acts runs `tests/test_tour.py`).
