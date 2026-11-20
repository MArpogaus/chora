# black-pane report

## r1 (2026-11-20, branch claude/black-pane at 2a1e2638, from bb16f2ff)

Status: NOT FIXED, no product change. The red test the brief asks for cannot be red with the companions
running: the premise of sd-h1 and of the render half of sh-23 does not reproduce. The PM decides the design
question below.

| Item | Red sha | Fix sha | Test | Gate cell | Capture |
|------|---------|---------|------|-----------|---------|
| sd-h1 empty tiling output beside xp | none (green from the start) | none | `tests/gate-tiling-empty-output.sh outputs` | not wired into merge-gate | `reports/black-pane/after/two-screens-tiling.png` (before: `before/`) |
| sh-23 render half, tiling pane stays black | none (green from the start) | none | `tests/gate-tiling-empty-output.sh panes` | not wired into merge-gate | `reports/black-pane/after/pane-tiling.png`, `panes-screen.png` |

Commit: 2a1e2638 `test(tiling): gate an empty tiling output and pane (sd-h1, sh-23)` (the gate only).

### What I found
- Step 0 passed (unshallow, `cargo check --workspace`). The branch already contained
  origin/claude/autonomous-dev-session-3w-i33pg7 (same head bb16f2ff), so there was nothing to merge.
- The bar of tiling is waybar and the wallpaper is swaybg. Both are companions that a session starts
  (`flavors/tiling/waybar/waybar.sh`, `wallpaper.sh`, or the units); chora-tiling does not draw them.
- Two outputs (xp on HEADLESS-1, tiling on HEADLESS-2, no window anywhere) with `waybar.sh` and
  `wallpaper.sh` running: HEADLESS-2 shows the bar, the Apps door and the wallpaper at once, and a click on
  Apps at x 55 of HEADLESS-2 starts the launcher (the fuzzel stand-in is called).
- xp and tiling in two construct panes with the same companions: the output of the tiling pane
  (VIRTUAL-2) shows bar and wallpaper with no click. `waybar.sh` follows the new virtual output within its
  2 s poll.
- The black screen of the tour (05-01-two-screens-R.png) and the black pane are the state without the
  companions: a bare `chora-tiling` on an output. `COMPANIONS=0 tests/gate-tiling-empty-output.sh` reproduces
  it (both parts fail on pure black `0 0 0 255` at the bar and in the body; captures in
  `reports/black-pane/before/`). multilook-r1 reached the same reading ("Not changed: a tiling pane with no
  window is black. Wallpaper and bar are companions").
- So the open question is whether the tour's handover chapter (and `pane.manager` in a session without
  units) starts the companions. The tour tools for v4 to v6 are not in the repo or on this branch, so I could
  not check what chapter 5 starts.

### Why I stopped before a fix
The only change that makes a bare tiling output draw is that chora-tiling starts `waybar.sh` and
`wallpaper.sh` itself when no companion runs. That has a wide effect:
- 15 gates start `waybar.sh` themselves (gate-tiling-bar, -doors, -power, -look-r2, tests/doors/tiling.sh
  and others); an auto-started panel would double the bar there, and the panel's exclusive zone moves the
  layout rects of every gate that runs bare tiling.
- A wallpaper started by tiling replaces the black background that the pixel cells of tiling-look and
  tiling-audit measure.
- In a real session the units (`chora-tiling-waybar.service`, `chora-tiling-swaybg.service`) start the same
  companions after tiling, so tiling would need a guard (no `INVOCATION_ID`, or the pid files) to avoid two
  panels.

### Options for the PM
1. Opt-in flag `chora-tiling --with-companions`, used by `pane.manager` for tiling and by the tour handover
   script. Small, no gate changes; the red cell is then "a bare `chora-tiling --with-companions` output is
   black" and the tour passes the flag. Recommended.
2. Automatic start when no `INVOCATION_ID` and no `panel.pid`/`wallpaper.pid` companion runs: no flag, but
   the 15 gates above need `COMPANIONS=0`-style opt-outs and the pixel baselines need a re-check.
3. Tour only: start `waybar.sh` and `wallpaper.sh` in chapter 5 as chapter 2 does; no product change.

### Not run
The merge gate cells (`pre-commit`, `ci-check`, `build`, `test`, `clippy`, `layout`, `matrix`, `doors`,
`tiling-look`, `tiling-audit`, `wm-pane`) were not run: the diff is one new gate script and changes no
product code. The pre-commit hooks ran on the commit and passed. The gate runs 2 of 2 parts green
(`outputs`, `panes`) and red with `COMPANIONS=0` (both parts). `crates/chora` `input.rs` is untouched.
