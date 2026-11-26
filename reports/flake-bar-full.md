# flake-bar-full: the tiling bar stays over a fullscreen window

## r1

- Base: `6daaaab8` (session head, t147 green). Head: `db34b699` on
  `claude/flake-bar-full-r1`. Commits: red `11e88ba8` (test), fix `db34b699`.
- Diffstat (base..head): `plugins/chora-tiling/src/overlay.rs` +29 -15,
  `plugins/chora-tiling/src/overlay/tests_moved.rs` +36,
  `tests/gate-tiling-audit.sh` +5; 3 files, 70 insertions, 15 deletions.

### Red of train 148

`media/2026-11-26/train148/tiling-audit-red.log`: `FAIL: fullscreen
ink-1280x800: the bar does not hide over the fullscreen window within 20 s:
{"window":[0.0,0.0,1280.0,800.0],"output":[1280,800],"work_area":[0,40,1280,760]}`,
then `the bar shows over the window at x 33`; the same on ink-2560x1600@2.
The fullscreen window covers the output and the work area keeps the 40 px
of the bar: waybar is visible and holds its exclusive zone.

### Counts on 6daaaab8 (no red reproduced)

- `tests/merge-gate.sh --jobs 2 --only tiling-audit`: 20 runs alone, 20
  green (about 250 s each), plus 1 run under a parallel release `cargo
  build -j4` (load average 14): green. 0 red in 21.
- Extra, `tests/gate-tiling-audit.sh` directly, 8 busy loops on 4 cores: 12
  runs of the fullscreen cell only (ink, 1280x800) and 6 full ink rounds:
  18 of 18 green.
- So the red of train 148 is not reproduced here: below 1 in 39 rounds.

### Cause (product, a defect that gives exactly this state; the
train's own trigger is not proven)

`hide_bar_for_fullscreen` returned early when its record said "hidden"
(`bar_hidden == Some((_, want))`) without looking at the pid of the panel,
so a waybar that started again while a window was fullscreen (new pid,
visible, USR1 ignored until its handlers are set) was never told to hide
and floated over the window, with its 40 px zone held, until the window
ended: the state of the red log. The harness was not the cause of the
order: the cell waits on `fullscreen_rests` (state, 20 s) before it audits.
What restarted or hid the panel in train 148 stays open: no log of the
panel or of the core was kept from that train, and the cell printed only
the fail line. Not ruled out: a USR1 that is lost or coalesced while
waybar is slow (the toggle is never verified against the work area).

### Pin and fix

- Red `11e88ba8`: `bar_step` is the old logic as a pure function (the
  panel lookup passed in); `a_panel_that_starts_again_under_a_fullscreen_window_is_hidden`
  and `a_panel_that_ends_under_a_fullscreen_window_is_forgotten` fail on it
  every time, `the_panel_is_toggled_once_for_a_fullscreen_window` passes.
- Fix `db34b699`: `bar_step` compares the pid at each event while a window
  is fullscreen or a hidden panel is recorded, keeps no record while the
  panel shows. The cell now prints, when the bar does not hide: pid and
  `SigIgn`/`SigCgt` of waybar, the fullscreen flag, the outputs (work
  area) and the tail of the waybar log.
- After: `tiling-audit` 20 of 20 green alone.

### Gate

Full gate `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push) on
`db34b699`: EXIT 0, "all steps passed; no --push, nothing is pushed";
tiling-audit PASS 248 s, test PASS, clippy PASS, e2e PASS 272 s, no FAIL
or SKIP line. `pgrep -fa 'chora --headless'` empty afterwards.
