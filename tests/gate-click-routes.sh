#!/bin/bash
# Gate of the click that chooses a window manager (D55): chora-mc manages
# the left output and has the keyboard focus, chora-tiling manages the
# right output, which is empty.
#
# 1. With the flag --test-injected-drag-gives of the core (headless
#    only), where an injected press counts as one of the user: after a
#    click on the right output the next new window is a window of tiling
#    on that output.
# 2. Without the flag an injected click chooses nobody: the next new
#    window goes to mc, which has the focus.
#
# Usage: tests/gate-click-routes.sh   BIN: directory with the binaries
# shellcheck disable=SC2016
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-mc chora-tiling chora-probe
temp_config
work=$XDG_CONFIG_HOME

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
windows() {
    ctl query "app_id:chora-probe kind:surface" | python3 -I -c '
import json, sys
print(" ".join(sorted("%s:%s:%s" % (n["id"], n.get("owner"), ",".join(n["rects"]) or "-")
    for n in json.load(sys.stdin))))'
}
claims() {
    ctl query kind:group | python3 -I -c '
import json, sys
print(" ".join(sorted(n["owner"] for n in json.load(sys.stdin) if n.get("claim"))))'
}
open_window() {
    "$bin/chora-probe" window "$1" 120 200 >"$work/probe-$1.log" 2>&1 &
    pids+=($!)
}

click_right() {
    env_file=$work/$1.env session_start "1280x800,1280x800+1280+0" -- \
        sh -c "$bin/chora-mc --manage-windows & $bin/chora-tiling --output HEADLESS-2 & wait"
    grant input.inject
    until_ok 'test "$(claims)" = "mc tiling"' 80 || fail "mc and tiling hold no claims: $(claims)"
    echo '[{"pointer_move": {"position": [600, 400]}}]' | ctl inject >/dev/null || fail "inject"
    open_window 40
    until_ok 'windows | grep -q "^s1:mc:HEADLESS-1$"' 80 || fail "the first window is no window of mc: $(windows)"
    echo '[{"pointer_move": {"position": [600, 400], "output": "HEADLESS-2"}},
        {"button": {"state": "pressed"}}, {"button": {"state": "released"}}]' |
        ctl inject >/dev/null || fail "inject"
    open_window 90
    until_ok 'windows | grep -q "s2:[a-z]"' 80 || fail "the second window has no owner: $(windows)"
    sleep 1
}
end_session() {
    kill "${pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
}

CHORA_TEST_FLAGS=--test-injected-drag-gives click_right chosen
[ "$(windows)" = "s1:mc:HEADLESS-1 s2:tiling:HEADLESS-2" ] ||
    fail "after a click on the empty output of tiling the new window is not there: $(windows)"
ok "after a click on the empty output of tiling the new window is a window of tiling: $(windows)"
end_session

click_right injected
case "$(windows)" in
*s2:mc:*) ;;
*) fail "an injected click chose a window manager: $(windows)" ;;
esac
ok "an injected click chooses nobody, the new window goes to the focused mc: $(windows)"
end_session
