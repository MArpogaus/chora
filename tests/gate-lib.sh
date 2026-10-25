#!/bin/bash
# Shared helpers for the gate scripts of plan 0.8 (tests/gate-0.8-<n>.sh).
# A gate script answers "ok: ..." or "SKIP: <why>" with exit 0, or
# "FAIL: <why>" with exit 1. Sources cleanly from any directory.
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
bin=${BIN:-$repo/target/debug}
# software EGL for the headless renderer (CONTRIBUTING.md)
export __EGL_VENDOR_LIBRARY_DIRS=${__EGL_VENDOR_LIBRARY_DIRS:-$HOME/.local/opt/mesa-egl/usr/share/glvnd/egl_vendor.d}
export LD_LIBRARY_PATH=$HOME/.local/opt/mesa-egl/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
skip() {
    echo "SKIP: $*"
    exit 0
}
ok() { echo "ok: $*"; }
fail() {
    echo "FAIL: $*" >&2
    exit 1
}
need() { command -v "$1" >/dev/null 2>&1 || skip "$1 is not installed here"; }
pids=()
session_start() {
    local env_file=${env_file:-$(mktemp /tmp/gate-env.XXXXXX)}
    GATE_ENV=$env_file
    "$repo/tests/headless-session.sh" "$@" >"$env_file" 2>&1 &
    pids+=($!)
    trap 'kill "${pids[@]}" "${run_pids[@]}" 2>/dev/null; wait 2>/dev/null' EXIT
    for _ in $(seq 60); do
        grep -q CHORA_SOCKET "$env_file" && break
        sleep 0.5
    done
    grep -q CHORA_SOCKET "$env_file" || fail "no core: $(tail -n 3 "$env_file")"
    export CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$env_file")
    export WAYLAND_DISPLAY=$(sed -n 's/^WAYLAND_DISPLAY=//p' "$env_file")
    sleep 2 # the plugin needs a moment before its nodes answer queries
}
grant() { "$bin/choractl" grant agent "$@" || fail "the grant $*"; }
ctl() { timeout 10 "$bin/choractl" --name agent "$@"; }
# run_pid <pid> remembers a program of `choractl run` (D42): it lives in
# a session of its own, so the EXIT trap of session_start must end it.
run_pids=()
run_pid() { run_pids+=("$1"); }
pixel() { python3 -I "$repo/tests/png-pixel.py" "$@"; }
# mcp_py runs one python MCP check with tests/gate-mcp.py importable.
mcp_py() {
    PYTHONPATH=$repo/tests python3 - "$@"
}
