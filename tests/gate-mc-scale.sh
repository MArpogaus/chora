#!/bin/bash
# chora-mc with a scene of 300 panels: scene.save, scene.show and
# settings.set each answer within half the 10 s that a method call of
# the core waits, on a debug build, and the core drops no late answer.
#
# Measured on 2026-11-11 with llvmpipe: a settings.set that changes more
# than the theme builds all scenes anew and takes 2.3 to 2.9 s on a debug
# build and 0.5 s on a release build; a change of the theme alone fades
# the colors and takes 1.7 s and 0.3 s. Most of it is the core, which
# applies about 1300 commands in one transaction (1.5 s debug, 0.3 s
# release); the new grabs of the keys take 0.4 s debug and 0.06 s
# release. So there is no partial rebuild: the release time is no stall.
#
# Usage: tests/gate-mc-scale.sh   BIN: directory with the binaries
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-mc
export PATH=$bin:$PATH
export CHORA_BIN=$bin
unset DBUS_SESSION_BUS_ADDRESS
work=$(mktemp -d /tmp/gate-mc-scale.XXXXXX)
tmp_dirs+=("$work")
temp_config
export CHORA_LOG=$work/chora.log
session_start 640x480 -- "$bin/chora-mc" --manage-windows
mc() { ctl call mc "$@" >/dev/null 2>&1; }
for _ in $(seq 75); do
    mc scene.list '{}' && break
    sleep 0.2
done

answers_within() {
    local limit=$1 what=$2 started took
    shift 2
    started=$(date +%s%3N)
    mc "$@" || fail "$what fails after $(($(date +%s%3N) - started)) ms"
    took=$(($(date +%s%3N) - started))
    [ "$took" -le "$limit" ] || fail "$what takes $took ms, more than $limit ms"
    ok "$what in $took ms"
}

panels=$(python3 -I -c 'import json
print(json.dumps([{"id": "p%d" % i, "cell": [i % 20, i // 20, 1, 1],
    "title": "T%d" % i, "lines": ["%d\tline" % i]} for i in range(300)]))')
answers_within 5000 "scene.save of 300 panels" scene.save \
    "{\"name\": \"many\", \"grid\": [20, 15], \"panels\": $panels}"
mc scene.save '{"name": "day", "grid": [1, 1], "panels": [
    {"id": "a", "cell": [0, 0, 1, 1], "title": "A", "lines": ["one"]}]}'
answers_within 5000 "scene.show of 300 panels" scene.show '{"name": "many"}'
answers_within 5000 "settings.set of a theme beside 300 panels" settings.set '{"theme": "light"}'
answers_within 5000 "scene.show of a small scene after it" scene.show '{"name": "day"}'
answers_within 5000 "scene.show of 300 panels again" scene.show '{"name": "many"}'
answers_within 5000 "settings.set of the write time beside 300 panels" settings.set \
    '{"write": {"duration_ms": 300}}'
late=$(grep -ac "no call .* waits for this connection" "$CHORA_LOG")
[ "$late" = 0 ] || fail "the core dropped $late late answers of chora-mc"
ok "no answer of chora-mc came late"
