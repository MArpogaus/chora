#!/bin/bash
# Starts a private headless chora session with a flavor for a simulated
# user (persona), and grants the client `persona` screen.capture and
# input.inject only. It stays in the foreground until SIGTERM or INT, then
# stops the core by its pid and removes the session directory, unless the
# caller gave it in PERSONA_DIR.
#
# Usage: tests/persona/session.sh FLAVOR [WxH]
#        tests/persona/session.sh --stop ENV_FILE
#   FLAVOR  tiling, xp, mc or construct
#   WxH     the size of the virtual output (default 1280x800)
#
# Environment:
#   PERSONA_DIR  the session directory (default: a new /tmp/persona.XXXXXX,
#                which the script removes at the end; a directory that
#                you give stays)
#   PERSONA_KEEP a directory that gets a copy of shots/ and actions.jsonl
#                when the session ends (default: nothing is kept)
#   CHORA_BIN    the directory with the binaries (default: target/debug)
#
# The companions that the units of the flavor start (flavors/FLAVOR/) start
# too, with choractl run, when their programs exist on the machine; the
# list of each flavor is the `case` on FLAVOR in this script.
#
# When the session is up, it writes KEY=VALUE lines to
# $PERSONA_DIR/session.env and prints "PERSONA_ENV=<that file>" as its
# first output line. The file has PERSONA_ENV, PERSONA_DIR, PERSONA_LOG,
# PERSONA_SHOTS, CHORA_SOCKET, XDG_RUNTIME_DIR, PATH (with the binaries),
# CHORA_CORE_PID and PERSONA_SESSION_PID. act.py reads it.
set -euo pipefail

repo=$(cd "$(dirname "$0")/../.." && pwd)
bin=${CHORA_BIN:-$repo/target/debug}

# --stop: end a session by the pid in its env file (not by name).
if [[ ${1:-} == --stop ]]; then
    [[ $# -eq 2 && -f $2 ]] || {
        echo "usage: $0 --stop ENV_FILE" >&2
        exit 2
    }
    spid=$(sed -n 's/^PERSONA_SESSION_PID=//p' "$2")
    [[ -n $spid ]] || exit 1
    kill -TERM "$spid" 2>/dev/null || true
    for _ in $(seq 100); do
        kill -0 "$spid" 2>/dev/null || exit 0
        sleep 0.1
    done
    echo "session $spid did not end" >&2
    exit 1
fi

[[ $# -ge 1 && $# -le 2 ]] || {
    echo "usage: $0 FLAVOR [WxH]" >&2
    exit 2
}
flavor=$1
size=${2:-1280x800}

created_dir=
if [[ -z ${PERSONA_DIR:-} ]]; then
    PERSONA_DIR=$(mktemp -d /tmp/persona.XXXXXX)
    created_dir=$PERSONA_DIR
fi
dir=$PERSONA_DIR
mkdir -p "$dir/shots" "$dir/config"
dir=$(cd "$dir" && pwd)
env_file=$dir/session.env

# The plugins start apps with `choractl run` from PATH (D42).
export PATH="$bin:$PATH"
unset DBUS_SESSION_BUS_ADDRESS
# A private config home: the settings of the user stay out of the run.
export XDG_CONFIG_HOME=$dir/config
# Software EGL where the system has no libEGL, as tests/battle/lib.sh does.
egl=$HOME/.local/opt/mesa-egl
if [[ -d $egl ]]; then
    export LD_LIBRARY_PATH=$egl/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
    export __EGL_VENDOR_LIBRARY_FILENAMES=$egl/usr/share/glvnd/egl_vendor.d/50_mesa.json
fi
export GALLIUM_DRIVER=llvmpipe LIBGL_ALWAYS_SOFTWARE=1

case $flavor in
tiling) startup=("$bin/chora-tiling") ;;
xp)
    # No sounds in a headless run.
    sed 's/^  sounds: true/  sounds: false/' "$repo/plugins/chora-xp/xp.yaml" >"$dir/xp.yaml"
    startup=("$bin/chora-xp" --theme "$repo/flavors/xp/theme" --config "$dir/xp.yaml")
    ;;
mc) startup=("$bin/chora-mc" --manage-windows) ;;
construct) startup=("$bin/chora-construct") ;;
*)
    echo "unknown flavor $flavor (tiling, xp, mc, construct)" >&2
    exit 2
    ;;
esac

case $flavor in
tiling)
    companions=(
        "swaybg --color #2e3440"
        "waybar --config $repo/flavors/tiling/waybar/config.jsonc"
    )
    ;;
*) companions=() ;;
esac

out=$dir/headless.out
session=
companion_pids=()
# shellcheck disable=SC2329 # runs as the EXIT trap
cleanup() {
    trap - EXIT INT TERM HUP
    if [[ -n $session ]]; then
        # headless-session.sh stops the core and its process group on TERM.
        kill -TERM "$session" 2>/dev/null || true
        for _ in $(seq 100); do
            kill -0 "$session" 2>/dev/null || break
            sleep 0.1
        done
        kill -KILL "$session" 2>/dev/null || true
    fi
    for pid in "${companion_pids[@]}"; do
        kill -TERM -- "-$pid" 2>/dev/null || true
    done
    if [[ -n ${PERSONA_KEEP:-} ]]; then
        mkdir -p "$PERSONA_KEEP"
        cp -r "$dir/shots" "$PERSONA_KEEP/"
        cp "$dir/actions.jsonl" "$PERSONA_KEEP/" 2>/dev/null || true
    fi
    if [[ -n $created_dir ]]; then
        rm -rf "$created_dir"
    fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP

CHORA_LOG=$dir/chora.log "$repo/tests/headless-session.sh" "$size" -- "${startup[@]}" \
    >"$out" 2>"$out.err" &
session=$!

for _ in $(seq 150); do
    grep -q '^CHORA_SOCKET=' "$out" 2>/dev/null && break
    kill -0 "$session" 2>/dev/null || {
        cat "$out.err" >&2
        exit 1
    }
    sleep 0.2
done
grep -q '^CHORA_SOCKET=' "$out" || {
    echo "the session did not start; see $out.err" >&2
    exit 1
}
socket=$(sed -n 's/^CHORA_SOCKET=//p' "$out")
runtime=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$out")
core=$(pgrep -P "$session" -x chora | head -n 1 || true)

# The admin socket grants; persona gets these two and nothing else.
CHORA_SOCKET=$socket XDG_RUNTIME_DIR=$runtime "$bin/choractl" grant persona screen.capture input.inject >/dev/null

for line in "${companions[@]}"; do
    read -ra argv <<<"$line"
    command -v "${argv[0]}" >/dev/null || continue
    started=$(
        WAYLAND_DISPLAY=$(sed -n 's/^WAYLAND_DISPLAY=//p' "$out") CHORA_FLAVOR_DIR=$repo/flavors/$flavor \
            CHORA_SOCKET=$socket XDG_RUNTIME_DIR=$runtime \
            "$bin/choractl" run setsid dbus-run-session -- "${argv[@]}"
    )
    companion_pids+=("$(jq -r .pid <<<"$started")")
done

{
    echo "PERSONA_ENV=$env_file"
    echo "PERSONA_DIR=$dir"
    echo "PERSONA_LOG=$dir/actions.jsonl"
    echo "PERSONA_SHOTS=$dir/shots"
    echo "CHORA_SOCKET=$socket"
    echo "XDG_RUNTIME_DIR=$runtime"
    echo "PATH=$PATH"
    echo "CHORA_CORE_PID=$core"
    echo "PERSONA_SESSION_PID=$$"
} >"$env_file"
echo "PERSONA_ENV=$env_file"

# Wait until the core ends or a signal arrives.
rc=0
wait "$session" || rc=$?
session=
exit "$rc"
