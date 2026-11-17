# tiling-look r3 report, 2026-11-17

Build aa245abd (claude/tiling-look = session branch 4105f8d1 + one ci commit). Sonnet worker, one round.

## Verdict on the three MAJORs: not product faults; the tour starts waybar bare
The tour (media `demo/tour/v5_tiling.py`) starts `waybar --config flavors/tiling/waybar/config.jsonc` with no `--style`
and no waybar.sh. Without the style, waybar draws no padding and no power icon. Same build, both launches:
- bare waybar (the tour): `media/2026-11-17/tiling-look-r3/tiling-bar-bare-waybar-aa245abd-2026-11-17.png`
  shows "No helperNo sound", Connected pill, "Keys", no power icon (= j1-4, rv5-1).
- the unit path (waybar.sh, as chora-tiling-waybar.service runs it):
  `.../tiling-bar-unit-path-aa245abd-2026-11-17.png`: islands, gaps, power icon at the right.

| id | cell | result on aa245abd |
|---|---|---|
| j1-4 gap | tests/gate-tiling-spacing.sh (12 px between every item of both islands) | passed alone on 4105f8d in the probe; but the cell is STALE (see below), so not claimed |
| rv5-1 power icon | tests/gate-tiling-power.sh "the power icon opens the menu 3 of 3 times", Super+Shift+e | pass |
| j1-5 notice | gate-tiling-power.sh "Super+Print writes a file and notifies"; gate-tiling-notice.sh (real mako); gate-tiling-look-r2 notice cells | pass |

j1-5: a real-press probe (Super+Print, real mako, no stub) was red only until the session kept its D-Bus
(`CHORA_TEST_KEEP_BUS=1`; headless-session.sh drops the bus). With the bus it is green and the notice shows. The tour
session has no bus for the key-run script, so notify-send has nowhere to go: a harness fault again.
No product change was needed, so there is no red commit; none was made up.
Fix for media: start waybar through `sh flavors/tiling/waybar/waybar.sh flavors/tiling/waybar` and keep the bus
(`CHORA_TEST_KEEP_BUS=1`) in the tour session.

## Done
- aa245abd `ci: run the power cell of tiling in the gate`: gate-tiling-power.sh is in the `tiling-look` step (item 4).
- gate-tiling-spacing.sh is NOT added: it fails on the current design (gaps 1-2 px, not 12; the layout word is two
  runs). Stale since the islands redesign; it was never in a gate. Needs a rewrite or removal (PM call; MINOR now).

## merge-gate (one run on the head, after fetching full history; the clone was shallow, so 2d-guard's base 8742dee was missing)
35 steps PASS (red-first .. tiling-look incl. power, chat-keys, ..), `e2e` FAIL: tests/gate-tiling-apps-click.sh
"a click on the Zebra row did not start zebra", 3 of 3 alone. The Zebra row is at y=533 as the cell expects
(capture `.../tiling-apps-list-aa245abd-2026-11-17.png`); this container has fuzzel 1.9.2, the cell is written for
1.11.1 pointer semantics. Environmental, unrelated to this diff; I may not install packages. The gate is not green
as run here: needs a run where fuzzel is 1.11.x. Branch pushed anyway (ci-only diff), not merge-ready until then.
Earlier gate attempts: one aborted by me (two started by mistake), one failed on the stale spacing cell.

## Notes
- Stop-hook asked for noreply@anthropic.com authors; kept the owner identity as the brief says.
