#!/bin/bash
# Shared helpers for the battle workflows. Source this, then call
# session_start, health, session_stop. Run files: $RUN set by the driver.
repo=/home/marcel/Projekte/chora
cd "$repo" || exit 1
bin=$repo/target/debug
findings=$repo/tests/battle/findings.md
export PATH=/home/agent/.local/share/mamba/envs/chora/bin:$PATH
# software EGL for the headless backend (system libEGL is not installed)
export LD_LIBRARY_PATH=/home/agent/.local/opt/mesa-egl/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
export __EGL_VENDOR_LIBRARY_FILENAMES=/home/agent/.local/opt/mesa-egl/usr/share/glvnd/egl_vendor.d/50_mesa.json
export GALLIUM_DRIVER=llvmpipe LIBGL_ALWAYS_SOFTWARE=1

step() { echo "-- $*" >> "$RUN"; }
ctl() { timeout 10 "$bin/choractl" --name agent "$@"; }
admin() { timeout 10 "$bin/choractl" "$@"; }

# health <what just happened>  -> a finding with severity crash if the core
# does not answer within 5 s.
health() {
  if ! timeout 5 "$bin/choractl" --name agent query >/dev/null 2>&1; then
    finding "core dead after: $*" crash "health check during $WF"
    return 1
  fi
}

# finding <title> <severity> <steps...>   (workflow name: global WF)
finding() {
  local log=-
  [ -n "${CHORA_LOG:-}" ] && [ -f "$CHORA_LOG" ] && log=$(tail -5 "$CHORA_LOG" | tr '\n' ' | ')
  {
    printf '## %s\nworkflow: %s   severity: %s\n' "$1" "${WF:-?}" "$2"
    shift 2
    printf 'steps: %s\n' "$*"
    printf 'expected: core stays correct   actual: see title\nlog: %s\nstatus: open\n\n' "$log"
  } >> "$findings"
  echo "FINDING: $1 ($2)" >> "$RUN"
}

# session_start <outputs> [plugin args...]  -> hl.env sourced, $HLPID set
session_start() {
  local outputs=$1; shift
  # the sandbox shares a tight box: wait for memory instead of thrashing
  local waited=0 free
  while :; do
    free=$(awk '/MemAvailable/ {print int($2/1048576)}' /proc/meminfo)
    [ "$free" -ge 2 ] && break
    [ "$waited" -ge 600 ] && break
    sleep 30; waited=$((waited+30))
  done
  hlenv=/tmp/bt-$WF.env
  : > "$hlenv"
  export CHORA_LOG=$repo/tests/battle/chora-$WF.log
  tests/headless-session.sh "$outputs" ${1:+-- "$@"} > "$hlenv" 2> "$hlenv.err" &
  HLPID=$!
  for _ in $(seq 120); do grep -q CHORA_SOCKET "$hlenv" && break; sleep 0.5; done
  grep -q CHORA_SOCKET "$hlenv" || { finding "session did not start" crash "setup" "headless-session.sh $outputs -- $*"; return 1; }
  set -a; . "$hlenv"; set +a
  sleep 2  # the plugin needs a moment before its nodes answer queries
  admin grant agent process.spawn screen.capture input.inject >> "$RUN" 2>&1
  health "session started"
}

session_stop() {
  [ -n "$HLPID" ] && kill "$HLPID" 2>/dev/null
  wait "$HLPID" 2>/dev/null
  HLPID=
}

# open <r g b>  -> one probe window owned by agent, prints its pid
open() { ctl spawn "$bin/chora-probe" window "$@" | sed -n 's/.*"pid": \([0-9]*\).*/\1/p'; sleep 0.4; }

# count_windows -> number of chora-probe toplevels
count_windows() { ctl query "app_id:chora-probe kind:surface" | grep -c '"id"'; }

# inject <json>
inject() { echo "$1" | timeout 10 "$bin/choractl" --name agent inject; }

# stopfile: Marcel stops everything with PHASE=done
stopped() { grep -q PHASE=done "$repo/target/review-loop/state.txt"; }
