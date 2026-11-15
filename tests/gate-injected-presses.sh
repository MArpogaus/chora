#!/bin/bash
# Gate of the injected input that a session refuses (review security a
# r1): with test_accept_injected_keys off, a click or a key of
# input.inject acts for no person. Each cell has a control that turns the
# switch on and shows that the same input acts then.
#
# 1. mc: a critical notification stands in the footer; an injected click
#    on its line leaves it, a click with the switch on clears it.
# 2. The construct over tiling: Tab opens the window list; an injected
#    Return and an injected click on a row take no window, with the
#    switch on Return takes the window as a pane.
#
# Usage: tests/gate-injected-presses.sh   BIN: directory with the binaries
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-mc chora-tiling chora-probe
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
urgent_text() {
    call mc region.list | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["contents"]["urgent"]["text"])'
}

press_urgent() {
    session_start 1280x800 -- "$bin/chora-mc"
    grant input.inject methods.call methods.privileged
    call mc notify.push '{"title": "Disk is full", "urgency": "critical"}' >/dev/null || fail "notify.push"
    local hit=mc-r1-footer-urgent-hit
    until_ok "ctl query id:$hit | grep -q rects" || fail "no line for the urgent notification"
    until_ok '[ -n "$(urgent_text)" ]' || fail "the notification did not reach the urgent text"
    inject "[{\"pointer_move\": {\"position\": [$(center "$hit")]}}, {\"click\": {}}]"
    sleep 1
}

CHORA_TEST_ACCEPT_INJECTED_KEYS=0 press_urgent
[ -n "$(urgent_text)" ] || fail "an injected click cleared the urgent notification"
ok "mc: an injected click on the urgent line leaves it"
end_session

CHORA_TEST_ACCEPT_INJECTED_KEYS=1 press_urgent
[ -z "$(urgent_text)" ] || fail "the click did not clear the urgent notification with the switch on: $(urgent_text)"
ok "mc: with the switch on the same click clears the urgent line"
end_session

probes=0
taken() { call construct window.list | python3 -I -c 'import json, sys; print(next((w["pane"] for w in json.load(sys.stdin)["windows"] if w["name"] == "chora-probe"), "missing"))'; }
list_with_probe() {
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
    sleep 1
    [ "$(taken)" = none ] || fail "the probe is $(taken) before the list opens"
    key Tab
    ctl wait --timeout 5 "id:construct-list meta:state=open" >/dev/null || fail "Tab opened no list"
}

CHORA_TEST_ACCEPT_INJECTED_KEYS=0 list_with_probe
key Return
sleep 1
[ "$(taken)" = none ] || fail "an injected Return took the window: $(taken)"
inject "[{\"pointer_move\": {\"position\": [$(center construct-list-row-0)]}}, {\"click\": {}}]"
sleep 1
[ "$(taken)" = none ] || fail "an injected click on a row took the window: $(taken)"
ok "the window list: an injected Return and an injected click take no window"
end_session

CHORA_TEST_ACCEPT_INJECTED_KEYS=1 list_with_probe
key Return
until_ok '[ "$(taken)" != none ]' || fail "with the switch on Return took no window"
ok "the window list: with the switch on Return takes the window as a pane"
end_session
