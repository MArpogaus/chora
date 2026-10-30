#!/bin/bash
# W7 safety: SIGUSR1 emergency reset during a tiling session.
WF=w7; . "$(dirname "$0")/lib.sh"
session_start "1280x800" "$bin/chora-tiling" || exit 1
for i in $(seq 6); do open 30 $((i*40)) 120; done
wait_count "app_id:chora-probe kind:surface" 6 || finding "W7: $(count_windows) windows instead of 6" wrong "run 6 windows"
health "6 windows tiled"

core=$(pgrep -P "$HLPID" -x chora)
[ -n "$core" ] || { finding "W7: no chora process to signal" wrong "pgrep -P \$HLPID -x chora"; session_stop; exit 1; }
kill -USR1 "$core"
# the emergency reset revokes all grants: grant again to continue testing
sleep 1; admin grant agent screen.capture input.inject >> "$RUN" 2>&1
# the core must answer within 5 s again
ok=0
for _ in $(seq 10); do timeout 2 "$bin/choractl" --name agent query >/dev/null 2>&1 && { ok=1; break; }; sleep 0.5; done
[ $ok = 1 ] || finding "W7: core silent after SIGUSR1" crash "kill -USR1 $core"
# the reset disconnects every client, so no window manager lives; the grid
# itself is the work of the core

# windows must sit in a grid: no two share a place, checked on the rects
ok=0
for _ in $(seq 25); do
  grid=$(python3 tests/battle/check_grid.py 6) && { ok=1; break; }
  sleep 0.2
done
step "grid check: ${grid:-FINDING}"
[ $ok = 1 ] || finding "W7: windows overlap after SIGUSR1 reset" wrong "kill -USR1, compare pos"
health "after reset"
ctl capture "$repo/tests/battle/w7-final.png" --output HEADLESS-1 >> "$RUN" 2>&1
session_stop
