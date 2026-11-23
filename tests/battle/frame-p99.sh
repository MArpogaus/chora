#!/bin/bash
# frame-p99: the frame interval of a client during a relayout and during a
# resize drag (sim-long r2, item 3). One session: chora-tiling on the first
# output, six probe windows, then `chora-probe live FRAMES` runs as a client
# of the session. Its toplevel commits a frame on each frame callback and
# prints "N frames, mean M ms, p99 P ms, max X ms" (tests/probe, `live`).
# Relayout: the layouts cycle (set_layout) while the probe runs. Resize drag:
# no pointer drag is injected; the master width of tiling is swept from 0.3
# to 0.7 by settings.set, which resizes the master layer frame by frame.
# Each phase runs PASSES times. Output: RUN (default
# /tmp/bt-frame-p99.run) and the lines printed by the probe in PROBE_OUT.
WF=frame; . "$(dirname "$0")/lib.sh"
RUN=${RUN:-/tmp/bt-$WF.run}; : > "$RUN"
FRAMES=${FRAMES:-400}; PASSES=${PASSES:-3}
PROBE_OUT=${PROBE_OUT:-/tmp/sim-long-r2/frame-p99.txt}
mkdir -p "$(dirname "$PROBE_OUT")"; : > "$PROBE_OUT"
config_home
session_start "1280x800" tests/battle/keep.sh "/tmp/bt-$WF-tiling.pid" "$bin/chora-tiling" || exit 1
ready tiling get_layout '{}' || { echo "frame-p99: tiling silent" >&2; session_stop; exit 2; }
for i in $(seq 6); do open $((i*40)) 90 $((255-i*30)) >/dev/null; done
wait_count "app_id:chora-probe kind:surface" 6 || step "frame-p99: $(count_windows) of 6 windows"
layouts=(tile fair spiral dwindle max floating tile_left tile_bottom)
probe() {
  local out line
  out=$(mktemp)
  "$bin/chora-probe" live "$FRAMES" > "$out" 2>&1 &
  local pp=$!
  sleep 1.5
  "$1"
  wait "$pp"
  line=$(grep -a "frames, mean" "$out" | tail -1)
  rm -f "$out"
  echo "frame-p99: $2: ${line:-no line}" | tee -a "$PROBE_OUT" >> "$RUN"
}
relayout() { for i in $(seq 12); do ctl call tiling set_layout "{\"layout\": \"${layouts[$((i % 8))]}\"}" >/dev/null 2>&1; sleep 0.25; done; }
drag() { for i in $(seq 0 20); do ctl call tiling settings.set "{\"master_width\": 0.$((30 + 2 * i))}" >/dev/null 2>&1; sleep 0.1; done; }
for p in $(seq "$PASSES"); do
  probe relayout "relayout pass $p"
  probe drag "resize drag pass $p"
done
ctl call tiling settings.set '{"master_width": 0.5}' >/dev/null 2>&1
health "frame-p99 end"
log_scan
session_stop
echo "frame-p99: done" >> "$RUN"
