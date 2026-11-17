# tiling-look r4 report, 2026-11-17

Build 29ee9ebc (claude/tiling-look = session branch aa8aa8c0 + 2 commits). Sonnet worker, one round.

## Cause of j1-5 (Super+Print shows nothing)
The tour session has no session bus (demo/tour/launch.py pops DBUS_SESSION_BUS_ADDRESS; waybar gets a private one).
screenshot.sh sent the notice with notify-send; with no bus it fails, and the CHORA_NOTIFIER fallback
(`chora-tiling --launch mako`) needs the bus too, so the result went to stderr only. The r3 cell used a stub
notify-send, so it never saw this. A second fault, found on the way: on a bus where mako is not yet running,
D-Bus activation starts the system mako (no chora theme, top right, other colours) before the themed one.

## Done
- ebf2bad1 `test:` red first: tests/gate-tiling-print-notice.sh runs the real key (inject Super+Print), real mako,
  no stub, no bus; reads the notice pixel (26 29 39 at 1000,700). Red: "no notice at the bottom right, pixel 0 0 0".
- 29ee9ebc `fix:` flavors/shared/screenshot.sh: no bus -> a bus of its own for 7 s with the notifier and the
  notice (visible 6+ s; the cell asserts 3 s); a bus without a daemon -> the notifier of the look starts first.
  Item 2: power menu (power icon of the bar, Super+Shift+e) gets a fifth row "Screenshot" (German
  "Bildschirmfoto"); tiling passes CHORA_SCREENSHOT so the notifier fallback applies; mouse line of the key list
  and docs/configuration.md say so. Cell: same script, second half (click the power icon, type Screenshot, Return:
  file + notice + 3 s, window audit).
- merge-gate: tiling-look step now includes the new cell.

## Evidence
Cell tests/gate-tiling-print-notice.sh: 3 ok (green). Captures on this branch under
media/2026-11-17/tiling-look-r4/: tiling-print-notice-29ee9ebc-2026-11-17.png,
tiling-power-screenshot-notice-29ee9ebc-2026-11-17.png. Also green: gate-tiling-power, -system, -bar-keys,
cargo test -p chora-tiling (141).
tests/merge-gate.sh once on 29ee9ebc (shallow clone fixed first by full fetch): all 38 steps passed, incl. e2e
(fuzzel cell green here), tiling-look, doors. Not pushed to the session branch (PM merges).

## Open / not done
- The tour must still be rerun to show it; the tour harness starts waybar bare (see r3) and has no mako, but the
  product now answers there.
- A bar pill "Screenshot" was not added (KISS: one place). Power-menu label for a screenshot is a compromise.
- No MINOR work done (N-5 bar on the handover screen is next round).

## Refactor proposals
1. flavors/shared/screenshot.sh + power.sh: the notice code (bus check, own bus, daemon start) should be one shared
   `notify.sh` that lock-failure notices in power.sh also use; power.sh calls bare notify-send, same fault. Size S.
2. tests/gate-tiling-*.sh: each repeats keys/click/menu_open/pixel helpers (gate-power, print-notice, notice);
   move them into gate-lib.sh. Size S-M.
3. demo/tour/launch.py: give every look's tour a bus and mako like a real session (target unit order), instead of
   per-chapter companion lists. Size M.
Cut check: no new plugin breach; the Screenshot row uses only the shared scripts and chora-tiling --launch.
