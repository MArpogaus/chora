#!/bin/bash
# Starts a private headless chora session with a flavor for a simulated
# user (persona), and grants the client `persona` screen.capture and
# input.inject only, and the client `persona-check` (tests/persona/check.py)
# screen.capture and methods.call. It stays in the foreground until SIGTERM
# or INT, then stops the core by its pid and removes the session directory,
# unless the caller gave it in PERSONA_DIR.
#
# The programs that a click can run on the host are stand-ins in the PATH of
# the session: systemctl (xp Stand By runs `systemctl suspend`), loginctl,
# wpctl, grim and notify-send write `NAME ARGS` to PERSONA_CALLS, and never
# reach the host.
#
# Usage: tests/persona/session.sh FLAVOR[+FLAVOR] [WxH]
#        tests/persona/session.sh --stop ENV_FILE
#   FLAVOR  tiling, xp, mc or construct; with a second one (tiling+xp) the
#           first starts with the core and the second joins it, as a window
#           can be handed from one to the other
#   WxH     the size of the virtual output (default 1280x800)
#
# Environment:
#   PERSONA_DIR  the session directory (default: a new /tmp/persona.XXXXXX,
#                which the script removes at the end; a directory that
#                you give stays)
#   PERSONA_KEEP a directory that gets a copy of shots/, actions.jsonl,
#                breaches.jsonl, evidence/ and the stand-in calls.log
#                when the session ends (default: nothing is kept)
#   PERSONA_CHECK  1: act.py runs tests/persona/check.py after each action
#                (written to the env file, so the driver need not export it)
#   CHORA_BIN    the directory with the binaries (default: target/debug); a
#                snapshot is fine, the session gives chora-xp its theme
#                (CHORA_XP_THEME) for the xp of a cube too
#   CHORA_TEST_FLAGS  flags of the core, as tests/headless-session.sh
#                takes them; --test-injected-press-is-real lets the
#                injected keys of a persona reach the chat field
#
# The companions that the units of the flavor start (flavors/FLAVOR/) start
# too, with choractl run, when their programs exist on the machine; the
# list of each flavor is the `case` on FLAVOR in this script.
#
# When the session is up, it writes KEY=VALUE lines to
# $PERSONA_DIR/session.env and prints "PERSONA_ENV=<that file>" as its
# first output line. The file has PERSONA_ENV, PERSONA_DIR, PERSONA_LOG,
# PERSONA_SHOTS, CHORA_SOCKET, XDG_RUNTIME_DIR, PATH (with the binaries),
# CHORA_CORE_PID, PERSONA_SESSION_PID, PERSONA_CALLS, PERSONA_VOICE,
# PERSONA_PROBE (the report of persona-probe, for check.py --probe) and
# PERSONA_FLAVORS (the flavors joined by +). act.py reads it.
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
IFS=+ read -ra flavors <<<"$1"
flavor=${flavors[0]}
size=${2:-1280x800}
((${#flavors[@]} <= 2)) && [[ ${flavors[1]:-x} != "$flavor" ]] || {
    echo "usage: $0 FLAVOR[+FLAVOR] [WxH] (two different flavors at most)" >&2
    exit 2
}
for f in "${flavors[@]}"; do
    case $f in
    tiling | xp | mc | construct) ;;
    *)
        echo "unknown flavor $f (tiling, xp, mc, construct)" >&2
        exit 2
        ;;
    esac
done
has_flavor() { [[ " ${flavors[*]} " == *" $1 "* ]]; }

if [[ -n ${PERSONA_KEEP:-} && -n $(ls -A "$PERSONA_KEEP" 2>/dev/null) ]]; then
    echo "PERSONA_KEEP $PERSONA_KEEP is not empty; give a new or empty directory" >&2
    exit 2
fi
created_dir=
if [[ -z ${PERSONA_DIR:-} ]]; then
    PERSONA_DIR=$(mktemp -d /tmp/persona.XXXXXX)
    created_dir=$PERSONA_DIR
fi
dir=$PERSONA_DIR
mkdir -p "$dir/shots" "$dir/config"
rm -f "$dir/last_look.json"
dir=$(cd "$dir" && pwd)
env_file=$dir/session.env

real_systemctl=$(command -v systemctl || true)
fakes=$dir/fake
mkdir -p "$fakes" "$dir/voice" "$dir/data/chora/huds"
marks=$fakes/calls.log
: >"$marks"
cat >"$fakes/notify-send" <<EOF
#!/bin/sh
echo "notify-send \$*" >>"$marks"
if [ -n "\${DBUS_SESSION_BUS_ADDRESS:-}" ] && command -v gdbus >/dev/null; then
    gdbus call --session --dest org.freedesktop.Notifications \\
        --object-path /org/freedesktop/Notifications \\
        --method org.freedesktop.Notifications.Notify notify-send 0 '' "\${1:-}" "\${2:-}" '[]' '{}' 5000 >/dev/null
fi
exit 0
EOF
cat >"$fakes/loginctl" <<EOF
#!/bin/sh
echo "loginctl \$*" >>"$marks"
exit 0
EOF
cat >"$fakes/swaylock" <<'EOF'
#!/bin/sh
echo swaylock "$@" >>"@MARKS@"
(timeout 8 "@BIN@/chora-probe" window 16 24 48 --title "Screen locked" >/dev/null 2>&1 &)
EOF
cat >"$fakes/mousepad" <<'EOF'
#!/bin/sh
if command -v foot >/dev/null; then
    echo "mousepad stand-in foot" >>"@MARKS@"
    exec foot --title "Untitled - Notepad" sh -c 'exec tee -a "@NOTEPAD@"'
fi
echo "mousepad stand-in probe" >>"@MARKS@"
exec "@BIN@/chora-probe" window 240 240 240 --title "Untitled - Notepad" --server-deco
EOF
cat >"$fakes/persona-probe" <<'EOF'
#!/bin/sh
exec "@BIN@/chora-probe" window 200 200 200 --title "Probe window" --server-deco --report "$@" >>"@PROBE@"
EOF
shared=$dir/share/chora/flavors/shared
mkdir -p "$shared"
cp -r "$repo"/flavors/shared/. "$shared/"
printf '#!/bin/sh\necho systemctl "$@" >>"%s"\n' "$marks" >"$fakes/systemctl"
cat >"$fakes/wpctl" <<'EOF'
#!/bin/sh
echo wpctl "$@" >>"@MARKS@"
state="@STATE@"
[ -f "$state" ] || echo "0.50 0" >"$state"
read -r volume mute <"$state"
case "$1" in
get-volume)
    if [ "$mute" = 1 ]; then echo "Volume: $volume [MUTED]"; else echo "Volume: $volume"; fi ;;
set-mute)
    case "$3" in 1) mute=1 ;; 0) mute=0 ;; toggle) mute=$((1 - mute)) ;; esac
    echo "$volume $mute" >"$state" ;;
set-volume)
    for last; do :; done
    case "$last" in
    *%+) volume=$(awk -v v="$volume" -v p="${last%\%+}" 'BEGIN{v+=p/100; if (v>1) v=1; printf "%.2f", v}') ;;
    *%-) volume=$(awk -v v="$volume" -v p="${last%\%-}" 'BEGIN{v-=p/100; if (v<0) v=0; printf "%.2f", v}') ;;
    *%) volume=$(awk -v p="${last%\%}" 'BEGIN{printf "%.2f", p/100}') ;;
    esac
    echo "$volume $mute" >"$state" ;;
esac
EOF
sed -i -e "s|@MARKS@|$marks|g" -e "s|@BIN@|$bin|g" -e "s|@NOTEPAD@|$dir/notepad.txt|g" \
    -e "s|@PROBE@|$dir/probe.report|g" -e "s|@STATE@|$fakes/wpctl.state|g" \
    "$fakes/swaylock" "$fakes/mousepad" "$fakes/persona-probe" "$fakes/wpctl"
# shellcheck disable=SC2016 # the stand-in expands its own arguments
printf '#!/bin/sh\necho grim "$@" >>"%s"\nfor last; do :; done\n: >"$last"\n' "$marks" >"$fakes/grim"
mkdir -p "$dir/pictures"
printf '#!/bin/sh\necho "%s"\n' "$dir/pictures" >"$fakes/xdg-user-dir"
chmod +x "$fakes/xdg-user-dir"
chmod +x "$fakes/swaylock" "$fakes/mousepad" "$fakes/persona-probe" "$fakes/notify-send" "$fakes/loginctl" "$fakes/systemctl" "$fakes/wpctl" "$fakes/grim"
while read -r name hud; do
    ln -sfn "$hud" "$dir/data/chora/huds/$name"
done < <("$repo/tests/hud-folders.sh")
echo '[]' >"$dir/voice/entries.json"

# The plugins start apps with `choractl run` from PATH (D42).
export PATH="$fakes:$bin:$PATH"
export XDG_DATA_HOME=$dir/data
export CHORA_SHARED_DIR=$shared
export CHORA_XP_THEME=$repo/flavors/xp/theme
export XDG_PICTURES_DIR=$dir/pictures
export CHORA_TEST_ACCEPT_INJECTED_KEYS=${CHORA_TEST_ACCEPT_INJECTED_KEYS:-1}
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

startup_of() {
    case $1 in
    tiling) startup=("$bin/chora-tiling") ;;
    xp)
        sed -e 's/^  sounds: true/  sounds: false/' -e 's/^editor:.*/editor: mousepad/' "$repo/plugins/chora-xp/xp.yaml" >"$dir/xp.yaml"
        startup=(env "DBUS_SESSION_BUS_ADDRESS=unix:path=$dir/bus" "$bin/chora-xp" --theme "$repo/flavors/xp/theme" --config "$dir/xp.yaml")
        ;;
    mc) startup=("$bin/chora-mc" --manage-windows) ;;
    construct) startup=("$bin/chora-construct") ;;
    esac
}
startup_of "$flavor"

companions=()
for f in "${flavors[@]}"; do
    case $f in
    tiling)
        companions+=(
            "tiling sh $repo/flavors/tiling/wallpaper.sh"
            "tiling sh $repo/flavors/tiling/waybar/waybar.sh $repo/flavors/tiling/waybar"
            "tiling chora-assistant"
        )
        ;;
    xp)
        companions+=(
            "xp chora-notify --assets $repo/flavors/xp/assets/notify"
            "xp chora-assistant"
        )
        ;;
    mc | construct) companions+=("$f chora-assistant") ;;
    esac
done

if ((${#companions[@]})); then
    for tool in jq setsid dbus-daemon dbus-run-session; do
        command -v "$tool" >/dev/null || {
            echo "$tool is needed to start the companions of $flavor" >&2
            exit 2
        }
    done
fi

out=$dir/headless.out
session=
companion_units=()
bus_pid=
voice_pid=
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
    if [[ -n $voice_pid ]]; then
        kill -TERM "$voice_pid" 2>/dev/null || true
    fi
    if [[ -n $bus_pid ]]; then
        kill -TERM "$bus_pid" 2>/dev/null || true
    fi
    for unit in "${companion_units[@]}"; do
        [[ -z $real_systemctl ]] || "$real_systemctl" --user stop "$unit" 2>/dev/null || true
    done
    for pid in "${companion_pids[@]}"; do
        kill -TERM -- "-$pid" 2>/dev/null || true
    done
    if [[ -n ${PERSONA_KEEP:-} ]]; then
        mkdir -p "$PERSONA_KEEP"
        cp -r "$dir/shots" "$PERSONA_KEEP/"
        cp "$dir/actions.jsonl" "$dir/breaches.jsonl" "$fakes/calls.log" "$PERSONA_KEEP/" 2>/dev/null || true
        cp -r "$dir/evidence" "$PERSONA_KEEP/" 2>/dev/null || true
    fi
    if [[ -n $created_dir ]]; then
        rm -rf "$created_dir"
    fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP

if has_flavor xp; then
    export DBUS_SESSION_BUS_ADDRESS=unix:path=$dir/bus
    dbus-daemon --session --nofork --address="$DBUS_SESSION_BUS_ADDRESS" >"$dir/dbus.log" 2>&1 &
    bus_pid=$!
fi

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
CHORA_SOCKET=$socket XDG_RUNTIME_DIR=$runtime "$bin/choractl" grant persona-check screen.capture methods.call >/dev/null

mkfifo "$dir/voice/in"
exec 4<>"$dir/voice/in"
CHORA_SOCKET=$socket XDG_RUNTIME_DIR=$runtime FAKE_VOICE_ENTRIES=$dir/voice/entries.json FAKE_VOICE_ANSWER=Hello \
    python3 -I "$repo/tests/fake-voice.py" "$dir/voice/claims" <&4 >"$dir/voice/voice.log" 2>&1 &
voice_pid=$!
echo 'voice.state {"state": "listening", "task": "", "said": ""}' >&4

if ((${#flavors[@]} == 2)); then
    startup_of "${flavors[1]}"
    companions=("${flavors[1]} ${startup[*]}" "${companions[@]}")
fi
started_once=" "
for line in "${companions[@]}"; do
    read -ra argv <<<"$line"
    cflavor=${argv[0]}
    argv=("${argv[@]:1}")
    [[ $started_once != *" ${argv[*]} "* ]] || continue
    started_once+="${argv[*]} "
    command -v "${argv[0]}" >/dev/null || {
        echo "companion ${argv[0]} not found" >>"$dir/companions.log"
        continue
    }
    session_wrapper=(dbus-run-session --)
    if [[ $cflavor == xp ]]; then
        session_wrapper=()
    fi
    started=$(
        WAYLAND_DISPLAY=$(sed -n 's/^WAYLAND_DISPLAY=//p' "$out") CHORA_FLAVOR_DIR=$repo/flavors/$cflavor \
            CHORA_SOCKET=$socket XDG_RUNTIME_DIR=$runtime \
            "$bin/choractl" run setsid "${session_wrapper[@]}" "${argv[@]}"
    )
    unit=$(jq -r '.unit // empty' <<<"$started")
    if [[ -n $unit ]]; then
        companion_units+=("$unit")
    else
        companion_pids+=("$(jq -r .pid <<<"$started")")
    fi
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
    echo "PERSONA_CALLS=$marks"
    echo "PERSONA_VOICE=$dir/voice"
    echo "PERSONA_PROBE=$dir/probe.report"
    echo "PERSONA_FLAVORS=$1"
    [[ -z ${PERSONA_CHECK:-} ]] || echo "PERSONA_CHECK=$PERSONA_CHECK"
} >"$env_file"
echo "PERSONA_ENV=$env_file"

# Wait until the core ends or a signal arrives.
rc=0
wait "$session" || rc=$?
session=
exit "$rc"
