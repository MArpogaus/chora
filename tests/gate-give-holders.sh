#!/bin/bash
# The give keys (Super+Alt+1 to 4) with each window manager as the one that
# holds them: xp, tiling and mc start first in turn, construct last, and
# two cells run per holder through tests/gate-handover-matrix.sh. One cell
# gives a window of the holder (the holder hands it over); the other gives
# a window of another manager (the holder forwards to its owner). The
# default run of the matrix gate has construct as the holder. A last run
# turns the setting off that lets the managers take injected keys, and
# expects that the injected give keys give nothing.
#
# Usage: tests/gate-give-holders.sh   BIN: directory with the binaries
#   SHOTS: directory for the logs (default: a temp dir per holder)
set -uo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
cells=(
    "xp|^(xp mc|tiling mc) key$"
    "tiling|^(tiling xp|mc xp) key$"
    "mc|^(mc tiling|xp tiling) key$"
)
for entry in "${cells[@]}"; do
    holder=${entry%%|*}
    only=${entry#*|}
    echo "holder $holder"
    HOLDER=$holder ONLY=$only CAP=120 SHOTS=${SHOTS:+$SHOTS/$holder} "$repo/tests/gate-handover-matrix.sh" || {
        echo "FAIL: gate-give-holders: $holder as the holder" >&2
        exit 1
    }
done
echo "holder construct, injected keys refused"
CHORA_TEST_ACCEPT_INJECTED_KEYS=0 ONLY='^(tiling xp|xp mc|mc tiling) key$' CAP=120 \
    SHOTS=${SHOTS:+$SHOTS/refused} "$repo/tests/gate-handover-matrix.sh" || {
    echo "FAIL: gate-give-holders: an injected key gave a window" >&2
    exit 1
}
echo "ok: gate-give-holders: xp, tiling and mc each served the give keys"
