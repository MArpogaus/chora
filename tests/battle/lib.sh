#!/bin/bash
# Shared helpers for the battle workflows. Source this, then call
# session_start, health, session_stop. Run files: $RUN set by the driver.
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$repo" || exit 1
bin=${BIN:-$repo/target/debug}
findings=$repo/tests/battle/findings.md
mamba=$HOME/.local/share/mamba/envs/chora/bin
[ -d "$mamba" ] && export PATH=$mamba:$PATH
# plugins start apps with `choractl run` from PATH (D42)
export PATH=$bin:$PATH
# no user manager of the host: run starts children that the session ends
unset DBUS_SESSION_BUS_ADDRESS
# software EGL for the headless backend where the system has no libEGL
egl=$HOME/.local/opt/mesa-egl
if [ -d "$egl" ]; then
  export LD_LIBRARY_PATH=$egl/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
  export __EGL_VENDOR_LIBRARY_FILENAMES=$egl/usr/share/glvnd/egl_vendor.d/50_mesa.json
fi
export GALLIUM_DRIVER=llvmpipe LIBGL_ALWAYS_SOFTWARE=1

step() { echo "-- $*" >> "$RUN"; }
# xp_settings writes the settings of chora-xp without sounds to
# /tmp/bt-$WF-xp.yaml, apart from the settings of the user.
xp_settings() {
  sed 's/^  sounds: true/  sounds: false/' plugins/chora-xp/xp.yaml > "/tmp/bt-$WF-xp.yaml"
}
ctl() { timeout 10 "$bin/choractl" --name agent "$@"; }
admin() { timeout 10 "$bin/choractl" "$@"; }
# restart_plugin <name> ends a plugin that keep.sh runs with the pid file
# /tmp/bt-$WF-<name>.pid; keep.sh starts it again.
restart_plugin() {
  local file=/tmp/bt-$WF-$1.pid old
  old=$(cat "$file") && kill "$old" || return 1
  # wait for the new pid, so that the next restart ends the new plugin
  for _ in $(seq 50); do [ "$(cat "$file")" != "$old" ] && return 0; sleep 0.1; done
  return 1
}

# health <what just happened>  -> a finding with severity crash if the core
# does not answer within 5 s.
health() {
  if ! timeout 5 "$bin/choractl" --name agent query >/dev/null 2>&1; then
    # Dead or only hung? The process facts go into the finding, not into
    # its title, so repeats of this finding stay one title.
    local core state=
    core=$(pgrep -P "$HLPID" -x chora)
    if [ -n "$core" ]; then
      state=$(awk '/^(State|VmRSS|VmSwap):/ {printf "%s=%s ", $1, $2 $3}' "/proc/$core/status" 2>/dev/null)
      state="$state wchan=$(head -c 100 "/proc/$core/wchan" 2>/dev/null)"
    else
      state=no-process
    fi
    finding "core dead after: $*" crash "health check during $WF" "process: $state"
    # W9-W12 end at the first death instead of piling up findings
    if [ -n "${STOP_ON_DEATH:-}" ]; then log_scan; session_stop; exit 1; fi
    return 1
  fi
}

# finding <title> <severity> <steps...>   (workflow name: global WF)
finding() {
  local log=- title=$1 severity=$2
  [ -n "${CHORA_LOG:-}" ] && [ -f "$CHORA_LOG" ] && log=$(tail -5 "$CHORA_LOG" | tr '\n' ' | ')
  {
    printf '## %s\nworkflow: %s   severity: %s\n' "$1" "${WF:-?}" "$2"
    shift 2
    printf 'steps: %s\n' "$*"
    printf 'expected: core stays correct   actual: see title\nlog: %s\nstatus: open\n\n' "$log"
  } >> "$findings"
  echo "FINDING: $title ($severity)" >> "$RUN"
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
  : > "/tmp/bt-$WF.pids"
  export CHORA_LOG=$repo/tests/battle/chora-$WF.log
  tests/headless-session.sh "$outputs" ${1:+-- "$@"} > "$hlenv" 2> "$hlenv.err" &
  HLPID=$!
  for _ in $(seq 120); do grep -q CHORA_SOCKET "$hlenv" && break; sleep 0.5; done
  grep -q CHORA_SOCKET "$hlenv" || { finding "session did not start" crash "setup" "headless-session.sh $outputs -- $*"; return 1; }
  set -a
  # shellcheck disable=SC1090 # the env file is written by headless-session.sh
  . "$hlenv"
  set +a
  # the core of this session alone, for health and kill-session.sh
  pgrep -P "$HLPID" -x chora > "/tmp/bt-$WF.core"
  sleep 2  # the plugin needs a moment before its nodes answer queries
  admin grant agent screen.capture input.inject >> "$RUN" 2>&1
  health "session started"
}

session_stop() {
  [ -n "$HLPID" ] && kill "$HLPID" 2>/dev/null
  wait "$HLPID" 2>/dev/null
  HLPID=
  # `choractl run` starts its programs in the process group of this
  # script (D42), outside the group of the core, so this script ends them. Their pids are
  # in a file because `$(open ...)` runs `open` in a subshell.
  # shellcheck disable=SC2046 # no pids must give no argument; they contain no blanks
  kill $(cat "/tmp/bt-$WF.pids" 2>/dev/null) 2>/dev/null
  : > "/tmp/bt-$WF.pids"
}

# open <r g b>  -> one probe window owned by agent, prints its pid.
# Safe in `$(open ...)`: the pid reaches session_stop through the file.
open() {
  local pid
  pid=$(ctl run "$bin/chora-probe" window "$@" | sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p')
  echo "$pid" >> "/tmp/bt-$WF.pids"
  echo "$pid"
  sleep 0.4
}

# count_windows -> number of chora-probe toplevels
count_windows() { ctl query "app_id:chora-probe kind:surface" | grep -c '"id"'; }

# wait_sel <selector> [secs] -> wait until the selector matches (choractl wait)
wait_sel() {
  local sel=$1 secs=${2:-10}
  timeout "$((secs + 5))" "$bin/choractl" --name agent wait "$sel" \
    --timeout "$secs" >/dev/null 2>&1
}

# wait_count <selector> <n>  -> wait until at least n nodes match
wait_count() {
  for _ in $(seq 100); do
    [ "$(ctl query "$1" | grep -c '"id"')" -ge "$2" ] && return 0
    sleep 0.2
  done
  return 1
}

# ready <client> <method> [params]  -> wait until the plugin answers calls
# to one of its methods again; the core rejects them while a restart runs
ready() {
  local client=$1; shift
  for _ in $(seq 75); do
    ctl call "$client" "$@" >/dev/null 2>&1 && return 0
    sleep 0.2
  done
  return 1
}

# inject <json>
inject() { echo "$1" | timeout 10 "$bin/choractl" --name agent inject; }

# stopfile: Marcel stops everything with PHASE=done
stopped() { grep -q PHASE=done "$repo/target/review-loop/state.txt"; }

# --- helpers of W9-W12 ---------------------------------------------------
# captures and log copies; SHOTS moves them
shots=${SHOTS:-$repo/target/battle-round}
mkdir -p "$shots"
# config_home: a config home of the workflow, so that settings.set and
# scene.save never write the files of the user
config_home() {
  export XDG_CONFIG_HOME=/tmp/bt-$WF-config
  rm -rf "$XDG_CONFIG_HOME"
  mkdir -p "$XDG_CONFIG_HOME/chora"
}
# rss <process name> -> RSS in kB of that process of this session: the
# core leads its own session (setsid), other sessions on the box differ
# wait_bg -> waits for the background jobs of the workflow; a bare
# `wait` would also wait for the session itself
wait_bg() {
  local j
  for j in $(jobs -p); do [ "$j" = "$HLPID" ] || wait "$j"; done
}
rss() {
  local core p
  core=$(pgrep -P "$HLPID" -x chora) || return 0
  for p in $(pgrep -s "$core" -x "$1"); do
    awk '/^VmRSS:/ {print $2}' "/proc/$p/status" 2>/dev/null
  done | tail -1
}
rss_note() { step "rss $1: chora=$(rss chora) tiling=$(rss chora-tiling) xp=$(rss chora-xp) mc=$(rss chora-mc)"; }
# rss_growth <start kB> <end kB> <what> -> a finding past +50 % and +30 MB
rss_growth() {
  [ -n "$1" ] && [ -n "$2" ] || return 0
  if [ "$2" -gt $(($1 * 3 / 2)) ] && [ $(($2 - $1)) -gt 30000 ]; then
    finding "$WF: $3 RSS grew from $1 kB to $2 kB" slow "rss at start and end of $WF"
  fi
}
# log_scan -> a finding for panics in the core log; copies the log
log_scan() {
  [ -f "$CHORA_LOG" ] || return 0
  local n
  n=$(grep -c "panicked" "$CHORA_LOG")
  if [ "$n" -gt 0 ]; then
    finding "$WF: $n panics in the log: $(grep -m1 -A1 panicked "$CHORA_LOG" | tr '\n' ' ' | cut -c1-160)" crash "see $shots/chora-$WF.log"
  fi
  step "watchdog lines: $(grep -ci watchdog "$CHORA_LOG")"
  cp "$CHORA_LOG" "$shots/chora-$WF.log"
}
# shot <name> [output] -> captures into $shots and prints the path
shot() {
  local f=$shots/$WF-$1.png
  ctl capture "$f" --output "${2:-${OUT:-HEADLESS-1}}" >/dev/null 2>&1 && echo "$f"
}
# alive_frames <label> <command...> -> a finding when the output shows
# the same picture before and after a command that changes it
alive_frames() {
  local label=$1 a b
  shift
  a=$(shot "$label-a") || { finding "$WF: capture failed ($label)" wrong "choractl capture"; return 0; }
  "$@" >/dev/null 2>&1
  sleep 1.5
  b=$(shot "$label-b") || { finding "$WF: capture failed ($label)" wrong "choractl capture"; return 0; }
  if cmp -s "$a" "$b"; then
    finding "$WF: output frozen: same picture after $label" wrong "capture, $*, capture"
  fi
}
