#!/bin/bash
# Smoke test of the persona driver: a tiling session, a look, Super+Return
# opens foot, a look, typing into it; the two captures must differ.
#
# Usage: tests/persona/smoke.sh
# Needs: target/debug binaries and foot.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
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

PERSONA_DIR=$dir/s "$here/session.sh" tiling >"$dir/out" 2>"$dir/err" &
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

log=$dir/s/actions.jsonl
n=$(wc -l <"$log")
((n == 9)) || fail "the log has $n lines, expected 9"
grep -q '"ok": false' "$log" && fail "a failed action in the log"
# A command outside the list is refused.
act ls >/dev/null 2>&1 && fail "act.py took an unknown command"
echo "log: $n actions, none failed"
echo "PASS"
