#!/bin/bash
# Gate of the mc v2 look (design/looks/mc-v2.md, the build list): the
# margin column of chora-mc on a headless output of 1280x800. Each step of
# the list adds its checks here; a check reads the texts and the rects of
# the nodes of mc with `choractl query`.
#
# Usage: tests/gate-mc-column.sh   BIN: directory with the binaries
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
