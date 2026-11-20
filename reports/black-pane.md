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

## r2 (2026-11-20, branch claude/black-pane, option 1 of r1)

Status: FIXED. Red sha c35404a9, fix sha 91e6a114 (pushed). Dev branch
origin/claude/autonomous-dev-session-3w-i33pg7 (b97369d1) merged first (bdfd523c). Step 0 passed.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,layout,matrix,doors,tiling-look,tiling-audit,wm-pane,tiling-empty`:
all 12 cells PASS (tiling-look 522 s, doors 350 s, matrix 274 s, tiling-audit 273 s, tiling-empty 21 s).
`crates/chora` `input.rs` is untouched.

| Item | Red sha | Fix sha | Test | Gate cell | Capture |
|------|---------|---------|------|-----------|---------|
| 1 red first: `--with-companions`, COMPANIONS=0 | c35404a9 | 91e6a114 | `tests/gate-tiling-empty-output.sh flag` (pure black `0 0 0 255` before) | tiling-empty | before `r2/before/flag-tiling.png`, after `r2/after/flag-tiling.png` |
| 2 start waybar.sh and wallpaper.sh once, pid-file guard | c35404a9 | 91e6a114 | `... flag` (draws) and `... guard` (gate-started companions first, then the flag: one `waybar.sh`, one `waybar`, one `wallpaper.sh`) | tiling-empty | `r2/after/guard-tiling.png` |
| 3 pane.manager passes the flag, doc | c35404a9 | 91e6a114 | `COMPANIONS=0 ... panes` (black in r1 `before/pane-tiling.png`, draws now) | tiling-empty, wm-pane | `r2/after/pane-tiling.png`, `r2/after/panes-screen.png` |
| 4 `: "${XDG_RUNTIME_DIR:?}"` in waybar.sh and wallpaper.sh (KIT-SEC MINOR-1) | - | 91e6a114 | covered by every cell that starts them | tiling-empty, tiling-look | - |
| 5 `tiling-empty` cell in merge-gate | - | 91e6a114 | `tests/gate-tiling-empty-output.sh` (outputs, panes, flag, guard) | tiling-empty | - |

### What changed
- `chora-tiling --with-companions` (plugins/chora-tiling/src/main.rs, `theme::start_companions`): after start-up
  it runs `sh waybar.sh FOLDER` and `sh wallpaper.sh` from the flavor directory, each only when its pid file
  (`panel.pid`, `wallpaper.pid`) names no running process of that script. No flag: nothing changes, so the
  15 gates and the pixel baselines are as before (tiling-look, tiling-audit, doors PASS).
- A panel in a session without a D-Bus session bus runs in `dbus-run-session`: the headless sessions of the
  gates have none, and waybar draws nothing without one (found by the first green attempt: wallpaper, no bar).
- `managers.tiling` defaults to `chora-tiling --with-companions` (plugins/chora-construct/src/config.rs,
  docs/configuration.md `managers` row).
- `waybar.sh` and `wallpaper.sh` use `: "${XDG_RUNTIME_DIR:?}"`. `wallpaper.sh` now also ends, with its pid
  file removed, when the Wayland socket of its session is gone, so one that tiling started outlives no session
  (it looped forever on a dead display before).

### Notes for the PM
- The first red commit tested "a second `--with-companions` tiling" as the second start. A second chora-tiling
  cannot connect under the name `tiling` (one pane per manager), so that part was vacuous: a mutant without the
  guard stayed green. The `guard` mode (companions started first, as the units or a gate do) replaces it and
  goes red on that mutant (`2 waybar.sh in the session, not one`). It is in the fix commit, not the red commit.
- ci-check failed once on the copy row `XDG_RUNTIME_DIR (KIT1)` (my error message named the variable); reworded
  and the gate rerun in full, all cells PASS.
- A race of two starts within the few ms before `waybar.sh` writes its pid file is not guarded; pane.manager
  starts one tiling per session, so only a unit and a pane starting at the same instant could meet it.
- Not touched: the tour tools of v4 to v6 (not in the repo); chapter 5 of the tour can pass `--with-companions`
  to its tiling.

## Review r2 (PM, 02:55; PM not the builder)
Base dev..91e6a114: 2a1e2638, c35404a9 red, 91e6a114 fix (8 files, +212 -8).
- Brief items 1-5: met. Flag off keeps every baseline (tiling-look, tiling-audit, doors PASS); `managers.tiling` default passes it, so a pane.manager tiling pane and a second output draw bar and wallpaper (captures r2/after/{flag,pane,guard}-tiling.png). Children are reaped (theme.rs:596 waits in a thread); wallpaper.sh ends with its socket.
- Red-first: the pure-black flag mode is red in c35404a9; the guard mode came with the fix after the first red was found vacuous (honestly reported). Accepted.
- Nit bp-1: `bus` is keyed on `folder` (theme.rs start_companions), so "needs D-Bus" rides on "takes its folder"; name it per companion. Nit bp-2: the pid-file race of two starts within ms is unguarded (documented).
Verdict: MERGE-OK (no open BLOCK or MAJOR). sd-h1 and sh-23 render half close with the merge. Train t84 with stray-grab.
