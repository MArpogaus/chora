#!/bin/bash
# The text fields of xp (xp-24) stand on the grid of xp-v2.md: the edit box
# of the chat is 28 px high with a 1 px border of #7F9DB9, and its top and
# bottom edges are those of the Send button beside it, at scale 1 and 2.
# The Run box is fuzzel: its settings stay on the grid of 4 px, and the
# label is in the text colour.
# The Run box, rendered: a row of 28 px and its label, from the pad, 5 px right of the rows (the horizontal pad of fuzzel.ini less 7: what a pad of 12 makes; the glyph bearing of the font, which differs by 1 px between 1x and 2x, cancels), at 1x and 2x; a pad of 6 fails; fuzzel before 1.11 does not read the file of xp, and the cell skips there.
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
    grant screen.capture
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
    gate_cleanup 0
    pids=()
}
[ -n "${RUNBOX_INI:-}" ] || need_fuzzel 1.11
runbox_cell 1
runbox_cell 2
if [ -z "${RUNBOX_ONLY:-}" ]; then
    wrong=$shots/pad6.ini
    sed 's/^horizontal-pad=.*/horizontal-pad=6/' "$fuzzel_ini" >"$wrong"
    if RUNBOX_ONLY=1 RUNBOX_INI=$wrong SHOTS=$shots/pad6 "$0" >"$shots/pad6.log" 2>&1; then
        fail "the Run box with pad 6 passes the label check"
    fi
    grep -q "px right of its rows" "$shots/pad6.log" || fail "the Run box with pad 6 failed, but not at the label: $(tail -n 3 "$shots/pad6.log")"
    ok "a wrong pad (6) fails the label check"
fi
