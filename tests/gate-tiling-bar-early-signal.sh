#!/bin/bash
# A signal that chora-tiling sends to waybar right after waybar.sh wrote
# waybar.pid (SIGRTMIN+1 after a change of the tags, SIGUSR1 for a
# fullscreen window) does not end a waybar that has not yet installed its
# handlers. A stand-in for waybar installs them only after 1.5 s; the
# signals arrive at once, and the stand-in must still be alive and
# answer a signal that arrives after the handlers.
#
# Usage: tests/gate-tiling-bar-early-signal.sh
set -u
. "$(dirname "$0")/gate-lib.sh"
need python3
need jq
temp_config
work=$XDG_CONFIG_HOME
export XDG_RUNTIME_DIR=$work/run
# shellcheck disable=SC2174
mkdir -p -m 700 "$XDG_RUNTIME_DIR" "$work/fakes" "$work/flavor"
cat >"$work/fakes/choractl" <<'EOF'
#!/bin/sh
case "$1" in
    call) echo '{"outputs":[{"name":"X"}]}' ;;
    outputs) echo '[{"name":"X"}]' ;;
esac
EOF
cat >"$work/fakes/waybar" <<'EOF'
#!/usr/bin/python3 -I
import os, signal, sys, time
log = os.environ["FAKE_WAYBAR_LOG"]
time.sleep(1.5)
def answer(number, frame):
    open(log, "a").write("answered\n")
signal.signal(signal.SIGRTMIN + 1, answer)
open(log, "a").write("ready\n")
while True:
    time.sleep(0.1)
EOF
chmod +x "$work/fakes/choractl" "$work/fakes/waybar"
mkdir -p "$work/flavor/icons"
touch "$work/flavor/icons/a.svg" "$work/flavor/style.css" "$work/flavor/colors.css" "$work/flavor/config.jsonc"
export FAKE_WAYBAR_LOG=$work/waybar.log
PATH=$work/fakes:$PATH setsid sh "$repo/flavors/tiling/waybar/waybar.sh" "$work/flavor" >"$work/panel.log" 2>&1 &
run_pid $!
panel_pids=$XDG_RUNTIME_DIR/chora-tiling
for _ in $(seq 50); do
    [ -s "$panel_pids/waybar.pid" ] && break
    sleep 0.1
done
waybar_pid=$(cat "$panel_pids/waybar.pid" 2>/dev/null) || fail "waybar.sh wrote no waybar.pid"
run_pid "$waybar_pid"
kill -s RTMIN+1 "$waybar_pid"
kill -s USR1 "$waybar_pid"
for _ in $(seq 50); do
    grep -q ready "$work/waybar.log" 2>/dev/null && break
    sleep 0.1
done
grep -q ready "$work/waybar.log" 2>/dev/null || fail "waybar did not survive the early signals: panel says $(cat "$work/panel.log")"
kill -s RTMIN+1 "$waybar_pid"
for _ in $(seq 20); do
    grep -q answered "$work/waybar.log" && break
    sleep 0.1
done
grep -q answered "$work/waybar.log" || fail "waybar does not answer a signal after its start"
ok "waybar survives signals before its handlers and answers after them"
