#!/bin/bash
# Gate of the single focus holder in a face of the cube (invariant I3 of
# tests/acts, persona round 5): the construct opens the cube, one click
# lands on the desktop of the xp face, and then exactly one of construct and
# xp says it holds the focus in `state.get` (xp, which has the node of its
# icons focused; the construct must say nobody).
#
# Usage: tests/gate-construct-i3.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-xp
temp_config
export PATH=$bin:$PATH

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
holder() { ctl call "$1" state.get '{}' | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["focus"].get("holder") or "-")'; }

session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
ctl call construct cube.open >/dev/null || fail "cube.open failed"
until_ok 'ctl query id:xp-bar-VIRTUAL-1 | grep -q xp-bar' 80 || fail "xp shows no taskbar on the front face"
echo '[{"pointer_move": {"position": [640, 300]}}, {"click": {}}]' | ctl inject >/dev/null || fail "the click did not inject"
sleep 2
construct=$(holder construct)
xp=$(holder xp)
if [ "$construct" != - ] && [ "$xp" != - ]; then
    fail "construct and xp each say they hold the focus after a click on the desktop of the xp face (construct: $construct, xp: $xp)"
fi
ok "one focus holder after a click in the xp face (construct: $construct, xp: $xp)"
