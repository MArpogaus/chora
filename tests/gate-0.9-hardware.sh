#!/bin/bash
# Gate check 4 of 0.9 (and check 7, the hardware checks again): the
# checklist on the reference hardware, run by the user inside the chora
# session. It asks before every step, collects the evidence of the core
# (choractl, journal, coredumpctl, units, loginctl) and writes one log.
# It never suspends or changes anything: the user does each action.
#
# Usage: tests/gate-0.9-hardware.sh [LOGFILE]
#   Default log: ./gate-0.9-hardware-YYYYmmdd-HHMMSS.log
#   The events of the core go to LOGFILE.events.
set -u
log=${1:-gate-0.9-hardware-$(date +%Y%m%d-%H%M%S).log}
results=()

say() { printf '%s\n' "$*" | tee -a "$log"; }
note() { printf '[%s] %s\n' "$(date +%T)" "$*" >>"$log"; }
yn() { # yn QUESTION: true when the user answers y
    local a
    read -r -p "$1 [y/n] " a
    note "asked: $1 -> $a"
    [ "$a" = y ]
}
journal_since() { # warnings of chora since the step began
    journalctl --user -u 'chora@*' -u 'wayland-wm@*' --since "$1" -p warning \
        --no-pager 2>&1 | tail -n 60 >>"$log"
}
snapshot() { # snapshot LABEL: outputs, session and crashes
    note "== $1: choractl outputs"
    timeout 10 choractl outputs >>"$log" 2>&1
    note "== $1: loginctl"
    loginctl session-status "${XDG_SESSION_ID:-}" 2>&1 | head -n 12 >>"$log"
    note "== $1: coredumpctl list chora"
    coredumpctl list chora --no-pager 2>&1 | tail -n 5 >>"$log"
}
step() { # step NAME ACTION [REPEAT]
    local name=$1 what=$2 n=${3:-1} i since pass=1
    say ""
    say "### $name"
    for ((i = 1; i <= n; i++)); do
        since=$(date '+%Y-%m-%d %H:%M:%S')
        snapshot "$name $i/$n before"
        if ! yn "$what ($i/$n): done?"; then
            note "$name $i/$n not done"
            pass=0
            break
        fi
        sleep 3
        snapshot "$name $i/$n after"
        journal_since "$since"
        if ! yn "$name $i/$n: screen, windows and input are fine?"; then
            pass=0
            break
        fi
    done
    if [ $pass = 1 ]; then results+=("PASS $name"); else results+=("FAIL $name"); fi
    say "${results[-1]}"
}

command -v choractl >/dev/null || {
    echo "FAIL: choractl is not in PATH; run this inside the chora session" >&2
    exit 1
}
: >"$log"
say "chora gate 0.9 hardware checklist, $(date -Is), log: $log"
note "host: $(uname -srm); chora: $(chora --version 2>&1 | head -n1)"
note "== units"
systemctl --user status --no-pager 'chora-*.target' 'chora@*' 'wayland-wm@*' 2>&1 |
    head -n 50 >>"$log"
# the events of the core while the checklist runs; ends with the script
timeout 3h choractl events >>"$log.events" 2>&1 &
ev=$!
trap 'kill $ev 2>/dev/null' EXIT

step "monitor hotplug" "Plug the external monitor in, wait, then unplug it" 10
step "suspend and resume" "Suspend the laptop yourself (systemctl suspend), then wake it" 10
step "lid" "Close the lid, wait 5 s, open it"
step "VT switch" "Switch to another VT (Ctrl+Alt+F3), then back"
step "lock and unlock" "Lock the session (swaylock), then unlock it"
step "screen share" "Share a screen or a window in a browser (a web conference test page)"
step "X11 app" "Start an X11 app (xeyes) and see it as a tiled window"

say ""
say "### end state"
snapshot "end"
note "== errors of chora in this boot"
journalctl --user -b -u 'chora@*' -u 'wayland-wm@*' -p err --no-pager 2>&1 |
    tail -n 40 >>"$log"
note "== flavor units"
systemctl --user list-units --no-pager 'chora-*' >>"$log" 2>&1
say "Summary:"
printf '  %s\n' "${results[@]}" | tee -a "$log"
say "Attach $log and $log.events to the gate report."
