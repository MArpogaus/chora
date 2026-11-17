# Review tiling-look r4 (head 29ee9ebc, red ebf2bad1), read-only, no build

Verdict: MERGE-OK (no open BLOCK or MAJOR; 3 MINOR, 1 COSMETIC).

## Checks
1. Cause and fix real: yes. Tour/test session has no bus, so notify-send and the mako fallback both failed
   (flavors/shared/screenshot.sh:21,42-56). Real user session (bus + mako running): has_bus true (:23-25),
   daemon_runs true (:36-40), so no notifier start and the plain notify-send path (:55) as before. Bus but no
   mako: themed mako started first, polled 3 s (:47-53), fixing the unthemed D-Bus-activated system mako.
   No injection: notifier goes through sh -c "$CHORA_NOTIFIER" (set only by chora-tiling main.rs:257), title/body
   passed as $1/$2 args (:28-34), file name from date. Compositor not blocked: own-bus path is setsid + & and
   returns at once (:26-35,43-46); the daemon-start path blocks the script (not the compositor) up to 3 s.
2. Visibility: own bus lives ~7 s (sleep 6 after send, :34), notify at 0.3-3 s; mako.conf default-timeout 10000
   (flavors/tiling/mako.conf:13) so the bus end bounds it to 6+ s. Themed: mako started with the look's conf;
   cell asserts bg pixel 26 29 39 for 3 s (tests/gate-tiling-print-notice.sh:42-45,54-60,71).
3. Red cell: real key via ctl inject Super+Print (:69), real mako/notify-send/grim, bus unset (:27), no stub;
   builder reports "pixel 0 0 0" on old code, consistent with the old code's failing notify-send. Wired at
   tests/merge-gate.sh:173. Merge-gate ran green on the head per the builder (not re-run by me).
4. Mouse path: waybar custom/power on-click -> chora-tiling --launch power (flavors/tiling/waybar/config.jsonc:132)
   -> power.sh row "Screenshot" (flavors/shared/power.sh:17,22,40-43) -> CHORA_SCREENSHOT (main.rs:254) ->
   screenshot with notifier. sleep 0.4 lets the menu close before grim. Cell clicks (1238,24) and picks the row
   (gate:78-87). Key list line (keys.rs:715-726) and docs/configuration.md:1171,1679 updated.
5. Other looks: screenshot.sh is unchanged unless CHORA_NOTIFIER is set, and only tiling sets it, so xp, mc,
   construct behave as before. power.sh is shared: all looks get a fifth row; doors tests (tests/doors/*.sh:160,
   207,114) check only the four old rows and still pass.

## Findings
- MINOR power.sh:40-43 (and screenshot.sh in the other looks): mc/construct/xp get a Screenshot row without
  notifier, so with no bus their notice goes to stderr only; their docs and mc config.rs:59 still say four rows.
- MINOR gate-tiling-print-notice.sh: no check that the own-bus mako exits after 7 s (leak); mako normally quits
  when its bus dies, unverified. Also hard-coded click x=1238 (:78) is layout-fragile.
- MINOR power.sh:42 unquoted ${CHORA_SCREENSHOT:-sh "$(dirname "$0")/..."}: the default breaks for an install
  path with spaces; no injection (env set by our own plugin).
- COSMETIC power.sh header usage line (:10) omits "screenshot".
- Not covered (known): tour still lacks mako/bus; needs rerun to show it in the tour.

## Refactor proposals
1. Shared notify.sh for screenshot.sh and power.sh (power.sh:28 uses bare notify-send, same no-bus fault). S.
2. Move keys/click/notice_here helpers of the tiling gates into gate-lib.sh. S-M.
3. Let power.sh pass a per-look row list so looks without screenshot support do not show the row. S.
