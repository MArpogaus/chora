#!/bin/bash
# W8 oversize stress: 40 windows (the sandbox holds 3 GB of memory and
# software rendering; 100 windows thrash the host, see FRICTION.md), every
# layout three times, 50 hotplug rounds, 20 plugin restarts.
WF=w8; . "$(dirname "$0")/lib.sh"
free_gb=$(awk '/MemAvailable/ {print int($2/1048576)}' /proc/meminfo)
[ "$free_gb" -ge 8 ] || { echo "W8 skipped: only ${free_gb} GB free" >> "$RUN"; exit 0; }
# supervise keeps tiling alive and answers `choractl restart` (D42)
session_start "1280x800,1280x800+1280+0" "$bin/choractl" supervise tiling -- "$bin/chora-tiling" || exit 1

step "opening 40 windows"
N=40
for i in $(seq $N); do
  # `choractl run` detaches; open records the pid for session_stop
  open $((i*2 % 255)) $((i*3 % 255)) $((i*5 % 255)) > /dev/null
done
wait_count "app_id:chora-probe kind:surface" $((N-2)) \
  || finding "W8: only $(count_windows) of $N windows mapped" wrong "$N runs"
c=$(count_windows)
step "$c windows mapped"
health "$N windows"

layouts="tile tile_left tile_bottom tile_top fair fair_horizontal spiral dwindle max fullscreen magnifier floating"
for round in 1 2 3; do
  for l in $layouts; do ctl call tiling set_layout "{\"layout\":\"$l\"}" > /dev/null 2>&1; done
  health "36 layout changes round $round"
done

step "50 hotplug rounds with $N windows"
for i in $(seq 50); do
  name=$(admin add-output "800x600+2560+0" | sed -n 's/.*"name": "\([^"]*\).*/\1/p')
  [ -n "$name" ] || { finding "W8: add-output failed in round $i" wrong "add-output 800x600"; break; }
  admin remove-output "$name" > /dev/null 2>&1
  health "hotplug round $i"
done

step "20 plugin restarts"
# supervise allows at most five restarts per minute (D42): keep the
# restarts fifteen seconds apart
for i in $(seq 20); do
  ctl restart tiling > /dev/null 2>&1 || { finding "W8: restart $i failed" wrong "choractl restart tiling"; break; }
  ready tiling get_layout '{}' || { finding "W8: tiling silent after restart $i" wrong "choractl restart tiling"; break; }
  health "restart $i"
  sleep 15
done
c=$(count_windows)
[ $c -ge $((N-2)) ] || finding "W8: $c windows left after the stress" wrong "$N windows, 36x3 layouts, 50 hotplug, 20 restarts"
ctl capture "$repo/tests/battle/w8-final.png" --output HEADLESS-1 >> "$RUN" 2>&1
session_stop
