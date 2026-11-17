# xp-polish r5 (rc 1): Notepad that opens for Erika, honest when it cannot

Branch claude/xp-polish at f40b2ddc (session branch 4105f8d1 merged first). Sonnet, one round, 2026-11-17.

## Done
| item | result | cell | capture |
|---|---|---|---|
| 1 j1-2/N1 Notepad falls back | fixed | tests/gate-xp-notepad-fallback.sh (3 cells, in merge-gate step xp-notepad); 2 new unit tests in launch.rs, 1 in apps.rs | media/2026-11-17/xp-polish-r5/xp-notepad-fallback_xp_f40b2ddc_2026-11-17.png (xp, f40b2ddc, 2026-11-17) |
| 2 config plain text, default list documented | done | docs/configuration.md (`editor`), plugins/chora-xp/xp.yaml comment (the plugin has no README) | - |

Behaviour: Notepad tries the setting `editor`, then lowriter, mousepad, gedit, gnome-text-editor, kwrite, kate, xed,
pluma, featherpad, leafpad, l3afpad, geany, then VISUAL, EDITOR, nano, vim, vi in a terminal window. A start that falls
back shows "Notepad opened with X" with "Y is not installed / ended without a window / showed no window in 30 seconds /
could not be started". When none opens: "Cannot open Notepad". An editor that ends without a window is detected after 2 s
(Notepad only; other programs keep the 30 s limit).

## Evidence
- Red: 23a8c476 (cell only) on the unchanged build: window opens, no notice: "FAIL: no notice Notepad opened with gedit / chora-no-such-editor is not installed".
- Green: fd11858d + f40b2ddc: cells "editor missing", "editor ends without a window", "no editor opens" pass; cargo test -p chora-xp 183 ok; clippy clean.
- Fakes on PATH (gedit opens a foot, lowriter exits 1); no packages installed.

## Merge gate (one run on f40b2ddc, 46 m 37 s): FAIL at e2e, not caused by this diff
All steps before e2e PASS, including xp, xp-notepad, hud, layout, tiling-audit, doors, cube, keys, pixels, mc-paths, mc-sofa.
e2e failed in gate-tiling-apps-click ("a click on the Zebra row did not start zebra"), a tiling cell with real fuzzel
(1.9.2 here; the cell comment names 1.11.1). It fails the same way alone on the head (3 of 3) and on 4105f8d1 built
separately (2 of 2): pre-existing and environmental, the diff touches only chora-xp, docs, xp.yaml and tests.
Not rerun in full (brief: one run). The PM decides whether this cell is a known gap of this container.

## Open
- A second failing fallback shows a second notice (no replacement of the first); the last one says "Cannot open Notepad".
- The notice itself is not in the capture: the gate runs no chora-notify; the cell reads it from the D-Bus call.
- The skipped-editor notice shows on every click of Notepad while the setting stays missing.
