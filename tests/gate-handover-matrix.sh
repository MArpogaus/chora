#!/bin/bash
# Matrix gate of the handover between window managers (D55): one headless
# session with four outputs, the construct flavor on the first, then xp,
# tiling and mc (`--manage-windows`), each with `--output`. For each of the
# 12 ordered pairs and each path, one foot is placed with the first window
# manager, the path moves it, and the cell is measured:
#
#   pass  the window is owned by the second window manager, every surface
#         is attached, visible and of nonzero size, and the window has the
#         keyboard focus
#   gap   anything else; the window must still have an owner and no
#         surface may be lost
#
# Paths: give (`window.give` of the owner), cgive (`choractl give TO`, the
# focused window), cgive-s (`choractl give TO --surface ID`), mcp (the tool
# of `choractl mcp`, tests/mcp-client.py give), drag (the title bar of xp;
# Super with the left button for the others), key (the shortcut
# Super+Alt+1 to 4 for tiling, xp, mc and construct as the target; in
# construct Tab, g and Return of the window list on a window behind the
# camera), voice (the real chora-voice with the fakes of
# tests/battle/w16.sh, a tool call `call_method` of the fake LLM). A voice
# turn takes about 7 s; all 12 pairs run, and the whole matrix takes about
# 150 s of the 240 s that it may take. The table can mark a cell `skip`,
# which runs nothing; no cell is one now.
#
# The gate fails on every cell that differs from the table EXPECTED, in
# both directions, so a fix must flip its line. It fails too when a window
# is lost or the whole run, startup and last cell included, takes more
# than 4 minutes. It prints the matrix and
# one line per cell in $SHOTS/matrix.log.
#
# Usage: tests/gate-handover-matrix.sh   BIN: directory with the binaries
#   SHOTS: directory for the log (default: a temp dir)
#   HOLDER: the window manager that starts first and so holds the give
#   keys (default construct); only that one may serve the keys, and the
#   gate fails when another does
#   CHORA_TEST_ACCEPT_INJECTED_KEYS: 1 (default) lets the window managers
#   take the keys of `input.inject`; 0 is production: the key cells of
#   tiling, xp and mc expect gap, and no manager may serve a key
#   CAP: the most seconds of the run (default 240)
#   ONLY: a regular expression that must match all of "SRC DST PATH"; only
#   those cells run, the table is not fully checked, and the gate says so
# shellcheck disable=SC2016
set -uo pipefail

EXPECTED='
tiling xp key pass
tiling xp drag pass
tiling xp give pass
tiling xp cgive pass
tiling xp cgive-s pass
tiling xp mcp pass
tiling xp voice pass
tiling mc key pass
tiling mc drag pass
tiling mc give pass
tiling mc cgive pass
tiling mc cgive-s pass
tiling mc mcp pass
tiling mc voice pass
tiling construct key pass
tiling construct drag pass
tiling construct give pass
tiling construct cgive pass
tiling construct cgive-s pass
tiling construct mcp pass
tiling construct voice pass
xp tiling key pass
xp tiling drag pass
xp tiling give pass
xp tiling cgive pass
xp tiling cgive-s pass
xp tiling mcp pass
xp tiling voice pass
xp mc key pass
xp mc drag pass
xp mc give pass
xp mc cgive pass
xp mc cgive-s pass
xp mc mcp pass
xp mc voice pass
xp construct key pass
xp construct drag pass
xp construct give pass
xp construct cgive pass
xp construct cgive-s pass
xp construct mcp pass
xp construct voice pass
mc tiling key pass
mc tiling drag pass
mc tiling give pass
mc tiling cgive pass
mc tiling cgive-s pass
mc tiling mcp pass
mc tiling voice pass
mc xp key pass
mc xp drag pass
mc xp give pass
mc xp cgive pass
mc xp cgive-s pass
mc xp mcp pass
mc xp voice pass
mc construct key pass
mc construct drag pass
mc construct give pass
mc construct cgive pass
mc construct cgive-s pass
mc construct mcp pass
mc construct voice pass
construct tiling key pass
construct tiling drag pass
construct tiling give pass
construct tiling cgive pass
construct tiling cgive-s pass
construct tiling mcp pass
construct tiling voice pass
construct xp key pass
construct xp drag pass
construct xp give pass
construct xp cgive pass
construct xp cgive-s pass
construct xp mcp pass
construct xp voice pass
construct mc key pass
mc construct leak pass
construct mc drag pass
construct mc give pass
construct mc cgive pass
construct mc cgive-s pass
construct mc mcp pass
construct mc voice pass
'

. "$(dirname "$0")/gate-lib.sh"
need jq
need foot
need python3
need_bins chora choractl chora-construct chora-tiling chora-xp chora-mc chora-voice
temp_config
work=$XDG_CONFIG_HOME
shots=${SHOTS:-$work}
mkdir -p "$shots" "$work/chora"
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
theme=$repo/flavors/xp/theme
time_cap=${CAP:-240}
log=$shots/matrix.log
: >"$log"

injected_keys=${CHORA_TEST_ACCEPT_INJECTED_KEYS:-1}
holder=${HOLDER:-construct}
case $holder in construct | xp | tiling | mc) ;; *) fail "HOLDER $holder is no window manager" ;; esac

session_start 1280x800
grant input.inject screen.capture methods.call methods.privileged

start_construct() {
    "$bin/chora-construct" >"$shots/construct.log" 2>&1 &
    pids+=($!)
    ctl wait --timeout 20 id:construct-look >/dev/null || fail "the flavor shows no space: $(tail -n 3 "$shots/construct.log")"
}
[ "$holder" != construct ] || start_construct
for x in 1280 2560 3840; do
    ctl add-output "1280x800+$x+0" >/dev/null || fail "add-output +$x"
done
mapfile -t outs < <(ctl outputs | jq -r 'sort_by(.position[0]) | .[].name')
[ "${#outs[@]}" = 4 ] || fail "four outputs: ${outs[*]}"
declare -A out_of=([construct]=${outs[0]} [xp]=${outs[1]} [tiling]=${outs[2]} [mc]=${outs[3]})

plugin() {
    local name=$1
    shift
    "$bin/chora-$name" "$@" >"$shots/$name.log" 2>&1 &
    pids+=($!)
}
start_plugin() {
    case $1 in
    xp) plugin xp --theme "$theme" --output "${out_of[xp]}" ;;
    tiling) plugin tiling --output "${out_of[tiling]}" ;;
    mc) plugin mc --manage-windows --output "${out_of[mc]}" ;;
    esac
    ctl wait "owner:$1" --timeout 20 >/dev/null || fail "$1 does not start: $(tail -n 3 "$shots/$1.log")"
}
if [ "$holder" = construct ]; then
    for c in xp tiling mc; do start_plugin "$c"; done
else
    start_plugin "$holder"
    for c in xp tiling mc; do [ "$c" = "$holder" ] || start_plugin "$c"; done
    start_construct
fi

inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
inject "[{\"pointer_move\": {\"position\": [640, 300], \"output\": \"${out_of[xp]}\"}}]"
foot -f "DejaVu Sans Mono:size=10" -a probe sh -c 'sleep 900' >"$shots/foot.log" 2>&1 &
run_pid $!
w=$(ctl wait "app_id:probe kind:surface" --timeout 20 | jq -r '.[0].id') ||
    fail "foot does not show"
for _ in $(seq 50); do
    first_owner=$(ctl query "id:$w" | jq -r '.[0].owner // "none"')
    [ "$first_owner" != none ] && break
    sleep 0.2
done
if [ "$first_owner" != xp ]; then
    ctl call "$first_owner" window.give "{\"surface\": \"$w\", \"to\": \"xp\"}" >/dev/null 2>&1 ||
        fail "foot starts with $first_owner and does not go to xp"
    ctl wait "id:$w owner:xp" --timeout 20 >/dev/null || fail "foot does not show in xp"
fi

owner_of() { ctl query "id:$w" | jq -r '.[0].owner // "none"'; }

surfaces_clean='all(.[] | select(.id == $w); .owner != null and .parent != null and ((.rects // {}) | length) > 0
    and ([.rects[] | select(.[2] > 0 and .[3] > 0)] | length) > 0)'

arrived() {
    local all owned=0
    for _ in $(seq "$(awk -v s="$2" 'BEGIN {print s * 5}')"); do
        all=$(ctl query kind:surface 2>/dev/null) || all='[]'
        last_all=$all
        echo "$all" | jq -e --arg w "$w" --arg o "$1" 'any(.[]; .id == $w and .owner == $o)' >/dev/null &&
            owned=1 && break
        sleep 0.2
    done
    [ "$owned" = 1 ] || return 1
    for _ in $(seq "$(awk -v s="$3" 'BEGIN {print s * 5}')"); do
        all=$(ctl query kind:surface 2>/dev/null) || all='[]'
        last_all=$all
        echo "$all" | jq -e --arg w "$w" --arg o "$1" "
            $surfaces_clean and any(.[]; .id == \$w and .owner == \$o and .focused == true)" >/dev/null &&
            return 0
        sleep 0.2
    done
    return 1
}

construct_pane() {
    ctl call construct window.list 2>/dev/null | jq -r '.windows[] | select(.name == "probe") | .pane'
}

rects_stable() {
    local before after
    before=$(ctl query "id:$w" | jq -c '.[0].rects')
    for _ in $(seq 15); do
        sleep 0.3
        after=$(ctl query "id:$w" | jq -c '.[0].rects')
        [ "$before" != "$after" ] || return 0
        before=$after
    done
}

repairs=0
repair() {
    repairs=$((repairs + 1))
    echo "repair $*" >>"$log"
}

has_focus() {
    for _ in $(seq 50); do
        ctl query "id:$w" | jq -e '.[0].focused == true' >/dev/null && return 0
        sleep 0.1
    done
    return 1
}

place() {
    local o
    o=$(owner_of)
    [ "$o" != "$1" ] || return 0
    case $o in none | null | "") fail "the window is lost before a cell: owner ${o:-nobody}" ;; esac
    if ! ctl call "$o" window.give "{\"surface\": \"$w\", \"to\": \"$1\"}" >/dev/null 2>&1; then
        repair "give $o to $1 through tiling"
        ctl call "$o" window.give "{\"surface\": \"$w\", \"to\": \"tiling\"}" >/dev/null 2>&1
        sleep 0.5
        ctl call tiling window.give "{\"surface\": \"$w\", \"to\": \"$1\"}" >/dev/null 2>&1
    fi
    for _ in $(seq 50); do
        [ "$(owner_of)" = "$1" ] && break
        sleep 0.2
    done
    has_focus || fail "the window has no focus with $1 before a cell: $(ctl query "id:$w" | jq -c '.[0] | {owner, focused, rects}')"
    [ "$(owner_of)" = "$1" ] || fail "cannot place the window with $1: owner $(owner_of); construct: $(ctl call construct window.list 2>&1 | jq -c .windows)"
    rects_stable
}

super_drag() {
    local point='[100,60]'
    if [ "$2" = construct ]; then
        point=$(ctl query "id:$w" | jq -c '.[0].rects | to_entries[0].value | [.[2] / 2, .[3] / 2]')
    fi
    inject "[{\"pointer_move\": {\"node\": \"$w\", \"point\": $point}},
      {\"key\": {\"key\": \"Super_L\", \"state\": \"pressed\"}},
      {\"button\": {\"state\": \"pressed\", \"hold\": true}},
      {\"pointer_move\": {\"position\": [640, 400], \"output\": \"${out_of[$1]}\"}},
      {\"button\": {\"state\": \"released\"}},
      {\"key\": {\"key\": \"Super_L\", \"state\": \"released\"}}]"
}

title_drag() {
    inject "[{\"pointer_move\": {\"node\": \"xp-title-$w\", \"point\": [60, 10]}},
      {\"button\": {\"state\": \"pressed\", \"hold\": true}},
      {\"pointer_move\": {\"position\": [300, 300], \"output\": \"${out_of[xp]}\"}},
      {\"pointer_move\": {\"position\": [640, 400], \"output\": \"${out_of[$1]}\"}},
      {\"button\": {\"state\": \"released\"}}]"
}

construct_key() {
    local row names
    ctl call construct window.show '{"name": "probe"}' >/dev/null 2>&1
    sleep 1.5
    key Alt+Escape
    ctl wait --timeout 3 "id:construct-look focused:true" >/dev/null 2>&1
    ctl call construct pane.move "{\"name\": \"probe\", \"ahead\": -6000}" >/dev/null 2>&1
    sleep 0.5
    row=$(ctl call construct window.list 2>/dev/null |
        jq -r '[.windows[] | .name] | index("probe") // 0')
    key Tab
    ctl wait --timeout 3 "id:construct-list meta:state=open" >/dev/null 2>&1 || return 0
    for _ in $(seq "$row"); do key Down; done
    key g
    sleep 0.3
    names=$(printf '%s\n' xp tiling mc | sort)
    for _ in $(seq "$(echo "$names" | grep -nx "$1" | cut -d: -f1 | awk '{print $1 - 1}')"); do key Down; done
    key Return
}

vlog=$shots/voice.log
voice_ready=0
voice_start() {
    python3 -I - "$repo/services/voice/tests/data/corpus/hallo-chora/mic.wav" "$work/speech.pcm" <<'EOF'
import sys, wave
with wave.open(sys.argv[1]) as w, open(sys.argv[2], "wb") as out:
    out.write(w.readframes(w.getnframes()))
EOF
    : >"$work/mic.cmd"
    local fakes=$repo/tests/battle/w16-fakes.py
    python3 -I "$fakes" servers "$work" >"$shots/fakes.log" 2>&1 &
    pids+=($!)
    for _ in $(seq 50); do
        [ -s "$work/ports" ] && break
        sleep 0.2
    done
    [ -s "$work/ports" ] || fail "the fakes write no ports"
    local asr tts llm
    asr=$(jq -r .asr "$work/ports")
    tts=$(jq -r .tts "$work/ports")
    llm_port=$(jq -r .llm "$work/ports")
    llm=$llm_port
    cat >"$work/chora/chora-voice.yaml" <<EOF
asr_url: ws://127.0.0.1:$asr
tts_url: ws://127.0.0.1:$tts
llm_url: http://127.0.0.1:$llm/v1
capture_cmd: [python3, -I, $fakes, mic, $work]
playback_cmd: [python3, -I, $fakes, play, $work]
playback_buffer_ms: 200
earcons: false
allow_methods: [construct.window.give, xp.window.give, tiling.window.give, mc.window.give]
EOF
    "$bin/choractl" grant voice-mcp methods.call methods.privileged >/dev/null || fail "grant voice-mcp"
    RUST_LOG="chora_voice=debug,info,tract_linalg=warn,rmcp=warn" "$bin/chora-voice" >"$vlog" 2>&1 &
    pids+=($!)
    for _ in $(seq 150); do
        ctl call voice state >/dev/null 2>&1 && break
        sleep 0.2
    done
    ctl call voice state >/dev/null 2>&1 || fail "voice does not answer: $(tail -n 3 "$vlog")"
    for _ in $(seq 150); do
        grep -q Ready "$work/tts.log" 2>/dev/null && break
        sleep 0.2
    done
    sleep 2
    voice_ready=1
}
push() { curl -s -X POST --data-binary "$2" "http://127.0.0.1:$llm_port/$1" >/dev/null; }

voice_l0=0
voice_turn=0
voice_idle() {
    [ "$voice_turn" = 1 ] || return 0
    for _ in $(seq 100); do
        tail -n "+$((voice_l0 + 1))" "$vlog" | grep -q 'llm: stop' && return 0
        sleep 0.2
    done
}

voice_give() {
    [ "$voice_ready" = 1 ] || voice_start
    voice_idle
    voice_l0=$(wc -l <"$vlog")
    voice_turn=1
    push _llm "{\"tool\": \"call_method\", \"args\": {\"client\": \"$1\", \"method\": \"window.give\", \"params\": {\"surface\": \"$w\", \"to\": \"$2\"}}}"
    push _llm '{"text": "Done."}'
    push _asr "{\"text\": \"move it to $2\"}"
    echo speech >>"$work/mic.cmd"
}

give_key() {
    case $1 in tiling) echo 1 ;; xp) echo 2 ;; mc) echo 3 ;; construct) echo 4 ;; esac
}

path_run() {
    case $1 in
    give) ctl call "$2" window.give "{\"surface\": \"$w\", \"to\": \"$3\"}" >/dev/null 2>&1 ;;
    cgive) ctl give "$3" >/dev/null 2>&1 ;;
    cgive-s) ctl give "$3" --surface "$w" >/dev/null 2>&1 ;;
    mcp) timeout 20 python3 -I "$repo/tests/mcp-client.py" "$bin/choractl" "$work" give "$2" "$w" "$3" >"$work/mcp.out" 2>&1 ;;
    drag)
        if [ "$2" = xp ]; then title_drag "$3"; else super_drag "$3" "$2"; fi
        ;;
    key)
        if [ "$2" = construct ]; then construct_key "$3"; else key "Super+Alt+$(give_key "$3")"; fi
        ;;
    voice) voice_give "$2" "$3" ;;
    esac
    return 0
}

last_all=[]
declare -A got
mismatches=()
expected_of() { echo "$EXPECTED" | awk -v k="$1 $2 $3" '$1" "$2" "$3 == k {print $4}'; }

cell() {
    local src=$1 dst=$2 path=$3 want result owner_s settle_s=1 t0=$SECONDS o
    [ "$SECONDS" -le "$time_cap" ] || fail "the run takes more than $time_cap s at $src $dst $path"
    want=$(expected_of "$src" "$dst" "$path")
    [ -n "$want" ] || fail "no EXPECTED line for $src $dst $path"
    if [ "$injected_keys" = 0 ] && [ "$path" = key ] && [ "$src" != construct ]; then
        want=gap
    fi
    case $path in drag | key) owner_s=1.5 ;; voice) owner_s=12 ;; *) owner_s=0.8 ;; esac
    if [ "$want" = pass ]; then
        settle_s=10
        [ "$path" = voice ] || owner_s=5
    fi
    if [ "$want" = skip ]; then
        got["$src $dst $path"]=skip
        echo "$src $dst $path skip" >>"$log"
        return 0
    fi
    place "$src"
    path_run "$path" "$src" "$dst"
    if arrived "$dst" "$owner_s" "$settle_s"; then result=pass; else result=gap; fi
    for _ in $(seq 25); do
        echo "$last_all" | jq -e --arg w "$w" "$surfaces_clean" >/dev/null && break
        sleep 0.2
        last_all=$(ctl query kind:surface)
    done
    echo "$last_all" | jq -e --arg w "$w" "$surfaces_clean" >/dev/null ||
        fail "a surface is lost after $src $dst $path: $(echo "$last_all" | jq -c 'map({id, owner, parent, rects})')"
    o=$(echo "$last_all" | jq -r --arg w "$w" '.[] | select(.id == $w) | .owner // "none"')
    case $o in none | null | "") fail "the window is lost after $src $dst $path" ;; esac
    if [ "$src" = construct ] && [ "$path" = key ]; then
        key Escape
    fi
    got["$src $dst $path"]=$result
    echo "$src $dst $path $result owner=$o focus=$(echo "$last_all" | jq -r --arg w "$w" '.[] | select(.id == $w) | .focused') pane=$([ "$o" = construct ] && construct_pane)$([ "$path" = mcp ] && echo " tool=$(sed -n 's/^tool: //p' "$work/mcp.out")") $((SECONDS - t0))s" >>"$log"
    [ "$result" = "$want" ] || mismatches+=("$src $dst $path: measured $result, expected $want")
}

leak_cell() {
    local want result
    want=$(expected_of mc construct leak)
    [ -n "$want" ] || fail "no EXPECTED line for mc construct leak"
    place construct
    if [ "$(construct_pane)" = shown ]; then result=pass; else result=gap; fi
    got["mc construct leak"]=$result
    echo "mc construct leak $result pane=$(construct_pane)" >>"$log"
    [ "$result" = "$want" ] || mismatches+=("mc construct leak: measured $result, expected $want")
}

pairs=()
for s in tiling xp construct mc; do
    for d in tiling xp mc construct; do
        [ "$s" = "$d" ] || pairs+=("$s $d")
    done
done
paths=(give cgive cgive-s mcp voice drag key)
for path in "${paths[@]}"; do
    for pair in "${pairs[@]}"; do
        # shellcheck disable=SC2086
        set -- $pair
        [[ "$1 $2 $path" =~ ^(${ONLY:-.*})$ ]] || continue
        cell "$1" "$2" "$path"
    done
    if [ "$path" = drag ] && [[ "mc construct leak" =~ ^(${ONLY:-.*})$ ]]; then
        leak_cell
    fi
done

printf '\n%-22s' pair
printf '%-9s' "${paths[@]}"
echo
for pair in "${pairs[@]}"; do
    printf '%-22s' "$pair"
    for path in "${paths[@]}"; do
        printf '%-9s' "${got["$pair $path"]:--}"
    done
    echo
done
served_by() {
    local c
    for c in construct xp tiling mc; do
        if grep -q "key give" "$shots/$c.log" 2>/dev/null; then echo "$c"; fi
    done
}
keys_ran=0
for k in "${!got[@]}"; do [[ $k == *" key" ]] && keys_ran=1; done
if [ "$keys_ran" = 1 ] && [ "$injected_keys" = 0 ] && [ -n "$(served_by)" ]; then
    fail "an injected give key was served by [$(served_by | tr '\n' ' ')] with the setting off"
fi
if [ "$keys_ran" = 1 ] && [ "$injected_keys" != 0 ] && [ "$(served_by | tr '\n' ' ')" != "$holder " ]; then
    fail "the give keys were served by [$(served_by | tr '\n' ' ')], expected only $holder"
fi
[ -z "${ONLY:-}" ] || echo "ONLY is set: the table is not fully checked"
echo "repairs of the setup: $repairs (see $log)"
echo "run time: ${SECONDS}s"
if [ "${#mismatches[@]}" != 0 ]; then
    printf 'MISMATCH %s\n' "${mismatches[@]}" >&2
    fail "${#mismatches[@]} cells differ from EXPECTED; see $log"
fi
[ "$SECONDS" -le "$time_cap" ] || fail "the run took ${SECONDS}s, more than $time_cap s"
ok "gate-handover-matrix: every cell as in EXPECTED${ONLY:+ that ran} in ${SECONDS}s"
