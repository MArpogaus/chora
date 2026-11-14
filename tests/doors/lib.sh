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
#                   volume keys as Vol+, Vol- and Mute, with the words
#                   Volume up, Volume down and Mute
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
#   center ID       the middle of the node, `X Y`; click_node ID clicks it
#   doors_voice [none]  starts chora-voice with the stand-in helper of
#                   services/voice/tests/fake-acp.sh and the project demo;
#                   with `none` it has no project, so no helper can exist.
#                   voice_stop ends it
#   doors_assistant starts the HUD host chora-assistant
#   helper_start    starts the helper `coder` for the project demo
#   doors_window    shows a window that writes `key CODE` for each key it
#                   gets to $work/window; got_key CODE reads it
#   fuzzel_stand_in a fuzzel in $fakes that writes `fuzzel ARGS`, a line
#                   `data XDG_DATA_DIRS` and its rows as `row TEXT` to
#                   $marks, and answers the content of $fakes/choice
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
center() {
    ctl query "id:$1" 2>/dev/null |
        jq -er '.[0].rects | to_entries[0].value | "\(.[0] + .[2] / 2 | floor) \(.[1] + .[3] / 2 | floor)"'
}
click_node() {
    local at
    at=$(center "$1") || return 1
    # shellcheck disable=SC2086 # two words, X and Y
    click $at
}
doors_voice() {
    work=$XDG_CONFIG_HOME
    export XDG_DATA_HOME=$work/data XDG_STATE_HOME=$work/state
    mkdir -p "$work/chora" "$work/demo"
    local projects="{demo: $work/demo}"
    [ "${1:-}" != none ] || projects="{}"
    cat >"$work/chora/chora-voice.yaml" <<YAML
capture_cmd: [sh, -c, exec sleep 1000]
playback_cmd: [sh, -c, exec cat >/dev/null]
asr_url: ws://127.0.0.1:9
tts_url: ws://127.0.0.1:9
llm_url: http://127.0.0.1:9/v1
earcons: false
indicator: false
unsandboxed: true
agent_cmd: [sh, $repo/services/voice/tests/fake-acp.sh]
agent_projects: $projects
YAML
    "$bin/chora-voice" >>"$work/voice.log" 2>&1 &
    voice_pid=$!
    pids+=("$voice_pid")
    until_ok 'ctl call voice state >/dev/null 2>&1' 80
}
voice_stop() {
    kill "$voice_pid" 2>/dev/null
    wait "$voice_pid" 2>/dev/null
    until_ok '! ctl call voice state >/dev/null 2>&1' 40
}
doors_assistant() {
    "$bin/chora-assistant" >"$work/assistant.log" 2>&1 &
    pids+=($!)
    until_ok "ctl query owner:assistant | jq -e 'length > 0' >/dev/null" 80
}
helper_start() {
    ctl call voice agent '{"tool": "agent_start", "project": "demo", "name": "coder"}' >/dev/null
}
doors_window() {
    "$bin/chora-probe" window 40 120 200 --report >"$work/window" 2>&1 &
    pids+=($!)
    sleep 2
}
got_key() { until_ok "grep -qx 'key $1' \"$work/window\"" 12; }
fuzzel_stand_in() {
    cat >"$fakes/fuzzel" <<SH
#!/bin/sh
echo fuzzel "\$@" >>"$marks"
echo "data \$XDG_DATA_DIRS" >>"$marks"
case " \$* " in
*" --dmenu "*)
    sed 's/^/row /' >>"$marks"
    cat "$fakes/choice" 2>/dev/null
    ;;
esac
SH
    chmod +x "$fakes/fuzzel"
}

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
