#!/bin/bash
# Gate of the handover between window managers on virtual outputs (D69,
# D55): one headless session with one screen. tests/pane-host.py owns two
# virtual outputs and shows each in a forwarding pane on the screen. mc
# (`--manage-windows`) manages the screen, xp the first virtual output and
# tiling the second, each with `--output`. For each of the 6 ordered pairs
# and each path, one foot is placed with the first window manager, the
# path moves it, and the cell is measured:
#
#   pass     the window is owned by the second window manager, it is
#            attached, shows on the output of that one with a size, and
#            has the keyboard focus
#   gap      anything else; the window must still have an owner
#   missing  the gate does not build the cell yet
#
# Paths: give (`window.give` of the owner), cgive (`choractl give TO`, the
# focused window), key (Super+Alt+1 to 3 for tiling, xp and mc as the
# target, which mc serves as the first window manager), drag (the title
# bar of xp; Super with the left button for the others). A drag goes over
# the screen: the pointer enters a virtual output through its pane, and a
# release over a pane names the window manager behind it.
#
# The input of the gate is injected, and an injected press does not hold
# the pointer behind its pane as the press of a real device does. A drag
# of a real device out of a pane is a manual cell.
#
# The gate fails on every cell that differs from the table EXPECTED, in
# both directions, so a fix must flip its line. It fails too when a window
# is lost or the run takes more than 4 minutes. It prints the matrix and
# one line per cell in $SHOTS/virtual.log.
#
# The panes show their outputs at 1:1 at (20, 20) and (660, 20), so a node
# of a virtual output is on the screen at the place of its pane plus its
# rect. The pointer goes to the middle of the area of a window manager:
# the free part below the panes for mc, and the pane of the other two.
#
# Usage: tests/gate-handover-virtual.sh   BIN: directory with the binaries
#   SHOTS: directory for the log (default: a temp dir)
#   CAP: the most seconds of the run (default 240)
#   ONLY: a regular expression that must match all of "SRC DST PATH"; only
#   those cells run, the table is not fully checked, and the gate says so
set -uo pipefail

EXPECTED='
mc xp give missing
mc xp cgive missing
mc xp key missing
mc xp drag missing
mc tiling give missing
mc tiling cgive missing
mc tiling key missing
mc tiling drag missing
xp mc give missing
xp mc cgive missing
xp mc key missing
xp mc drag missing
xp tiling give missing
xp tiling cgive missing
xp tiling key missing
xp tiling drag missing
tiling mc give missing
tiling mc cgive missing
tiling mc key missing
tiling mc drag missing
tiling xp give missing
tiling xp cgive missing
tiling xp key missing
tiling xp drag missing
'

. "$(dirname "$0")/gate-lib.sh"
need jq
need foot
need python3
need_bins chora choractl chora-tiling chora-xp chora-mc
temp_config
shots=${SHOTS:-$XDG_CONFIG_HOME}
mkdir -p "$shots"
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
theme=$repo/flavors/xp/theme
time_cap=${CAP:-240}
log=$shots/virtual.log
: >"$log"

session_start 1280x800
grant input.inject screen.capture methods.call methods.privileged
screen=$(ctl outputs | jq -r '.[0].name')

python3 -I "$repo/tests/pane-host.py" "$screen" 600x400+20+20 600x400+660+20 >"$shots/pane-host.log" 2>&1 &
pids+=($!)
for _ in $(seq 50); do
    grep -qx ready "$shots/pane-host.log" && break
    sleep 0.2
done
grep -qx ready "$shots/pane-host.log" || fail "no pane host: $(tail -n 3 "$shots/pane-host.log")"
mapfile -t faces < <(awk 'NF == 5 {print $1}' "$shots/pane-host.log")
[ "${#faces[@]}" = 2 ] || fail "two virtual outputs: ${faces[*]}"
declare -A out_of=([mc]=$screen [xp]=${faces[0]} [tiling]=${faces[1]})
declare -A middle=([mc]="640, 620" [xp]="320, 220" [tiling]="960, 220")

plugin() {
    local name=$1
    shift
    "$bin/chora-$name" "$@" >"$shots/$name.log" 2>&1 &
    pids+=($!)
    ctl wait "owner:$name" --timeout 20 >/dev/null || fail "$name does not start: $(tail -n 3 "$shots/$name.log")"
}
plugin mc --manage-windows --output "${out_of[mc]}"
plugin xp --theme "$theme" --output "${out_of[xp]}"
plugin tiling --output "${out_of[tiling]}"

inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
key() { inject "[{\"shortcut\": {\"keys\": \"$1\"}}]"; }
inject "[{\"pointer_move\": {\"position\": [${middle[xp]}]}}]"
foot -f "DejaVu Sans Mono:size=10" -a probe sh -c 'sleep 900' >"$shots/foot.log" 2>&1 &
run_pid $!
w=$(ctl wait "app_id:probe kind:surface" --timeout 20 | jq -r '.[0].id') || fail "foot does not show"
owner_of() { ctl query "id:$w" | jq -r '.[0].owner // "none"'; }

with_owner() {
    local shown
    shown=".[0] | .owner == \$o and .parent != null and .focused == true
        and ((.rects[\$out] // [0, 0, 0, 0]) | .[2] > 0 and .[3] > 0)"
    for _ in $(seq "$(awk -v s="$2" 'BEGIN {print s * 5}')"); do
        ctl query "id:$w" 2>/dev/null | jq -e --arg o "$1" --arg out "${out_of[$1]}" "$shown" >/dev/null && return 0
        sleep 0.2
    done
    return 1
}

with_owner xp 20 ||
    fail "a new window under the pane of xp does not go to xp with the focus: $(ctl query "id:$w" | jq -c '.[0] | {owner, focused, rects}')"
sleep 1

give() { ctl call "$1" window.give "{\"surface\": \"$w\", \"to\": \"$2\"}" >/dev/null 2>&1; }

place() {
    local o other=xp
    o=$(owner_of)
    case $o in none | null | "") fail "the window is lost before a cell" ;; esac
    [ "$1" != xp ] || other=tiling
    if [ "$o" != "$1" ]; then
        give "$o" "$1"
    elif ! with_owner "$1" 1; then
        echo "repair: $1 has the window without the focus" >>"$log"
        give "$1" "$other"
        with_owner "$other" 10 || fail "cannot move the window from $1 to $other"
        give "$other" "$1"
    fi
    with_owner "$1" 10 || fail "cannot place the window with $1: $(ctl query "id:$w" | jq -c '.[0] | {owner, focused, rects}')"
    sleep 0.6
}

super_drag() {
    local x=${middle[$1]%%,*}
    inject "[{\"pointer_move\": {\"position\": [${middle[$1]}]}},
      {\"key\": {\"key\": \"Super_L\", \"state\": \"pressed\"}},
      {\"button\": {\"state\": \"pressed\", \"hold\": true}},
      {\"pointer_move\": {\"position\": [$((x + 40)), 300]}},
      {\"pointer_move\": {\"position\": [${middle[$2]}]}},
      {\"button\": {\"state\": \"released\"}},
      {\"key\": {\"key\": \"Super_L\", \"state\": \"released\"}}]"
}

title_drag() {
    local at
    at=$(ctl query "id:xp-title-$w" | jq -c --arg out "${out_of[xp]}" '.[0].rects[$out] | [20 + .[0] + 60, 20 + .[1] + 10]')
    inject "[{\"pointer_move\": {\"position\": $at}},
      {\"button\": {\"state\": \"pressed\", \"hold\": true}},
      {\"pointer_move\": {\"position\": [200, 150]}},
      {\"pointer_move\": {\"position\": [${middle[$1]}]}},
      {\"button\": {\"state\": \"released\"}}]"
}

give_key() {
    case $1 in tiling) echo 1 ;; xp) echo 2 ;; mc) echo 3 ;; esac
}

path_run() {
    case $1 in
    give) give "$2" "$3" ;;
    cgive) ctl give "$3" >/dev/null 2>&1 ;;
    key) key "Super+Alt+$(give_key "$3")" ;;
    drag) if [ "$2" = xp ]; then title_drag "$3"; else super_drag "$2" "$3"; fi ;;
    esac
    return 0
}

declare -A got
mismatches=()
expected_of() { echo "$EXPECTED" | awk -v k="$1 $2 $3" '$1" "$2" "$3 == k {print $4}'; }

cell() {
    local src=$1 dst=$2 path=$3 want result wait_s=2 t0=$SECONDS o
    [ "$SECONDS" -le "$time_cap" ] || fail "the run takes more than $time_cap s at $src $dst $path"
    want=$(expected_of "$src" "$dst" "$path")
    [ -n "$want" ] || fail "no EXPECTED line for $src $dst $path"
    if [ "$want" = missing ]; then
        got["$src $dst $path"]=missing
        echo "$src $dst $path missing" >>"$log"
        return 0
    fi
    [ "$want" != pass ] || wait_s=10
    place "$src"
    path_run "$path" "$src" "$dst"
    if with_owner "$dst" "$wait_s"; then result=pass; else result=gap; fi
    o=$(owner_of)
    case $o in none | null | "")
        sleep 6
        o=$(owner_of)
        ;;
    esac
    case $o in none | null | "") fail "the window is lost after $src $dst $path" ;; esac
    got["$src $dst $path"]=$result
    echo "$src $dst $path $result owner=$o $((SECONDS - t0))s" >>"$log"
    [ "$result" = "$want" ] || mismatches+=("$src $dst $path: measured $result, expected $want")
}

managers=(mc xp tiling)
paths=(give cgive key drag)
pairs=()
for s in "${managers[@]}"; do
    for d in "${managers[@]}"; do
        [ "$s" = "$d" ] || pairs+=("$s $d")
    done
done
for path in "${paths[@]}"; do
    for pair in "${pairs[@]}"; do
        # shellcheck disable=SC2086
        set -- $pair
        [[ "$1 $2 $path" =~ ^(${ONLY:-.*})$ ]] || continue
        cell "$1" "$2" "$path"
    done
done

printf '\n%-16s' pair
printf '%-9s' "${paths[@]}"
echo
for pair in "${pairs[@]}"; do
    printf '%-16s' "$pair"
    for path in "${paths[@]}"; do
        printf '%-9s' "${got["$pair $path"]:--}"
    done
    echo
done
[ -z "${ONLY:-}" ] || echo "ONLY is set: the table is not fully checked"
echo "run time: ${SECONDS}s (see $log)"
if [ "${#mismatches[@]}" != 0 ]; then
    printf 'MISMATCH %s\n' "${mismatches[@]}" >&2
    fail "${#mismatches[@]} cells differ from EXPECTED; see $log"
fi
[ "$SECONDS" -le "$time_cap" ] || fail "the run took ${SECONDS}s, more than $time_cap s"
ok "gate-handover-virtual: every cell as in EXPECTED${ONLY:+ that ran} in ${SECONDS}s"
