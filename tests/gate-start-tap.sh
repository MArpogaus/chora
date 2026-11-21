#!/bin/bash
# Gate of the Start menu of xp on a Super tap beside tiling on two outputs
# (sh-23, sd-h1, sd-h2, ca-4 of the handover). xp starts first on
# HEADLESS-1, tiling on HEADLESS-2; both take injected keys. The Start menu
# opens on a release of Super only when no other key, button or drag came
# between press and release, as in XP. Cells:
#
#   tap     Super alone opens the Start menu, a second tap closes it (the
#           control: without it the other cells would pass on a dead tap)
#   cube    Super held, a drag on the desktop of xp (the cube turn) and the
#           release of Super: the Start menu stays closed (ca-4)
#   drag    a foot of tiling, Super held and a drag of it onto the output
#           of xp, then the release: the Start menu stays closed (sd-h2)
#   return  a click on the empty output of tiling, then Super+Return: a new
#           foot of tiling takes the keyboard, the Start menu stays closed
#           (sh-23)
#   give    the Start menu is open and the foot of tiling goes to xp
#           (window.give of tiling): the menu closes, it does not cover the
#           window (sd-h2)
#
# The drags inject the press, each step of the motion and the release
# apart, about 0.1 s between them, as the tour does: a drag in one inject
# hides ca-4. The core runs with --test-injected-press-is-real, so an
# injected click chooses a window manager as one of a person does (D55).
# A cell prints `cell NAME pass|red` and the gate fails on a red one.
# Usage: tests/gate-start-tap.sh [CELL...]   (default: all)
#   BIN: directory with the binaries   SHOTS: directory for the logs
set -uo pipefail
. "$(dirname "$0")/gate-lib.sh"
need jq
need foot
need_bins chora choractl chora-tiling chora-xp
temp_config
mkdir -p "$XDG_CONFIG_HOME/chora"
for c in tiling xp; do printf 'test_accept_injected_keys: true\n' >"$XDG_CONFIG_HOME/chora/chora-$c.yaml"; done
shots=${SHOTS:-$XDG_CONFIG_HOME}
mkdir -p "$shots"
export PATH=$bin:$PATH
cells=("$@")
[ "${#cells[@]}" != 0 ] || cells=(tap cube drag return give)

CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x800
grant input.inject screen.capture methods.call methods.privileged
ctl add-output 1280x800+1280+0 >/dev/null || fail "add-output"
mapfile -t outs < <(ctl outputs | jq -r 'sort_by(.position[0]) | .[].name')
XP=${outs[0]} TI=${outs[1]}
"$bin/chora-xp" --theme "$repo/flavors/xp/theme" --output "$XP" >"$shots/xp.log" 2>&1 &
pids+=($!)
ctl wait id:xp-icons --timeout 20 >/dev/null || fail "xp does not start: $(tail -n 3 "$shots/xp.log")"
"$bin/chora-tiling" --output "$TI" >"$shots/tiling.log" 2>&1 &
pids+=($!)
ctl wait owner:tiling --timeout 20 >/dev/null || fail "tiling does not start: $(tail -n 3 "$shots/tiling.log")"
sleep 1

inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
super() { inject "[{\"key\": {\"key\": \"Super_L\", \"state\": \"$1\"}}]"; }
menu_open() { [ "$(ctl query id:xp-menu | jq -r ".[0].visible")" = true ]; }
menu_after() {
    for _ in $(seq 5); do
        sleep 0.2
        if menu_open; then [ "$1" = open ] && return 0; [ "$1" = closed ] && return 1; fi
    done
    [ "$1" = closed ] && return 0
    return 1
}
close_menu() {
    ctl call xp start_menu '{"open": false}' >/dev/null 2>&1
    menu_after closed || fail "the Start menu of xp does not close"
}
reds=()
verdict() {
    echo "cell $1 $2${3:+: $3}"
    [ "$2" = pass ] || reds+=("$1")
}
foot_of_tiling() {
    local id
    foot -f "DejaVu Sans Mono:size=10" -a "$1" sh -c 'sleep 900' >>"$shots/foot.log" 2>&1 &
    run_pid $!
    id=$(ctl wait "app_id:$1 kind:surface" --timeout 20 | jq -r '.[0].id') || fail "foot $1 does not show"
    for _ in $(seq 50); do
        case $(ctl query "id:$id" | jq -r '.[0].owner // "none"') in
        tiling) break ;;
        none) sleep 0.2 ;;
        *) ctl call xp window.give "{\"surface\": \"$id\", \"to\": \"tiling\"}" >/dev/null 2>&1; sleep 0.2 ;;
        esac
    done
    ctl wait "id:$id owner:tiling" --timeout 10 >/dev/null || fail "foot $1 is not of tiling"
    echo "$id"
}

cell_tap() {
    super pressed
    super released
    if ! menu_after open; then verdict tap red "a Super tap does not open the Start menu"; return; fi
    super pressed
    super released
    if menu_after closed; then verdict tap pass; else verdict tap red "a second tap leaves it open"; fi
    close_menu
}

cell_cube() {
    local x
    inject "[{\"pointer_move\": {\"position\": [900, 250], \"output\": \"$XP\"}}]"
    sleep 0.3
    inject '[{"key": {"key": "Super_L", "state": "pressed"}}, {"button": {"button": 272, "state": "pressed", "hold": true}}]'
    sleep 0.1
    for x in 850 800 750 700 650 600 550 500; do
        inject "[{\"pointer_move\": {\"position\": [$x, 250], \"output\": \"$XP\"}}]"
        sleep 0.1
    done
    inject '[{"button": {"button": 272, "state": "released"}}, {"key": {"key": "Super_L", "state": "released"}}]'
    if menu_after closed; then verdict cube pass; else verdict cube red "the Start menu opens after a Super drag on the desktop"; fi
    close_menu
}

cell_drag() {
    local w x
    w=$(foot_of_tiling drag)
    inject "[{\"pointer_move\": {\"node\": \"$w\", \"point\": [100, 60]}}]"
    sleep 0.3
    inject '[{"key": {"key": "Super_L", "state": "pressed"}}, {"button": {"button": 272, "state": "pressed", "hold": true}}]'
    sleep 0.1
    for x in 1100 900 700 640; do
        inject "[{\"pointer_move\": {\"position\": [$x, 400], \"output\": \"$XP\"}}]"
        sleep 0.1
    done
    inject '[{"button": {"button": 272, "state": "released"}}, {"key": {"key": "Super_L", "state": "released"}}]'
    if menu_after closed; then verdict drag pass; else verdict drag red "the Start menu opens after a Super drag of a window (owner $(ctl query "id:$w" | jq -r '.[0].owner'))"; fi
    close_menu
}

cell_return() {
    local before after id
    before=$(ctl query "owner:tiling kind:surface" | jq length)
    inject "[{\"pointer_move\": {\"position\": [640, 500], \"output\": \"$TI\"}},
      {\"button\": {\"state\": \"pressed\"}}, {\"button\": {\"state\": \"released\"}}]"
    for _ in $(seq 25); do
        ctl query '*' | jq -e 'all(.[]; .focused != true)' >/dev/null && break
        sleep 0.2
    done
    inject '[{"key": {"key": "Super_L", "state": "pressed"}}, {"key": {"key": "Return", "state": "pressed"}},
      {"key": {"key": "Return", "state": "released"}}, {"key": {"key": "Super_L", "state": "released"}}]'
    local opened=0
    menu_after closed || opened=1
    for _ in $(seq 50); do
        after=$(ctl query "owner:tiling kind:surface" | jq length)
        [ "$after" -gt "$before" ] && break
        sleep 0.2
    done
    for _ in $(seq 25); do
        id=$(ctl query "owner:tiling kind:surface" | jq -r '[.[] | select(.focused == true)][0].id // empty')
        [ -n "$id" ] && break
        sleep 0.2
    done
    if [ "$opened" = 1 ]; then
        verdict return red "Super+Return on the tiling output opens the Start menu of xp (tiling surfaces $before -> $after)"
    elif [ "$after" -le "$before" ] || [ -z "$id" ]; then
        verdict return red "Super+Return on the tiling output gives tiling no focused new window ($before -> $after, focused ${id:-none})"
    else
        verdict return pass
    fi
    close_menu
}

cell_give() {
    local w
    w=$(foot_of_tiling give)
    inject "[{\"pointer_move\": {\"node\": \"$w\", \"point\": [100, 60]}},
      {\"button\": {\"state\": \"pressed\"}}, {\"button\": {\"state\": \"released\"}}]"
    ctl call xp start_menu '{"open": true}' >/dev/null || fail "give: open the Start menu"
    menu_after open || fail "give: the Start menu does not open"
    ctl call tiling window.give "{\"surface\": \"$w\", \"to\": \"xp\"}" >/dev/null 2>&1
    ctl wait "id:$w owner:xp" --timeout 10 >/dev/null || { verdict give red "the window does not reach xp"; return; }
    if menu_after closed; then verdict give pass; else verdict give red "the Start menu stays open over the given window"; fi
    close_menu
}

for c in "${cells[@]}"; do
    case $c in tap | cube | drag | return | give) "cell_$c" ;; *) fail "no cell $c" ;; esac
done
[ "${#reds[@]}" = 0 ] || fail "red cells: ${reds[*]} (logs in $shots)"
ok "gate-start-tap: ${cells[*]} pass"
