#!/bin/bash
# Gate check 5 of 0.8 (D58): a client sets the opacity of a tiling window
# to 0.5 with an `update` that the core forwards to chora-tiling as
# `node.requested`. After a relayout, a move to another tag and a restart
# of the plugin, a query still shows 0.5. The scripts gate-0.8-N.sh are
# checks of the 0.7 gate, so this one has a name of its own.
#
# Usage: tests/gate-0.8-tiling-look.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl chora-tiling chora-probe; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
config=$(mktemp -d /tmp/gate-look.XXXXXX)
export XDG_CONFIG_HOME=$config
pidfile=$config/tiling.pid
# keep.sh starts the plugin again when it ends, as its systemd unit does.
session_start 1280x800 -- "$repo/tests/battle/keep.sh" "$pidfile" "$bin/chora-tiling"

pid=$(ctl run "$bin/chora-probe" window 255 0 0 |
    sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p')
[ -n "$pid" ] || fail "the run of the window"
run_pid "$pid"
for _ in $(seq 50); do
    node=$(ctl query 'app_id:chora-probe kind:surface')
    echo "$node" | grep -q '"owner": *"tiling"' && break
    sleep 0.2
done
echo "$node" | grep -q '"owner": *"tiling"' || fail "tiling did not take the window: $node"
id=$(echo "$node" | sed -n 's/.*"id": *"\([^"]*\)".*/\1/p' | head -n 1)

# opacity <what> waits until a query shows the opacity 0.5.
opacity() {
    for _ in $(seq 25); do
        node=$(ctl query "id:$id")
        echo "$node" | grep -q '"opacity": *0.5' && return 0
        sleep 0.2
    done
    fail "$1: $node"
}
echo "[{\"update\": {\"id\": \"$id\", \"props\": {\"opacity\": 0.5}}}]" | ctl apply ||
    fail "the update of the opacity"
opacity "the requested opacity"
ctl call tiling set_layout '{"layout": "max"}' >/dev/null || fail "the relayout"
opacity "after the relayout"
ctl call tiling move_to_tag "{\"surface\": \"$id\", \"tag\": 2}" >/dev/null &&
    ctl call tiling view_tag '{"tag": 2}' >/dev/null || fail "the move to tag 2"
opacity "after the move to tag 2"
old=$(cat "$pidfile")
kill "$old" || fail "the end of the plugin"
for _ in $(seq 50); do
    [ "$(cat "$pidfile")" != "$old" ] && ctl call tiling get_layout >/dev/null 2>&1 && break
    sleep 0.2
done
ctl call tiling get_layout >/dev/null 2>&1 || fail "tiling did not come back"
echo "$(ctl query "id:$id")" | grep -q '"owner": *"tiling"' || fail "the window after the restart"
opacity "after the restart"
rm -rf "$config"
ok "the requested opacity 0.5 stays over a relayout, a tag move and a restart"
