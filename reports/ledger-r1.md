# ledger r1 (rc 1): open BLOCK and MAJOR on d349a6da

Reviewer lane: opus, cloud, read-only on code. Build: `cargo build --locked --workspace` of d349a6da in
/home/user/wt/head (3 min). Each line of findings/*.md with BLOCK or MAJOR now carries its state.

## Runs on d349a6da (all exit 0)
- tests/gate-doors.sh: 80 of 80 cells, checks `xp log-off`, `construct helper-exit`, `pane-corner`.
- tests/gate-mc-paths.sh: 10 cells and the window audit. gate-xp-run, gate-xp-notepad, gate-tiling-apps,
  gate-tiling-bar-keys, gate-construct-keys, gate-construct-cube: every line ok.
- tests/gate-handover-matrix.sh: every cell as in EXPECTED (192 s), `mc construct panes` included.
- tests/gate-tiling-notice.sh: SKIP, mako is not installed (setup.sh does not install it).
- Probes: the setup of tests/doors/LOOK.sh with the cells replaced by a capture script (scratch copy in the
  worker, gate-lib helpers, 1280x800, 1x).

## Counts (BLOCK + MAJOR not fixed)
| Look | Before | After | Open after |
|---|---|---|---|
| xp | 4 | 0 | |
| tiling | 4 (1 BLOCK) | 2 (1 BLOCK) | ti-1, ti-4 |
| mc | 9 | 1 | mc-30 |
| construct | 4 (1 BLOCK) | 0 | |
| shared | 11 | 4 | sh-5, sh-7, sh-10, sh-11 |
| all | 32 | 7 | |

## Still open, with evidence
- ti-1 BLOCK: fuzzel 1.9.2 in this container (no mouse selection); not re-clicked. Waits for PK1 (fuzzel >= 1.11).
- ti-4 MAJOR: not verifiable here; gate-tiling-notice skips without mako. Needs mako in setup.sh, then one run.
- mc-30 MAJOR: reproduced. mc with voice, HUD host and helper `coder` at 1280x800: `coder` lies under the word
  Mute, the date is cut to `SUNDAY 15 NOVEM` by the Listening island, the clock digits sit under the red dot.
- sh-5 MAJOR: walk view shows the strip clock; with a cube stood back (cube.open, view.step_back) no clock and
  no strip show (node construct-door-clock not visible), only the tiny tray clock of the xp face. The strip hides
  while a cube is open by design (dd8669a4), so lock, power, volume and screenshot have no visible door there.
- sh-7 MAJOR: Super+Alt+1..4 work (matrix key path) but neither the tiling Keys sheet nor the mc All keys list
  names them; the cell keys-names does not ask for them.
- sh-10 MAJOR: tiling, mc, construct say No helper; the xp balloon title is still cut:
  `No helper is set up. Ask the pers...`. The waiting word of a question was not rechecked.
- sh-11 MAJOR: with one helper, Super+a still opens the list (tiling). Typed text is kept and the field says
  `Pick a helper first: type its number and press Enter, or click it.`; Enter does not send it. Better than the
  finding (no text lost), but the "one helper opens direct" part is open.

## Fixed, by merge
- doors (d349a6da): xp-1, xp-3, mc-2, co-4, sh-1, sh-2, sh-3, sh-4, sh-6, sh-8, sh-9.
- xtc (99f40faf): xp-2, xp-4, ti-2 (typing-state word stays MINOR xr-2), ti-3, co-1, co-2.
- mc-fix (35f0c227): mc-1, mc-3 .. mc-8.
- co-3: bfc34490 (parent-death signal of the manager children, before train 4); the cube gate kills the
  construct with -9 and sees the xp end.

## New BLOCK or MAJOR
None beyond sh-5's widened evidence (no visible doors while a cube stands back); left in sh-5, not a new id.

## Captures at 1x (in the worker, not committed)
/tmp/claude-0/-home-user-chora/755813e5-ebf1-5e7d-8597-1bddd2a7e795/scratchpad/caps/: mc-header.png,
mc-keys.png, tiling-keys.png, tiling-chat-open.png, tiling-chat-typed.png, xp-nohelper.png,
construct-walk.png, construct-stoodback.png
