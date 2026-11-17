# mc r6 (rc 1): a window handed to mc is visible at once

Branch claude/mc at e84e4d30 (pushed), on the session head 4105f8d1. Lane: sonnet, 2026-11-17.

## Cause (rv5-2)

Not the render path of a window that mc owns: give and key already draw it within 0.5 s on the
head (cells give 408 ms, key 455 ms, drop 462 ms after the event). The tour hides the window
during the pull: xp keeps the window until the button goes up; a title-bar drag hands it over
only at the release (wm.rs hand_over). While the button is down over the mc output the window is
an xp node, and the opaque page of mc (root built after xp, so drawn above it) covers it. On the
tiling output nothing covers it, so that leg looks fine. The tour holds the button over the mc
output for about 3 s, then DRAG 5 follows; to a viewer the window "never arrives".
Evidence: tour chunk 05-handover-006, frames 56 to 104 show neither window nor pane on the mc
screen on 4105f8d1; owner xp, rects on the mc output, held capture shows only the Agenda page.

## Fix

Red first: 7fa87352 tests/gate-mc-handover.sh, cells give, key, drag-held, drag-dropped (red
pixel of the window in a capture of the mc output within 1000 ms, surface count unchanged).
Red on 4105f8d1: give ok, key ok, drag-held FAIL (not drawn within 1000 ms, owner xp).
Green: 18795b9e fix(mc): the root of mc moves below the roots of the other plugins at each build
(plugins/chora-mc/src/main.rs under_other_plugins). Cells: give 408 ms, key 451 ms, drag-held
2 ms, drag-dropped 462 ms. e84e4d30 ci: the cells run in merge-gate.sh as step mc-handover.
Tour re-run on 18795b9e (v5_handover.py, claude/media 0f941e2): the window now follows the
pointer over the mc page and mc owns it after the release (chunk 05-handover-006).

## Captures (media/2026-11-17/mc-r6/)

mc-4105f8d1-2026-11-17-xp-to-mc-held.png, mc-18795b9e-2026-11-17-xp-to-mc-held.png (tour frame
80 of chunk 006, before and after), mc-4105f8d1-2026-11-17-gate-drag-held-red.png,
mc-18795b9e-2026-11-17-gate-drag-held-green.png (gate capture of the mc output, held).

## Gate

tests/merge-gate.sh once on e84e4d30: every step PASS (incl. mc-paths, mc-sofa, mc-handover 10 s)
except e2e: gate-tiling-apps-click "a click on the Zebra row did not start zebra", 3 of 3 alone
too. It uses chora-tiling and the real fuzzel only; the diff touches chora-mc and tests, so it
is not from this lane, but I did not run it on 4105f8d1 to prove it. Open: the tiling lane.

## Open / not done

- The drag still shows the window as an xp window over the mc page until the release; mc scene
  and panel appear 0.46 s after the drop (fade 160 ms plus the give). No MINOR work (rules.md 6).
- The tour video of v5 was not re-recorded in full; only chunk 006 was re-run.
