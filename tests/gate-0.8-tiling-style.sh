#!/bin/bash
# Gate check 6 of 0.8 (D62, #99): `settings.set` changes the default
# style of chora-tiling, a new window gets this style, and the settings
# file holds it. A wrong value gets an error. The scripts gate-0.8-N.sh
# are checks of the 0.7 gate, so this one has a name of its own.
#
# Usage: tests/gate-0.8-tiling-style.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl chora-tiling chora-probe; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
config=$(mktemp -d /tmp/gate-style.XXXXXX)
export XDG_CONFIG_HOME=$config
session_start 1280x800 -- "$bin/chora-tiling"
file=$config/chora/chora-tiling.yaml

if ctl call tiling settings.set '{"style": {"opacity": 2, "effects": []}}' 2>/dev/null; then
    fail "settings.set took the opacity 2"
fi
test -e "$file" && fail "a wrong value wrote $file"
ctl call tiling settings.set '{"style": {"opacity": 0.5, "effects": []}}' >/dev/null ||
    fail "settings.set of the style"
ctl call tiling settings.get | grep -q '"opacity": *0.5' ||
    fail "settings.get: $(ctl call tiling settings.get)"
grep -q 'opacity: 0.5' "$file" || fail "the file has no opacity 0.5: $(cat "$file")"

pid=$(ctl run "$bin/chora-probe" window 255 0 0 |
    sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p')
[ -n "$pid" ] || fail "the run of the window"
run_pid "$pid"
for _ in $(seq 50); do
    node=$(ctl query 'app_id:chora-probe kind:surface')
    echo "$node" | grep -q '"owner": *"tiling"' && echo "$node" | grep -q '"opacity": *0.5' && break
    sleep 0.2
done
echo "$node" | grep -q '"opacity": *0.5' || fail "the new window: $node"
ok "settings.set gave the new window the opacity 0.5, and the file keeps it"
