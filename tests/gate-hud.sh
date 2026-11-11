#!/bin/bash
# Gate of the HUD sprint (design/hud-sprint.md, step H0): a table of cells,
# one per acceptance check that a script can verify headlessly with
# tests/headless-session.sh, `choractl capture` and `input.inject`.
# Each cell is measured as
#
#   pass     the feature works as the spec says
#   missing  anything else
#
# The table EXPECTED holds the state of every cell. A cell is `missing`
# until the step that builds its feature has merged. The gate fails on each cell that differs from the table, in
# both directions, so a step that builds a feature must flip its lines to
# `pass` in the same branch. It fails too when a cell has no line, when a
# line has no cell, and when the run takes more than 4 minutes. One line
# per cell goes to $SHOTS/hud.log, with the timeouts that expired.
#
# Cells, with the checks of design/ux-hud-sprint.md section 7:
#   check-*                 chora-assistant --check-hud on the four shipped
#                           folders (A15), the fixture tests/hud/mine (A12)
#   check-names-four-errors the fixture tests/hud/fancy (A13): a symlink
#                           out, an href to /etc/passwd, a 30000 px SVG and
#                           chat.size 10; exit 1, each error with its file
#   list-huds, use-hud-writes-setting   --list-huds, --use-hud (A12)
#   neutral-equals-today    the figure and the sentence of the neutral folder
#                           against captures of 92e7539 (tests/hud/today)
#   card-*                  the question card of a physical question (A1,
#                           A2, A11)
#   chat-*, typed-yes-held  the "Chat ..." row, chat.show and chat.hide,
#                           Super+a and Esc (A3, A9), a typed yes (A5)
#   agents-row-text         the helper row (A7)
#   xp-tray-rows            the xp folder shows the row of a working helper
#                           in its tray balloon (H8, F7)
#   mc-host-hud             chora-mc with `voice_hud: host` and the mc
#                           folder: the host draws the figure and the card,
#                           mc shows no sentence, no question, no claim (H9)
#   mc-chat-drawer          the chat of the mc folder fills the box that mc
#                           announces with mc.chat.slot (H9, F8)
#   construct-host-hud      chora-construct with `voice_hud: host` and the
#                           construct folder: the host draws the figure and
#                           the card, construct draws no HUD, no question
#                           glass and makes no claim (H10)
#   construct-chat-panel    the chat of the construct folder is a panel at
#                           the right, 480 wide, 32 from the edges of the
#                           work area (H10, F9)
#   voice-state-rate, layers-dropped-empty, host-idle-cpu   100 voice.state
#                           in 5 s, then no dropped layers and under 1 % CPU
#
# The node ids that the cells read are part of the contract of the steps
# H5 to H7: hud-figure, hud-state, hud-row-chat, hud-row-NAME, hud-card,
# hud-card-yes, hud-chat and hud-chat-hint.
#
# Persona only (a person reads a capture or the checks need the whole
# stack): A4 the answer of the helper in the chat, A6 the xp balloon that
# becomes the chat, A8 the muted badge and "Hört zu", A12 the colours of the
# capture, A13 the notice of 8 s, A14 stray characters, U1 to U5.
# Elsewhere: A10 and the windows with a chat open in gate-hud-handover (H11);
# A14 and A15 in gate-chat (H7); chat Enter to first agent text in 300 ms
# in gate-chat (H7), which needs the fake agent.
#
# Usage: tests/gate-hud.sh [--self-test]   BIN: directory with the binaries
#   SHOTS: directory for the log (default: a temp dir)
#   ONLY: a regular expression that must match a cell id; only those cells
#   run, the table is not fully checked, and the gate says so
#   FORCE_LINE: "ID STATE" replaces that line of the table, to show that a
#   wrong line fails the gate
#   --self-test: no session; the comparison must pass for a table equal to
#   the measurements and fail for a forced wrong line
# shellcheck disable=SC2016
set -uo pipefail

EXPECTED='
check-neutral pass
check-xp pass
check-mc pass
check-construct pass
check-ok-names-use pass
check-names-four-errors pass
list-huds missing
use-hud-writes-setting pass
card-names-file pass
card-injected-click-refused pass
context-held-for-question pass
context-picks-folder pass
card-stays-on-focus-move pass
chat-row-always pass
chat-row-click-opens pass
chat-show-hide pass
chat-super-a-esc missing
typed-yes-held pass
agents-row-text missing
xp-tray-rows missing
mc-host-hud pass
mc-chat-drawer missing
construct-host-hud pass
construct-chat-panel missing
voice-state-rate pass
layers-dropped-empty pass
host-idle-cpu pass
neutral-equals-today pass
late-host-shows-figure pass
reconnected-voice-shows-figure pass
hung-voice-leaves-host-responsive pass
'

. "$(dirname "$0")/gate-lib.sh"
time_cap=240
declare -A got
mismatches=()

table() {
    if [ -n "${FORCE_LINE:-}" ]; then
        echo "$EXPECTED" | awk -v f="$FORCE_LINE" 'BEGIN {split(f, p, " ")} NF && $1 == p[1] {print p[1], p[2]; next} NF'
    else
        echo "$EXPECTED" | awk 'NF'
    fi
}

compare() {
    local id want
    mismatches=()
    while read -r id want; do
        [ -n "${ONLY:-}" ] && ! [[ $id =~ ^(${ONLY})$ ]] && continue
        [ -n "${got[$id]:-}" ] || {
            mismatches+=("$id: no measurement, expected $want")
            continue
        }
        [ "${got[$id]}" = "$want" ] || mismatches+=("$id: measured ${got[$id]}, expected $want")
    done < <(table)
    for id in "${!got[@]}"; do
        table | awk -v k="$id" '$1 == k {f = 1} END {exit !f}' || mismatches+=("$id: a cell with no line in EXPECTED")
    done
    [ "${#mismatches[@]}" = 0 ]
}

if [ "${1:-}" = --self-test ]; then
    while read -r id _; do got[$id]=missing; done < <(table)
    compare || fail "self-test: the table does not match all-missing: ${mismatches[*]}"
    first=$(table | awk 'NR == 1 {print $1}')
    FORCE_LINE="$first pass"
    compare && fail "self-test: a forced wrong line for $first did not fail"
    [[ ${mismatches[*]} == *"$first: measured missing, expected pass"* ]] ||
        fail "self-test: the failure does not name $first: ${mismatches[*]}"
    ok "gate-hud self-test: a forced wrong line fails and names its cell"
    exit 0
fi

need jq
need foot
need python3
need_bins chora choractl chora-assistant chora-tiling chora-mc chora-construct
temp_config
work=$XDG_CONFIG_HOME
shots=${SHOTS:-$work}
mkdir -p "$shots" "$work/chora"
export PATH=$bin:$PATH
export CHORA_BIN=$bin
log=$shots/hud.log
: >"$log"
fixtures=$repo/tests/hud
settle_s=5
grace_s=15

waiting() {
    local limit=$1 n
    shift
    n=$(awk -v s="$limit" 'BEGIN {print s * 5}')
    for _ in $(seq "$n"); do
        "$@" >/dev/null 2>&1 && return 0
        sleep 0.2
    done
    echo "timeout ${limit}s: $*" >>"$log"
    return 1
}

watch_for() {
    local seconds=$1 n
    shift
    n=$(awk -v s="$seconds" 'BEGIN {print s * 5}')
    for _ in $(seq "$n"); do
        "$@" >/dev/null 2>&1 || return 1
        sleep 0.2
    done
}

node_exists() { ctl query "id:$1" | jq -e 'length > 0' >/dev/null; }
node_has() { ctl query "id:$1" | jq -r '.[0] // {} | tostring' | grep -qiF -- "$2"; }
node_rect() { ctl query "id:$1" | jq -c '.[0].rects // null'; }
inject() { echo "$1" | ctl inject >/dev/null; }
key() { inject "[{\"shortcut\": {\"keys\": \"$1\"}}]"; }
click() {
    inject "[{\"pointer_move\": {\"node\": \"$1\", \"point\": [${2:-10}, ${3:-10}]}},
      {\"button\": {\"state\": \"pressed\"}}, {\"button\": {\"state\": \"released\"}}]"
}

has_focus() {
    for _ in $(seq 50); do
        ctl query "id:$1" | jq -e '.[0].focused == true' >/dev/null && return 0
        sleep 0.1
    done
    return 1
}

hud_check() {
    hud_out=$(timeout 10 "$bin/chora-assistant" --check-hud "$1" 2>&1)
    hud_code=$?
}
check_ships() {
    hud_check "$1"
    [ "$hud_code" = 0 ] && grep -q '^OK' <<<"$hud_out"
}
cell_check_neutral() { check_ships "$repo/plugins/chora-assistant/huds/neutral"; }
cell_check_xp() { check_ships "$repo/flavors/xp/assets/assistant"; }
cell_check_mc() { check_ships "$repo/plugins/chora-mc/hud"; }
cell_check_construct() { check_ships "$repo/plugins/chora-construct/hud"; }
cell_check_ok_names_use() {
    hud_check "$fixtures/mine"
    [ "$hud_code" = 0 ] && grep -q '^OK' <<<"$hud_out" && grep -qF -- '--use-hud mine' <<<"$hud_out"
}
cell_check_names_four_errors() {
    local secret
    hud_check "$fixtures/fancy"
    [ "$hud_code" = 1 ] || return 1
    grep -q 'hud.yaml:[0-9]*.*chat.size' <<<"$hud_out" || return 1
    for f in link.svg href.svg huge.svg; do
        grep -qF -- "$f" <<<"$hud_out" || return 1
    done
    for secret in "$(cat /etc/hostname 2>/dev/null)" 'root:x:0'; do
        [ "${#secret}" -lt 4 ] || ! grep -qF -- "$secret" <<<"$hud_out" || return 1
    done
}

private_dirs() {
    data=$(mktemp -d /tmp/gate-hud-data.XXXXXX)
    cfg=$(mktemp -d /tmp/gate-hud-cfg.XXXXXX)
    tmp_dirs+=("$data" "$cfg")
    mkdir -p "$data/chora/huds" "$cfg/chora"
    cp -r "$fixtures/mine" "$data/chora/huds/mine"
}
cell_list_huds() {
    local listed
    private_dirs
    listed=$(XDG_DATA_HOME=$data XDG_CONFIG_HOME=$cfg timeout 10 "$bin/chora-assistant" --list-huds 2>&1) || return 1
    for n in neutral xp mc construct mine; do
        grep -qw -- "$n" <<<"$listed" || return 1
    done
}
cell_use_hud_writes_setting() {
    private_dirs
    XDG_DATA_HOME=$data XDG_CONFIG_HOME=$cfg timeout 10 "$bin/chora-assistant" --use-hud mine >/dev/null 2>&1 || return 1
    grep -q '^hud: *mine' "$cfg/chora/chora-assistant.yaml" || return 1
    XDG_DATA_HOME=$data XDG_CONFIG_HOME=$cfg timeout 10 "$bin/chora-assistant" --use-hud default >/dev/null 2>&1 || return 1
    ! grep -q '^hud: *mine' "$cfg/chora/chora-assistant.yaml"
}

voice_in=$work/voice.in
session_up=0
assistant_pid=0
emit() { echo "$1 $2" >&3; }
ask_notes() {
    emit voice.state '{"state": "question", "task": "Write file", "said": ""}'
    emit voice.confirm '{"id": "51", "prompt": "Atlas wants to write the file notes.txt.", "detail": "~/projects/notes/notes.txt (412 bytes)", "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true}'
}
end_question() {
    emit voice.confirm '{"id": "51", "done": true}'
    emit voice.state '{"state": "listening", "task": "", "said": ""}'
}

voice_pid=0
start_voice() {
    rm -f "$voice_in"
    mkfifo "$voice_in"
    python3 -I "$repo/tests/fake-voice.py" "$work/claims" <"$voice_in" >"$shots/voice.log" 2>&1 &
    voice_pid=$!
    pids+=("$voice_pid")
    exec 3>"$voice_in"
}
start_assistant() {
    XDG_DATA_HOME=$work/data "$bin/chora-assistant" >"$shots/assistant.log" 2>&1 &
    assistant_pid=$!
    pids+=("$assistant_pid")
    waiting 20 host_connected
}
host_connected() { ctl query owner:assistant | jq -e 'length > 0'; }
stop_voice() {
    exec 3>&-
    kill "$voice_pid"
    wait "$voice_pid" 2>/dev/null
}
stop_assistant() {
    kill "$assistant_pid"
    wait "$assistant_pid" 2>/dev/null
}

ensure_session() {
    [ "$session_up" = 0 ] || return 0
    session_up=1
    session_start 1280x800
    grant input.inject screen.capture methods.call
    start_voice
    "$bin/chora-tiling" >"$shots/tiling.log" 2>&1 &
    pids+=($!)
    ctl wait owner:tiling --timeout 20 >/dev/null || fail "tiling does not start: $(tail -n 3 "$shots/tiling.log")"
    mkdir -p "$work/data/chora/huds"
    cp -r "$fixtures/mine" "$work/data/chora/huds/mine"
    cp -rL "$repo/flavors/xp/assets/assistant" "$work/data/chora/huds/xp"
    [ ! -d "$repo/plugins/chora-mc/hud" ] || cp -rL "$repo/plugins/chora-mc/hud" "$work/data/chora/huds/mc"
    [ ! -d "$repo/plugins/chora-construct/hud" ] || cp -rL "$repo/plugins/chora-construct/hud" "$work/data/chora/huds/construct"
    printf 'enabled: true\nposition: null\nhud: neutral\nhuds:\n  - {when: {app: probe2}, folder: mine}\n' >"$work/chora/chora-assistant.yaml"
    start_assistant || fail "chora-assistant does not connect: $(tail -n 3 "$shots/assistant.log")"
    foot -f "DejaVu Sans Mono:size=10" -a probe sh -c 'sleep 900' >"$shots/foot.log" 2>&1 &
    run_pid $!
    probe=$(ctl wait "app_id:probe kind:surface" --timeout 20 | jq -r '.[0].id') || fail "foot does not show"
    has_focus "$probe" || fail "foot has no focus at the start"
}

cell_neutral_equals_today() {
    emit voice.state '{"state": "listening", "task": "", "said": ""}'
    waiting "$settle_s" node_exists hud-figure || return 1
    sleep 2
    ctl capture "$shots/figure.png" --node hud-figure >/dev/null || return 1
    emit voice.say '{"text": "Your updates are ready to install.", "duration_ms": 4000}'
    waiting "$settle_s" node_exists hud-line || return 1
    sleep 1
    ctl capture "$shots/line.png" --node hud-line >/dev/null || return 1
    local part
    for part in figure line; do
        [ "$(python3 -I "$repo/tests/png-same.py" "$shots/$part.png" "$fixtures/today/$part.png")" = "0 0" ] || return 1
    done
}
cell_card_names_file() {
    ask_notes
    waiting "$settle_s" node_exists hud-card || return 1
    for text in notes.txt Atlas Alt+y; do node_has hud-card "$text" || return 1; done
    node_exists hud-card-yes && node_exists hud-card-no
}
cell_card_injected_click_refused() {
    ask_notes
    waiting "$settle_s" node_exists hud-card-yes || return 1
    click hud-card-yes
    waiting "$settle_s" node_has hud-card "Please use your own mouse or keyboard." || return 1
    node_exists hud-card-yes || return 1
    [ ! -s "$work/claims.answers" ]
}
figure_near() {
    jq -ne --argjson a "$1" --argjson b "$(node_rect hud-figure)" \
        '($a | to_entries[0].value) as $p | ($b | to_entries[0].value) as $q |
         (($p[0] - $q[0]) | fabs) < 4 and (($p[1] - $q[1]) | fabs) < 4' >/dev/null 2>&1
}
cell_context_held_for_question() {
    local card figure second
    ask_notes
    waiting "$settle_s" node_exists hud-card || return 1
    card=$(node_rect hud-card)
    figure=$(node_rect hud-figure)
    foot -f "DejaVu Sans Mono:size=10" -a probe2 sh -c 'sleep 900' >"$shots/foot-ctx.log" 2>&1 &
    run_pid $!
    second=$(ctl wait "app_id:probe2 kind:surface" --timeout 20 | jq -r '.[0].id') || return 1
    has_focus "$second" || return 1
    watch_for 1.5 card_rect_is "$card" || return 1
    figure_near "$figure"
}
figure_left() { ! figure_near "$1"; }
cell_context_picks_folder() {
    local figure
    figure=$(node_rect hud-figure)
    end_question
    waiting "$settle_s" figure_left "$figure"
}
card_rect_is() { [ "$(node_rect hud-card)" = "$1" ]; }
new_window() {
    ctl query "app_id:$1 kind:surface" | jq -re --argjson known "$2" '[.[].id] - $known | .[0] // empty'
}
cell_card_stays_on_focus_move() {
    local before known second
    ask_notes
    waiting "$settle_s" node_exists hud-card || return 1
    before=$(node_rect hud-card)
    known=$(ctl query "app_id:probe2 kind:surface" | jq -c '[.[].id]') || return 1
    foot -f "DejaVu Sans Mono:size=10" -a probe2 sh -c 'sleep 900' >"$shots/foot2.log" 2>&1 &
    run_pid $!
    waiting 20 new_window probe2 "$known" || return 1
    second=$(new_window probe2 "$known") || return 1
    has_focus "$second" || return 1
    watch_for 1.5 card_rect_is "$before"
}
cell_chat_row_always() { waiting "$settle_s" node_exists hud-row-chat; }
cell_chat_row_click_opens() {
    waiting "$settle_s" node_exists hud-row-chat || return 1
    click hud-row-chat
    waiting "$settle_s" node_exists hud-chat
}
cell_chat_show_hide() {
    ctl call assistant chat.show '{}' >/dev/null 2>&1 || return 1
    waiting "$settle_s" node_exists hud-chat || return 1
    ctl call assistant chat.hide '{}' >/dev/null 2>&1 || return 1
    ctl wait id:hud-chat --gone --timeout "$settle_s" >/dev/null
}
cell_chat_super_a_esc() {
    key Super+a
    waiting "$settle_s" node_exists hud-chat || return 1
    key Escape
    ctl wait id:hud-chat --gone --timeout "$settle_s" >/dev/null || return 1
    has_focus "$probe"
}
cell_typed_yes_held() {
    ask_notes
    waiting "$settle_s" node_exists hud-card || return 1
    click hud-row-chat
    waiting "$settle_s" node_exists hud-chat || return 1
    inject '[{"text": {"text": "ja\n"}}]'
    waiting "$settle_s" node_has hud-chat-hint "Typing does not answer" || return 1
    [ ! -s "$work/claims.answers" ]
}
cell_agents_row_text() {
    emit voice.state '{"name": "Atlas", "state": "working", "task": "Write file", "step": 2, "steps": 2}'
    waiting "$settle_s" node_has hud-row-Atlas "Atlas · working · Write file (2/2)"
}

use_folder() { ctl call assistant settings.set "{\"hud\": \"$1\"}" >/dev/null 2>&1; }
cell_xp_tray_rows() {
    local shown
    use_folder xp || return 1
    emit voice.state '{"name": "Atlas", "state": "working", "task": "Write file", "step": 2, "steps": 2}'
    waiting "$settle_s" node_has hud-row-Atlas "Atlas · working · Write file (2/2)"
    shown=$?
    use_folder neutral
    return "$shown"
}

mc_pid=0
start_mc() {
    [ "$mc_pid" = 0 ] || return 0
    printf 'voice_hud: host\n' >"$work/chora/chora-mc.yaml"
    "$bin/chora-mc" >"$shots/mc.log" 2>&1 &
    mc_pid=$!
    pids+=("$mc_pid")
    ctl wait owner:mc --timeout 20 >/dev/null
}
stop_mc() {
    [ "$mc_pid" != 0 ] || return 0
    kill "$mc_pid"
    wait "$mc_pid" 2>/dev/null
    mc_pid=0
}
claims_now() { cat "$work/claims" 2>/dev/null || echo 0; }
node_shows() { ctl query "id:$1" | jq -e '.[0] | . != null and .visible != false and (.opacity // 1) > 0' >/dev/null; }
owner_says() { ctl query "owner:$1" | jq -e --arg t "$2" 'any(.[]; (.meta.text // "") | contains($t))'; }
cell_mc_host_hud() {
    local before
    use_folder mc || return 1
    start_mc || return 1
    before=$(claims_now)
    emit voice.state '{"state": "listening", "task": "", "said": ""}'
    emit voice.say '{"text": "Your updates are ready to install.", "duration_ms": 1000}'
    waiting "$settle_s" node_exists hud-figure || return 1
    waiting "$settle_s" node_exists hud-line || return 1
    ! owner_says mc "Your updates" || return 1
    ask_notes
    waiting "$settle_s" node_exists hud-card || return 1
    sleep 1
    ! node_exists mc-answers || return 1
    ! owner_says mc notes.txt || return 1
    [ "$(claims_now)" = "$before" ]
}
slot_rect() {
    jq -c 'select(.event == "mc.chat.slot" and .data.rect != null) | .data.rect' "$slots_file" | tail -n 1
}
chat_in_slot() {
    local slot
    slot=$(slot_rect)
    [ -n "$slot" ] || return 1
    jq -ne --argjson s "$slot" --argjson c "$(node_rect hud-chat)" \
        '($c | to_entries[0].value) as $r |
         ([range(4)] | all(. as $i | (($r[$i] - $s[$i]) | fabs) < 2))' >/dev/null 2>&1
}
slots_file=$shots/slots.jsonl
cell_mc_chat_drawer() {
    local shown
    use_folder mc || return 1
    start_mc || return 1
    ctl events mc.chat.slot assistant.chat >"$slots_file" 2>/dev/null &
    pids+=($!)
    ctl call assistant chat.show '{"name": "atlas"}' >/dev/null 2>&1 || return 1
    waiting "$settle_s" node_exists hud-chat || return 1
    waiting "$settle_s" chat_in_slot
    shown=$?
    stop_mc
    use_folder neutral
    return "$shown"
}

construct_pid=0
start_construct() {
    [ "$construct_pid" = 0 ] || return 0
    printf 'voice_hud: host\n' >"$work/chora/chora-construct.yaml"
    "$bin/chora-construct" >"$shots/construct.log" 2>&1 &
    construct_pid=$!
    pids+=("$construct_pid")
    ctl wait owner:construct --timeout 20 >/dev/null
}
stop_construct() {
    [ "$construct_pid" != 0 ] || return 0
    kill "$construct_pid"
    wait "$construct_pid" 2>/dev/null
    construct_pid=0
}
cell_construct_host_hud() {
    local before
    use_folder construct || return 1
    start_construct || return 1
    before=$(claims_now)
    emit voice.state '{"state": "listening", "task": "", "said": ""}'
    emit voice.say '{"text": "Your updates are ready to install.", "duration_ms": 1000}'
    waiting "$settle_s" node_exists hud-figure || return 1
    waiting "$settle_s" node_exists hud-line || return 1
    sleep 1
    ! node_shows construct-hud || return 1
    ask_notes
    waiting "$settle_s" node_exists hud-card || return 1
    sleep 1
    ! node_shows construct-ask || return 1
    [ "$(claims_now)" = "$before" ]
}
chat_at_right() {
    local area
    area=$(ctl outputs | jq -c '.[0].work_area')
    jq -ne --argjson a "$area" --argjson c "$(node_rect hud-chat)" \
        '($c | to_entries[0].value) as $r |
         $r[2] == 480 and $r[0] + $r[2] == $a[0] + $a[2] - 32 and $r[1] == $a[1] + 32' >/dev/null 2>&1
}
cell_construct_chat_panel() {
    local shown
    use_folder construct || return 1
    start_construct || return 1
    ctl call assistant chat.show '{"name": "atlas"}' >/dev/null 2>&1 || return 1
    waiting "$settle_s" node_exists hud-chat || return 1
    waiting "$settle_s" chat_at_right
    shown=$?
    stop_construct
    use_folder neutral
    return "$shown"
}

figure_shows() { node_exists hud-figure; }
cpu_ticks() { awk '{print $14 + $15}' "/proc/$assistant_pid/stat"; }
idle_window() { timeout 6 sleep "$1"; }
drops_file=$work/drops.jsonl
drops_pid=0
burst_done=0
burst() {
    [ "$burst_done" = 0 ] || return 0
    burst_done=1
    ctl events >"$drops_file" 2>/dev/null &
    drops_pid=$!
    pids+=("$drops_pid")
    for i in $(seq 100); do
        emit voice.state "{\"state\": \"$([ $((i % 2)) = 0 ] && echo thinking || echo working)\", \"task\": \"\", \"said\": \"\"}"
        sleep 0.05
    done
    emit voice.state '{"state": "listening", "task": "", "said": ""}'
}
cell_voice_state_rate() {
    figure_shows || return 1
    burst
    waiting "$settle_s" node_has hud-state Listening
}
cell_layers_dropped_empty() {
    figure_shows || return 1
    burst
    ! jq -e 'select(.event == "layers.dropped") | select((.data.nodes // []) | length > 0)' "$drops_file" >/dev/null 2>&1
}
cell_host_idle_cpu() {
    local before after hz
    figure_shows || return 1
    end_question
    waiting "$settle_s" node_exists hud-state || return 1
    hz=$(getconf CLK_TCK)
    before=$(cpu_ticks)
    idle_window 5
    after=$(cpu_ticks)
    [ $((after - before)) -lt $((5 * hz / 100)) ]
}

cell_late_host_shows_figure() {
    emit voice.state '{"state": "thinking", "task": "", "said": ""}'
    waiting "$settle_s" node_has hud-state Thinking || return 1
    stop_assistant
    waiting "$grace_s" figure_gone || return 1
    emit voice.state '{"state": "listening", "task": "", "said": ""}'
    start_assistant || return 1
    waiting "$settle_s" node_has hud-state Listening
}
cell_reconnected_voice_shows_figure() {
    stop_voice
    waiting "$settle_s" figure_gone || return 1
    FAKE_VOICE_STATE_DELAY=1 FAKE_VOICE_STATE='{"state": "listening", "task": "", "said": ""}' start_voice
    waiting "$settle_s" node_has hud-state Listening
}
cell_hung_voice_leaves_host_responsive() {
    stop_voice
    waiting "$settle_s" figure_gone || return 1
    FAKE_VOICE_HANG_STATE=1 start_voice
    emit voice.state '{"state": "thinking", "task": "", "said": ""}'
    waiting "$settle_s" node_has hud-state Thinking
}
figure_gone() { ! node_exists hud-figure; }

run_cell() {
    local id=$1 fn=$2 session=$3 t0=$SECONDS
    [ $SECONDS -le $time_cap ] || fail "the run takes more than $time_cap s at $id"
    [ -z "${ONLY:-}" ] || [[ $id =~ ^(${ONLY})$ ]] || return 0
    [ "$session" = static ] || ensure_session
    if "$fn"; then got[$id]=pass; else got[$id]=missing; fi
    echo "$id ${got[$id]} $((SECONDS - t0))s" >>"$log"
    if [ "$session" != static ] && [ "$session_up" = 1 ]; then
        end_question
        ctl call assistant chat.hide '{}' >/dev/null 2>&1
        emit voice.state '{"name": "Atlas", "state": "idle", "task": "", "said": ""}'
    fi
}

run_cell check-neutral cell_check_neutral static
run_cell check-xp cell_check_xp static
run_cell check-mc cell_check_mc static
run_cell check-construct cell_check_construct static
run_cell check-ok-names-use cell_check_ok_names_use static
run_cell check-names-four-errors cell_check_names_four_errors static
run_cell list-huds cell_list_huds static
run_cell use-hud-writes-setting cell_use_hud_writes_setting static
run_cell neutral-equals-today cell_neutral_equals_today session
run_cell card-names-file cell_card_names_file session
run_cell card-injected-click-refused cell_card_injected_click_refused session
run_cell context-held-for-question cell_context_held_for_question session
run_cell context-picks-folder cell_context_picks_folder session
run_cell card-stays-on-focus-move cell_card_stays_on_focus_move session
run_cell chat-row-always cell_chat_row_always session
run_cell chat-row-click-opens cell_chat_row_click_opens session
run_cell chat-show-hide cell_chat_show_hide session
run_cell chat-super-a-esc cell_chat_super_a_esc session
run_cell typed-yes-held cell_typed_yes_held session
run_cell agents-row-text cell_agents_row_text session
run_cell xp-tray-rows cell_xp_tray_rows session
run_cell mc-host-hud cell_mc_host_hud session
run_cell mc-chat-drawer cell_mc_chat_drawer session
run_cell construct-host-hud cell_construct_host_hud session
run_cell construct-chat-panel cell_construct_chat_panel session
run_cell voice-state-rate cell_voice_state_rate session
run_cell layers-dropped-empty cell_layers_dropped_empty session
run_cell host-idle-cpu cell_host_idle_cpu session
run_cell late-host-shows-figure cell_late_host_shows_figure session
run_cell reconnected-voice-shows-figure cell_reconnected_voice_shows_figure session
run_cell hung-voice-leaves-host-responsive cell_hung_voice_leaves_host_responsive session

printf '\n%-34s %-9s %s\n' cell measured expected
while read -r id want; do
    printf '%-34s %-9s %s\n' "$id" "${got[$id]:--}" "$want"
done < <(table)
[ -z "${ONLY:-}" ] || echo "ONLY is set: the table is not fully checked"
echo "run time: ${SECONDS}s (see $log)"
if ! compare; then
    printf 'MISMATCH %s\n' "${mismatches[@]}" >&2
    fail "${#mismatches[@]} cells differ from EXPECTED; see $log"
fi
[ $SECONDS -le $time_cap ] || fail "the run took ${SECONDS}s, more than $time_cap s"
ok "gate-hud: every cell as in EXPECTED${ONLY:+ that ran} in ${SECONDS}s"
