#!/bin/bash
# The layout audit of the tiling look (reports/pixel-tiling-v2.md of the
# log, "Automatable checks"): tests/tiling-audit.py in each state, in each
# theme and at each scale. Each cell prints `ok:` or `FAIL:` with the
# findings of its flaw classes:
#   tokens      a helper surface fill that is no token of the theme, an
#               unfocused tile border that is not `line`
#   radius      corners of the helper surfaces outside the radius family
#   overlap     a helper surface under the bar or on a tile border
#   fullscreen  the bar islands over a fullscreen window
# The cells: desktop (two tiles and the "Chat ..." row), chat (the chat
# open), question (the card), fullscreen (a fullscreen window).
#
# Usage: tests/gate-tiling-audit.sh   BIN: directory with the binaries
#   SIZES   the outputs (default "1280x800 2560x1600@2")
#   THEMES  the themes (default "ink paper")
#   CELLS   a regular expression of the cells that run (default all)
#   SHOTS   a directory that keeps a capture and the nodes of each cell
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-tiling chora-assistant
need foot
need waybar
need jq
need mkfifo
temp_config
work=$XDG_CONFIG_HOME
shots=${SHOTS:-$work/shots}
mkdir -p "$work/chora" "$work/data/chora/huds" "$shots"
cp -rL "$repo/plugins/chora-assistant/huds/neutral" "$work/data/chora/huds/neutral"
export PATH=$bin:$PATH
export CHORA_BIN=$bin
flavor=$repo/flavors/tiling
failed=0
bus=$(dbus-daemon --session --print-address --fork 2>/dev/null | head -n 1 | cut -d, -f1)
[ -n "$bus" ] || bus=
export DBUS_SESSION_BUS_ADDRESS=$bus

emit() { echo "$1 $2" >&3; }
shows() { ctl query "id:$1" | jq -e '.[0].rects | length > 0' >/dev/null 2>&1; }
until_shows() {
    for _ in $(seq 40); do
        shows "$1" && return 0
        sleep 0.25
    done
    return 1
}

audit() {
    local cell=$1 tag=$2 theme=$3 scale=$4 out bar=
    [[ $cell =~ ^(${CELLS:-.*})$ ]] || return 0
    sleep 1
    ctl query '*' >"$shots/audit-$cell-$tag.json"
    ctl capture "$shots/audit-$cell-$tag.png" >/dev/null 2>&1
    [ "$(pixel "$shots/audit-$cell-$tag.png" $((24 * ${scale%.*})) $((24 * ${scale%.*})))" != "$(pixel "$shots/audit-$cell-$tag.png" $((24 * ${scale%.*})) $((300 * ${scale%.*})))" ] && bar=bar
    if out=$(python3 -I "$repo/tests/tiling-audit.py" "$shots/audit-$cell-$tag.png" "$shots/audit-$cell-$tag.json" "$theme" "$scale" "$cell" $bar); then
        ok "$cell $tag${bar:+ (bar)}"
    else
        echo "FAIL: $cell $tag:"
        printf '  %s\n' "$out"
        failed=1
    fi
}

round() {
    local theme=$1 size=$2 tag=$1-$2 scale=1
    [[ $size == *@* ]] && scale=${size##*@}
    mkdir -p "$work/chora"
    printf 'enabled: true\nposition: null\nhud: neutral\nhuds: []\n' >"$work/chora/chora-assistant.yaml"
    "$bin/chora-tiling" --print-config >"$work/chora/chora-tiling.yaml"
    sed -i "s/^  theme: system/  theme: $theme/" "$work/chora/chora-tiling.yaml"
    session_start "$size" -- "$bin/chora-tiling"
    grant screen.capture input.inject methods.call
    CHORA_FLAVOR_DIR=$flavor setsid sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/dev/null 2>&1 &
    run_pid $!
    rm -f "$work/voice.in"
    mkfifo "$work/voice.in"
    python3 -I "$repo/tests/fake-voice.py" "$work/claims" <"$work/voice.in" >"$shots/voice-$tag.log" 2>&1 &
    pids+=($!)
    exec 3>"$work/voice.in"
    XDG_DATA_HOME=$work/data "$bin/chora-assistant" >"$shots/assistant-$tag.log" 2>&1 &
    pids+=($!)
    for i in 1 2; do ctl run foot -T "w$i" sh -c 'echo hello; sleep 600' >/dev/null || fail "$tag: foot"; done
    ctl wait id:hud-row-chat --timeout 20 >/dev/null 2>&1
    for _ in $(seq 30); do
        sleep 1
        ctl capture "$shots/probe-$tag.png" >/dev/null 2>&1 &&
            [ "$(pixel "$shots/probe-$tag.png" $((24 * ${scale%.*})) $((24 * ${scale%.*})))" != "$(pixel "$shots/probe-$tag.png" $((24 * ${scale%.*})) $((300 * ${scale%.*})))" ] && break
    done
    audit desktop "$tag" "$theme" "$scale"

    emit voice.state '{"state": "question", "task": "Write file", "said": ""}'
    emit voice.confirm '{"id": "51", "name": "Atlas", "prompt": "Atlas wants to write the file notes.txt.", "detail": "~/notes/notes.txt", "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true, "timeout_s": 42}'
    until_shows hud-card || echo "FAIL: $tag: no question card"
    audit question "$tag" "$theme" "$scale"
    emit voice.confirm '{"id": "51", "done": true}'
    emit voice.state '{"state": "listening", "task": "", "said": ""}'
    sleep 0.5

    ctl call assistant chat.show '{"name": "atlas"}' >/dev/null 2>&1
    until_shows hud-chat || echo "FAIL: $tag: no chat"
    audit chat "$tag" "$theme" "$scale"
    ctl call assistant chat.hide >/dev/null 2>&1

    ctl run foot -F -T fullscreen sh -c 'echo fullscreen; sleep 600' >/dev/null || fail "$tag: foot"
    sleep 3
    audit fullscreen "$tag" "$theme" "$scale"
    exec 3>&-
    for p in "${pids[@]}" "${run_pids[@]}"; do kill "$p" 2>/dev/null; done
    wait 2>/dev/null
    pids=()
    run_pids=()
}

for theme in ${THEMES:-ink paper}; do
    for size in ${SIZES:-1280x800 2560x1600@2}; do
        round "$theme" "$size"
    done
done
[ "$failed" = 0 ] || fail "the layout audit found flaws"
ok "the layout audit"
