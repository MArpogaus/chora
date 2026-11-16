# tiling look r2 (rc 1, 11)

Branch claude/tiling-look at e1dda18f (session branch merged first, 2b0ccd2e). Captures: media/2026-11-16/tiling-look-r2/ (Ink and Paper, 1280x800).
Cell: tests/gate-tiling-look-r2.sh (tooltip, door, power, notice; ink and paper), in the merge gate step tiling-look.
Red first: 633fc0d3 (tooltip, door, notice red on r1; power already green). Each red re-checked against the old code after a cell fixup.

| id | result | red | fix | gate cell | capture |
|---|---|---|---|---|---|
| ti-22 | fixed | 633fc0d3, 81ecfe99 | b1412720 | notice: Super+Print writes the file, `makoctl list` has "Screenshot saved" and the path, and the bottom right changes; with grim failing, "Screenshot failed" and the bottom right changes | tiling-screenshot-saved-{ink,paper}.png, tiling-screenshot-failed-{ink,paper}.png |
| ti-23 | fixed | 633fc0d3 | c1adc900 | tooltip: pointer at rest on Apps, helper word, Keys, power for 3 s, then Super+Space and Escape: nothing of the bar under the first tile (exact pixel diff 0) | tiling-tooltip-Apps-{ink,paper}.png; before-stale-tooltip-ink.png |
| ti-24 | fixed (pill); power icon already there | 633fc0d3, b3c95c8e | 3ba4fb22 | door: bar word "No helper", no `hud-row-*` node, also with a helper at work; power: icon pixels at the right end of the bar | tiling-door-none-*, tiling-door-working-*, tiling-bar-power-* |
| r1 MINOR | fixed | - | d220b3e6 | docs/xp-split.md says wobbly is on by default | - |

## What was wrong
- ti-22: the screenshot script gave the notice to notify-send only. On a bus where nothing owns org.freedesktop.Notifications (the tour; mako's own D-Bus activation fails without the display) the notice is lost, saved or failed. Now `chora-tiling --launch screenshot` sets CHORA_NOTIFIER and the script starts the flavor's mako and sends again; without any notifier the result goes to stderr.
- ti-23: reproduced only with the pointer at rest on Apps while the launcher opens by key or a window opens (the tooltip stays; a click already hid it). Fix: the four doors (Apps, helper word, Keys, power) have no tooltip; the key list names their keys. The clock, network and volume tooltips stay and still show while hovered.
- ti-24: the power icon is in the bar and opens the menu with Lock (gate-tiling-power.sh, 3 of 3 clicks); the tour bar was from before r1. The pill was the neutral HUD folder's "Chat ..." row over the first tile, beside the bar word. The tiling look now has its own HUD folder (`tiling`: dot opens the chat, no rows), picked by the shipped rule `space: tiling`; packaging, docs, glossary-free, tests in step.

## Evidence
- New cells: 10 of 10 runs green, both themes (11 ok lines each). Two earlier runs failed on a threshold of mine (Paper failure notice 1727 px of 2000); now 500, absent notice is 0.
- tests/merge-gate.sh: all steps passed once (test 59 s, 2d-guard 142 s after `git fetch --unshallow`, tiling-audit, tiling-look 561 s, doors, cube, e2e). Log of the run: /tmp/merge-gate.9qM1I4 (container).
- cargo test -p chora-tiling 141, -p chora-assistant 228 pass.

## Left open / notes
- The notice shows the path cut with an ellipsis (mako width 360); the full path is in the notification (`makoctl list`).
- Data tooltips (clock, network, volume) can still stand over a tile while the pointer rests on them.
- gate-tiling-audit now runs the tiling HUD folder; its desktop cell has no helper surface by design.
- Captures are part of this commit on session-log (13 PNG), beside the report.
