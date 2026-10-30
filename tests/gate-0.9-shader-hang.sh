#!/bin/bash
# Gate check 2 of 0.9: a shader node with an endless loop hangs the main
# loop on llvmpipe. The watchdog thread logs the stall within 6 s, and the
# log line names the action (D64). A second session with a short
# CHORA_HANG_ABORT_S checks the action: the core aborts with SIGABRT.
# Run it under a memory limit, for example `prlimit --as=6000000000`.
#
# Usage: tests/gate-0.9-shader-hang.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl
ulimit -c 0 # the abort leaves no core file here
temp_config
work=$XDG_CONFIG_HOME
shader=$work/hang.frag
# x only grows, and depends on a uniform, so no compiler drops the loop.
cat >"$shader" <<'GLSL'
precision highp float;
varying vec2 v_uv;
uniform float time;
void main() {
    float x = 1.0 + v_uv.x;
    while (x > 0.0) {
        x += 1.0 + time;
    }
    gl_FragColor = vec4(x, 0.0, 0.0, 1.0);
}
GLSL

# hang ABORT_S: starts a session, puts the shader node and waits up to 6 s
# for the stall line of the watchdog. Sets log and core.
hang() {
    log=$work/chora-$1.log
    CHORA_HANG_ABORT_S=$1 CHORA_LOG=$log session_start 400x300
    core=${pids[-1]}
    # The core hangs inside the frame, maybe before it answers.
    echo '[{"create": {"id": "hang", "kind": "shader", "parent": "screen:HEADLESS-1",
        "props": {"size": [100, 100], "shader": "'"$shader"'"}}}]' |
        timeout 20 "$bin/choractl" --name agent apply - >"$work/apply-$1.log" 2>&1 &
    for _ in $(seq 60); do
        grep -q "the main loop stalls" "$log" && return
        sleep 0.1
    done
    fail "no stall line within 6 s: $(tail -n 5 "$log")"
}

hang 600
grep -q "the main loop stalls; the watchdog aborts chora with a core dump after 600 s" "$log" ||
    fail "the stall line names no action: $(grep 'main loop' "$log")"
ok "the watchdog thread logged the stall and its action within 6 s"
kill "${pids[@]}" 2>/dev/null
wait 2>/dev/null
pids=()

hang 5
timeout 15 tail --pid="$core" -f /dev/null || fail "the core still runs after 15 s"
grep -q "the main loop hangs; the watchdog aborts chora with a core dump" "$log" ||
    fail "no abort line: $(tail -n 5 "$log")"
grep -q "chora: SIGABRT" "$log" || fail "no SIGABRT: $(tail -n 5 "$log")"
ok "the watchdog aborted the hung core after CHORA_HANG_ABORT_S"
