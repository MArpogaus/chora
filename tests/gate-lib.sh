#!/bin/bash
# Shared helpers for the gate scripts (tests/gate-*.sh).
# A gate script answers "ok: ..." or "SKIP: <why>" with exit 0, or
# "FAIL: <why>" with exit 1. Sources cleanly from any directory.
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
pids=()
tmp_dirs=()
# gate_cleanup [STATUS]: the EXIT trap. It ends the session and the
# programs, and removes the temporary directories when the gate did not
# fail; a failed gate keeps them for its logs.
gate_cleanup() {
    local status=${1:-$?}
    kill "${pids[@]}" "${run_pids[@]}" 2>/dev/null
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
    export CHORA_SOCKET WAYLAND_DISPLAY
    sleep 2 # the plugin needs a moment before its nodes answer queries
}
grant() { "$bin/choractl" grant agent "$@" || fail "the grant $*"; }
ctl() { timeout 10 "$bin/choractl" --name agent "$@"; }
# run_pid <pid> remembers a program of `choractl run` (D42): it lives in
# a unit or outside the group of the core, so gate_cleanup must end it.
run_pids=()
run_pid() { run_pids+=("$1"); }
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
