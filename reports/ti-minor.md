# ti-minor

## r1

Base fee9c695, branch claude/ti-minor, no code change: all three items were already fixed on the head, so no red cell can be written (a cell that is green before the fix is not a red cell).

| item | state on fee9c695 | where | cell |
|---|---|---|---|
| ti-5 | chora-notify reads the open chat box at start (`notify.ready`, the host answers `assistant.chat`) and the card steps aside; the finding came from the mako era | 03e06230 (red bdb5af3a), unit `the_card_steps_aside_of_the_chat_panel` | tiling-audit (unit in test) |
| ti-6, ti-19 | `"tooltip": false` on Apps, helper, Keys and power | c1adc900 | tiling-look (cell tooltip of gate-tiling-look-r2) |
| ti-7 | `hide_bar_for_fullscreen` sends SIGUSR1 to waybar while a window is fullscreen | 6092aee3 | tiling-audit (cell fullscreen) |

Gate on fee9c695: `tests/merge-gate.sh --jobs 2 --only build,tiling-audit,tiling-look,tiling-empty` PASS (build 132 s, tiling-audit 253 s, tiling-empty 21 s, tiling-look 533 s). Not run: pre-commit, ci-check, test, clippy, gate-lib, layout, chat-keys, doors (nothing changed).

Open: the findings come from older captures; a fresh capture on the head would confirm the eye sees it too. No cell checks a toast started while the chat is open end to end (only the unit does).

## PM review r1

Checked: 03e06230, bdb5af3a, c1adc900 and 6092aee3 are ancestors of fee9c695 (`git merge-base --is-ancestor`); the cells named are in train 89b green (tiling-audit, tiling-look, tiling-empty PASS, ce2c436b). Verdict: nothing to merge (no branch); ti-5, ti-6/19, ti-7 stay fixed. The gap (toast with the chat open, end to end) goes to p2-r2-tiling item 2, not a new lane. Worker archived.

## r1

Branch claude/ti-minor, head 136bbd82 (base a1f3c8d7 plus a merge of fresh origin/claude/autonomous-dev-session-3w-i33pg7, a3921f96; pushed).

**ti-40** (left island width). Red: `FAIL: the left island ends at 436 with tile_left, at 411 with tile` (gate-tiling-bar.sh). Fix: `#custom-layout { min-width: 100px }` in flavors/tiling/waybar/style.css (fits "Kacheln links"/"Tile bottom"). Green: "ok: the left island keeps its width through four layout words". The gate check stays.

**ti-41** (separator to layout word). tiling-v2.md line 59 names "the layout word" and line 17 only a 4 px unit; no guide line gives a separator-to-word number, so the gate follows the code: the chip edge is 6 px from the separator (as the other items) and the word one padding in (12 measured, 10 wanted, within 2). Red: `separator to the layout word: 6 px, not 12`. Hidden second red behind it: the tooltip check of the same gate wanted a tooltip over Apps/Keys, which c1adc900 removed on purpose; it now asserts no tooltip under a pointer at rest. Green: all four oks of gate-tiling-spacing.sh.

**ti-42** (no "Power menu not installed"). Cause: not a lost notice. tell_launch_error (main.rs:257) connects to the core of the gate session and notify.show succeeds; chora-notify shows one card at a time (card.rs header, `push`: `waiting`) and the screenshot card (default 10 s) was still up: state.get listed `"waiting": [2]`, and `shown_notices` reads only `shown`. The gate waited 5 s. Product behaviour (one card, queue) left as is; the gate dismisses the screenshot card first (`dismiss_notices`), as a person would. Note for the PM: a critical notice still waits up to 10 s behind a normal card; a design question for chora-notify, not filed here. Green: "a launch that fails ends with failure and chora-notify shows the reason".

**ti-45** (gate-join-keys). Asserts the holders before the key (tiling-first: tiling none, xp none; xp-first: tiling none, xp xp), after the key (tiling-first: tiling holds; xp-first: xp holds), and one click case (click on the tile after the second key: tiling holds, xp none; core with --test-injected-press-is-real). Green on a1f3c8d7: `tiling-first: after a click on the tile at 640 405: tiling {"holder":"tiling","surface":"s2"} xp {"holder":null}` and the final ok. Red on 3b14c825 (built apart): `tiling-first: before the key: tiling {"holder":null} xp {"holder":"xp"}` then `FAIL: tiling-first before the key: the holders are tiling none, xp xp; wanted tiling none, xp none`.

**Counts** (each cell 5 times in a row; /tmp/chora-gate.lock serializes gates, so jobs 1 in effect): gate-tiling-bar 5/5, gate-tiling-spacing 5/5, gate-tiling-notice 5/5, gate-join-keys 5/5; 0 red.

**known-red.** tests/acts/known-red.toml holds no row of these four (its rows are the construct and tiling-keys guide checks); nothing removed there. The three known-red steps of tests/merge-gate.sh (known-red-tiling-bar, -spacing, -notice) are gone; their gates run as the new step `tiling-bar`. known-red-tiling-theme and known-red-mc-host stay. The rows ti-40, ti-41, ti-42, ti-45 of findings/tiling.md are yours to close.

**Gate** `tests/merge-gate.sh --jobs 2` on 136bbd82 (fresh origin merged, up to date): EXIT 1, 39 steps PASS. Red: step `acts` (232 pass, 5 known red, 2 blocking), both construct, not mine:
- `FAIL: construct I6: in act 'co-42: the hint Keys of a face stays after 3 s and opens the key list by a click': on HEADLESS-1 'PANES' (construct-help-body-head-6) is 11 px, the guide wants 14 px`
- `FAIL: construct act:the hint Keys of a face names a key that opens the key list: ... RuntimeError: the hint keys names no key`
After the failure the steps after `pixels` (mc-paths ... e2e, incl. the new tiling-bar) did not run in the full run; `--only tiling-bar --jobs 2`: PASS 136 s.

## PM review r1 (136bbd82)

Base a1f3c8d7..136bbd82 without the merge a3921f96: 12 files +92 -46 (PM review: under 200 lines, PM not the builder). Checked: ti-40 min-width on the chip, gate check kept; ti-41 the gate now measures chip edge 6 px and word 10 px in the chip (guide tiling-v2.md names no number; accepted, the 12 px rule of the islands still holds for every other gap); ti-42 dismiss_notices before the launch, cause with lines (card.rs one card, queue); ti-45 holders asserted before, after the key and after a click; red on 3b14c825 pasted. The three known-red tiling steps become step tiling-bar in the full run.
Findings:
- rw-1 MINOR tests/gate-join-keys.sh (click case): `sleep 0.5` then expect_holders; a sleep-only wait in a new assert. Fix: until_ok on the holder. Next tiling round.
- rw-2 MINOR (design, chora-notify): a failure notice waits up to 10 s behind a normal card (report ti-42). Backlog NOTIFY-PRIO.
Gate: foreign reds named (acts, construct: hint keys = flake-hint lane; PANES 11 px = co-45 family); steps after pixels run in the train. Verdict computed: 0 BLOCK, 0 MAJOR: MERGE-OK; joins t110. Worker archived.
