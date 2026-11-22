#!/bin/bash
# Self-test of the leaks of a gate (F5, DB21 oi-2): after one gate script
# that starts a session, no /tmp/gate-env.* file and no wallpaper.sh of the
# session is left. It counts both before and after, and fails on growth.
# A file of another gate in the same minute counts too: run it alone.
#
# Usage: tests/gate-leak-selftest.sh [GATE_SCRIPT [ARGS...]]
#   default: tests/gate-tiling-empty-output.sh flag (starts wallpaper.sh)
repo=$(cd "$(dirname "$0")/.." && pwd)
[ $# -gt 0 ] || set -- "$repo/tests/gate-tiling-empty-output.sh" flag
count_env() { find /tmp -maxdepth 1 -name 'gate-env.*' 2>/dev/null | wc -l; }
# shellcheck disable=SC2009 # counts only; pgrep -f by name is barred in tests
count_wall() { ps -eo args= | grep -c '[w]allpaper\.sh' || true; }
env_before=$(count_env)
wall_before=$(count_wall)
"$@" >/dev/null 2>&1
sleep 2
env_after=$(count_env)
wall_after=$(count_wall)
status=0
if [ "$env_after" -gt "$env_before" ]; then
    echo "FAIL: /tmp/gate-env.* grew from $env_before to $env_after"
    status=1
fi
if [ "$wall_after" -gt "$wall_before" ]; then
    echo "FAIL: wallpaper.sh processes grew from $wall_before to $wall_after"
    status=1
fi
[ "$status" != 0 ] || echo "ok: gate-env files $env_before -> $env_after, wallpaper.sh $wall_before -> $wall_after"
exit $status
