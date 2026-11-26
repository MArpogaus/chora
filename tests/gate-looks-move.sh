#!/bin/bash
# The keyboard stays where the person looks after "Move window to the xp
# desktop" (D119, tiling-focus r1, ti-57): in tiling+xp, tiling manages
# HEADLESS-1 and xp HEADLESS-2 (tests/persona/session.sh). The person
# opens two terminals on tiling and sends them to xp one after the other
# with the Looks door of the bar and the row of its picker, with the hands
# of a persona (act.py), as the ricer of the rounds does it (T7).
#
# 0. The give key Super+Alt+2 sends a third terminal to xp: a give of the
#    person as the row (tiling-focus r2, rtf1-1), so tiling holds the
#    keyboard on the terminal focused before it, and xp holds none.
# 1. The first move: tiling holds the keyboard on the terminal left on
#    HEADLESS-1; xp holds none, though it got a window.
# 2. The second move takes the last window of tiling: tiling holds the
#    keyboard with its desktop, xp still none.
# 3. L3 after it (Super+2, Super+1) holds with tiling: the contract of
#    tests/persona/check.py.
# 4. Super+F1 opens the key list of tiling, Escape closes it, and the
#    keyboard does not go to xp then.
#
# Usage: tests/gate-looks-move.sh   BIN: directory with the binaries
#   SHOTS: a folder for the captures (before and after the moves)
# shellcheck disable=SC2016
set -u
. "$(dirname "$0")/gate-lib.sh"
need python3
need foot
need_fuzzel 1.11
need jq
need_bins chora choractl chora-tiling chora-xp

act() { PERSONA_ENV=$dir/session.env python3 -I "$repo/tests/persona/act.py" "$@"; }
check() { PERSONA_ENV=$dir/session.env python3 -I "$repo/tests/persona/check.py" "$@"; }
admin() {
    CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$dir/session.env") \
        XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$dir/session.env") "$bin/choractl" "$@"
}
launcher_up() {
    XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$dir/session.env") session_pids -x fuzzel >/dev/null
}
owned() { admin query 'kind:surface app_id:foot' | jq -r --arg o "$1" '[.[] | select(.owner == $o) | .id] | join(" ")'; }
holder() { admin state "$1" | jq -c '.focus'; }
holders() { echo "tiling $(holder tiling), xp $(holder xp)"; }
keys_open() { admin state tiling | jq -e '[.open[]? | .id] | index("tiling-keys") != null' >/dev/null; }

spid=
dir=$(mktemp -d /tmp/looks-move.XXXXXX)
tmp_dirs+=("$dir")
stop_session() {
    [ -z "$spid" ] || {
        "$repo/tests/persona/session.sh" --stop "$dir/session.env" >/dev/null 2>&1
        wait "$spid" 2>/dev/null
        spid=
    }
}
trap 'status=$?; stop_session; gate_cleanup $status' EXIT
shot() {
    [ -n "${SHOTS:-}" ] || return 0
    mkdir -p "$SHOTS"
    admin capture "$SHOTS/$1.png" >/dev/null
}

(PERSONA_CHECK=1 PERSONA_DIR=$dir CHORA_BIN=$bin exec "$repo/tests/persona/session.sh" tiling+xp >"$dir.out" 2>"$dir.err") &
spid=$!
until_ok "grep -q '^PERSONA_ENV=' '$dir.out'" 240 || fail "the tiling+xp session did not start: $(cat "$dir.err")"
admin grant choractl screen.capture >/dev/null || fail "grant of the capture"
check start >/dev/null || fail "check.py start"

for n in 1 2 3; do
    act key Super+Return >/dev/null || fail "Super+Return"
    until_ok "[ \$(owned tiling | wc -w) = $n ]" 80 || fail "terminal $n did not open on tiling: $(owned tiling)"
done
read -r first second third <<<"$(owned tiling)"
until_ok "[ \"\$(holder tiling)\" = '{\"holder\":\"tiling\",\"surface\":\"$third\"}' ]" 40 ||
    fail "the third terminal has no keyboard before the give key: $(holders)"
steady_holders() { settled holders 10; }

shot before-key
act key Super+Alt+2 >/dev/null || fail "Super+Alt+2"
until_ok "owned xp | grep -qw $third" 40 || fail "Super+Alt+2 did not move $third to xp: tiling [$(owned tiling)], xp [$(owned xp)]"
steady_holders
shot after-key
[ "$(holder xp)" = '{"holder":null}' ] || fail "the give key: xp took the keyboard: $(holders)"
[ "$(holder tiling)" = "{\"holder\":\"tiling\",\"surface\":\"$second\"}" ] ||
    fail "the give key: the keyboard is not with $second of tiling: $(holders)"
ok "the give key: $third went to xp on HEADLESS-2, the keyboard stays with $second on tiling ($(holders))"

move() {
    local window=$1
    act look >/dev/null || fail "look"
    act click 82 15 >/dev/null || fail "click on the Looks door"
    until_ok launcher_up 80 || fail "the picker of the Looks door did not start"
    local try
    for try in 1 2 3 4 5; do
        act key Return >/dev/null || fail "Return on the row of xp"
        until_ok "owned xp | grep -qw $window" 4 && break
    done
    owned xp | grep -qw "$window" || fail "$window did not move to xp in $try tries: tiling [$(owned tiling)], xp [$(owned xp)]"
}
shot before-move
move "$second"
steady_holders
shot after-move-1
[ "$(holder xp)" = '{"holder":null}' ] || fail "the first move: xp took the keyboard: $(holders)"
[ "$(holder tiling)" = "{\"holder\":\"tiling\",\"surface\":\"$first\"}" ] ||
    fail "the first move: the keyboard is not with $first of tiling: $(holders)"
ok "the first move: $second went to xp on HEADLESS-2, the keyboard stays with $first on tiling ($(holders))"

move "$first"
steady_holders
shot after-move-2
[ "$(holder xp)" = '{"holder":null}' ] || fail "the second move: xp took the keyboard: $(holders)"
[ "$(holder tiling)" = '{"holder":"tiling"}' ] ||
    fail "the second move: tiling does not hold the keyboard with its desktop: $(holders)"
ok "the second move: the last window went to xp, tiling holds the keyboard with its desktop ($(holders))"

check begin >/dev/null || fail "check.py begin"
shown_tag() { admin state tiling | jq -e --argjson t "$1" '.desks[0].shown == [$t]' >/dev/null; }
for t in 2 1; do
    act key "Super+$t" >/dev/null || fail "Super+$t"
    until_ok "shown_tag $t" 20 || fail "Super+$t did not show tag $t: $(admin state tiling | jq -c .desks)"
done
l3=$(check end L3 2>&1 | grep '^task L3')
case $l3 in "task L3 pass"*) ;; *) fail "L3 after the moves: $l3 ($(holders))" ;; esac
ok "L3 after the moves: $l3"

act key Super+F1 >/dev/null || fail "Super+F1"
until_ok keys_open 20 || fail "Super+F1 opened no key list: $(admin state tiling | jq -c .open) ($(holders))"
shot keys
act key Escape >/dev/null || fail "Escape"
until_ok '! keys_open' 20 || fail "Escape did not close the key list ($(holders))"
[ "$(holder xp)" = '{"holder":null}' ] || fail "after the key list, xp has the keyboard: $(holders)"
shot after-keys
ok "Super+F1 opens the key list, Escape closes it, and the keyboard stays off xp ($(holders))"
