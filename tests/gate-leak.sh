#!/bin/bash
# Gate of sh-33 (rc 9, F5): a long session must not pile up memory or file
# descriptors in the core. One headless core lives through ROUNDS rounds;
# each round is a look switch (tiling, xp, mc in turn: the look starts,
# manages the windows of the round and ends) and an open/close churn of
# WINDOWS probe windows. After each round the gate samples the core:
# VmRSS of /proc/PID/status, the open fds of /proc/PID/fd (with a split by
# kind), the live windows of the scene and the connected clients.
#
# Red when, over the rounds after WARMUP:
#   - the least-squares slope of the RSS is above 1024 kB per round (set
#     before the first run: a leaked 200x200 probe buffer per round is
#     160 kB, a 1280x800 window texture 4 MB; allocator noise is far below);
#   - the fds grow by more than one per 10 rounds (last minus first);
#   - a round ends with more windows or clients than the base before
#     round 1.
#
# Usage: tests/gate-leak.sh   BIN: directory with the binaries
#   ROUNDS (default 12, at most 20), WINDOWS (default 8), WARMUP (default 3)
#   LEAK_LOG: the per-round table (default: a temp file, printed at the end)
# Helpers: fds (the fds of the core by kind), clients (clients.list but
# the gate), look NAME and unlook NAME (start and end the look of a round).
set -uo pipefail
. "$(dirname "$0")/gate-lib.sh"
need jq
need_bins chora choractl chora-probe chora-tiling chora-xp chora-mc
temp_config
export PATH=$bin:$PATH
ROUNDS=${ROUNDS:-12}
WINDOWS=${WINDOWS:-8}
WARMUP=${WARMUP:-3}
RSS_SLOPE_KB=1024
[ "$ROUNDS" -le 20 ] || fail "ROUNDS=$ROUNDS: at most 20 (the gate runs under 10 min)"
[ "$ROUNDS" -gt $((WARMUP + 3)) ] || fail "ROUNDS=$ROUNDS: need more than WARMUP+3 rounds for a slope"
work=$XDG_CONFIG_HOME
table=${LEAK_LOG:-$work/leak.tsv}

session_start 1280x800,1280x800+1280+0
core=$(XDG_RUNTIME_DIR=$session_runtime session_pids -x chora) || fail "no core pid"
[ "$(wc -w <<<"$core")" = 1 ] || fail "more than one core: $core"

fds() {
    find "/proc/$core/fd" -mindepth 1 -maxdepth 1 -printf '%l\n' 2>/dev/null |
        sed -E 's/^anon_inode:\[?([a-z_]*).*/anon:\1/; s/^(socket|pipe):.*/\1/; s#^/memfd:.*#memfd#; s#^/dev/dri/.*#dri#; s#^/dev/.*#dev#; s#^/(tmp|proc|run).*#\1#' |
        sort | uniq -c | awk '{ total += $1; split_ = split_ $2 "=" $1 " " } END { printf "%d\t%s\n", total, split_ }'
}
rss() { awk '/^VmRSS:/ { print $2 }' "/proc/$core/status"; }
windows() { ctl query 'kind:surface' | jq 'length'; }
clients() {
    timeout 10 python3 -I -c '
import json, os, socket
s = socket.socket(socket.AF_UNIX); s.connect(os.environ["CHORA_SOCKET"]); f = s.makefile("r")
for i, (m, p) in enumerate((("hello", {"name": "leak-gate"}), ("clients.list", None)), 1):
    s.sendall((json.dumps({"jsonrpc": "2.0", "id": i, "method": m, "params": p}) + "\n").encode())
    while (a := json.loads(f.readline())).get("id") != i: pass
print(sum(1 for c in a["result"] if c["name"] != "leak-gate"))'
}
wait_windows() {
    until_ok "[ \"\$(windows)\" = $1 ]" "${2:-40}"
}

look_pid=
look() {
    case $1 in
    tiling) "$bin/chora-tiling" >>"$work/tiling.log" 2>&1 & ;;
    xp) "$bin/chora-xp" --theme "$repo/flavors/xp/theme" >>"$work/xp.log" 2>&1 & ;;
    mc) "$bin/chora-mc" --manage-windows >>"$work/mc.log" 2>&1 & ;;
    esac
    look_pid=$!
    pids+=("$look_pid")
    until_ok "ctl query 'owner:$1' | jq -e 'length > 0' >/dev/null" 60 ||
        fail "round $round: the look $1 shows no nodes"
}
unlook() {
    kill "$look_pid" 2>/dev/null
    wait "$look_pid" 2>/dev/null
    until_ok "! ctl query 'owner:$1' | jq -e 'length > 0' >/dev/null" 40 ||
        fail "round $round: the nodes of $1 stay after it ended"
}

sleep 1
base_windows=$(windows)
base_clients=$(clients)
looks=(tiling xp mc)
printf 'round\tlook\trss_kb\tfds\twindows\tclients\tfd_kinds\n' >"$table"
printf '0\t-\t%s\t%s\t%s\t%s\n' "$(rss)" "$(fds)" "$base_windows" "$base_clients" |
    awk -F'\t' -v OFS='\t' '{ print $1, $2, $3, $4, $6, $7, $5 }' >>"$table"
for round in $(seq "$ROUNDS"); do
    name=${looks[$(((round - 1) % 3))]}
    look "$name"
    probes=()
    for i in $(seq "$WINDOWS"); do
        "$bin/chora-probe" window $((i * 30)) $((round * 20 % 256)) 90 >/dev/null 2>&1 &
        probes+=($!)
        pids+=($!)
    done
    wait_windows $((base_windows + WINDOWS)) ||
        fail "round $round ($name): $(windows) windows, expected $((base_windows + WINDOWS))"
    sleep 0.5
    kill "${probes[@]}" 2>/dev/null
    wait "${probes[@]}" 2>/dev/null
    wait_windows "$base_windows" || fail "round $round ($name): $(windows) windows after the close, base $base_windows"
    unlook "$name"
    sleep 1.5
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$round" "$name" "$(rss)" "$(fds)" "$(windows)" "$(clients)" |
        awk -F'\t' -v OFS='\t' '{ print $1, $2, $3, $4, $6, $7, $5 }' >>"$table"
done
cat "$table"

verdict=$(awk -F'\t' -v warm="$WARMUP" -v bound="$RSS_SLOPE_KB" -v bw="$base_windows" -v bc="$base_clients" '
    NR == 1 || $1 == 0 { next }
    $5 > bw { print "FAIL: round " $1 ": " $5 " windows, base " bw }
    $6 > bc { print "FAIL: round " $1 ": " $6 " clients, base " bc }
    $1 <= warm { next }
    { n++; x = $1; y = $3; sx += x; sy += y; sxx += x * x; sxy += x * y
      if (n == 1) { f0 = $4; r0 = $1 } f1 = $4; r1 = $1 }
    END {
        slope = (n * sxy - sx * sy) / (n * sxx - sx * sx)
        fd_bound = int((r1 - r0) / 10) + (((r1 - r0) % 10) > 0 ? 1 : 0)
        printf "rss slope %.0f kB/round (bound %d), fds %d -> %d over rounds %d..%d (bound +%d)\n", slope, bound, f0, f1, r0, r1, fd_bound
        if (slope > bound) printf "FAIL: the RSS of the core grows %.0f kB per round (bound %d)\n", slope, bound
        if (f1 - f0 > fd_bound) printf "FAIL: the fds of the core grow from %d to %d over %d rounds (bound +%d)\n", f0, f1, r1 - r0, fd_bound
    }' "$table")
echo "$verdict"
grep -q '^FAIL:' <<<"$verdict" && exit 1
ok "$ROUNDS rounds of $WINDOWS windows and a look switch: RSS and fds flat, windows and clients back to the base"
