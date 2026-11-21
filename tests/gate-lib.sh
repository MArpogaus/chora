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
# gate_probes prints the pids of the chora-probe programs of the gate: each
# carries CHORA_GATE_PID, the pid of the gate shell, in its environment (an
# orphan too), or the XDG_RUNTIME_DIR of the session of the gate (one that
# `choractl run` started). gate_cleanup ends them with the other programs of
# the gate (sh-30: a leaked probe spun at a full core for hours).
# fuzzel_older MIN is true when the fuzzel on PATH is older than MIN (1.11);
# fuzzel_version holds the version found ("unknown" when it has none).
# need_fuzzel MIN is the one version check of the fuzzel cells (launcher-click,
# the Run box of xp-fields): below MIN the gate is SKIP with the version, not red.
# cell_skip NAME REASON: a cell of a script of several cells does not run; its
# line says SKIP, the others go on (under CI a skip fails, as skip does).
# cells_end RAN SKIPPED ends such a script: with no cell run it is SKIP, else ok.
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
fuzzel_version=
fuzzel_older() {
    fuzzel_version=$(fuzzel --version 2>/dev/null | sed -n 's/^fuzzel version: \([0-9][0-9.]*\).*/\1/p')
    [ -n "$fuzzel_version" ] || {
        fuzzel_version=unknown
        return 0
    }
    [ "$(printf '%s\n%s\n' "$1" "$fuzzel_version" | sort -V | head -n 1)" != "$1" ]
}
need_fuzzel() {
    need fuzzel
    ! fuzzel_older "$1" || skip "fuzzel $fuzzel_version < $1"
}
cell_skip() {
    echo "SKIP: cell $1: $2"
    [ -z "${CI:-}" ] || {
        echo "FAIL: a skip does not pass in CI: cell $1: $2" >&2
        exit 1
    }
    return 0
}
cells_end() {
    [ "$1" != 0 ] || skip "no subtest ran (skipped: ${2:-none})"
    ok "$1 cells ran"
}
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
export CHORA_GATE_PID=$$
session_runtime=
gate_elapsed() { echo $((SECONDS - gate_t0)); }
pids=()
tmp_dirs=()
# gate_cleanup [STATUS]: the EXIT trap. It ends the session and the
# programs, and removes the temporary directories when the gate did not
# fail; a failed gate keeps them for its logs.
gate_probes() {
    local pid
    for pid in $(pgrep -x chora-probe); do
        ! grep -qzx "CHORA_GATE_PID=$$" "/proc/$pid/environ" 2>/dev/null || echo "$pid"
    done
    [ -z "$session_runtime" ] || XDG_RUNTIME_DIR=$session_runtime session_pids -x chora-probe
    return 0
}
gate_cleanup() {
    local status=${1:-$?} probes
    mapfile -t probes < <(gate_probes | sort -u)
    kill "${pids[@]}" "${run_pids[@]}" "${probes[@]}" 2>/dev/null
    kill -CONT "${pids[@]}" "${run_pids[@]}" "${probes[@]}" 2>/dev/null
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
    session_runtime=$XDG_RUNTIME_DIR
    local owner=''
    while [ $# -gt 0 ] && [ "$1" != -- ]; do shift; done
    [ $# -lt 2 ] || owner=$(basename "$2")
    case $owner in chora-*) owner=${owner#chora-} ;; *) owner='' ;; esac
    until_ok "ctl query 'owner:${owner:-core}' | jq -e 'length > 0' >/dev/null || { [ -z '$owner' ] && ctl query '*' >/dev/null; }" 60 ||
        fail "the nodes of ${owner:-the core} do not answer queries"
}
settled() {
    until_ok "a=\$($1); sleep 0.3; [ -n \"\$a\" ] && [ \"\$a\" = \"\$($1)\" ]" "${2:-30}" || true
}
settle() { settled "ctl render-stats | jq -c .outputs"; }
pace() { sleep "$1"; }
until_ok() {
    local rounds=${2:-40}
    local end=$((SECONDS + rounds / 2))
    while :; do
        eval "$1" && return 0
        [ "$SECONDS" -lt "$end" ] || return 1
        sleep 0.25
    done
}
grant() { "$bin/choractl" grant agent "$@" || fail "the grant $*"; }
holds() {
    local step
    step=$(awk -v w="${2:-1}" 'BEGIN { print w / 4 }')
    eval "$1" || return 1
    for _ in 1 2 3 4; do
        sleep "$step"
        eval "$1" || return 1
    done
}
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
