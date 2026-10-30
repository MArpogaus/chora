#!/bin/bash
# X11 programs: dist/session/chora-xwayland runs xwayland-satellite on a
# free display, as the X11 units of the flavors do, and xeyes on it shows
# up as a surface of chora, tiled by chora-tiling.
#
# Usage: tests/gate-0.9-x11.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need xwayland-satellite
need Xwayland
need xeyes
need_bins chora choractl chora-tiling
temp_config
session_start 640x480 -- "$bin/chora-tiling"
# xwayland-satellite needs a runtime dir; the one of the session serves.
XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$GATE_ENV")
export XDG_RUNTIME_DIR

n=0
while [ -e "/tmp/.X11-unix/X$n" ] || [ -e "/tmp/.X$n-lock" ]; do
    n=$((n + 1))
done
export DISPLAY=:$n
"$repo/dist/session/chora-xwayland" >"$XDG_CONFIG_HOME/satellite.log" 2>&1 &
pids+=($!)
for _ in $(seq 50); do
    [ -e "/tmp/.X11-unix/X$n" ] && break
    sleep 0.2
done
[ -e "/tmp/.X11-unix/X$n" ] || fail "no display $DISPLAY: $(tail -n 5 "$XDG_CONFIG_HOME/satellite.log")"
xeyes &
pids+=($!)
for _ in $(seq 50); do
    node=$(ctl query 'kind:surface')
    echo "$node" | grep -q '"owner": *"tiling"' && break
    sleep 0.2
done
echo "$node" | grep -q '"owner": *"tiling"' ||
    fail "no X11 window: $node
$(tail -n 5 "$XDG_CONFIG_HOME/satellite.log")"
ok "xeyes on $DISPLAY of xwayland-satellite is a tiled surface of chora"
