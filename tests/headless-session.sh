#!/bin/bash
# Runs a chora session on the headless backend, for tests and agents: no
# host display, GPU or seat. The core gets a private XDG_RUNTIME_DIR in a
# temp dir and never sees WAYLAND_DISPLAY or DISPLAY of the host.
#
# Usage: tests/headless-session.sh [OUTPUTS] [-- STARTUP [ARGS...]]
#   OUTPUTS  virtual outputs, WxH[@SCALE][+X+Y] comma separated
#            (default 1280x800)
#   STARTUP  the startup command, for example a window manager plugin
#
# When the IPC socket is up, it prints lines KEY=VALUE for the clients:
# XDG_RUNTIME_DIR, WAYLAND_DISPLAY (an absolute path) and CHORA_SOCKET.
# Then it waits until the core ends or until a signal; either way it
# stops the core with its children and removes the temp dir.
#
# Environment:
#   CHORA_BIN  the directory with chora (default: target/debug of the repo)
#   CHORA_LOG  the log of the core (default: chora.log in the temp dir)
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
bin=${CHORA_BIN:-$repo/target/debug}
outputs=1280x800
if [[ $# -gt 0 && $1 != -- ]]; then
    outputs=$1
    shift
fi
if [[ $# -gt 0 ]]; then
    [[ $1 == -- ]] || {
        echo "usage: $0 [OUTPUTS] [-- STARTUP [ARGS...]]" >&2
        exit 2
    }
    shift
fi

# A short path, because a socket path has at most 107 bytes.
runtime=$(mktemp -d /tmp/chora-hl.XXXXXX)
log=${CHORA_LOG:-$runtime/chora.log}
pid=
cleanup() {
    trap - EXIT INT TERM HUP
    if [[ -n $pid ]]; then
        # chora leads its process group, which holds the plugins and the
        # programs that it started.
        kill -TERM -- "-$pid" 2>/dev/null || true
        for _ in $(seq 50); do
            kill -0 "$pid" 2>/dev/null || break
            sleep 0.1
        done
        kill -KILL -- "-$pid" 2>/dev/null || true
    fi
    rm -rf "$runtime"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP

# llvmpipe starts a thread per core; 4 render a headless frame at half the
# CPU cost of 32.
export LP_NUM_THREADS=${LP_NUM_THREADS:-4}
env -u WAYLAND_DISPLAY -u DISPLAY -u WAYLAND_SOCKET XDG_RUNTIME_DIR="$runtime" \
    setsid "$bin/chora" --headless "$outputs" -- "$@" >"$log" 2>&1 &
pid=$!

for _ in $(seq 100); do
    socket=$(compgen -G "$runtime/chora/*.sock" || true)
    [[ -n $socket ]] && break
    kill -0 "$pid" 2>/dev/null || {
        cat "$log" >&2
        exit 1
    }
    sleep 0.1
done
[[ -n $socket ]] || {
    echo "chora opened no IPC socket; see $log" >&2
    exit 1
}
echo "XDG_RUNTIME_DIR=$runtime"
echo "WAYLAND_DISPLAY=$runtime/$(basename "$socket" .sock)"
echo "CHORA_SOCKET=$socket"
# wait returns at once on a signal, so that the traps run.
wait "$pid" || true
