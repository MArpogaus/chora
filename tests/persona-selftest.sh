#!/bin/bash
# Selftest of the persona session (tests/persona/session.sh): the programs
# that a click can run on a host (systemctl, wpctl, grim, loginctl) are
# stand-ins in the PATH of the session and write `NAME ARGS` to
# PERSONA_CALLS, so a persona that picks Stand By never suspends the host;
# a session can have a second look (FLAVOR+FLAVOR); the client persona-check
# has the grants that tests/persona/check.py needs.
#
# Usage: tests/persona-selftest.sh   BIN: the binaries (default target/debug)
set -u
. "$(dirname "$0")/gate-lib.sh"
need jq
need_bins chora choractl chora-mc chora-tiling chora-assistant chora-xp chora-notify chora-probe

dir=$(mktemp -d /tmp/persona-selftest.XXXXXX)
tmp_dirs+=("$dir")
spid=
stop_session() {
    [ -z "$spid" ] || {
        "$repo/tests/persona/session.sh" --stop "$dir/session.env" >/dev/null 2>&1
        wait "$spid" 2>/dev/null
        spid=
    }
}
trap 'status=$?; stop_session; gate_cleanup $status' EXIT

until_ok() {
    local _
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
start() {
    PERSONA_DIR=$dir CHORA_BIN=$bin "$repo/tests/persona/session.sh" "$@" >"$dir.out" 2>"$dir.err" &
    spid=$!
    until_ok "grep -q '^PERSONA_ENV=' '$dir.out'" 150 || fail "the session $* did not start: $(cat "$dir.err")"
    env_value() { sed -n "s/^$1=//p" "$dir/session.env"; }
}
admin() {
    CHORA_SOCKET=$(env_value CHORA_SOCKET) XDG_RUNTIME_DIR=$(env_value XDG_RUNTIME_DIR) "$bin/choractl" "$@"
}

start mc
calls=$(env_value PERSONA_CALLS)
session_path=$(env_value PATH)
for what in systemctl wpctl grim loginctl notify-send; do
    found=$(PATH=$session_path command -v "$what" || true)
    case $found in
    "$dir"/fake/*) ;;
    *) fail "$what in the PATH of the session is $found, not a stand-in of the session" ;;
    esac
done
core_path=$(tr '\0' '\n' <"/proc/$(env_value CHORA_CORE_PID)/environ" | sed -n 's/^PATH=//p')
case $core_path in
"$dir"/fake:*) ;;
*) fail "the PATH of the core, which starts the programs of a click, is $core_path" ;;
esac
ok "systemctl, wpctl, grim, loginctl and notify-send are stand-ins, first in the PATH of the core"

: >"$calls"
PATH=$session_path systemctl suspend || fail "the stand-in systemctl failed"
grep -qx 'systemctl suspend' "$calls" || fail "systemctl suspend did not reach PERSONA_CALLS: $(cat "$calls")"
ok "systemctl suspend reaches the stand-in"
volume=$(PATH=$session_path wpctl get-volume @DEFAULT_AUDIO_SINK@)
[ "$volume" = "Volume: 0.50" ] || fail "wpctl get-volume answered '$volume'"
PATH=$session_path wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
grep -qx 'wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+' "$calls" || fail "wpctl did not reach PERSONA_CALLS: $(cat "$calls")"
PATH=$session_path grim "$dir/shot.png"
[ -e "$dir/shot.png" ] || fail "grim wrote no file"
grep -qx "grim $dir/shot.png" "$calls" || fail "grim did not reach PERSONA_CALLS: $(cat "$calls")"
ok "wpctl answers and logs, grim writes its file and logs"

mapfile -d '' core_env <"/proc/$(env_value CHORA_CORE_PID)/environ"
host_before=$(find "$HOME" -path '*Screenshots/*.png' 2>/dev/null | sort)
env "${core_env[@]}" sh "$repo/flavors/shared/screenshot.sh" >/dev/null 2>&1
shot=$(find "$dir" -path '*Screenshots/*.png' | head -n1)
host_new=$(find "$HOME" -path '*Screenshots/*.png' 2>/dev/null | sort | comm -13 <(echo "$host_before") - | head -n1)
[ -n "$shot" ] || fail "screenshot.sh wrote no file under $dir (new on the host: ${host_new:-none})"
[ -z "$host_new" ] || fail "screenshot.sh wrote $host_new on the host"
ok "screenshot.sh in the environment of the core writes under the session directory"

grants=$(admin grants)
for method in screen.capture methods.call; do
    echo "$grants" | jq -e --arg m "$method" '.["persona-check"] // [] | index($m)' >/dev/null ||
        fail "persona-check has no grant of $method: $grants"
done
echo "$grants" | jq -e '.["persona-check"] // [] | index("input.inject") | not' >/dev/null ||
    fail "persona-check may inject input"
ok "persona-check has screen.capture and methods.call, not input.inject"

stop_session
rm -rf "$dir"
mkdir "$dir"
start mc+tiling
client_names() {
    python3 -I -c '
import sys
sys.path.insert(0, sys.argv[1] + "/tests/acts")
from harness import Core
print(" ".join(sorted(c["name"] for c in Core(sys.argv[2], name="selftest").call("clients.list"))))
' "$repo" "$(env_value CHORA_SOCKET)"
}
until_ok "client_names | grep -qw mc && client_names | grep -qw tiling" 80 ||
    fail "mc+tiling: the clients are $(client_names)"
[ "$(env_value PERSONA_FLAVORS)" = "mc+tiling" ] || fail "PERSONA_FLAVORS is '$(env_value PERSONA_FLAVORS)'"
ok "mc+tiling starts both looks"
stop_session

rm -rf "$dir"
mkdir "$dir"
start xp
calls=$(env_value PERSONA_CALLS)
session_path=$(env_value PATH)
mapfile -d '' core_env <"/proc/$(env_value CHORA_CORE_PID)/environ"
state() { admin call xp state.get '{}'; }
wayland=$(sed -n 's/^WAYLAND_DISPLAY=//p' "$dir/headless.out")
run_in_session() { PATH=$session_path WAYLAND_DISPLAY=$wayland admin run "$@" >/dev/null; }
until_ok "state >/dev/null 2>&1" 80 || fail "the look xp does not answer state.get"

shared=$(printf '%s\n' "${core_env[@]}" | sed -n 's/^CHORA_SHARED_DIR=//p')
[ -n "$shared" ] || fail "the core has no CHORA_SHARED_DIR: the look cannot find power.sh and screenshot.sh"
for script in power.sh screenshot.sh; do
    [ -f "$shared/$script" ] || fail "$shared has no $script"
done
case $shared in
"$repo"/*) fail "CHORA_SHARED_DIR $shared is in the checkout, not installed like a package" ;;
esac
found=$(PATH=$session_path command -v swaylock || true)
case $found in
"$dir"/fake/*) ;;
*) fail "swaylock in the PATH of the session is '$found', not a stand-in" ;;
esac
: >"$calls"
env "${core_env[@]}" sh "$shared/power.sh" lock || fail "power.sh lock failed"
grep -Eq '^(swaylock|loginctl lock-session)' "$calls" || fail "power.sh lock left no mark in PERSONA_CALLS: $(cat "$calls")"
ok "shared scripts installed under the session, lock leaves a mark"

found=$(PATH=$session_path command -v mousepad || true)
case $found in
"$dir"/fake/*) ;;
*) fail "mousepad in the PATH of the session is '$found', not a stand-in" ;;
esac
click_node() {
    local at cx cy
    at=$(admin query "id:$1" | jq -er '.[0].rects | to_entries[0].value | "\(.[0] + .[2] / 2 | floor) \(.[1] + .[3] / 2 | floor)"') || return 1
    read -r cx cy <<<"$at"
    PERSONA_ENV=$dir/session.env python3 -I "$repo/tests/persona/act.py" click "$cx" "$cy" >/dev/null
}
click_node xp-start-HEADLESS-1 || fail "no Start button"
notepad=$(admin query '*' | jq -r '.[] | select(.meta.text == "Notepad") | .id' | head -n1)
[ -n "$notepad" ] || fail "the Start menu has no Notepad"
click_node "$notepad" || fail "no click on Notepad"
until_ok "state | jq -e 'any(.windows[]; .title == \"Untitled - Notepad\")' >/dev/null" 40 ||
    fail "mousepad shows no window titled Untitled - Notepad: $(state | jq -c '[.windows[].title]')"
ok "mousepad stand-in starts a window Untitled - Notepad"

run_in_session persona-probe
until_ok "state | jq -e 'any(.windows[]; .title == \"Probe window\")' >/dev/null" 40 ||
    fail "persona-probe shows no window titled Probe window: $(state | jq -c '[.windows[].title]')"
surface=$(state | jq -r '.windows[] | select(.title == "Probe window") | .surface')
rect() { state | jq -c --arg s "$surface" '.windows[] | select(.surface == $s) | .rect'; }
before=$(rect)
title=$(admin query "id:xp-title-$surface" | jq -er '.[0].rects | to_entries[0].value | "\(.[0] + .[2] / 2 | floor) \(.[1] + .[3] / 2 | floor)"') ||
    fail "the probe window has no title bar node xp-title-$surface"
read -r tx ty <<<"$title"
PERSONA_ENV=$dir/session.env python3 -I "$repo/tests/persona/act.py" drag "$tx" "$ty" "$((tx + 300))" "$((ty + 150))" >/dev/null ||
    fail "act.py drag failed"
until_ok "[ \"\$(rect)\" != '$before' ]" 20 || fail "the probe window did not move: still $before"
ok "the probe window has a title bar and a drag moves it ($before to $(rect))"

until_ok "[ \"\$(state | jq -r .helper)\" = idle ]" 40 || fail "the helper is $(state | jq -c .helper), not idle, before anything was said"
admin call voice agent '{"tool": "agent_say", "name": "atlas", "text": "hello"}' >/dev/null ||
    fail "the call agent_say failed"
until_ok "[ \"\$(state | jq -r .helper)\" = working ]" 20 || fail "the helper never worked after hello: $(state | jq -c .helper)"
until_ok "[ \"\$(state | jq -r .helper)\" = idle ]" 80 || fail "the helper did not rest again: $(state | jq -c .helper)"
jq -e 'any(.[]; .role == "agent" and (.text | length > 0))' "$dir/voice/entries.json" >/dev/null ||
    fail "no agent entry answers hello: $(cat "$dir/voice/entries.json")"
ok "the helper works on hello, rests and answers"

stop_session
ok "persona session"
