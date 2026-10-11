#!/bin/bash
# End-to-end test of milestone M1: chora runs nested in a headless sway,
# chora-tiling tiles three foot windows, a crash of the plugin leaves the
# core running, and the emergency reset puts the windows in the grid.
# Further steps check fixed regressions, a taskbar, and the nested exit of
# M2: waybar, fuzzel and swaylock work, wlr-randr configures the output,
# the lock holds against other clients, and frames report their
# presentation time. The last step checks image nodes of M3.
#
# The test starts the reset with SIGUSR1. Ctrl+Alt+Escape runs the same
# code, but synthetic keys from wtype carry their own keymap, which a nested
# compositor cannot see.
#
# Usage: tests/nested.sh BIN_DIR [OUT_DIR]
#   BIN_DIR  holds chora, choractl, chora-tiling and chora-probe
#   OUT_DIR  receives the logs and screenshots (default: a temp dir)
# Needs: sway, foot, grim, jq, python3, pgrep and pkill, a Mesa driver,
# waybar with dbus-run-session, fuzzel, wlrctl, swaylock and wlr-randr.
set -euo pipefail

bin=$(realpath "$1")
out=$(realpath "${2:-$(mktemp -d)}")
mkdir -p "$out"
# A short path, because a socket path has at most 107 bytes.
XDG_RUNTIME_DIR=$(mktemp -d /tmp/chora-e2e.XXXXXX)
export XDG_RUNTIME_DIR
pids=()
cleanup() {
    kill "${pids[@]}" 2>/dev/null || true
    rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
fail() {
    echo "FAIL: $*" >&2
    exit 1
}
step() { echo "== $*"; }

# Waits up to 10 s for a command to succeed.
wait_for() {
    for _ in $(seq 50); do
        if "$@" >/dev/null 2>&1; then return 0; fi
        sleep 0.2
    done
    fail "timeout: $*"
}

step "host: headless sway"
printf 'output HEADLESS-1 resolution 1280x800\ndefault_border none\n' >"$out/sway.conf"
WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1 \
    sway -c "$out/sway.conf" >"$out/sway.log" 2>&1 &
pids+=($!)
wait_for test -S "$XDG_RUNTIME_DIR/wayland-1"
export WAYLAND_DISPLAY=wayland-1

step "1. chora -- chora-tiling starts nested"
"$bin/chora" -- "$bin/chora-tiling" >"$out/chora.log" 2>&1 &
chora_pid=$!
pids+=("$chora_pid")
wait_for compgen -G "$XDG_RUNTIME_DIR/chora/*.sock"
CHORA_SOCKET=$(compgen -G "$XDG_RUNTIME_DIR/chora/*.sock")
export CHORA_SOCKET
inner=$(basename "$CHORA_SOCKET" .sock)
wait_for pgrep -x chora-tiling

surfaces() { "$bin/choractl" query kind:surface; }
# Prints the color of a pixel of a new screenshot as "r g b".
pixel() {
    grim -t ppm - | tail -c +$((17 + ($2 * 1280 + $1) * 3)) | head -c 3 |
        od -An -tu1 | awk '{print $1, $2, $3}'
}
# Saves a screenshot as FILE and prints the number of its colors. The
# header of a 1280x800 PPM file has 16 bytes.
colors() {
    grim "$1"
    grim -t ppm - | tail -c +17 | od -An -v -tu1 -w3 | sort -u | wc -l
}
# Succeeds when exactly N surfaces match the jq condition.
count_is() { test "$(surfaces | jq "[.[] | select($2)] | length")" = "$1"; }

step "2. three foot windows appear without overlap"
feet=()
for _ in 1 2 3; do
    WAYLAND_DISPLAY=$inner foot >>"$out/foot.log" 2>&1 &
    feet+=($!)
done
pids+=("${feet[@]}")
wait_for count_is 3 '.owner == "tiling"'
sleep 0.5
surfaces >"$out/tiled.json"
jq -e '
  [.[] | {x: .transform.matrix[12], y: .transform.matrix[13],
          w: .surface.size[0], h: .surface.size[1]}] as $r
  | [range(0; $r | length) as $i | range($i + 1; $r | length) as $j
     | ($r[$i]) as $a | ($r[$j]) as $b
     | ($a.x < $b.x + $b.w and $b.x < $a.x + $a.w and
        $a.y < $b.y + $b.h and $b.y < $a.y + $a.h)]
  | any | not' "$out/tiled.json" >/dev/null || fail "windows overlap: $out/tiled.json"
test "$(colors "$out/tiled.png")" -gt 3 || fail "the screenshot is empty"

step "3. a crash of chora-tiling leaves chora running and detaches the surfaces"
pkill -KILL -x chora-tiling
wait_for count_is 3 '.parent == null and .owner == null'
kill -0 "$chora_pid" || fail "chora died"

# The later steps check fixed regressions.

# Sends a hello as "e2e" and one scene.apply per argument on one
# connection, closes its side, and prints each answer with its duration.
rpc() {
    timeout 20 python3 -I - "$CHORA_SOCKET" "$@" <<'EOF'
import json, socket, sys, time
path, calls = sys.argv[1], [("hello", {"name": "e2e", "keep_nodes": True})]
calls += [("scene.apply", {"commands": json.loads(c)}) for c in sys.argv[2:]]
s = socket.socket(socket.AF_UNIX)
s.settimeout(10)
s.connect(path)
f = s.makefile()
for i, (method, params) in enumerate(calls):
    start = time.monotonic()
    s.sendall(json.dumps({"jsonrpc": "2.0", "id": i, "method": method,
                          "params": params}).encode() + b"\n")
    if i == len(calls) - 1:
        s.shutdown(socket.SHUT_WR)
    answer = json.loads(f.readline())
    answer["seconds"] = time.monotonic() - start
    print(json.dumps(answer))
EOF
}
# Succeeds when every answer in the file is a result.
all_results() { jq -se 'all(has("result"))' "$1" >/dev/null; }

step "4. a transaction that arrives with the end of the stream still applies"
first=$(surfaces | jq -r '[.[].id] | sort | first')
rpc "[{\"attach\": {\"surface\": \"$first\", \"parent\": \"root\"}},
      {\"configure\": {\"surface\": \"$first\", \"size\": [300, 200]}}]" \
    >"$out/eof.json" || fail "no answers: $out/eof.json"
all_results "$out/eof.json" || fail "answers: $out/eof.json"
count_is 1 '.owner == "e2e"' || fail "the transaction did not apply"

step "5. a client that misses a deadline delays one transaction, not each"
configure() { echo "[{\"configure\": {\"surface\": \"$first\", \"size\": $1}}]"; }
kill -STOP "${feet[@]}"
rpc "$(configure '[310, 210]')" "$(configure '[320, 220]')" >"$out/late.json" || true
kill -CONT "${feet[@]}"
all_results "$out/late.json" || fail "answers: $out/late.json"
jq -se '.[1].seconds >= 0.1 and .[2].seconds < 0.1' "$out/late.json" >/dev/null ||
    fail "waits: $out/late.json"

step "6. a connection without hello closes after 5 s"
timeout 20 python3 -I - "$CHORA_SOCKET" <<'EOF' || fail "the idle connection stayed open"
import socket, sys
s = socket.socket(socket.AF_UNIX)
s.settimeout(8)
s.connect(sys.argv[1])
sys.exit(0 if s.recv(1) == b"" else 1)
EOF

step "7. a camera pans the view of the output"
work_area() { "$bin/choractl" outputs | jq -c '.[0].work_area'; }
echo '[{"camera": {"output": "winit", "transform": {"translate": [-100, -50]}}}]' |
    "$bin/choractl" --name e2e apply || fail "the camera command failed"
test "$(work_area)" = "[100,50,1280,800]" || fail "work area: $(work_area)"
sleep 0.3
test "$(colors "$out/pan.png")" -ge 2 || fail "the panned view is empty"
# A zoom of 2 doubles the 320x220 window at the scene origin: the point
# 600,400 of the output shows the window, not the black background.
echo '[{"camera": {"output": "winit", "transform": {"scale": [2, 2, 1]}}}]' |
    "$bin/choractl" --name e2e apply || fail "the zoom failed"
sleep 0.3
test "$(pixel 600 400)" != "0 0 0" || fail "the zoom does not resize the window"

step "8. the emergency reset puts the three surfaces in the grid"
kill -USR1 "$chora_pid"
wait_for count_is 3 '.parent == "root"'
test "$(work_area)" = "[0,0,1280,800]" || fail "the reset kept the camera: $(work_area)"
surfaces >"$out/grid.json"
jq -e '[.[] | [.transform.matrix[12], .transform.matrix[13]]] | sort == [[0,0],[0,400],[640,0]]' \
    "$out/grid.json" >/dev/null || fail "unexpected grid: $out/grid.json"
grim "$out/grid.png"

step "9. SIGUSR1 resets while the startup command runs, which blocks no signal"
rm -f "$out/sigblk"
# shellcheck disable=SC2016 # $1 expands in the inner shell
"$bin/chora" -- sh -c 'grep SigBlk /proc/self/status >"$1"; exec sleep 60' sh "$out/sigblk" \
    >"$out/chora-signal.log" 2>&1 &
signal_pid=$!
pids+=("$signal_pid")
wait_for grep -q "chora runs" "$out/chora-signal.log"
wait_for test -s "$out/sigblk"
grep -Eq 'SigBlk:[[:space:]]*0+$' "$out/sigblk" || fail "blocked signals: $(cat "$out/sigblk")"
kill -USR1 "$signal_pid"
wait_for grep -q "emergency reset" "$out/chora-signal.log"
kill -0 "$signal_pid" || fail "chora died of SIGUSR1"
# Gives the whole host output back to the first chora.
kill "$signal_pid"

step "10. a panel reserves its zone of the work area until it exits"
echo '{"layer": "top", "position": "top", "height": 30, "modules-center": ["clock"]}' \
    >"$out/waybar.json"
WAYLAND_DISPLAY=$inner dbus-run-session waybar -c "$out/waybar.json" >"$out/waybar.log" 2>&1 &
pids+=($!)
work_area_is() { test "$(work_area)" = "$1"; }
wait_for work_area_is "[0,30,1280,770]"
"$bin/choractl" events output.changed >"$out/panel-events.jsonl" &
events_pid=$!
pids+=("$events_pid")
sleep 0.3
pkill -x waybar
wait_for grep -q '"work_area":\[0,0,1280,800\]' "$out/panel-events.jsonl"
kill "$events_pid"

step "11. a launcher shows above the scene"
WAYLAND_DISPLAY=$inner fuzzel >"$out/fuzzel.log" 2>&1 &
fuzzel_pid=$!
pids+=("$fuzzel_pid")
sleep 1
kill -0 "$fuzzel_pid" || fail "fuzzel exited: $out/fuzzel.log"
test "$(colors "$out/launcher.png")" -gt 3 || fail "the launcher view is empty"
kill "$fuzzel_pid"

step "12. a taskbar lists the windows, and plugins get its requests"
"$bin/choractl" events surface.requested >"$out/taskbar-events.jsonl" &
events_pid=$!
pids+=("$events_pid")
sleep 0.3
taskbar() { WAYLAND_DISPLAY=$inner wlrctl toplevel "$@"; }
taskbar list >"$out/toplevels.txt" || fail "no toplevel list"
test "$(grep -c '^foot: ' "$out/toplevels.txt")" = 3 || fail "toplevels: $out/toplevels.txt"
# The reset focused one surface, so one toplevel is active.
taskbar find state:active || fail "no active toplevel"
taskbar focus app_id:foot || fail "the activation failed"
wait_for grep -q '"request":"activate"' "$out/taskbar-events.jsonl"
# Since the reset, no plugin runs, and the core does not close a window by
# itself.
taskbar close app_id:foot || fail "the close request failed"
wait_for grep -q '"request":"close"' "$out/taskbar-events.jsonl"
sleep 0.3
count_is 3 true || fail "the core closed a window"
kill "$events_pid"

step "13. wlr-randr lists the output and changes its scale and position"
randr() { WAYLAND_DISPLAY=$inner wlr-randr "$@"; }
# Succeeds when the jq condition holds for the first output.
output_has() { "$bin/choractl" outputs | jq -e ".[0] | $1" >/dev/null; }
randr >"$out/randr.txt" || fail "wlr-randr cannot list the outputs"
grep -q '^winit ' "$out/randr.txt" || fail "no head for winit: $out/randr.txt"
randr --output winit --scale 2 || fail "the scale change failed"
wait_for output_has '.scale == 2 and .size == [640, 400]'
randr --output winit --pos 100,50 --scale 1 || fail "the position change failed"
wait_for output_has '.position == [100, 50] and .size == [1280, 800]'
# The only output cannot turn off, and the window decides its mode.
if randr --output winit --off 2>>"$out/randr.txt"; then fail "the winit output turned off"; fi
randr --output winit --pos 0,0 || fail "the position reset failed"
wait_for output_has '.position == [0, 0] and .work_area == [0, 0, 1280, 800]'

step "14. the lock screen hides the scene and survives a crash of its client"
# Prints the most frequent color of a new screenshot as "r g b".
top_color() {
    grim -t ppm - | tail -c +17 | od -An -v -tu1 -w3 | sort | uniq -c | sort -rn |
        head -1 | awk '{print $2, $3, $4}'
}
probe() { WAYLAND_DISPLAY=$inner timeout 20 "$bin/chora-probe" "$@"; }
WAYLAND_DISPLAY=$inner swaylock -c ff0000 >"$out/swaylock.log" 2>&1 &
swaylock_pid=$!
pids+=("$swaylock_pid")
color_is() { test "$(top_color)" = "$1"; }
wait_for color_is "255 0 0"
probe second-lock >>"$out/probe.log" 2>&1 || fail "a second lock: $out/probe.log"
probe popup-grab >>"$out/probe.log" 2>&1 || fail "a grab while locked: $out/probe.log"
color_is "255 0 0" || fail "the lock screen is gone"
kill -KILL "$swaylock_pid"
wait_for color_is "0 0 0"
kill -0 "$chora_pid" || fail "chora died"
# A new lock client takes over and unlocks; the grid shows again.
probe take-lock >>"$out/probe.log" 2>&1 || fail "the takeover: $out/probe.log"
not_black() { test "$(pixel 100 100)" != "0 0 0"; }
wait_for not_black

step "15. a fullscreen request reaches plugins after surface.created"
"$bin/choractl" events surface.created surface.requested >"$out/requests.jsonl" &
events_pid=$!
pids+=("$events_pid")
sleep 0.3
WAYLAND_DISPLAY=$inner foot --fullscreen >>"$out/foot.log" 2>&1 &
pids+=($!)
wait_for grep -q '"request":"fullscreen"' "$out/requests.jsonl"
jq -se 'map(.event) | index("surface.created") < index("surface.requested")' \
    "$out/requests.jsonl" >/dev/null || fail "order: $out/requests.jsonl"
kill "$events_pid"

step "16. a frame tells its client when it showed"
probe presentation >>"$out/probe.log" 2>&1 &
probe_pid=$!
pids+=("$probe_pid")
probe_surface() { "$bin/choractl" query app_id:chora-probe | jq -er '.[0].id'; }
wait_for probe_surface
echo "[{\"attach\": {\"surface\": \"$(probe_surface)\", \"parent\": \"root\"}}]" |
    "$bin/choractl" --name e2e apply || fail "attaching the probe failed"
wait "$probe_pid" || fail "presentation time: $out/probe.log"

step "17. the new buffer of a transaction shows with the transaction"
probe held-commit 300 300 >"$out/held.log" 2>&1 &
probe_pid=$!
pids+=("$probe_pid")
wait_for probe_surface
held=$(probe_surface)
attach() { echo "{\"attach\": {\"surface\": \"$1\", \"parent\": \"root\"}}"; }
rpc "[$(attach "$held"), $(attach "$first")]" >"$out/held-attach.json"
all_results "$out/held-attach.json" || fail "answers: $out/held-attach.json"
wait_for grep -q ready "$out/held.log"
# A stopped foot misses the deadline, so the transaction applies after
# 150 ms; the buffer of the probe must wait for it.
kill -STOP "${feet[@]}"
rpc "[{\"configure\": {\"surface\": \"$held\", \"size\": [300, 300]}},
      {\"configure\": {\"surface\": \"$first\", \"size\": [330, 230]}}]" \
    >"$out/held-configure.json" || true
kill -CONT "${feet[@]}"
wait "$probe_pid" || fail "the held commit: $out/held.log"
all_results "$out/held-configure.json" || fail "answers: $out/held-configure.json"

step "18. an image node stretches a PNG file by its slice"
# Writes a 3x3 PNG file: red corners, green edges and a center of the
# color "r,g,b".
png() {
    python3 -I - "$1" "$2" <<'EOF'
import struct, sys, zlib
center = tuple(int(c) for c in sys.argv[2].split(",")) + (255,)
red, green = (255, 0, 0, 255), (0, 255, 0, 255)
rows = [[red, green, red], [green, center, green], [red, green, red]]
raw = b"".join(b"\0" + bytes(c for px in row for c in px) for row in rows)
def chunk(kind, data):
    crc = zlib.crc32(kind + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", crc)
header = struct.pack(">IIBBBBB", 3, 3, 8, 6, 0, 0, 0)
with open(sys.argv[1], "wb") as f:
    f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
            + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))
EOF
}
# Prints the props of the image node for a file.
image_props() {
    echo "{\"image\": \"$1\", \"size\": [300, 200], \"slice\": [1, 1, 1, 1],
           \"transform\": {\"translate\": [700, 450]}}"
}
png "$out/frame.png" 0,0,255
rpc "[{\"create\": {\"id\": \"frame\", \"kind\": \"image\", \"parent\": \"root\",
       \"props\": $(image_props "$out/frame.png")}}]" >"$out/image.json"
all_results "$out/image.json" || fail "answers: $out/image.json"
# Succeeds when a pixel is within 8 of the color "r g b" in each channel;
# texture filtering of the GPU may differ by a little.
pixel_is() {
    local got want d i
    read -ra got <<<"$(pixel "$1" "$2")"
    read -ra want <<<"$3"
    for i in 0 1 2; do
        d=$((got[i] - want[i]))
        test "${d#-}" -le 8 || return 1
    done
}
wait_for pixel_is 850 550 "0 0 255"
pixel_is 700 450 "255 0 0" || fail "corner: $(pixel 700 450)"
pixel_is 850 450 "0 255 0" || fail "edge: $(pixel 850 450)"
# A changed file loads again; a missing file rejects the transaction.
png "$out/frame.png" 255,255,0
update_image() { rpc "[{\"update\": {\"id\": \"frame\", \"props\": $(image_props "$1")}}]"; }
update_image "$out/frame.png" >"$out/image-reload.json"
all_results "$out/image-reload.json" || fail "answers: $out/image-reload.json"
wait_for pixel_is 850 550 "255 255 0"
update_image "$out/missing.png" >"$out/image-missing.json"
jq -se '.[1] | has("error")' "$out/image-missing.json" >/dev/null ||
    fail "a missing file applied: $out/image-missing.json"

step "19. an animation moves a node and tells when it ends"
"$bin/choractl" events animation.done >"$out/animation.jsonl" &
events_pid=$!
pids+=("$events_pid")
sleep 0.3
rpc '[{"animate": {"id": "frame", "props": {"transform": {"translate": [900, 450]}},
                   "duration": 300, "easing": "ease_out"}}]' >"$out/animate.json"
all_results "$out/animate.json" || fail "answers: $out/animate.json"
wait_for grep -q '"node":"frame"' "$out/animation.jsonl"
kill "$events_pid"
pixel_is 1050 550 "255 255 0" || fail "the moved center: $(pixel 1050 550)"
pixel_is 750 550 "0 0 0" || fail "the old place: $(pixel 750 550)"

echo "PASS (logs and screenshots in $out)"
