#!/bin/bash
# Gate of the layout of the construct HUD (design/construct-hud-spec.md
# of the log): the layout audit of tests/hud-audit.py in each state of
# the spec. The construct runs with `voice_hud: host`, the host
# chora-assistant with the folder plugins/chora-construct/hud, a fake
# voice agent with a transcript of the helper Atlas and a foot pane with a
# long title.
#
# The cells, each an audit of the whole screen, in each theme and output:
#   start        the start card of the empty floor, centred, with no hints
#   walk         the pane label and the walk hints
#   question     a physical question with Yes and No
#   chat         the chat open with its transcript
#   chat-ask     the chat open while a question waits
#   list         the window list (Tab)
#   list-search  the window list filtered by typed letters
#   help         the help (F1)
#   help-search  the help filtered by "pane"
# The audit fails on helper surfaces that overlap, a surface or a text
# outside the work area, an image that is not drawn pixel for pixel, and a
# text past its surface. Each cell prints `ok:` or `FAIL:` with the
# findings; the gate fails when a cell fails. Each round ends with the
# window audit: the window of the pane is still there.
#
# The empty floor of each round is also driven: a real press on the row
# Terminal of the start card starts a terminal that opens flown in and
# takes typed keys, and the card fades; the close of its pane brings the
# card back; Super+T after that starts the terminal again (the keyboard
# after the act), and its close ends it.
#
# Then the use mode is driven: Alt+Escape during the fly-in steps back;
# Return flies in and the hint Alt Esc Step back is up; a real press on
# that hint steps back; Super+T from use mode steps back, places a second
# pane right of the first at the same distance and flies into it; the
# typed keys reach the right terminal; Alt+Escape steps back; the window
# audit finds every terminal as a pane.
#
# The new terminal never shows a grey slab: from the moment its pane
# stands, each sample finds the placeholder with the ring or the first
# buffer of the window. The label of its pane reads Terminal, not the app
# id, no label shows in use mode, and none is left 400 ms after the pane
# closes. A give to a window manager that does not run says so in a notice
# with the reason, and loses no window.
#
# The walk hints show only what applies, at most five, centred and 32 from
# the bottom: with one pane `W A S D Walk`, `Fly in`, `Super T Terminal`
# and `F1 All keys`, with three panes also `Tab Windows`. The last cells
# of a round use three panes: the hint and the rows of the window list
# take a hover and a real press (a click on a row flies into its window, a
# click on the hint Terminal starts a terminal that opens flown in), each
# followed by the key Alt+Escape (the keyboard after the act), and the
# round ends with the window audit.
#
# Usage: tests/gate-construct-layout.sh   BIN: directory with the binaries
#   SIZES   the outputs (default "1280x720 2560x1440@2")
#   THEMES  the themes (default "light dark")
#   CELLS   a regular expression of the cells that run (default all)
#   SHOTS   a directory that keeps a capture and the nodes of each cell
#           as layout-CELL-THEME-SIZE.png and .json
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-assistant
need foot
need jq
need mkfifo
temp_config
work=$XDG_CONFIG_HOME
shots=${SHOTS:-$work/shots}
mkdir -p "$work/chora" "$work/data/chora/huds" "$shots"
cp -rL "$repo/plugins/chora-construct/hud" "$work/data/chora/huds/construct"
export PATH=$bin:$PATH
export CHORA_BIN=$bin
failed=0

key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null; }
typed() { echo "[{\"text\": {\"text\": \"$1\"}}]" | ctl inject >/dev/null; }
emit() { echo "$1 $2" >&3; }
rect_of() { ctl query "id:$1" | jq -r '.[0].rects | to_entries[0].value | @tsv' 2>/dev/null; }
card_shows() { ctl query id:construct-start | jq -e 'length > 0 and (.[0].visible != false) and (.[0].opacity // 1) > 0.9' >/dev/null 2>&1; }
card_gone() { ! ctl query id:construct-start | jq -e '.[0] | (.visible != false) and (.opacity // 1) > 0.05' >/dev/null 2>&1; }
until_ok() {
    for _ in $(seq 40); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
pane_named() { ctl call construct construct.list | jq -e --arg n "$1" '.panes | any(.name | test($n; "i"))' >/dev/null 2>&1; }
shows() { ctl query "id:$1" | jq -e '.[0].rects | length > 0' >/dev/null 2>&1; }
hint_ids() { ctl query '*' | jq -r '[.[] | select(.kind == "rect" and (.id | test("^construct-hint-[a-z-]+$")) and (.rects | length > 0)) | .id | sub("construct-hint-"; "")] | sort | join(" ")'; }
hints_are() { [ "$(hint_ids)" = "$1" ]; }
logical() {
    local scale=1 w=${1%x*} h=${1#*x}
    [[ $1 == *@* ]] && scale=${1#*@}
    awk -v n="${w%@*}" -v m="${h%@*}" -v s="$scale" 'BEGIN { print n / s, m / s }'
}
any_front() { ctl query 'kind:surface' | jq -e 'any(.[]; .parent == "construct-front")' >/dev/null 2>&1; }
list_gone() { ctl query id:construct-list-body | jq -e 'length == 0 or (.[0].rects | length == 0)' >/dev/null 2>&1; }
hover_at() {
    local x y w h
    read -r x y w h < <(rect_of "$1")
    [ -n "${x:-}" ] || return 1
    echo "[{\"pointer_move\": {\"position\": [$((${x%.*} + ${w%.*} / 2)), $((${y%.*} + ${h%.*} / 2))]}}]" | ctl inject >/dev/null
}
filled() { ctl query "id:$1" | jq -e '.[0].color[3] > 0' >/dev/null 2>&1; }
press_at() {
    local x y w h
    read -r x y w h < <(rect_of "$1")
    [ -n "${x:-}" ] || return 1
    echo "[{\"pointer_move\": {\"position\": [$((${x%.*} + ${w%.*} / 2)), $((${y%.*} + ${h%.*} / 2))]}}, {\"click\": {}}]" | ctl inject >/dev/null
}
label_is() { [ "$(ctl query id:construct-label-body-text | jq -r '.[0].text // empty')" = "$1" ]; }
label_gone() { ctl query id:construct-label-body | jq -e 'length == 0 or .[0].visible == false or (.[0].opacity // 1) < 0.05' >/dev/null 2>&1; }
notice_is() { [ "$(ctl query id:construct-notice-text | jq -r '.[0].text // empty')" = "$1" ]; }
has_node() { ctl query "id:$1" | jq -e 'length > 0' >/dev/null 2>&1; }
has_buffer() { ctl query "id:$1" | jq -e '.[0].surface.size | (. != null) and .[0] > 0 and .[1] > 0' >/dev/null 2>&1; }
frames_have_content() {
    local surface=$1 seen=0 n
    for n in $(seq 80); do
        if has_node "construct-wait-$surface"; then
            seen=1
        elif has_buffer "$surface"; then
            echo "$seen"
            return 0
        else
            echo "no placeholder and no buffer in sample $n"
            return 1
        fi
    done
    echo "the window shows no buffer after 80 samples"
    return 1
}
front_is() { [ "$(front_count)" = "$1" ]; }
panes_are() { [ "$(panes_n)" = "$1" ]; }
parent_is() { [ "$(ctl query "id:$1" | jq -r '.[0].parent')" = "$2" ]; }
front_count() { ctl query 'app_id:startcell kind:surface' | jq '[.[] | select(.parent == "construct-front")] | length'; }
panes_n() { ctl call construct construct.list | jq '.panes | length'; }
hint_up() { ctl query id:construct-hint-step-back | jq -e 'length > 0 and (.[0].rects | length > 0)' >/dev/null 2>&1; }
typed_is() { [ "$(cat "$work/typed" 2>/dev/null)" = "$1" ]; }
until_shows() {
    for _ in $(seq 40); do
        shows "$1" && return 0
        sleep 0.25
    done
    return 1
}

entries() {
    cat <<'JSON'
[
 {"seq": 1, "role": "user", "text": "Please tidy the notes for Friday."},
 {"seq": 2, "role": "agent", "text": "I read notes.txt. I will sort the items by day and keep your wording."},
 {"seq": 3, "role": "tool", "text": "Read notes.txt"},
 {"seq": 4, "role": "tool", "text": "Sort 14 items by day"},
 {"seq": 5, "role": "user", "text": "Put the dentist first on Friday."},
 {"seq": 6, "role": "agent", "text": "Done. Friday has three items now, the dentist first."}
]
JSON
}

ask() {
    emit voice.state '{"state": "question", "task": "Write file", "said": ""}'
    emit voice.confirm '{"id": "51", "name": "Atlas", "prompt": "Atlas wants to write the file notes.txt.", "detail": "~/notes/notes.txt\n412 bytes · replaces 9 lines", "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true, "timeout_s": 42}'
}
answered() {
    emit voice.confirm '{"id": "51", "done": true}'
    emit voice.state '{"state": "listening", "task": "", "said": ""}'
}

audit() {
    local cell=$1 tag=$2 out
    [[ $cell =~ ^(${CELLS:-.*})$ ]] || return 0
    sleep 0.8
    ctl outputs >"$shots/layout-$cell-$tag.outputs.json"
    ctl query '*' >"$shots/layout-$cell-$tag.json"
    ctl capture "$shots/layout-$cell-$tag.png" >/dev/null 2>&1
    if out=$(python3 -I "$repo/tests/hud-audit.py" "$shots/layout-$cell-$tag.outputs.json" \
        "$shots/layout-$cell-$tag.json" construct assistant); then
        ok "$cell $tag: $out"
    else
        echo "FAIL: $cell $tag:"
        printf '  %s\n' "$out"
        failed=1
    fi
}

first_floor() {
    local tag=$1 scale=1 w=${2%x*} h=${2#*x} x y cw ch
    [[ $2 == *@* ]] && scale=${2#*@}
    w=$(awk -v n="${w%@*}" -v s="$scale" 'BEGIN { print n / s }')
    h=$(awk -v n="${h%@*}" -v s="$scale" 'BEGIN { print n / s }')
    until_ok card_shows || { echo "FAIL: $tag: the empty floor shows no start card"; failed=1; return; }
    audit start "$tag"
    read -r x y cw ch < <(rect_of construct-start-glass)
    awk -v x="$x" -v y="$y" -v cw="$cw" -v ch="$ch" -v w="$w" -v h="$h" 'BEGIN {
        exit !(cw == 448 && x + cw / 2 == w / 2 && y + ch / 2 == h / 2)
    }' || { echo "FAIL: $tag: the card $x,$y ${cw}x$ch is not 448 wide and centred in ${w}x$h"; failed=1; }
    ctl query id:construct-hints | jq -e 'length == 0 or (.[0].rects | length == 0) or .[0].visible == false' >/dev/null ||
        { echo "FAIL: $tag: a hint bar shows with the start card"; failed=1; }
    read -r x y cw ch < <(rect_of construct-start-row-0)
    echo "[{\"pointer_move\": {\"position\": [$((${x%.*} + ${cw%.*} / 2)), $((${y%.*} + ${ch%.*} / 2))]}}, {\"click\": {}}]" | ctl inject >/dev/null
    until_ok 'pane_named startcell' || { echo "FAIL: $tag: a click on the row Terminal started no terminal"; failed=1; return; }
    until_ok card_gone || { echo "FAIL: $tag: the card stays with a pane"; failed=1; }
    ok "$tag: a click on the row Terminal started a terminal and the card went"
    until_ok 'front_is 1' || { echo "FAIL: $tag: the terminal of the click did not open flown in"; failed=1; }
    : >"$work/typed"
    typed c
    until_ok 'typed_is c' || { echo "FAIL: $tag: a key typed after the click did not reach the terminal"; failed=1; }
    sleep 1
    key Alt+Escape
    sleep 1
    key Super+x
    until_ok 'ctl call construct construct.list | jq -e ".panes | length == 0" >/dev/null' ||
        { echo "FAIL: $tag: the terminal pane did not close: $(ctl call construct construct.list | jq -c .panes)"; failed=1; return; }
    until_ok card_shows || { echo "FAIL: $tag: the card did not come back on the empty floor"; failed=1; return; }
    key Super+t
    until_ok 'pane_named startcell' || { echo "FAIL: $tag: Super+T after the click started no terminal"; failed=1; return; }
    until_ok card_gone || { echo "FAIL: $tag: the card stays after Super+T"; failed=1; }
    ok "$tag: Super+T started a terminal and the card went"
    sleep 1
    key Alt+Escape
    sleep 1
    key Super+x
    until_ok 'ctl query app_id:startcell | jq -e "length == 0" >/dev/null' ||
        { echo "FAIL: $tag: the window of the start cell is left"; failed=1; }
}

use_mode() {
    local tag=$1 x y w h first second seen
    : >"$work/typed"
    key Super+t
    until_ok 'pane_named startcell' || { echo "FAIL: $tag: Super+T started no terminal"; failed=1; return; }
    if seen=$(frames_have_content "$(ctl call construct construct.list | jq -r '.panes[0].surface')"); then
        ok "$tag: every sample of the new pane showed content (placeholder seen: $seen)"
    else
        echo "FAIL: $tag: the new pane showed a slab: $seen"
        failed=1
    fi
    key Alt+Escape
    sleep 2
    until_ok 'label_is Terminal' || { echo "FAIL: $tag: the label of the terminal pane is '$(ctl query id:construct-label-body-text | jq -r '.[0].text')', not Terminal"; failed=1; }
    if front_is 0; then
        ok "$tag: Alt+Escape during the fly-in stepped back"
    else
        echo "FAIL: $tag: Alt+Escape during the fly-in did not step back"
        failed=1
        key Alt+Escape
        sleep 1
    fi
    key Return
    until_ok 'front_is 1' || { echo "FAIL: $tag: Return did not fly into the pane"; failed=1; return; }
    hint_up || { echo "FAIL: $tag: no hint Alt Esc Step back in use mode"; failed=1; return; }
    until_ok label_gone || { echo "FAIL: $tag: a pane label shows in use mode"; failed=1; }
    typed d
    until_ok 'typed_is d' || { echo "FAIL: $tag: a key typed in use mode did not reach the terminal: $(cat "$work/typed")"; failed=1; }
    key Alt+Escape
    until_ok 'front_is 0' || { echo "FAIL: $tag: Alt+Escape did not step back"; failed=1; return; }
    key Return
    until_ok 'front_is 1' || { echo "FAIL: $tag: Return did not fly in again"; failed=1; return; }
    until_ok hint_up || { echo "FAIL: $tag: no hint after the second fly-in"; failed=1; return; }
    read -r x y w h < <(rect_of construct-hint-step-back)
    [ -n "${x:-}" ] || { echo "FAIL: $tag: the hint Step back is not on the screen"; failed=1; return; }
    echo "[{\"pointer_move\": {\"position\": [$((${x%.*} + ${w%.*} / 2)), $((${y%.*} + ${h%.*} / 2))]}}, {\"click\": {}}]" | ctl inject >/dev/null
    until_ok 'front_is 0' || { echo "FAIL: $tag: a click on the hint Step back did not step back"; failed=1; return; }
    ok "$tag: a click on the hint Step back stepped back"
    echo '[{"pointer_move": {"position": [8, 8]}}]' | ctl inject >/dev/null
    key Return
    until_ok 'front_is 1' || { echo "FAIL: $tag: Return after the click did not fly in"; failed=1; return; }
    key Super+t
    until_ok 'panes_are 2' || { echo "FAIL: $tag: Super+T in use mode started no second terminal"; failed=1; return; }
    first=$(ctl call construct construct.list | jq -r '.panes[0].surface')
    second=$(ctl call construct construct.list | jq -r '.panes[1].surface')
    if seen=$(frames_have_content "$second"); then
        ok "$tag: every sample of the second pane showed content (placeholder seen: $seen)"
    else
        echo "FAIL: $tag: the second pane showed a slab: $seen"
        failed=1
    fi
    until_ok "parent_is $second construct-front" ||
        { echo "FAIL: $tag: the second terminal did not open flown in"; failed=1; return; }
    ! parent_is "$first" construct-front ||
        { echo "FAIL: $tag: the first terminal stays in front"; failed=1; }
    ctl call construct construct.list | jq -e '.panes | (.[1].at[0] > .[0].at[0]) and ((.[1].at[1] - .[0].at[1]) | fabs) < 1' >/dev/null ||
        { echo "FAIL: $tag: the second pane is not right of the first at the same distance: $(ctl call construct construct.list | jq -c '[.panes[].at]')"; failed=1; }
    ok "$tag: Super+T in use mode stepped back and flew into a second pane right of the first"
    : >"$work/typed"
    typed e
    until_ok 'typed_is e' || { echo "FAIL: $tag: a key typed after the second terminal did not reach it: $(cat "$work/typed")"; failed=1; }
    key Alt+Escape
    until_ok 'front_is 0' || { echo "FAIL: $tag: Alt+Escape after the second terminal did not step back"; failed=1; return; }
    key Super+t
    until_ok 'panes_are 3' || { echo "FAIL: $tag: Super+T in the walk started no third terminal"; failed=1; return; }
    until_ok 'front_is 1' || { echo "FAIL: $tag: the third terminal did not open flown in"; failed=1; return; }
    if ctl call construct construct.list | jq -e '.panes | ((.[2].at[1] - .[0].at[1]) | fabs) < 1 and (.[2].at[0] != .[0].at[0]) and (.[2].at[0] != .[1].at[0])' >/dev/null; then
        ok "$tag: Super+T in the walk opened a third pane beside the others at the same distance"
    else
        echo "FAIL: $tag: the third pane is not beside the others at the same distance: $(ctl call construct construct.list | jq -c '[.panes[].at]')"
        failed=1
    fi
    key Alt+Escape
    until_ok 'front_is 0' || { echo "FAIL: $tag: Alt+Escape after the third terminal did not step back"; failed=1; return; }
    for _ in 1 2 3 4; do
        panes_are 0 && break
        key Super+x
        sleep 1
    done
    until_ok 'ctl query app_id:startcell | jq -e "length == 0" >/dev/null' ||
        { echo "FAIL: $tag: a terminal window is left after the close"; failed=1; }
    sleep 0.4
    label_gone || { echo "FAIL: $tag: a pane label is left 400 ms after the last pane closed"; failed=1; }
}

hint_bar() {
    local tag=$1 w h x y bw bh
    read -r w h < <(logical "$2")
    read -r x y bw bh < <(rect_of construct-hints-glass)
    awk -v x="${x:-0}" -v y="${y:-0}" -v bw="${bw:-0}" -v bh="${bh:-0}" -v w="$w" -v h="$h" 'BEGIN {
        d = x + bw / 2 - w / 2
        exit !(bw > 0 && d > -0.5 && d < 0.5 && y + bh == h - 32)
    }' || { echo "FAIL: $tag: the hint bar $x,$y ${bw}x$bh is not centred and 32 from the bottom of ${w}x$h"; failed=1; }
}

more_panes() {
    local tag=$1 size=$2 n
    for n in 1 2; do
        foot -a "extra$n" -T "Extra $n" sh -c 'sleep 600' >/dev/null 2>&1 &
        run_pid $!
    done
    until_ok 'panes_are 3' || { echo "FAIL: $tag: three panes did not stand: $(ctl call construct construct.list | jq -c '[.panes[].name]')"; failed=1; return; }
    key Return
    until_ok any_front || { echo "FAIL: $tag: Return did not fly into a pane of three"; failed=1; return; }
    key Alt+Escape
    until_ok '! any_front' || { echo "FAIL: $tag: Alt+Escape did not step back from a pane of three"; failed=1; return; }
    if until_ok 'hints_are "fly-in keys terminal walk windows"'; then
        ok "$tag: three panes: $(hint_ids)"
    else
        echo "FAIL: $tag: the hints with three panes are: $(hint_ids)"
        failed=1
    fi
    hint_bar "$tag" "$size"
    hover_at construct-hint-fly-in || echo "FAIL: $tag: no hint Fly in on the screen"
    until_ok 'filled construct-hint-fly-in' || { echo "FAIL: $tag: the hint Fly in has no hover fill under the pointer"; failed=1; }
    key Tab
    until_shows construct-list-row-1 || { echo "FAIL: $tag: the window list shows no second row"; failed=1; return; }
    sleep 0.5
    hover_at construct-list-row-1 || { echo "FAIL: $tag: no second row on the screen"; failed=1; return; }
    until_ok 'filled construct-list-row-1' || { echo "FAIL: $tag: the second row has no hover fill under the pointer"; failed=1; }
    press_at construct-list-row-1 || { echo "FAIL: $tag: no second row on the screen"; failed=1; return; }
    until_ok list_gone || { echo "FAIL: $tag: a click on a row of the window list left the list open"; failed=1; }
    until_ok any_front || { echo "FAIL: $tag: a click on a row of the window list did not fly into its window"; failed=1; return; }
    ok "$tag: a click on a row of the window list flew into its window"
    key Alt+Escape
    until_ok '! any_front' || { echo "FAIL: $tag: Alt+Escape after the row did not step back"; failed=1; return; }
    until_ok 'hints_are "fly-in keys terminal walk windows"' ||
        { echo "FAIL: $tag: the hints after the step back are: $(hint_ids)"; failed=1; return; }
    press_at construct-hint-terminal || { echo "FAIL: $tag: no hint Terminal on the screen"; failed=1; return; }
    until_ok 'pane_named startcell' || { echo "FAIL: $tag: a click on the hint Terminal started no terminal"; failed=1; return; }
    until_ok any_front || { echo "FAIL: $tag: the terminal of the hint did not open flown in"; failed=1; return; }
    ok "$tag: a click on the hint Terminal started a terminal flown in"
    key Alt+Escape
    until_ok '! any_front' || { echo "FAIL: $tag: Alt+Escape after the hint did not step back"; failed=1; return; }
    ctl query 'kind:surface' | jq -e 'length == 4' >/dev/null ||
        { echo "FAIL: $tag: a window is lost: $(ctl query 'kind:surface' | jq -c '[.[].surface.app_id]')"; failed=1; }
    key Super+Alt+2
    until_ok 'notice_is "XP did not take the window: it is not running."' ||
        { echo "FAIL: $tag: the notice of a refused give reads '$(ctl query id:construct-notice-text | jq -r '.[0].text')'"; failed=1; }
    key Alt+Escape
    ctl query 'kind:surface' | jq -e 'length == 4' >/dev/null ||
        { echo "FAIL: $tag: a refused give lost a window: $(ctl query 'kind:surface' | jq -c '[.[].surface.app_id]')"; failed=1; }
}

round() {
    local theme=$1 size=$2 tag=$1-$2 voice_pid
    cat >"$work/startcell.sh" <<SCRIPT
#!/bin/bash
while IFS= read -rn1 k; do printf %s "\$k" >>$work/typed; done
SCRIPT
    chmod +x "$work/startcell.sh"
    printf 'theme: %s\nvoice_hud: host\nterminal: foot -a startcell %s/startcell.sh\n' "$theme" "$work" >"$work/chora/chora-construct.yaml"
    printf 'enabled: true\nposition: null\nhud: construct\nhuds:\n  - {when: {space: construct}, folder: construct}\n' \
        >"$work/chora/chora-assistant.yaml"
    entries >"$work/entries.json"
    CHORA_TEST_FLAGS=--test-injected-press-is-real session_start "$size" -- "$bin/chora-construct"
    local session=${pids[-1]}
    grant screen.capture input.inject methods.call
    ctl wait id:construct-look --timeout 20 >/dev/null || fail "$tag: no space: $(tail -n 5 "$GATE_ENV")"
    first_floor "$tag" "$size"
    use_mode "$tag"
    rm -f "$work/voice.in"
    mkfifo "$work/voice.in"
    FAKE_VOICE_ENTRIES=$work/entries.json python3 -I "$repo/tests/fake-voice.py" "$work/claims" \
        <"$work/voice.in" >"$shots/voice-$tag.log" 2>&1 &
    voice_pid=$!
    pids+=("$voice_pid")
    exec 3>"$work/voice.in"
    XDG_DATA_HOME=$work/data "$bin/chora-assistant" >"$shots/assistant-$tag.log" 2>&1 &
    pids+=($!)
    emit voice.state '{"state": "working", "task": "Tidy notes", "said": ""}'
    ctl wait id:hud-figure --timeout 20 >/dev/null || fail "$tag: the host shows no figure"
    emit voice.state '{"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 1, "steps": 2}'

    foot -a sunset -T 'Sunset over the Donaudampfschifffahrtsgesellschaft harbour' \
        -f 'DejaVu Sans Mono:size=10' sh -c 'sleep 600' >/dev/null 2>&1 &
    run_pid $!
    for _ in $(seq 40); do
        ctl call construct construct.list | grep -q '"name": "sunset' && break
        sleep 0.25
    done
    key Return
    until_ok any_front || { echo "FAIL: $tag: Return did not fly into the pane of the sunset"; failed=1; }
    key Alt+Escape
    until_ok '! any_front' || { echo "FAIL: $tag: Alt+Escape did not step back from the pane of the sunset"; failed=1; }
    if until_ok 'hints_are "fly-in keys terminal walk"'; then
        ok "$tag: one pane: $(hint_ids)"
    else
        echo "FAIL: $tag: the hints with one pane are: $(hint_ids)"
        failed=1
    fi
    hint_bar "$tag" "$size"
    echo '[{"pointer_move": {"position": [24, 712]}}]' | ctl inject >/dev/null
    sleep 0.5
    audit walk "$tag"

    ask
    until_shows hud-card || echo "FAIL: $tag: no question card"
    audit question "$tag"

    ctl call assistant chat.show '{"name": "atlas"}' >/dev/null 2>&1
    until_shows hud-chat || echo "FAIL: $tag: no chat"
    audit chat-ask "$tag"
    answered
    sleep 0.5
    audit chat "$tag"
    ctl call assistant chat.hide '{}' >/dev/null 2>&1
    sleep 0.5

    key Tab
    sleep 0.6
    audit list "$tag"
    typed sun
    audit list-search "$tag"
    key Escape
    key Escape
    sleep 0.4

    key F1
    sleep 0.6
    audit help "$tag"
    typed pane
    audit help-search "$tag"
    key Escape
    key Escape
    ctl query 'app_id:sunset kind:surface' | jq -e 'length == 1' >/dev/null ||
        echo "FAIL: $tag: the window of the pane is lost"
    more_panes "$tag" "$size"

    exec 3>&-
    kill "${pids[@]}" "${run_pids[@]}" 2>/dev/null
    wait "$session" "$voice_pid" 2>/dev/null
    pids=()
    run_pids=()
}

for size in ${SIZES:-1280x720 2560x1440@2}; do
    for theme in ${THEMES:-light dark}; do
        round "$theme" "$size"
    done
done
[ "$failed" = 0 ] || fail "the layout audit of the construct HUD has findings"
ok "the construct HUD keeps its layout in every state"
