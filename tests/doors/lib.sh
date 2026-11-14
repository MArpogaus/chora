#!/bin/bash
# The frame of the doors gate (tests/gate-doors.sh): the cells that every
# look answers, and the helpers of the cell files tests/doors/LOOK.sh.
# A cell file sources this file, defines one function per cell, named
# cell_ID with `_` for `-`, and ends with `doors_run LOOK`. A cell returns
# 0 when the look has the key or the door; a cell without a function
# counts as missing.
#
# The cells, the same for every look:
#   keys-key        Super+F1 opens the key list
#   keys-alias      Super+/ opens it too
#   keys-names      the key list names Super+F1, Super+Space, Super+a,
#                   Super+Shift+Esc, Super+Shift+e, Super+Print and the
#                   volume keys
#   keys-door       a visible door opens the key list by a click
#   f1-free         F1 is no binding of the look: it reaches the client
#   launcher-key    Super+Space opens the launcher
#   launcher-door   a visible door opens the launcher by a click
#   launcher-chat   the launcher has the row Chat with a helper
#   helper-key      Super+a opens the chat with a helper
#   helper-door     a visible helper door shows a state word
#   helper-none     without a helper the door says No helper, and Super+a
#                   says so too
#   lock-key        Super+Shift+Esc locks (the stand-in swaylock)
#   lock-door       a visible door locks by a click
#   power-key       Super+Shift+e opens the power menu: Lock, Log out,
#                   Restart, Shut down
#   power-door      a visible door opens the power menu by a click
#   screenshot-key  Super+Print takes a screenshot (the stand-in grim)
#   screenshot-door a visible door takes a screenshot by a click
#   volume-key      XF86AudioRaiseVolume changes the volume (the stand-in
#                   wpctl)
#   volume-door     a visible door shows or changes the volume
#   clock-door      a clock is visible
#
# Helpers:
#   doors_fakes     a folder in front of PATH whose swaylock, loginctl,
#                   systemctl, notify-send and wpctl write a line
#                   `NAME ARGS` to $marks, and whose grim writes an empty
#                   file where its last argument says; wpctl answers
#                   `Volume: 0.50` to get-volume
#   marked PATTERN  a stand-in wrote a line that matches, within 5 s
#   unmark          empties $marks
#   until_ok COMMAND [TRIES]  the command succeeds within TRIES quarters
#                   of a second (default 40)
#   key KEYS, click X Y       an injected shortcut, an injected click
#   has_text TEXT   a node of the scene shows the text
#   doors_run LOOK  runs the cells, prints `cell LOOK ID pass|missing` for
#                   each and returns 1 when one is missing
#
# Usage: . tests/doors/lib.sh   (from tests/doors/LOOK.sh)
#   ONLY  a regular expression of cell names; the others do not run
# shellcheck source=../gate-lib.sh
. "$(dirname "${BASH_SOURCE[0]}")/../gate-lib.sh"

doors_cells="keys-key keys-alias keys-names keys-door f1-free launcher-key launcher-door launcher-chat helper-key helper-door helper-none lock-key lock-door power-key power-door screenshot-key screenshot-door volume-key volume-door clock-door"

doors_fakes() {
    fakes=$(mktemp -d /tmp/gate-doors.XXXXXX)
    tmp_dirs+=("$fakes")
    marks=$fakes/marks
    : >"$marks"
    local what
    for what in swaylock loginctl systemctl notify-send; do
        printf '#!/bin/sh\necho %s "$@" >>"%s"\n' "$what" "$marks" >"$fakes/$what"
    done
    printf '#!/bin/sh\necho wpctl "$@" >>"%s"\ncase " $* " in *" get-volume "*) echo "Volume: 0.50" ;; esac\n' "$marks" >"$fakes/wpctl"
    # shellcheck disable=SC2016 # the stand-in expands its own arguments
    printf '#!/bin/sh\necho grim "$@" >>"%s"\nfor last; do :; done\n: >"$last"\n' "$marks" >"$fakes/grim"
    chmod +x "$fakes"/*
    export PATH=$fakes:$bin:$PATH
}
marked() {
    until_ok "grep -Eq -- \"$1\" \"$marks\"" 20
}
unmark() { : >"$marks"; }
until_ok() {
    local _
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null; }
click() {
    echo "[{\"pointer_move\": {\"position\": [$1, $2]}}, {\"click\": {\"button\": 272}}]" | ctl inject - >/dev/null
}
has_text() { ctl query '*' 2>/dev/null | grep -Fq -- "$1"; }

doors_run() {
    local look=$1 id fn result bad=0
    for id in $doors_cells; do
        [ -z "${ONLY:-}" ] || [[ $id =~ ^(${ONLY})$ ]] || continue
        fn=cell_${id//-/_}
        result=missing
        if declare -F "$fn" >/dev/null && "$fn"; then
            result=pass
        fi
        [ "$result" = pass ] || bad=1
        echo "cell $look $id $result"
    done
    return "$bad"
}
