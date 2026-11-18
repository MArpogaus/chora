# net2 r1 adapters (F3): state.rs per look, mc fix, door acts

Branch claude/net2-adapters from f31ff880, merged origin/claude/net2-harness at 4052cd82
(5a7f5034), head 5da2712c (pushed). Merges after the harness.
Merge gate on 5da2712c, `tests/merge-gate.sh --jobs 3` (02:05-02:40 UTC): all steps PASS,
GATE_EXIT=0 (run dir /tmp/merge-gate.Zcmyrn, container-local).

## Done
- mc fix (net1 review), red first: 1b1ce329 test, c9051de1 fix. `focus.holder` is mc only while the
  core focus is on a window of mc, else null; `open` is left out (absent, not empty).
- Adapters, each `Plugin::state` plus the `state.get` export, unit tests on a fake core:
  - xp c67d9017 (wm/state.rs): windows bottom first (surface, title, app_id, output, desks, rect,
    shown, minimized, maximized, fullscreen false, floating = floats off a virtual output, z),
    desks per output, open (switcher, xp-session modal, xp-ctx, xp-menu, xp-dlg-* panels),
    busy (effects, cube, fade, slide, resize).
  - tiling 07371449 (state.rs): windows from the model and the last placement (rect, shown,
    tags as desks, z = stacking order), desks = shown tags, open tiling-keys (modal), busy
    (effects, chip).
  - construct 85744615 (state.rs): owned window panes and hidden windows are `windows`; panes
    of another client's window, a plugin node or a virtual output are `views` (never owners);
    open (construct-ask, construct-list, construct-help, launcher); busy (glide, walk, move,
    effects, cube, waiting). No rect (panes lie in a space).
- Focus rule found by the acts (I3), red first: 4daca325/6edb73d9 tests, b12fd058/1a320dde fixes.
  A node that takes the keyboard sends no `focus.changed`, so a look claims the focus from its
  own belief: xp holds it without a focused window until a window of another window manager
  takes it (its icons, menus and dialogs have the keyboard; a destroyed focused window clears
  it); tiling while its key list is open; construct on an owned window, or with its space as the
  flavor when no surface has the focus.
- Door acts 5da2712c (tests/acts/looks): `door_acts` in common.py, the key list by Super+F1 and
  Super+/ and its close in all four looks; xp also the Start button and Turn Off Computer by
  Super+Shift+e. `ready()` waits for every look's `state.get` (a look connects before it exports
  its methods; the first act read that gap: I1, I7 and the new-window owner failed).
- known-red.toml: the six adapter rows (I1, I3 of xp, tiling, construct) pass and are gone.
- docs/api.md 0552081d: state.get in all four looks.

## Acts on the head (gate step `acts`, /tmp/merge-gate.Zcmyrn/acts.log)
Live selftest 18 cells pass; acts: 47 pass, 5 known red, 0 blocking.
New known-red rows (finding ids for the PM to file in the ledgers):
- net2-a1 xp `act:Escape shuts Turn Off Computer`: Escape does not cancel the dialog (WM3).
- net2-a2 mc `I3`: after its window goes, mc gives the keyboard to its page node
  (mc-s-agenda-page) and, by the net1 review rule, holds no focus (WM2).
- net2-a3 mc `I4` and the two key-list open acts: mc reports no `open` (WM3).

## Deviations
- Two small fields outside the adapters, read only by `state.get`: construct `core_focus`
  (subscribes to `focus.changed`, 23 events), xp `core_focus` (set in `focus_moved`, cleared on
  destroy). Accessor visibility: xp `Floats::floats` and the test helper `wm_on_fake_core`
  pub(super), tiling `on_fake_core` pub(crate). No look logic changed.
- `open[].by` is left out everywhere: no look records who opened a surface.
- mc keeps the brief's holder rule; xp uses the broader one. With the xp rule mc's I3 would
  pass, but mc's `core_focus` feeds its logic, so changing it is a WM2 decision, not an adapter.

## Backlog
- The harness Session should wait for `state.get` itself (looks/common.py `ready` does it now).
- tiling windows have no title or app_id (core facts); the join belongs to `choractl state`.
- `maximized` of tiling is true only for both axes.
