#!/bin/bash
# repro-sl1: the core keeps the memory of windows that are gone (sl-1 of
# sim-long r1, W9 step 1 "open/close churn"). One window manager
# (chora-tiling), CYCLES times: open WINDOWS probe windows one by one, then
# end them all. The RSS of the core is sampled after each window and after
# each close, with a split of /proc/PID/smaps into heap/anon, memfd and
# other mappings. Red when the RSS after the last close is more than
# MAX_MB above the start. Exit 1 = red, 0 = green. INSPECT=<program> runs
# with the pid of the core before the session ends (gdb, smaps);
# TILING_SETTINGS='{"look": {...}}' is sent with settings.set first.
# mem LABEL prints the RSS and its split by mapping kind, in MB.
# Clean up a run that was cut off with tests/battle/kill-session.sh repro-sl1.
WF=repro-sl1; . "$(dirname "$0")/lib.sh"
RUN=${RUN:-/tmp/bt-$WF.run}; : > "$RUN"
CYCLES=${CYCLES:-3}; WINDOWS=${WINDOWS:-12}; MAX_MB=${MAX_MB:-60}
config_home
session_start "1280x800,1280x800+1280+0" tests/battle/keep.sh "/tmp/bt-$WF-tiling.pid" "$bin/chora-tiling" || exit 1
ready tiling get_layout '{}' || { echo "repro-sl1: tiling silent" >&2; session_stop; exit 2; }
core=$(cat "/tmp/bt-$WF.core")
[ -n "${TILING_SETTINGS:-}" ] && { ctl call tiling settings.set "$TILING_SETTINGS" >/dev/null || echo "repro-sl1: settings.set refused" >&2; }
mem() {
  awk -v label="$1" '
    /^[0-9a-f]+-[0-9a-f]+ / { kind = "other"; if ($6 == "" || $6 == "[heap]") kind = "anon"; else if ($6 ~ /^\/memfd:/) kind = "memfd"; else if ($6 ~ /^\/dev\/(dri|shm)|^\/SYSV|^\[anon:/) kind = "shared" }
    /^Rss:/ { rss[kind] += $2; total += $2 }
    END { printf "repro-sl1: %-22s rss=%4d MB anon=%4d memfd=%4d shared=%4d other=%4d\n", label, total/1024, rss["anon"]/1024, rss["memfd"]/1024, rss["shared"]/1024, rss["other"]/1024 }
  ' "/proc/$core/smaps"
}
settle() { sleep "${1:-1.5}"; }
settle 2; mem start; start=$(awk '/^VmRSS/{print $2}' "/proc/$core/status")
for c in $(seq "$CYCLES"); do
  pids=()
  for i in $(seq "$WINDOWS"); do
    pids+=("$(open $((i*20)) 90 $((255-i*20)))")
    [ $((i % 4)) = 0 ] && { wait_count "app_id:chora-probe kind:surface" "$i"; settle 1; mem "cycle $c: $i windows"; }
  done
  kill "${pids[@]}" 2>/dev/null
  for _ in $(seq 50); do [ "$(count_windows)" = 0 ] && break; sleep 0.2; done
  settle 3; mem "cycle $c: all closed"
done
end=$(awk '/^VmRSS/{print $2}' "/proc/$core/status")
[ -n "${INSPECT:-}" ] && "$INSPECT" "$core"
session_stop
echo "repro-sl1: core RSS $((start/1024)) MB at start, $((end/1024)) MB after the last close (bound +$MAX_MB MB)"
if [ $((end - start)) -gt $((MAX_MB * 1024)) ]; then echo "repro-sl1: RED"; exit 1; fi
echo "repro-sl1: GREEN"
