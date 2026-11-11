#!/bin/bash
# The bar follows the tiling (tiling-v2 review r2, 9): after a change of
# the tag or of the layout, the pill and the layout word change within
# 250 ms, because the plugin pushes the change to waybar with a signal
# and the bar does not wait for its next poll. The time runs from the call
# to the start of the first capture that shows the change; a capture
# takes about 100 ms, which is the resolution of this measure.
#
# Usage: tests/gate-tiling-follow.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need waybar
need jq
need_bins chora choractl chora-tiling
flavor=$repo/flavors/tiling
export PATH=$bin:$PATH
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
CHORA_FLAVOR_DIR=$flavor setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/dev/null 2>&1 &
run_pid $!
grant screen.capture
shot=$XDG_CONFIG_HOME/follow.png
snap() { ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1; }
for _ in $(seq 40); do
    sleep 0.5
    snap || continue
    [ "$(pixel "$shot" 400 24)" != "$(pixel "$shot" 400 300)" ] && break
done
sleep 2

now_ms() { echo $((($(date +%s%N)) / 1000000)); }

changes() {
    python3 -I - "$repo/tests/png-pixel.py" "$@" <<'PY'
import hashlib, importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
kind, start = sys.argv[2], int(sys.argv[3])
frames = []
for item in sys.argv[4:]:
    path, at = item.rsplit("@", 1)
    rows = [bytes(r) for r in mod.rows(open(path, "rb").read())]
    if kind == "pill":
        row = rows[24]
        key = next((x for x in range(100, 300) if tuple(row[x * 4:x * 4 + 3]) == (167, 139, 250)), None)
    else:
        key = hashlib.md5(b"".join(rows[y][360 * 4:460 * 4] for y in range(16, 32))).hexdigest()
    frames.append((int(at) - start, key))
first = frames[0][1]
print(next((at for at, key in frames if key is not None and key != first), 9999))
PY
}

burst() {
    local kind=$1 start=$2 list=() i at
    for i in $(seq 1 12); do
        at=$(now_ms)
        ctl capture --output HEADLESS-1 "$XDG_CONFIG_HOME/burst$i.png" >/dev/null 2>&1 || fail "the capture"
        list+=("$XDG_CONFIG_HOME/burst$i.png@$at")
    done
    changes "$kind" "$start" "$XDG_CONFIG_HOME/follow.png@$start" "${list[@]}"
}

for tag in 2 3 1; do
    snap
    start=$(now_ms)
    ctl call tiling view_tag "{\"tag\": $tag}" >/dev/null || fail "view_tag $tag"
    ms=$(burst pill "$start")
    echo "tag $tag: ${ms} ms"
    [ "$ms" -le 250 ] || fail "the pill took ${ms} ms to follow the switch to tag $tag"
done
ok "the pill follows a tag switch within 250 ms"

for _ in 1 2 3; do
    snap
    start=$(now_ms)
    ctl call tiling next_layout >/dev/null || fail "next_layout"
    ms=$(burst word "$start")
    echo "layout: ${ms} ms"
    [ "$ms" -le 250 ] || fail "the layout word took ${ms} ms to follow the change"
done
ok "the layout word follows a change within 250 ms"
