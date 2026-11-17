#!/bin/bash
# Smoke test of the persona driver: a tiling session, a look, Super+Return
# opens foot, a look, typing into it; the two captures must differ.
#
# Usage: tests/persona/smoke.sh
# Needs: target/debug binaries, foot, waybar, swaybg, jq, setsid and
# dbus-run-session.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
for tool in foot waybar swaybg jq setsid dbus-run-session; do
    command -v "$tool" >/dev/null || {
        echo "FAIL: $tool is needed" >&2
        exit 1
    }
done
dir=$(mktemp -d /tmp/persona-smoke.XXXXXX)
spid=
# shellcheck disable=SC2329 # runs as the EXIT trap
cleanup() {
    trap - EXIT
    if [[ -n $spid ]]; then
        kill -TERM "$spid" 2>/dev/null || true
        wait "$spid" 2>/dev/null || true
    fi
    rm -rf "$dir"
}
trap cleanup EXIT
fail() {
    echo "FAIL: $*" >&2
    exit 1
}

mkdir "$dir/s"
touch "$dir/s/sentinel"
echo '{"scale": 0.1, "origin": [0, 0]}' >"$dir/s/last_look.json"
mkdir "$dir/full"
touch "$dir/full/file"
PERSONA_KEEP=$dir/full "$here/session.sh" tiling >/dev/null 2>"$dir/refuse.err" && fail "a non-empty PERSONA_KEEP was accepted"
grep -q PERSONA_KEEP "$dir/refuse.err" || fail "no message about PERSONA_KEEP: $(cat "$dir/refuse.err")"
echo "keep: a non-empty directory is refused"
PERSONA_DIR=$dir/s PERSONA_KEEP=$dir/keep "$here/session.sh" tiling >"$dir/out" 2>"$dir/err" &
spid=$!
for _ in $(seq 150); do
    [[ -s $dir/out ]] && break
    kill -0 "$spid" 2>/dev/null || fail "session ended: $(cat "$dir/err")"
    sleep 0.2
done
[[ -s $dir/out ]] || fail "no session"
PERSONA_ENV=$(sed -n 's/^PERSONA_ENV=//p' "$dir/out")
export PERSONA_ENV
act() { python3 -I "$here/act.py" "$@"; }
env_value() { sed -n "s/^$1=//p" "$PERSONA_ENV"; }
admin() {
    CHORA_SOCKET=$(env_value CHORA_SOCKET) XDG_RUNTIME_DIR=$(env_value XDG_RUNTIME_DIR) \
        "${CHORA_BIN:-$repo/target/debug}/choractl" "$@"
}
sleep 2 # the plugin needs a moment before it takes keys

top=0
for _ in $(seq 100); do
    top=$(admin outputs | jq '.[0].work_area[1]')
    ((top > 0)) && break
    sleep 0.2
done
((top > 0)) || fail "waybar reserved no space: the work area starts at $top"
[[ ! -e $dir/s/last_look.json ]] || fail "the session kept a stale last_look.json"
echo "waybar: the work area starts at y=$top"

act look | cut -d" " -f1 >"$dir/a.path"
act key Super+Return
sleep 4 # foot starts and draws
act look | cut -d" " -f1 >"$dir/b.path"
act type 'echo hello'
act key Return
sleep 1
a=$(cat "$dir/a.path")
b=$(cat "$dir/b.path")
act look | cut -d" " -f1 >"$dir/c.path"
c=$(cat "$dir/c.path")

out=$(python3 -I "$repo/tests/png-same.py" "$a" "$b") || fail "png-same: $out"
echo "look 1 vs 2: $out"
echo "$out" | grep -qE '^0 ' && fail "the captures do not differ"
out=$(python3 -I "$repo/tests/png-same.py" "$b" "$c") || fail "png-same: $out"
echo "look 2 vs 3: $out"

act key Super+Return
sleep 4
look=$(act look --max-side 640)
act click 480 200
sleep 1
focused_x=$(admin query 'kind:surface' | jq '.[] | select(.focused) | .rects["HEADLESS-1"][0] | floor')
((focused_x > 640)) || fail "a click at 480,200 of a 640 px capture did not focus the right window (x=$focused_x)"
echo "click on the capture: the window at x=$focused_x has the focus"
echo "$look" | grep -q 'scale=0.5' || fail "look printed no scale: $look"

act type -x two words >/dev/null || fail "act.py type refused a text that starts with a dash"

log=$dir/s/actions.jsonl
typed=$(grep '"action": "type"' "$log" | tail -n 1 | jq -r .args.text)
[[ $typed == "-x two words" ]] || fail "typed '$typed', expected '-x two words'"
echo "type: '$typed'"
n=$(wc -l <"$log")
((n == 10)) || fail "the log has $n lines, expected 10"
grep -q '"ok": false' "$log" && fail "a failed action in the log"
# A command outside the list is refused.
act ls >/dev/null 2>&1 && fail "act.py took an unknown command"
echo "log: $n actions, none failed"

cp "$log" "$dir/log.before"
"$here/session.sh" --stop "$PERSONA_ENV"
wait "$spid" || true
spid=
shots=$(find "$dir/keep/shots" -name '*.png' 2>/dev/null | wc -l || true)
((shots == 4)) || fail "PERSONA_KEEP holds $shots captures, expected 4"
cmp -s "$dir/keep/actions.jsonl" "$dir/log.before" || fail "PERSONA_KEEP holds no copy of the action log"
echo "keep: $shots captures and the log"
[[ -e $dir/s/sentinel ]] || fail "the session removed the PERSONA_DIR of its caller"
echo "PERSONA_DIR of the caller is still there"
looks() {
    local flavor=$1 d=$dir/look-$1 pid env_file calls
    mkdir "$d"
    PERSONA_DIR=$d "$here/session.sh" "$flavor" >"$d.out" 2>"$d.err" &
    pid=$!
    for _ in $(seq 150); do
        [[ -s $d.out ]] && break
        kill -0 "$pid" 2>/dev/null || fail "$flavor: session ended: $(cat "$d.err")"
        sleep 0.2
    done
    [[ -s $d.out ]] || fail "$flavor: no session"
    env_file=$(sed -n 's/^PERSONA_ENV=//p' "$d.out")
    ctl() {
        CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$env_file") XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$env_file") \
            "${CHORA_BIN:-$repo/target/debug}/choractl" "$@"
    }
    session_path=$(sed -n 's/^PATH=//p' "$env_file")
    calls=$d/fake/calls.log
    ctl wait owner:assistant --timeout 20 >/dev/null 2>&1 || fail "$flavor: chora-assistant is not running"
    ctl wait id:hud-figure --timeout 20 >/dev/null 2>&1 || fail "$flavor: the assistant draws no figure"
    ctl call voice state '{}' >/dev/null 2>&1 || fail "$flavor: fake-voice does not answer"
    PATH=$session_path notify-send persona-smoke body || fail "$flavor: notify-send failed"
    grep -q 'notify-send persona-smoke' "$calls" 2>/dev/null || fail "$flavor: notify-send did not reach the stub"
    PATH=$session_path loginctl lock-session || fail "$flavor: loginctl failed"
    grep -q 'loginctl lock-session' "$calls" || fail "$flavor: loginctl lock-session did not reach the fake"
    echo "$flavor: assistant, voice, notify-send and lock-session are in place"
    "$here/session.sh" --stop "$env_file"
    wait "$pid" || true
}
for flavor in tiling xp mc construct; do
    looks "$flavor"
done
echo "PASS"
