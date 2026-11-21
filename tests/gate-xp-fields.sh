#!/bin/bash
# The text fields of xp (xp-24) stand on the grid of xp-v2.md: the edit box
# of the chat is 28 px high with a 1 px border of #7F9DB9, and its top and
# bottom edges are those of the Send button beside it, at scale 1 and 2.
# The Run box is fuzzel: its settings stay on the grid of 4 px, and the
# label is in the text colour.
# The Run box, rendered: a row of 28 px and its label, from the pad, 5 px right of the rows (the horizontal pad of fuzzel.ini less 7: what a pad of 12 makes; the glyph bearing of the font, which differs by 1 px between 1x and 2x, cancels), at 1x and 2x; a pad of 6 fails; fuzzel before 1.11 does not read the file of xp, and the cell skips there.
# The input row of fuzzel has no box (xp-style 13): the label and the text typed after it stand on one baseline, within 1 px.
# The fields of the dialogs (Date and Time, Display Properties): a field is 21 px high with a border of 1 px (#7F9DB9) and the text 3 px inside it (the first ink of the year, a digit without left bearing, is 3 px from the inner edge); a label beside a field stands on its baseline, within 1 px.
# Help at an output of 720 px: the key list ends inside its pane (ca-1): the window fits or the page scrolls (a wheel to the end), so that below the last rule of the table, the one of the last key row, the pane has no ink and at least 12 px of the padding of 16 are left.
# BIN: directory with the binaries, SHOTS: directory for the captures.
set -u
. "$(dirname "$0")/gate-lib.sh"
need jq
need fuzzel
need_bins chora choractl chora-xp chora-assistant
shots=${SHOTS:-$(mktemp -d /tmp/gate-xp-fields.XXXXXX)}
mkdir -p "$shots"

fuzzel_ini=$repo/flavors/xp/theme/fuzzel.ini
setting() { sed -n "s/^$1=//p" "$fuzzel_ini" | head -n 1; }
for key in horizontal-pad vertical-pad inner-pad; do
    value=$(setting "$key")
    [ $((value % 4)) = 0 ] || fail "the Run box: $key=$value is off the grid of 4 px"
done
[ "$(setting horizontal-pad)" = 12 ] || fail "the Run box: horizontal-pad is not 12"
grep -qx "prompt=000000ff" "$fuzzel_ini" || fail "the Run box: the label is not in the text colour"
ok "the Run box stands on the grid of 4 px with a label in the text colour"

field_cell() {
    local scale=$1 entries pid
    temp_config
    export HOME=$XDG_CONFIG_HOME/home
    mkdir -p "$HOME"
    session_start "1280x800@$scale" -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
    "$bin/chora-assistant" >"$shots/assistant-$scale.log" 2>&1 &
    pids+=($!)
    grant screen.capture
    entries=$shots/entries.json
    echo '[{"seq": 1, "role": "user", "text": "Please write notes.txt"}]' >"$entries"
    mkfifo "$shots/voice-$scale.in"
    FAKE_VOICE_ENTRIES=$entries python3 -I "$repo/tests/fake-voice.py" "$shots/claims-$scale" \
        <"$shots/voice-$scale.in" >"$shots/voice-$scale.log" 2>&1 &
    pid=$!
    pids+=("$pid")
    exec 5>"$shots/voice-$scale.in"
    echo 'voice.state {"name": "Atlas", "state": "working", "task": "Write file", "step": 2, "steps": 2, "seq": 2}' >&5
    sleep 2
    ctl call assistant chat.show '{"name": "atlas"}' >/dev/null || fail "chat.show failed"
    for _ in $(seq 50); do
        [ "$(ctl query id:hud-chat-send | jq length)" = 1 ] && break
        sleep 0.2
    done
    sleep 1.5
    ctl capture "$shots/chat-field-${scale}x.png" >/dev/null || fail "the capture failed"
    local sx sy
    read -r sx sy <<<"$(ctl query id:hud-chat-send | jq -r '.[0].rects | to_entries[0].value | [.[0], .[1]] | map(floor) | join(" ")')"
    local top height
    read -r top height <<<"$(python3 -I "$repo/tests/xp-field-rows.py" "$shots/chat-field-${scale}x.png" $((sx - 40)) "$scale")"
    [ "$top" != none ] || fail "${scale}x: no edit box found left of Send"
    [ "$height" = 28 ] || fail "${scale}x: the edit box is $height px high, not 28"
    [ "$top" = "$sy" ] || fail "${scale}x: the edit box starts at y $top, Send at $sy"
    ok "${scale}x: the edit box of the chat is 28 px high and level with Send"
    exec 5>&-
    gate_cleanup 0
    pids=()
}
if [ -z "${RUNBOX_ONLY:-}" ]; then
    field_cell 1
    field_cell 2
fi

runbox_cell() {
    local scale=$1 apps out
    temp_config
    export HOME=$XDG_CONFIG_HOME/home
    apps=$HOME/share/applications
    mkdir -p "$apps"
    for p in alpha:Alpha mid:Mid; do
        printf '[Desktop Entry]\nType=Application\nName=%s\nExec=foot -a %s\n' "${p#*:}" "${p%%:*}" >"$apps/${p%%:*}.desktop"
    done
    export XDG_DATA_DIRS=$HOME/share XDG_DATA_HOME=$HOME/none XDG_CACHE_HOME=$HOME/cache
    session_start "1280x800@$scale" -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
    grant screen.capture input.inject
    ctl run "$bin/chora-xp" --run-box "${RUNBOX_INI:-$fuzzel_ini}" foot >/dev/null || fail "the run of the Run box"
    for _ in $(seq 40); do
        session_pids -x fuzzel >/dev/null && break
        sleep 0.25
    done
    sleep 1.5
    ctl capture "$shots/runbox-${scale}x.png" >/dev/null || fail "the capture failed"
    out=$(python3 -I "$repo/tests/xp-runbox-rows.py" "$shots/runbox-${scale}x.png" "$scale")
    [ "$out" != none ] || fail "${scale}x: no Run box in the capture"
    local height text list gap
    gap=${RUNBOX_GAP:-$(($(setting horizontal-pad) - 7))}.0
    read -r height text list <<<"$out"
    [ "$height" = "${RUNBOX_ROW:-28}.0" ] || fail "${scale}x: the row of the Run box is $height px high, not ${RUNBOX_ROW:-28}"
    [ "$(python3 -I -c "print(float('$text') - float('$list') == $gap)")" = True ] ||
        fail "${scale}x: the text of the Run box stands $(python3 -I -c "print(float('$text') - float('$list'))") px right of its rows, not $gap (the pad of $fuzzel_ini less 7)"
    ok "${scale}x: the Run box has a row of $height px and its label $gap px right of the rows"
    echo '[{"text": {"text": "al"}}]' | ctl inject >/dev/null || fail "inject text"
    sleep 1
    ctl capture "$shots/runbox-typed-${scale}x.png" >/dev/null || fail "the capture failed"
    read -r label typed <<<"$(python3 -I "$repo/tests/xp-runbox-baseline.py" "$shots/runbox-typed-${scale}x.png" "$scale")"
    [ "${typed:-none}" != none ] || fail "${scale}x: no typed text in the Run box"
    [ "$(python3 -I -c "print(abs(float('$label') - float('$typed')) <= 1)")" = True ] ||
        fail "${scale}x: the label of the Run box ends on row $label, the typed text on row $typed"
    ok "${scale}x: the label and the typed text of the Run box stand on one baseline ($label, $typed)"
    gate_cleanup 0
    pids=()
}
runbox_ran=1
if [ -z "${RUNBOX_INI:-}" ] && fuzzel_older 1.11; then
    runbox_ran=0
    cell_skip runbox_cell "fuzzel $fuzzel_version < 1.11 does not read the file of xp: the baseline assert (the label and the typed text of the Run box on one baseline, within 1 px), the row and label asserts and the pad 6 check do not run"
fi
if [ "$runbox_ran" = 1 ]; then
    runbox_cell 1
    runbox_cell 2
fi
if [ -z "${RUNBOX_ONLY:-}" ] && [ "$runbox_ran" = 1 ]; then
    wrong=$shots/pad6.ini
    sed 's/^horizontal-pad=.*/horizontal-pad=6/' "$fuzzel_ini" >"$wrong"
    if RUNBOX_ONLY=1 RUNBOX_INI=$wrong SHOTS=$shots/pad6 "$0" >"$shots/pad6.log" 2>&1; then
        fail "the Run box with pad 6 passes the label check"
    fi
    grep -q "px right of its rows" "$shots/pad6.log" || fail "the Run box with pad 6 failed, but not at the label: $(tail -n 3 "$shots/pad6.log")"
    ok "a wrong pad (6) fails the label check"
fi

reds=()
red() {
    echo "RED: $*"
    reds+=("$*")
}
inj() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
centre() { ctl query "id:$1" | jq -r '.[0].rects | to_entries[0].value | [.[0] + .[2] / 2, .[1] + .[3] / 2] | map(floor) | join(",")'; }
rect() { ctl query "id:$1" | jq -r '.[0].rects | to_entries[0].value | map(floor) | join(" ")'; }
click_node() { inj "[{\"pointer_move\": {\"position\": [$(centre "$1")]}}, {\"click\": {}}]"; }
dialog_cell() {
    local scale=$1 w h x y fw out
    temp_config
    export HOME=$XDG_CONFIG_HOME/home
    mkdir -p "$HOME"
    session_start "1280x800@$scale" -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
    grant screen.capture input.inject
    read -r w h <<<"$(ctl outputs | jq -r '.[0].size | map(floor) | join(" ")')"
    sleep 2
    inj "[{\"pointer_move\": {\"position\": [$((w - 30)), $((h - 12))]}}, {\"click\": {}}, {\"click\": {}}]"
    ctl wait id:xp-dlg-datetime-content-c-year --timeout 10 >/dev/null || fail "${scale}x: Date and Time does not open"
    sleep 1
    ctl capture "$shots/datetime-${scale}x.png" >/dev/null || fail "the capture failed"
    read -r x y _ _ <<<"$(rect xp-dlg-datetime-content-c-year)"
    out=$(python3 -I "$repo/tests/xp-field-text.py" "$shots/datetime-${scale}x.png" "$scale" $((x - 55)) "$y" 54 21)
    [ "$out" != none ] || fail "${scale}x: no year field in Date and Time"
    read -r height thick pad _ _ <<<"$out"
    [ "$height" = 21.0 ] || red "${scale}x: the year field is $height px high, not 21"
    [ "$thick" = 1.0 ] || red "${scale}x: the border of the year field is $thick px, not 1"
    [ "$(python3 -I -c "print(3 <= float('$pad') <= 3.5)")" = True ] || red "${scale}x: the text of the year field stands $pad px inside its border, not 3 (3.5 at 2x: one device pixel of the glyph)"
    read -r x y fw _ <<<"$(rect xp-dlg-datetime-content-c-time)"
    out=$(python3 -I "$repo/tests/xp-field-text.py" "$shots/datetime-${scale}x.png" "$scale" "$x" "$y" $((fw - 18)) 21)
    [ "$out" != none ] || fail "${scale}x: no time field in Date and Time"
    read -r height thick pad _ _ <<<"$out"
    [ "$height" = 21.0 ] || red "${scale}x: the time field is $height px high, not 21"
    [ "$thick" = 1.0 ] || red "${scale}x: the border of the time field is $thick px, not 1"
    [ "$(python3 -I -c "print(3 <= float('$pad') <= 3.5)")" = True ] || red "${scale}x: the text of the time field stands $pad px inside its border, not 3"
    read -r x y _ _ <<<"$(rect xp-dlg-datetime-content-c-month)"
    out=$(python3 -I "$repo/tests/xp-field-text.py" "$shots/datetime-${scale}x.png" "$scale" "$x" "$y" 98 21)
    [ "$out" != none ] || fail "${scale}x: no month box in Date and Time"
    read -r height thick _ _ _ <<<"$out"
    [ "$height" = 21.0 ] || red "${scale}x: the month box is $height px high, not 21"
    [ "$thick" = 1.0 ] || red "${scale}x: the border of the month box is $thick px, not 1"
    click_node xp-dlg-datetime-content-c-tab-1
    ctl wait id:xp-dlg-datetime-content-c-ok --timeout 10 >/dev/null || fail "${scale}x: no Time Zone page"
    sleep 1
    ctl capture "$shots/timezone-${scale}x.png" >/dev/null || fail "the capture failed"
    read -r x y _ _ <<<"$(rect xp-dlg-datetime-content-c-ok)"
    out=$(python3 -I "$repo/tests/xp-field-text.py" "$shots/timezone-${scale}x.png" "$scale" $((x - 156 + 13)) $((y - 287 + 40)) 370 21)
    [ "$out" != none ] || fail "${scale}x: no time zone box in Date and Time"
    read -r height thick pad _ _ <<<"$out"
    [ "$height" = 21.0 ] || red "${scale}x: the time zone box is $height px high, not 21"
    [ "$thick" = 1.0 ] || red "${scale}x: the border of the time zone box is $thick px, not 1"
    [ "$(python3 -I -c "print(3 <= float('$pad') <= 3.5)")" = True ] || red "${scale}x: the text of the time zone box stands $pad px inside its border, not 3"
    zone_pad=$pad
    click_node xp-dlg-datetime-content-c-cancel
    ctl wait id:xp-dlg-datetime --gone --timeout 10 >/dev/null || fail "${scale}x: Date and Time stays"
    inj "[{\"pointer_move\": {\"position\": [$((w / 2 + 60)), 150]}}, {\"click\": {\"button\": 273}}]"
    ctl wait id:xp-ctx-0-6 --timeout 10 >/dev/null || fail "${scale}x: the menu of the desktop does not open"
    click_node xp-ctx-0-6
    ctl wait id:xp-dlg-display-content-c-tab-2 --timeout 10 >/dev/null || fail "${scale}x: Display Properties does not open"
    click_node xp-dlg-display-content-c-tab-2
    ctl wait id:xp-dlg-display-content-c-wait --timeout 10 >/dev/null || fail "${scale}x: no Screen Saver page"
    sleep 1
    ctl capture "$shots/display-${scale}x.png" >/dev/null || fail "the capture failed"
    read -r x y _ _ <<<"$(rect xp-dlg-display-content-c-wait)"
    out=$(python3 -I "$repo/tests/xp-field-text.py" "$shots/display-${scale}x.png" "$scale" "$x" "$y" 50 21 $((x - 30)) $((x - 2)))
    [ "$out" != none ] || fail "${scale}x: no Wait field in Display Properties"
    read -r height thick wait_pad base label <<<"$out"
    [ "$height" = 21.0 ] || red "${scale}x: the Wait field is $height px high, not 21"
    [ "$thick" = 1.0 ] || red "${scale}x: the border of the Wait field is $thick px, not 1"
    [ "$(python3 -I -c "print(3 <= float('$wait_pad') <= 3.5)")" = True ] || red "${scale}x: the text of the Wait field stands $wait_pad px inside its border, not 3"
    read -r x y _ _ <<<"$(rect xp-dlg-display-content-c-saver)"
    out=$(python3 -I "$repo/tests/xp-field-text.py" "$shots/display-${scale}x.png" "$scale" "$x" "$y" 170 21)
    [ "$out" != none ] || fail "${scale}x: no screen saver list in Display Properties"
    read -r height thick pad _ _ <<<"$out"
    [ "$height" = 21.0 ] || red "${scale}x: the screen saver list is $height px high, not 21"
    [ "$thick" = 1.0 ] || red "${scale}x: the border of the screen saver list is $thick px, not 1"
    [ "$(python3 -I -c "print(3 <= float('$pad') <= 3.5)")" = True ] || red "${scale}x: the text of the screen saver list stands $pad px inside its border, not 3"
    [ "$(python3 -I -c "print(abs(float('$base') - float('$label')) <= 1)")" = True ] ||
        red "${scale}x: the text of the Wait field ends on row $base, its label on row $label"
    gate_cleanup 0
    pids=()
    [ "${#reds[@]}" = 0 ] || fail "${#reds[@]} measures are off the grid: ${reds[*]}"
    ok "${scale}x: the year, time, month and time zone fields of Date and Time, the Wait field and the screen saver list of Display are 21 px high with a border of 1 px; the text stands 3 px inside (time zone $zone_pad, Wait $wait_pad); Wait and its field stand on one baseline ($base, $label)"
}
if [ -z "${RUNBOX_ONLY:-}" ]; then
    dialog_cell 1
    dialog_cell 2
fi

help_cell() {
    local x y bh col last ink below
    temp_config
    export HOME=$XDG_CONFIG_HOME/home
    mkdir -p "$HOME"
    session_start "1280x720@1" -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
    grant screen.capture input.inject
    sleep 2
    inj '[{"shortcut": {"keys": "Super+F1"}}]'
    ctl wait id:xp-dlg-help-content-body --timeout 10 >/dev/null || fail "720: Help does not open"
    read -r x y _ bh <<<"$(rect xp-dlg-help-content-body)"
    inj "[{\"pointer_move\": {\"position\": [$((x + 500)), $((y + 200))]}}, {\"scroll\": {\"dy\": 400}}]"
    sleep 1.5
    ctl capture "$shots/help-720.png" >/dev/null || fail "the capture failed"
    col=$((x + 244 + 195))
    read -r last ink below <<<"$(python3 -I "$repo/tests/xp-help-rows.py" "$shots/help-720.png" "$col" "$((x + 244 + 214))" "$y" "$bh")"
    [ "$last" != none ] || fail "720: no table in the key list of Help"
    [ "$ink" = 0 ] || red "720: the key list of Help shows $ink rows of ink below its last rule (row $last): its last row is cut by the pane"
    [ "$below" -ge 12 ] || red "720: the last rule of the key list stands $below px above the bottom of the pane, the padding of 16 px is cut"
    gate_cleanup 0
    pids=()
    [ "${#reds[@]}" = 0 ] || fail "${reds[*]}"
    ok "720: the last key row of Help lies inside its pane, $below px above its bottom"
}
if [ -z "${RUNBOX_ONLY:-}" ]; then
    help_cell
fi
