# Window loss

## r1

Lane `claude/window-loss-r1` (brief `briefs/window-loss-r1.md`, rc 9). Head
`7c6955c8` (session head). The lane adds only `tests/repeat-handover.sh`
(commit `96067388`); no product code changed. Logs: `media/2026-11-26/window-loss-r1/`.

Step 0: `git fetch --unshallow`, `cargo check --workspace` (rc 0),
`cargo build --workspace` (rc 0).

### Verdicts

| Item | Verdict | Count |
|---|---|---|
| A13 (mc drag onto construct loses panes) | CLOSED | 0 of 10 runs with `pane: none` after the drag (repair in place) |
| A17 (xp window without keyboard focus under load) | CLOSED | 0 of 5 runs alone, 0 of 5 beside a build load |

### A13: mc drag onto construct (cell `mc construct panes`)

- Brief arm, gate copy with the setup `window.show` (line 601) removed:
  `GATE=<copy> tests/repeat-handover.sh panes 10 <dir>`. Each run had 6
  `pane: none` construct windows in the round lines (2 fillers x 3 drags),
  60 in total. This arm does not measure loss: `setup_none` is 3 in every
  run, so the fillers already had `pane: none` before the first drag (no
  `window.show` gave them a pane). Its first failing line
  (`a13-removed-repair.log`, run 1): `MISMATCH mc construct panes: measured
  gap, expected pass: [filler1 aside, probe shown, filler2 none, filler3 none]`.
- Control, unmodified gate: `tests/repeat-handover.sh panes 10 <dir>`. 0 of 10
  runs had `pane: none` after the drag; all 10 pass (`a13-repair.log`).

A13 is CLOSED on the head for the gate's own setup. Open question, not a
window loss: a window given to construct without `window.show` shows
`pane: none` until it is shown. Not tested whether that is intended.

### A17: cgive and voice under load (matrix cells, both paths)

- Alone: `tests/repeat-handover.sh cells 5 <dir>`: 5 runs, all rc 0. Per run,
  6 xp-destination cgive/voice cells, 0 without keyboard focus; all 48
  cgive/voice cells pass in every run (`a17-alone.log`).
- Under load: `LOAD='while :; do git ls-files "*.rs" | xargs touch;
  CARGO_TARGET_DIR=<scratch>/load-target cargo build --workspace; done'
  tests/repeat-handover.sh cells 5 <dir>`: 5 runs, all rc 0; 16 builds completed
  in the load series, the loop is stopped (`a17-load.log`, `load-tail.log`).
  The load used its own target dir, so the binaries under test were not relinked.

A17 is CLOSED on the head: 0 of 10 runs (0 of 30 xp-destination cells per
series) lose keyboard focus.

### Reading the brief

"10 times each" for cgive and voice is read as 10 matrix runs; each run
covers both paths, so each path ran 10 times (5 alone, 5 under load).
