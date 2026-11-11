#!/bin/bash
# Gate of the mc v2 look (design/looks/mc-v2.md, the build list): the
# margin column of chora-mc on a headless output of 1280x800. Each step of
# the list adds its checks here; a check reads the texts and the rects of
# the nodes of mc with `choractl query`.
#
# Usage: tests/gate-mc-column.sh   BIN: directory with the binaries
# shellcheck disable=SC2016 # until_ok evaluates its first argument
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-mc
need jq
export PATH=$bin:$PATH
export CHORA_BIN=$bin
unset DBUS_SESSION_BUS_ADDRESS
temp_config
mkdir -p "$XDG_CONFIG_HOME/chora"
printf 'theme: light\ntest_accept_injected_keys: true\n' >"$XDG_CONFIG_HOME/chora/chora-mc.yaml"
session_start 1280x800
grant screen.capture input.inject
"$bin/chora-mc" >"$XDG_CONFIG_HOME/mc.log" 2>&1 &
pids+=($!)
ctl wait owner:mc --timeout 20 >/dev/null || fail "mc does not start"

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
shown='select(.visible and (.rects | length) > 0)'
texts() { ctl query "owner:mc kind:text" | jq -r ".[] | $shown | .text"; }
has_text() { texts | grep -qxF "$1"; }
text_rect() {
    ctl query "owner:mc kind:text" | jq -c --arg t "$1" \
        '[.[] | select(.visible and (.rects | length) > 0 and .text == $t)][0].rects | to_entries[0].value'
}
shot() { ctl capture "${SHOTS:-/tmp}/mc-column-$1.png" >/dev/null || fail "capture $1"; }

until_ok 'has_text "AGENDA"' || fail "no first screen: $(texts | tr '\n' '|')"

for t in "Super+F1" "All keys" "Super+← →" "Turn the page" "Super+Space" "Start a program" \
    "Super+n" "Hide this column" "Esc" "Back one step"; do
    has_text "$t" || fail "the key list lacks \"$t\" on the first screen"
done
rect=$(text_rect "Start a program")
[ "$(jq -n --argjson r "$rect" '$r[0] >= 700')" = true ] || fail "the key list is not in the right margin: $rect"
ok "the first screen shows the six rows of the key list in the right margin"

key Super+F1
until_ok 'has_text "Show overview"' || fail "Super+F1 does not open the full list"
ok "Super+F1 opens the full list of keys"
key Super+F1
until_ok '! has_text "Show overview"' || fail "Super+F1 does not close the full list"
has_text "Start a program" || fail "the six rows do not come back"
ok "Super+F1 closes the full list"

move_to() { echo "[{\"pointer_move\": {\"position\": [$1, $2]}}]" | ctl inject >/dev/null || fail "inject move"; }
click_at() {
    move_to "$1" "$2"
    echo '[{"button": {"state": "pressed"}}, {"button": {"state": "released"}}]' |
        ctl inject >/dev/null || fail "inject click"
}
center_of() {
    jq -r 'select(. != null) | "\(.[0] + 40) \(.[1] + .[3] / 2)"' <<<"$(text_rect "$1")"
}
hit_center() {
    ctl query "owner:mc kind:rect" | jq -r --arg s "$1-hit" \
        '[.[] | select((.id | endswith($s)) and (.rects | length) > 0)][0].rects | to_entries[0].value | "\(.[0] + 160) \(.[1] + .[3] / 2)"' 2>/dev/null
}
bars() { ctl query "owner:mc kind:rect" | jq -r '[.[] | select((.id | endswith("-mark")) and .opacity == 1 and (.rects | length) > 0)] | length'; }
current() { ctl call mc scene.list '{}' | jq -r '.current'; }

[ "$(bars)" = 0 ] || fail "a focus bar shows without the pointer"
read -r x y <<<"$(center_of "Start a program")"
move_to "$x" "$y"
until_ok '[ "$(bars)" = 1 ]' 8 || fail "no focus bar over the row Start a program"
ok "the pointer over a key row shows the focus bar"
move_to 400 400
until_ok '[ "$(bars)" = 0 ]' 8 || fail "the focus bar stays after the pointer left"
ok "the focus bar goes when the pointer leaves"

read -r x y <<<"$(center_of "All keys")"
click_at "$x" "$y"
until_ok 'has_text "Show overview"' || fail "a click on the row All keys does not show all keys"
sleep 1
read -r x y <<<"$(center_of "Show overview")"
click_at "$x" "$y"
until_ok '[ "$(current)" = overview ]' 20 || fail "a click on Show overview does not show the overview"
until_ok 'has_text "Overview" || [ -n "$(hit_center scenes-row-4)" ]' 20
sleep 1
read -r x y <<<"$(hit_center scenes-row-4)"
move_to "$x" "$y"
until_ok '[ "$(bars)" = 1 ]' 8 || fail "no focus bar over a line of the overview"
click_at "$x" "$y"
until_ok '[ "$(current)" = terminal ]' 20 || fail "a click on a line of the overview does not show that scene: $(current)"
ok "a mouse alone reaches All keys, the overview and the terminal scene"

small=$(ctl query "owner:mc kind:text" | jq -r '[.[] | select(.visible and (.rects | length) > 0 and (.rects | to_entries[0].value[0]) >= 860 and .font_size < 14) | "\(.text):\(.font_size)"] | join(" ")')
[ -z "$small" ] || fail "text under 14 px in the column: $small"
ok "nothing in the column is under 14 px"
