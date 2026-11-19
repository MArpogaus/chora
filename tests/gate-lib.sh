#!/bin/bash
# Shared helpers for the gate scripts (tests/gate-*.sh).
# A gate script answers "ok: ..." or "SKIP: <why>" with exit 0 (non-zero
# when CI is set), or
# "FAIL: <why>" with exit 1. Sources cleanly from any directory.
# session_pids PGREP_ARGS... prints the pids that pgrep finds whose
# environment has the XDG_RUNTIME_DIR of the gate, so that a gate beside
# it never counts, and exits 1 when it finds none.
# session_start OUTPUTS [-- STARTUP...] starts a headless session; its env
# file (env_file, else a new one) is emptied first, so that a session of a
# name used before never reads the socket of the old one (F21).
# gate_elapsed prints the seconds since the gate has the lock: the wait in
# the queue for the lock is not in it (SECONDS counts from the shell start).
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
bin=${BIN:-$repo/target/debug}
# software EGL for the headless renderer (CONTRIBUTING.md)
# where that copy exists; elsewhere the system EGL serves
if [ -d "$HOME/.local/opt/mesa-egl" ]; then
    export __EGL_VENDOR_LIBRARY_DIRS=${__EGL_VENDOR_LIBRARY_DIRS:-$HOME/.local/opt/mesa-egl/usr/share/glvnd/egl_vendor.d}
    export LD_LIBRARY_PATH=$HOME/.local/opt/mesa-egl/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
fi
skip() {
    echo "SKIP: $*"
    if [ -n "${CI:-}" ]; then
        echo "FAIL: a skip does not pass in CI: $*" >&2
        exit 1
    fi
    exit 0
}
ok() { echo "ok: $*"; }
fail() {
    echo "FAIL: $*" >&2
    exit 1
}
need() { command -v "$1" >/dev/null 2>&1 || skip "$1 is not installed here"; }
# need_bins NAME...: the binaries of the build in $bin.
need_bins() {
    local b
    for b in "$@"; do
        test -x "$bin/$b" || fail "no $bin/$b; build it first"
    done
}
export CHORA_TEST_ACCEPT_INJECTED_KEYS=${CHORA_TEST_ACCEPT_INJECTED_KEYS:-1}
. "$repo/tests/gate-lock.sh"
gate_lock "$(basename "$0")" || fail "cannot take the gate lock"
gate_t0=$SECONDS
gate_elapsed() { echo $((SECONDS - gate_t0)); }
pids=()
tmp_dirs=()
# gate_cleanup [STATUS]: the EXIT trap. It ends the session and the
# programs, and removes the temporary directories when the gate did not
# fail; a failed gate keeps them for its logs.
gate_cleanup() {
    local status=${1:-$?}
    kill "${pids[@]}" "${run_pids[@]}" 2>/dev/null
    kill -CONT "${pids[@]}" "${run_pids[@]}" 2>/dev/null
    wait 2>/dev/null
    [ "$status" != 0 ] || rm -rf "${tmp_dirs[@]}"
}
# temp_config: a new empty XDG_CONFIG_HOME, so that no file of the user
# reaches the gate; gate_cleanup removes it.
temp_config() {
    XDG_CONFIG_HOME=$(mktemp -d /tmp/gate-config.XXXXXX)
    export XDG_CONFIG_HOME
    tmp_dirs+=("$XDG_CONFIG_HOME")
    trap gate_cleanup EXIT
}
session_start() {
    local env_file=${env_file:-$(mktemp /tmp/gate-env.XXXXXX)}
    # shellcheck disable=SC2034 # read by the gate scripts that source this file
    GATE_ENV=$env_file
    : >"$env_file"
    "$repo/tests/headless-session.sh" "$@" >"$env_file" 2>&1 &
    pids+=($!)
    trap gate_cleanup EXIT
    for _ in $(seq 60); do
        grep -q CHORA_SOCKET "$env_file" && break
        sleep 0.5
    done
    grep -q CHORA_SOCKET "$env_file" || fail "no core: $(tail -n 3 "$env_file")"
    CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$env_file")
    WAYLAND_DISPLAY=$(sed -n 's/^WAYLAND_DISPLAY=//p' "$env_file")
    # The plugins that a gate starts write their shaders there, as in a
    # session; they refuse the shared /tmp.
    XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$env_file")
    export CHORA_SOCKET WAYLAND_DISPLAY XDG_RUNTIME_DIR
    sleep 2 # the plugin needs a moment before its nodes answer queries
}
grant() { "$bin/choractl" grant agent "$@" || fail "the grant $*"; }
ctl() { timeout 10 "$bin/choractl" --name agent "$@"; }
notify_start() {
    need_bins chora-notify
    env -u DBUS_SESSION_BUS_ADDRESS "$bin/chora-notify" \
        --assets "$repo/plugins/chora-notify/assets" >"${GATE_ENV:?}.notify" 2>&1 &
    pids+=($!)
    for _ in $(seq 40); do
        ctl call notify state.get >/dev/null 2>&1 && return 0
        sleep 0.25
    done
    fail "no chora-notify: $(tail -n 3 "$GATE_ENV.notify")"
}
notices() {
    ctl call notify state.get 2>/dev/null |
        python3 -I -c 'import json, sys; print("\n".join(n["summary"] for n in json.load(sys.stdin).get("shown", [])))'
}

start_notify() {
    "$bin/chora-notify" >"${1:-/dev/null}" 2>&1 &
    pids+=($!)
    for _ in $(seq 40); do
        ctl call notify state.get >/dev/null 2>&1 && return 0
        sleep 0.5
    done
    fail "chora-notify does not answer state.get"
}
shown_notices() { ctl call notify state.get 2>&1 | jq -r '.shown[]? | "\(.summary) \(.body)"'; }
dismiss_notices() {
    local id
    for id in $(ctl call notify state.get 2>/dev/null | jq -r '.shown[]?.id'); do
        ctl call notify notify.close "{\"id\": $id}" >/dev/null 2>&1
    done
}
layout_audit() {
    local tag=$1 dir out
    shift
    dir=$(mktemp -d /tmp/gate-audit.XXXXXX)
    tmp_dirs+=("$dir")
    ctl outputs >"$dir/outputs.json" || fail "$tag: no outputs"
    ctl query '*' >"$dir/nodes.json" || fail "$tag: no nodes"
    out=$(python3 -I "$repo/tests/hud-audit.py" "$dir/outputs.json" "$dir/nodes.json" "$@") ||
        fail "$tag: layout audit: $out"
    ok "$tag: $out"
}
# run_pid <pid> remembers a program of `choractl run` (D42): it lives in
# a unit or outside the group of the core, so gate_cleanup must end it.
run_pids=()
run_pid() { run_pids+=("$1"); }
session_pids() {
    local pid found=1
    for pid in $(pgrep "$@"); do
        grep -qzx "XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR" "/proc/$pid/environ" 2>/dev/null || continue
        echo "$pid"
        found=0
    done
    return "$found"
}
pixel() { python3 -I "$repo/tests/png-pixel.py" "$@"; }
# crash_lines SINCE: the journal lines of the chora units since a time
# that show a crash, a hang or a failed start. secureblue disables core
# dumps, so coredumpctl alone sees nothing there.
crash_lines() {
    journalctl --user -u 'chora@*' -u 'wayland-wm@*' --since "$1" --no-pager 2>/dev/null |
        grep -E 'the watchdog aborts|chora: SIG|the main loop stalls|the startup command failed'
}
# mcp_py runs one python MCP check with tests/gate_mcp.py importable.
mcp_py() {
    PYTHONPATH=$repo/tests python3 - "$@"
}
