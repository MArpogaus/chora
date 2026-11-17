#!/bin/bash
# Unbound Super chords type nothing (tiling-v2, item 3; M5, t-power/019):
# Super+q, Super+z and Super+F5, none of them bound, do not reach the
# program in foot, a plain key still does, and a bound key still works.
#
# Usage: tests/gate-tiling-swallow.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
need jq
need_bins chora choractl chora-tiling
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
grant input.inject
log=$XDG_CONFIG_HOME/typed
ctl run foot sh -c "stty raw -echo; cat >$log" >/dev/null || fail "the run of foot"
for _ in $(seq 60); do
    seen=$(ctl query 'app_id:foot kind:surface') || { sleep 0.5; continue; }
    grep -q '"owner": *"tiling"' <<<"$seen" && break
    sleep 0.5
done
sleep 2
keys() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the key $1"; }
for chord in Super+q Super+z Super+F5 Super+Shift+x; do
    keys "$chord"
done
sleep 1
keys x
sleep 1
typed=$(cat "$log" 2>/dev/null)
[ "$typed" = x ] || fail "the program typed '$typed' and not only x"
ok "Super+q, Super+z, Super+F5 and Super+Shift+x type nothing; x types x"
keys Super+2
sleep 1
tagged=$(ctl call tiling get_tags) || fail "ctl call failed"
jq -e '.outputs[0].tags[1].shown' <<<"$tagged" >/dev/null || fail "Super+2 did not view tag 2"
ok "a bound key still works"
