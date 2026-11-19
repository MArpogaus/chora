#!/bin/bash
# Gate of the injected input that a session refuses (review security a
# r1, r2, r3): with test_accept_injected_keys off, a click or a key of
# input.inject acts for no person. Each cell has a control that turns the
# switch on and shows that the same input acts then. A negative check
# waits for a round trip to the plugin (settings.reload), which the
# plugin answers after the input that came before it, not for a time.
#
# 1. The construct over tiling: the window list stands open (opened with
#    the switch on, which is then set off with settings.set); an injected
#    Return and an injected click on a row take no window, and with the
#    switch on each takes the window as a pane.
# 2. tiling and xp: two windows, the second has the focus; an injected
#    click on the first (xp: on its task button) leaves the focus, a click with the switch on moves it.
# 3. tiling with focus: follow_mouse (its default): an injected pointer move
#    over the first window leaves the focus, a move with the switch on moves it.
#
# Usage: tests/gate-injected-presses.sh   BIN: directory with the binaries
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-tiling chora-xp chora-probe
temp_config
work=$XDG_CONFIG_HOME
mkdir -p "$work/chora/constructs"
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
printf 'panes: []\n' >"$work/chora/constructs/empty.yaml"

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
call() { ctl call "$@"; }
barrier() { call "$1" settings.reload >/dev/null || fail "$1 does not answer"; }
click_on() { inject "[{\"pointer_move\": {\"position\": [$(center "$1")]}}, {\"click\": {}}]"; }
inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
key() { inject "[{\"shortcut\": {\"keys\": \"$1\"}}]"; }
end_session() {
    kill "${pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
}
center() {
    ctl query "id:$1" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
x, y, w, h = next(iter(nodes[0]["rects"].values()))
print(f"{x + w / 2:.1f}, {y + h / 2:.1f}")'
}
probes=0
taken() { call construct window.list | python3 -I -c 'import json, sys; print(next((w["pane"] for w in json.load(sys.stdin)["windows"] if w["name"] == "chora-probe"), "missing"))'; }
list_with_probe() {
    local switch=$1
    printf 'test_accept_injected_keys: true\n' >"$work/chora/chora-construct.yaml"
    session_start 1280x800 -- "$bin/chora-tiling"
    grant input.inject methods.call methods.privileged
    "$bin/chora-probe" window 200 40 40 >"$work/probe-$probes.log" 2>&1 &
    pids+=($!)
    probes=$((probes + 1))
    until_ok 'ctl query "kind:surface owner:tiling" | grep -q app_id' 80 || fail "tiling took no window"
    "$bin/chora-construct" --overlay >"$work/construct-$probes.log" 2>&1 &
    pids+=($!)
    until_ok 'call construct construct.list >/dev/null 2>&1' || fail "the overlay does not answer"
    call construct construct.load '{"name": "empty"}' >/dev/null || fail "construct.load"
    ctl wait --timeout 10 id:construct-look >/dev/null || fail "no space over tiling"
    until_ok '[ "$(taken)" = none ]' || fail "the probe is $(taken) before the list opens"
    barrier construct
    key Tab
    until_ok 'ctl wait --timeout 2 "id:construct-list meta:state=open" >/dev/null 2>&1' 5 || fail "Tab opened no list"
    if [ "$switch" = off ]; then
        call construct settings.set '{"test_accept_injected_keys": false}' >/dev/null || fail "settings.set"
        barrier construct
    fi
}

CHORA_TEST_ACCEPT_INJECTED_KEYS=0 list_with_probe off
key Return
barrier construct
[ "$(taken)" = none ] || fail "an injected Return took the window: $(taken)"
click_on construct-list-row-0
barrier construct
[ "$(taken)" = none ] || fail "an injected click on a row took the window: $(taken)"
ok "the window list: an injected Return and an injected click take no window"
end_session

CHORA_TEST_ACCEPT_INJECTED_KEYS=1 list_with_probe on
key Return
until_ok '[ "$(taken)" != none ]' || fail "with the switch on Return took no window"
ok "the window list: with the switch on Return takes the window as a pane"
end_session

CHORA_TEST_ACCEPT_INJECTED_KEYS=1 list_with_probe on
click_on construct-list-row-0
until_ok '[ "$(taken)" != none ]' || fail "with the switch on a click on a row took no window"
ok "the window list: with the switch on a click on a row takes the window as a pane"
end_session

two_windows() {
    local wm=$1
    session_start 1280x800 -- "$bin/chora-$wm"
    grant input.inject methods.call methods.privileged
    for color in 200 40; do
        "$bin/chora-probe" window "$color" 40 40 >"$work/probe-$wm-$color.log" 2>&1 &
        pids+=($!)
        until_ok '[ "$(ctl query "app_id:chora-probe kind:surface" | grep -c "\"id\"")" -ge '"$((color == 200 ? 1 : 2))"' ]' 80 || fail "$wm took no window"
    done
    until_ok '[ "$(unfocused)" = s1 ]' 80 || fail "$wm: the second window has no focus"
}
unfocused() {
    ctl query "app_id:chora-probe kind:surface" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
if len(nodes) == 2 and sum(1 for n in nodes if n.get("focused")) == 1:
    print(next(n["id"] for n in nodes if not n.get("focused")))'
}
focus_cell() {
    local wm=$1 switch=$2 target
    printf 'test_accept_injected_keys: %s\n' "$([ "$switch" = on ] && echo true || echo false)" >"$work/chora/chora-$wm.yaml"
    [ "$wm" = tiling ] && echo 'focus: click' >>"$work/chora/chora-$wm.yaml"
    CHORA_TEST_ACCEPT_INJECTED_KEYS=$([ "$switch" = on ] && echo 1 || echo 0) two_windows "$wm"
    target=$(unfocused)
    case "$wm" in
    xp) click_on "xp-task-$target-HEADLESS-1" ;;
    *) click_on "$target" ;;
    esac
    barrier "$wm"
    if [ "$switch" = on ]; then
        until_ok '[ -n "$(unfocused)" ] && [ "$(unfocused)" != "$target" ]' || fail "$wm: with the switch on the click did not focus $target"
        ok "$wm: with the switch on a click on the other window focuses it"
    else
        [ "$(unfocused)" = "$target" ] || fail "$wm: an injected click focused $target"
        ok "$wm: an injected click on the other window leaves the focus"
    fi
    end_session
}
for wm in tiling xp; do
    focus_cell "$wm" off
    focus_cell "$wm" on
done

follow_cell() {
    local switch=$1 target
    printf 'test_accept_injected_keys: %s\n' "$([ "$switch" = on ] && echo true || echo false)" >"$work/chora/chora-tiling.yaml"
    CHORA_TEST_ACCEPT_INJECTED_KEYS=$([ "$switch" = on ] && echo 1 || echo 0) two_windows tiling
    target=$(unfocused)
    inject "[{\"pointer_move\": {\"position\": [$(center "$target")]}}]"
    barrier tiling
    if [ "$switch" = on ]; then
        until_ok '[ -n "$(unfocused)" ] && [ "$(unfocused)" != "$target" ]' || fail "tiling: with the switch on a move did not focus $target"
        ok "tiling with follow_mouse: with the switch on an injected move focuses the window under it"
    else
        [ "$(unfocused)" = "$target" ] || fail "tiling: an injected move focused $target with follow_mouse"
        ok "tiling with follow_mouse: an injected move over the other window leaves the focus"
    fi
    end_session
}
follow_cell off
follow_cell on
