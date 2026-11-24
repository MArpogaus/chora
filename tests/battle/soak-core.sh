#!/bin/bash
# soak-core: one core kept alive for SOAK_MIN minutes (default 180), the
# follow-up of sim-long r1 (sl-7). One session for the whole run: chora-tiling
# on the first output, chora-mc (--manage-windows) on the second, and a
# confined client box0 as in W12. Each round runs steps taken from W9 (open
# and close churn, layout cycling, settings storm), W11 (scene switches,
# panel floods, scene.save) and W12 (confined client: a 200-create
# transaction, refused settings.set, removes), then closes every probe
# window. Every SAMPLE_S seconds the sampler writes the RSS (kB) and the fd
# count of the core and of each plugin, by pid, to SAMPLES. A round that
# finds the core dead ends the run (a finding, crash). Each 5th round
# restarts chora-mc. Not in this run: W10 (chora-xp needs its own theme
# session) and W16 (voice needs its own fakes), see RESULTS.md.
# Per round (sh-36): the settled RSS (after end_probes, only with 0 windows open)
# and the round's peak RSS (VmHWM, reset at the round start through clear_refs),
# of chora, chora-tiling and chora-mc, in kB, into the run file.
# Run: setsid nohup bash tests/battle/soak-core.sh >/dev/null 2>&1 </dev/null &
# Clean up with tests/battle/kill-session.sh soak (never by process name).
WF=soak; . "$(dirname "$0")/lib.sh"
RUN=${RUN:-/tmp/bt-$WF.run}; : > "$RUN"
SOAK_MIN=${SOAK_MIN:-180}; SAMPLE_S=${SAMPLE_S:-600}
SAMPLES=${SAMPLES:-/tmp/sim-long-r2/samples.csv}
mkdir -p "$(dirname "$SAMPLES")"
[ -f "$SAMPLES" ] || echo "t_s,process,pid,rss_kb,fds" > "$SAMPLES"
config_home
mkdir -p "$XDG_CONFIG_HOME/chora/mc/scenes"
session_start "1280x800,1280x800+1280+0" sh -c "tests/battle/keep.sh /tmp/bt-$WF-mc.pid $bin/chora-mc --manage-windows & tests/battle/keep.sh /tmp/bt-$WF-tiling.pid $bin/chora-tiling --output HEADLESS-2 & wait" || exit 1
ready mc scene.list '{}' || finding "soak: mc silent at start" wrong "start mc + tiling"
ready tiling get_layout '{}' || finding "soak: tiling silent at start" wrong "start mc + tiling"
box0=$(admin clients create box0 | python3 -c 'import json,sys; print(json.load(sys.stdin).get("path",""))')
[ -S "$box0" ] || { echo "soak: no box0 socket" >&2; session_stop; exit 2; }
mc() { ctl call mc "$@" >/dev/null 2>&1; }
mc scene.save '{"name": "day", "grid": [2, 2], "panels": [
  {"id": "a", "cell": [0, 0, 1, 1], "title": "A", "lines": ["one"]},
  {"id": "b", "cell": [1, 0, 1, 1], "title": "B", "lines": ["two"]},
  {"id": "c", "cell": [0, 1, 2, 1], "title": "C", "lines": ["09:00\tthree"]}]}' \
  || finding "soak: scene.save of a plain scene refused" wrong "scene.save day"
core=$(cat "/tmp/bt-$WF.core")
T0=$(date +%s)
echo "soak: core $core, start $(date -u +%FT%TZ), run $SOAK_MIN min" >> "$RUN"

sampler() {
  local end=$(( $(date +%s) + SOAK_MIN * 60 )) now p name pid
  while [ "$(date +%s)" -lt "$end" ]; do
    now=$(( $(date +%s) - T0 ))
    for name in chora chora-tiling chora-mc; do
      if [ "$name" = chora ]; then pid=$core; else pid=$(pgrep -s "$core" -x "$name" | tail -1); fi
      [ -n "$pid" ] && [ -d "/proc/$pid" ] || { echo "$now,$name,,," >> "$SAMPLES"; continue; }
      p=$(awk '/^VmRSS:/ {print $2}' "/proc/$pid/status" 2>/dev/null)
      echo "$now,$name,$pid,${p:-},$(find "/proc/$pid/fd" -mindepth 1 -maxdepth 1 2>/dev/null | wc -l)" >> "$SAMPLES"
    done
    sleep "$SAMPLE_S"
  done
}
sampler &
SAMPLER=$!

hwm_pid() {
  if [ "$1" = chora ]; then echo "$core"; else pgrep -s "$core" -x "$1" | tail -1; fi
}
hwm_reset() {
  local name p
  for name in chora chora-tiling chora-mc; do
    p=$(hwm_pid "$name")
    if [ -n "$p" ] && ! echo 5 > "/proc/$p/clear_refs" 2>/dev/null; then
      echo "soak: round $1: clear_refs refused for $name (peak not reset)" >> "$RUN"
    fi
  done
}
round_rss() {
  local n=$1 w=$2 name p s k rs='' hs=''
  for name in chora chora-tiling chora-mc; do
    p=$(hwm_pid "$name"); s=na; k=na
    if [ -n "$p" ] && [ -d "/proc/$p" ]; then
      s=$(awk '/^VmRSS:/ {print $2}' "/proc/$p/status" 2>/dev/null)
      k=$(awk '/^VmHWM:/ {print $2}' "/proc/$p/status" 2>/dev/null)
    fi
    rs="$rs $name=${s:-na}"; hs="$hs $name=${k:-na}"
  done
  if [ "$w" = 0 ]; then
    echo "soak: round $n: settled RSS kB:$rs" >> "$RUN"
  else
    echo "soak: round $n: settled RSS kB: n/a ($w windows open)" >> "$RUN"
  fi
  echo "soak: round $n: peak RSS kB (VmHWM since round start):$hs" >> "$RUN"
}

round() {
  local n=$1 i pids=() extra=() l w
  hwm_reset "$n"
  for i in $(seq 12); do pids+=("$(open $((i*20)) 90 $((255-i*20)))"); done
  wait_count "app_id:chora-probe kind:surface" 12 || finding "soak: $(count_windows) of 12 windows (round $n)" wrong "12 runs"
  for i in 0 2 4 6 8 10; do kill "${pids[$i]}" 2>/dev/null; done
  for i in $(seq 6); do extra+=("$(open 200 $((i*30)) 30)"); done
  wait_count "app_id:chora-probe kind:surface" 12 || finding "soak: $(count_windows) windows after churn (round $n)" wrong "kill 6, open 6"
  for l in tile fair spiral dwindle max floating tile_left; do
    ctl call tiling set_layout "{\"layout\": \"$l\"}" >/dev/null 2>&1 || finding "soak: set_layout $l refused (round $n)" wrong "set_layout"
  done
  ctl call tiling settings.set '{"gap": "x"}' >/dev/null 2>&1 && finding "soak: settings.set took a bad gap (round $n)" wrong "settings.set gap x"
  ctl call tiling settings.set "{\"gap\": $((n % 9))}" >/dev/null 2>&1 || finding "soak: settings.set gap refused (round $n)" wrong "settings.set gap"
  for s in day agenda day; do mc scene.show "{\"name\": \"$s\"}"; done
  for i in $(seq 20); do mc panel.set "{\"scene\": \"day\", \"panel\": \"a\", \"lines\": [\"line $n.$i\"]}"; done
  mc panel.set "{\"scene\": \"day\", \"panel\": \"b\", \"lines\": [\"$(head -c 4096 /dev/zero | tr '\0' x)\"]}"
  timeout 60 python3 -I - "$box0" "$n" >> "$RUN" 2>&1 <<'EOF'
import json, socket, sys
path, n = sys.argv[1], int(sys.argv[2])
s = socket.socket(socket.AF_UNIX); s.settimeout(50); s.connect(path); f = s.makefile()
def rpc(i, m, p):
    s.sendall(json.dumps({"jsonrpc": "2.0", "id": i, "method": m, "params": p}).encode() + b"\n")
    while True:
        line = f.readline()
        if not line: return {"error": {"message": "closed"}}
        r = json.loads(line)
        if r.get("id") == i: return r
rpc(1, "hello", {"name": "box0"})
ids = [f"s{n}-{i}" for i in range(200)]
r = rpc(2, "scene.apply", {"commands": [{"create": {"id": x, "kind": "rect", "parent": "root", "props": {"size": [1, 1]}}} for x in ids]})
print(f"soak: round {n}: 200 creates: {str(r.get('error') or 'ok')[:80]}")
r = rpc(3, "scene.apply", {"commands": [{"remove": {"id": x}} for x in ids]})
print(f"soak: round {n}: 200 removes: {str(r.get('error') or 'ok')[:80]}")
r = rpc(4, "methods.call", {"client": "tiling", "method": "settings.set", "params": {"gap": 40}})
print(f"soak: round {n}: box0 settings.set: {'refused' if 'error' in r else 'TOOK'}")
EOF
  kill "${pids[@]}" "${extra[@]}" 2>/dev/null
  for _ in $(seq 50); do [ "$(count_windows)" = 0 ] && break; sleep 0.2; done
  [ "$(count_windows)" = 0 ] || step "round $n: $(count_windows) windows left after close"
  end_probes
  for _ in $(seq 25); do [ "$(count_windows)" = 0 ] && break; sleep 0.2; done
  w=$(count_windows)
  echo "soak: round $n: windows open $w after end_probes" >> "$RUN"
  round_rss "$n" "$w"
  if [ $((n % 5)) = 0 ]; then
    if ! { restart_plugin mc && ready mc scene.list '{}'; }; then finding "soak: mc did not come back (round $n)" crash "restart_plugin mc"; fi
  fi
  health "soak round $n"
  rss_note "round $n end"
}

n=0
while [ $(( $(date +%s) - T0 )) -lt $(( SOAK_MIN * 60 )) ]; do
  n=$((n+1))
  round "$n" || break
  pgrep -P "${HLPID:-0}" -x chora >/dev/null || { step "core gone after round $n"; break; }
done
echo "soak: rounds $n, end $(date -u +%FT%TZ)" >> "$RUN"
kill "$SAMPLER" 2>/dev/null
now=$(( $(date +%s) - T0 )); p=$(awk '/^VmRSS:/ {print $2}' "/proc/$core/status" 2>/dev/null)
[ -d "/proc/$core" ] && echo "$now,chora,$core,${p:-},$(find "/proc/$core/fd" -mindepth 1 -maxdepth 1 2>/dev/null | wc -l)" >> "$SAMPLES"
rss_note end
log_scan
session_stop
echo "soak: DONE" >> "$RUN"
