#!/bin/sh
# The wallpaper of the tiling flavor without a unit: runs swaybg through
# `chora-tiling --launch swaybg`, which takes wallpaper.png in Ink and
# wallpaper-paper.png in Paper, and starts it again on SIGUSR2, which
# chora-tiling sends after a theme change. Its pid is in wallpaper.pid in
# chora-tiling of the runtime directory. A unit does not need this script:
# chora-tiling-swaybg.service runs swaybg itself and is restarted instead.
# It ends, with its pid file removed, when the Wayland socket of its
# session is gone, so that one that `chora-tiling --with-companions`
# started outlives no session.
# Usage: sh wallpaper.sh
runtime=${XDG_RUNTIME_DIR:-$(mktemp -d)}
pids=$runtime/chora-tiling
# shellcheck disable=SC2174
mkdir -p -m 700 "$pids"
socket_gone() {
    case ${WAYLAND_DISPLAY:-} in
    "" | /*) return 1 ;;
    esac
    [ ! -S "$runtime/$WAYLAND_DISPLAY" ]
}
pid=""
restart=""
trap '[ -z "$pid" ] || kill "$pid" 2>/dev/null; rm -f "$pids/wallpaper.pid"; exit 0' TERM INT
trap 'restart=1; [ -z "$pid" ] || kill "$pid" 2>/dev/null' USR2
echo $$ >"$pids/wallpaper.pid"
while :; do
    restart=""
    chora-tiling --launch swaybg &
    pid=$!
    while kill -0 "$pid" 2>/dev/null; do
        wait "$pid"
    done
    pid=""
    if socket_gone; then
        rm -f "$pids/wallpaper.pid"
        exit 0
    fi
    [ -n "$restart" ] || sleep 1
done
