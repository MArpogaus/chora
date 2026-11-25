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
# The bound is not a fixed 5000 ms (sh-37: a settings.set took 9316 ms in a
# lane that ran cargo beside the gate, 16 s in a train cell, and a baseline
# taken once at the start of the cell was too old when the load rose later).
# Before and after each call the cell times a fixed piece of work (cpu_probe_ms);
# the bound is 5000 ms times the slowdown of the slower of the two above the idle noise of the
# best probe of the cell (ratio 2), at least 5000 ms, at most 9000 ms. A loaded host slows the probe and
# the call alike; a call that is slow on an idle host still fails. A call that
# the core does not answer in the 10 s that it waits fails by itself, and the
# check at the end fails on a late answer.
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

probe_best=$(cpu_probe_ms)
answers_within() {
    local what=$1 started before after worst limit
    shift
    before=$(cpu_probe_ms)
    started=$(date +%s%3N)
    mc "$@" || fail "$what fails after $(($(date +%s%3N) - started)) ms"
    took=$(($(date +%s%3N) - started))
    after=$(cpu_probe_ms)
    worst=$((before > after ? before : after))
    probe_best=$((worst < probe_best ? worst : probe_best))
    limit=$(load_scaled_limit 5000 "$worst" "$probe_best")
    [ "$took" -le "$limit" ] || fail "$what takes $took ms, more than $limit ms (probe $worst ms, best $probe_best ms)"
    ok "$what in $took ms (limit $limit ms)"
}

panels=$(python3 -I -c 'import json
print(json.dumps([{"id": "p%d" % i, "cell": [i % 20, i // 20, 1, 1],
    "title": "T%d" % i, "lines": ["%d\tline" % i]} for i in range(300)]))')
answers_within "scene.save of 300 panels" scene.save \
    "{\"name\": \"many\", \"grid\": [20, 15], \"panels\": $panels}"
mc scene.save '{"name": "day", "grid": [1, 1], "panels": [
    {"id": "a", "cell": [0, 0, 1, 1], "title": "A", "lines": ["one"]}]}'
answers_within "scene.show of 300 panels" scene.show '{"name": "many"}'
answers_within "settings.set of a theme beside 300 panels" settings.set '{"theme": "light"}'
answers_within "scene.show of a small scene after it" scene.show '{"name": "day"}'
answers_within "scene.show of 300 panels again" scene.show '{"name": "many"}'
answers_within "settings.set of the write time beside 300 panels" settings.set \
    '{"write": {"duration_ms": 300}}'
late=$(grep -ac "no call .* waits for this connection" "$CHORA_LOG")
[ "$late" = 0 ] || fail "the core dropped $late late answers of chora-mc"
ok "no answer of chora-mc came late"
