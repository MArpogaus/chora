# flakes r5 (rc 3): report

Lane: cloud, branch `claude/flakes-r5` from `98291088`, head `366c1cb3` (pushed). Written 11-15 08:17 UTC.
Load: 8 busy shell loops on 4 cores (load average 9 to 12) unless said otherwise. Logs are under
`/home/user/logs` of the container only.

| Item | Result | Commits (red, fix) | Earliest gate |
|---|---|---|---|
| 1. F19 xp tray "a PNG file: NotFound" | fixed | `1f1f5389`, `975c43b7` | unit test of the helper (now by construction) |
| 2. F18 nested.sh step 14 not_black | not reproduced | none | |
| 3. F16 tiling "no bar" | not reproduced; the audit now names the cause | `429d9db3` | the audit step |
| 4. F20 review minors of r4 | done (5 of 5) | `2db659f0`, `a71464e7`, `61f8960f`, `411e3b6f`, `429d9db3` | clippy (compile time) for GR1 |
| new F21: audit loses borders at 2560x1600 under load | fixed (gate only) | `366c1cb3` | the audit step |
| 5. F7 | not reproduced | none | |
| 5. A5, A17 | not started | none | |

**F19.** Cause: the test helper `tray_runs` names its theme cache `tray-ink-{step}`, so
`the_speaker_has_ink_and_the_no_sign_is_the_palette_red` and the tray grid test both use
`chora-xp-tray-ink-1-<pid>` at the same time. The `Theme::load` of one empties the folder (`clear`),
or its `remove_dir_all` deletes it, while the other is about to load its PNG. r4 looked only for a
second user of `tray-red`. Reproduced: 52 of 200 runs of the two tests fail. Red: the new test
`a_second_theme_of_one_test_name_keeps_the_files_of_the_first` fails with "…png went with the cache
of the second theme". Fix: `sheets::tests::theme` adds a count of its calls to the folder name, so
no two calls share a folder, whatever name they pass. Green: 0 of 200 alone, 0 of 200 with the
8 loops; all 175 xp tests pass. CI runs nextest (one process per test), so CI could not show it.
About 20 other test helpers name a temp folder only by name and pid (chora-draw, chora-kit hud and
settings, assistant, mc, tiling, voice); a scan found no clashing literal names, but the F19 shape
(one helper called by two tests) is not visible to grep. Proposal: one counted helper for all.

**F18.** Not reproduced in 22 runs of `tests/nested.sh`: 12 on 4 cores at load 6 to 12, 10 with
the test and 8 loops pinned to 2 cores (`taskset -c 0,1`, load 9 to 13). Read: `unlock` in
`lock.rs` asks for no frame, but the winit backend redraws all the time (`redraw_later` after a
frame without damage), so a missed frame is no cause I can show. No change.

**F16.** Not reproduced: 6 cold runs of the first config (`drop_caches` before each), 3 of them under
load; waybar showed its bar about 1 s after it started. Found: the audit sent waybar to `/dev/null`
and waited 30 s for the bar without a word. It now keeps waybar's log and fails once with its last
lines and the answer of the last capture. Also: `choractl` is not on PATH in the audit and the
doors gate, so waybar.sh always waits its 10 s for `tiling get_tags` and then shows on every output.
No gate showed the bar missing this round (6 full gates).

**F20.** (a) The cube gate times only `call cube.open` with the bash clock, less a call of
`construct.list` just before it (the start of choractl under load). Red with a sleep of 1 s in
`open_cube`: "cube.open took 1105 ms, 1100 ms more than construct.list"; green 82 ms at load 11.
(b) `at_rest` asks for four equal widths over 750 ms; a stall longer than 750 ms can still pass it.
(c) Instead of a unit test on `sync_cube_keys`: clippy `disallowed-methods` bans
`Runtime::ungrab_all` in the workspace (compile time). The construct's `leave` keeps an allow
(every binding goes). **Voice still lets its keys go and grabs them again** (client.rs 684, 701: the
GR1 bug class); its two calls carry an allow that names it. Propose backlog GR2. (d) the stale
allow at wm.rs:688 is gone. (e) the doors message quotes the last capture.

**F21 (new, found by loaded gate 3 of the first series).** tiling-audit, `ink-2560x1600@2`, cells
desktop and question: no tile border, no glow, no shadow; the later cells of the round passed.
With `CHORA_FRAME_BUDGET_US=1` the audit fails with the same colours at the same pixels: the
watchdog of the core dropped all effects. At 2560x1600 the threads of the software renderer wait
for a CPU while the main thread sleeps, which the busy correction cannot see (safety.rs says so).
The audit now takes the 500 ms budget of the cube, HUD, 3D and K1 gates; the watchdog keeps its own
cells (nested step 26, HUD with budget 1). A failed cell quotes the watchdog lines of the core.
The core log of the failing run was gone (temp dir), so the cause is shown by its signature only.

**F7.** Not reproduced: 0 of 24 (4 at once plus 4 busy loops, all on 2 cores). The test already
has the `LATE` guard for a delayed watchdog thread. No change.

## Gates (full `tests/merge-gate.sh`, 27 steps)

| Run | Tree | Load | Result | Directory |
|---|---|---|---|---|
| L-1 | `411e3b6f` | 9.4 | PASS | `/home/user/logs/gates/merge-gate.O1Lm2F` |
| L-2 | `411e3b6f` | 9.2 | PASS | `merge-gate.PsJJaV` |
| L-3 | `411e3b6f` | 9.7 | FAIL tiling-audit (F21) | `merge-gate.oyq9WO` |
| M-1 | `366c1cb3` | 9.4 | PASS | `merge-gate.VYzJsJ` |
| M-2 | `366c1cb3` | 9.3 | PASS | `merge-gate.VCy5nT` |
| M-3 | `366c1cb3` | 9.3 | PASS | `merge-gate.IW5Jm8` |
| cold | `366c1cb3` | none | PASS (28 min) | `merge-gate.hGYzc6` |

M-1 to M-3 are three in a row under load. The cold run followed `rm -rf target`, `sync` and
`drop_caches`. `tests/ci-local.sh pre-commit` passed.

## Open

- F18, F16, F7 not reproduced; F16 and F21 now name their cause if they come back.
- Voice GR1 (ungrab_all, then grab): propose GR2.
- One counted temp-folder helper for the tests of all crates (the F19 class): propose.
- A5, A17 not started. No independent review of this round yet.
