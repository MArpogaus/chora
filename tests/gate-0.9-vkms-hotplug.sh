#!/bin/bash
# Gate check 5 of 0.9: a second output plugs in and out while chora runs
# on the tty backend with vkms. The core reports output.changed and
# output.removed, a window on the second output moves to the first one,
# and the core lives on. Captures of both outputs go to SHOTS.
#
# The CI job `vkms` runs this as root, with seatd and vkms as the only
# card. A second vkms device comes from configfs (Linux 6.17 and later):
# enabled 1 plugs it in. Where the kernel lets the status of a connector
# change while the device is on, the first unplug is the connector going
# to disconnected and back, as a monitor cable does; the device going
# off is the last unplug. Without configfs the closest test is the one
# connector of card0 forced off and on through sysfs: no second output.
#
# HOTPLUG=headless runs the same steps on the headless backend, with
# `choractl add-output` and `remove-output` as the monitor, for a check
# of the script without root.
#
# Usage: tests/gate-0.9-vkms-hotplug.sh
#   BIN: directory with the binaries, SHOTS: directory for the captures
set -u
. "$(dirname "$0")/gate-lib.sh"
need jq
for what in chora choractl chora-tiling chora-probe; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
mode=${HOTPLUG:-vkms}
XDG_CONFIG_HOME=$(mktemp -d /tmp/gate-hotplug.XXXXXX)
export XDG_CONFIG_HOME
shots=${SHOTS:-$XDG_CONFIG_HOME}
mkdir -p "$shots"
log=$shots/chora.log
cfg=/sys/kernel/config/vkms
dev=$cfg/chora-hotplug

wait_for() {
    for _ in $(seq 60); do
        "$@" >/dev/null 2>&1 && return 0
        sleep 0.5
    done
    fail "timeout: $*; $(tail -n 5 "$log" 2>/dev/null)"
}
outputs() { ctl outputs | jq -r '.[].name' | sort; }
count() { test "$(outputs | wc -l)" = "$1"; }
# mark: events before this line are old for `event`
mark() { mark=$(wc -l <"$shots/events.jsonl"); }
mark=0
# event NAME JQ: an event NAME after the mark matches JQ
event() {
    tail -n +$((mark + 1)) "$shots/events.jsonl" |
        jq -c --arg e "$1" "select(.event == \$e and ($2))" | grep -q .
}
# on WINDOW OUTPUT: the window shows on OUTPUT and nowhere else
on() {
    ctl query "id:$1" | jq -e --arg o "$2" '.[0].rects | keys == [$o]'
}
# window R G B: runs a window of that color; its surface id goes to $id
window() {
    local pid
    pid=$(ctl run "$bin/chora-probe" window "$@" | jq -r .pid)
    [ -n "$pid" ] || fail "the run of chora-probe $*"
    run_pid "$pid"
    for _ in $(seq 50); do
        id=$(ctl query "app_id:chora-probe kind:surface" | jq -r '.[].id' |
            grep -vxF -e "${red:-none}" | head -n 1)
        [ -n "$id" ] && return 0
        sleep 0.2
    done
    fail "no window $*"
}
# shot OUTPUT FILE: a capture of OUTPUT
shot() {
    ctl capture "$shots/$2" --output "$1" >/dev/null || fail "capture of $1"
}
# shows OUTPUT FILE WINDOW R G B: the middle of the window has the color
shows() {
    local x y r g b
    read -r x y < <(ctl query "id:$3" | jq -r --arg o "$1" \
        '.[0].rects[$o] | map(floor) | "\(.[0] + .[2] / 2) \(.[1] + .[3] / 2)"')
    read -r r g b _ < <(pixel "$shots/$2" "${x%.*}" "${y%.*}")
    [ "${r:-x}${g:-x}${b:-x}" != xxx ] &&
        [ $((r - $4)) -lt 40 ] && [ $(($4 - r)) -lt 40 ] &&
        [ $((g - $5)) -lt 40 ] && [ $(($5 - g)) -lt 40 ] &&
        [ $((b - $6)) -lt 40 ] && [ $(($6 - b)) -lt 40 ] ||
        fail "$3 on $1 shows '$r $g $b', not $4 $5 $6 ($shots/$2)"
}

# The monitor of each mode: plug, unplug and, where it has one, a
# connector that goes off and on while its device stays.
case $mode in
headless)
    session_start 1280x800 -- "$bin/chora-tiling"
    plug() { ctl add-output 1024x768+1280+0 >/dev/null; }
    unplug() { ctl remove-output "$second" >/dev/null; }
    cable() { if [ "$1" = 2 ]; then unplug; else plug; fi; }
    ;;
vkms)
    test "$(id -u)" = 0 && test -d /sys/module/vkms ||
        skip "needs root and the vkms module"
    mountpoint -q /sys/kernel/config ||
        mount -t configfs none /sys/kernel/config 2>/dev/null
    rt=$(mktemp -d /tmp/gate-hotplug.XXXXXX)
    env -u WAYLAND_DISPLAY -u DISPLAY XDG_RUNTIME_DIR="$rt" \
        "$bin/chora" -- "$bin/chora-tiling" >"$log" 2>&1 &
    pids+=($!)
    cleanup() {
        kill "${pids[@]}" "${run_pids[@]}" 2>/dev/null
        wait 2>/dev/null
        test -d "$dev" || return 0
        echo 0 >"$dev/enabled"
        rm -f "$dev"/*/*/possible_*/*
        rmdir "$dev"/*/* "$dev"
    }
    trap cleanup EXIT
    export CHORA_SOCKET=$rt/chora/wayland-1.sock
    wait_for test -S "$CHORA_SOCKET"
    sleep 2 # chora-tiling takes the outputs
    if test -d "$cfg"; then
        mkdir "$dev" "$dev/planes/plane0" "$dev/crtcs/crtc0" \
            "$dev/encoders/encoder0" "$dev/connectors/connector0" ||
            fail "a vkms device in configfs"
        echo 1 >"$dev/planes/plane0/type"
        ln -s "$dev/crtcs/crtc0" "$dev/planes/plane0/possible_crtcs"
        ln -s "$dev/crtcs/crtc0" "$dev/encoders/encoder0/possible_crtcs"
        ln -s "$dev/encoders/encoder0" \
            "$dev/connectors/connector0/possible_encoders"
        plug() { echo 1 >"$dev/enabled"; }
        unplug() { echo 0 >"$dev/enabled"; }
        status=$dev/connectors/connector0/status
        if test -e "$status"; then
            echo 1 >"$status" # connected
            cable() { echo "$1" 2>/dev/null >"$status"; }
        fi
        echo "note: the second output is a vkms device from configfs"
    else
        # ponytail: no second output without configfs; the one
        # connector of card0 goes off and on, a change uevent each time
        # since vkms does not poll its connector.
        set -- /sys/class/drm/card*-Virtual-*/status
        connector=$1
        test -w "$connector" || skip "no vkms connector in sysfs"
        card=${connector%%-*}
        force() {
            echo "$1" >"$connector" && echo change >"$card/uevent"
        }
        fallback=1
    fi
    ;;
*) fail "HOTPLUG is headless or vkms, not $mode" ;;
esac
grant screen.capture input.inject
"$bin/choractl" --name agent events output.changed output.removed \
    >"$shots/events.jsonl" 2>&1 &
pids+=($!)
wait_for count 1
first=$(outputs)

window 255 0 0
red=$id
wait_for on "$red" "$first"

if [ -n "${fallback:-}" ]; then
    mark
    force off || fail "forcing $connector off"
    wait_for event output.removed ".name == \"$first\""
    wait_for count 0
    ctl query "id:$red" | jq -e 'length == 1' >/dev/null ||
        fail "the window went with the output"
    force detect || fail "forcing $connector back"
    wait_for count 1
    first=$(outputs)
    wait_for on "$red" "$first"
    shot "$first" first.png
    shows "$first" first.png "$red" 255 0 0
    ok "$first went and came back with its window (fallback: no vkms" \
        "configfs in $(uname -r), so no second output)"
    exit 0
fi

mark
plug || fail "plugging in the second output"
wait_for count 2
second=$(outputs | grep -vxF "$first")
wait_for event output.changed ".output.name == \"$second\""
ok "$second plugged in next to $first"

# The second output takes the focus, so the next window opens on it.
echo '[{"shortcut": {"keys": "Super+Ctrl+j"}}]' | ctl inject >/dev/null ||
    fail "focus_next_output"
window 0 255 0
green=$id
wait_for on "$green" "$second"
shot "$first" first.png
shot "$second" second.png
shows "$first" first.png "$red" 255 0 0
shows "$second" second.png "$green" 0 255 0
ok "captures: red on $first, green on $second"

mark
if declare -F cable >/dev/null && cable 2; then
    wait_for event output.removed ".name == \"$second\""
    wait_for count 1
    wait_for on "$green" "$first"
    ok "the connector of $second went off, its window moved to $first"
    mark
    cable 1 || fail "the connector of $second back on"
    wait_for count 2
    second=$(outputs | grep -vxF "$first")
    wait_for event output.changed ".output.name == \"$second\""
    shot "$second" second-again.png
    ok "the connector came back as $second"
    mark
elif [ "$mode" = vkms ]; then
    echo "note: the connector status is fixed while the device is on"
fi

unplug || fail "unplugging $second"
wait_for event output.removed ".name == \"$second\""
wait_for count 1
wait_for on "$green" "$first"
wait_for on "$red" "$first"
shot "$first" first-after.png
shows "$first" first-after.png "$red" 255 0 0
shows "$first" first-after.png "$green" 0 255 0
ctl outputs >/dev/null || fail "the core is gone"
ok "$second unplugged, both windows on $first, the core lives"
