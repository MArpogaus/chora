#!/bin/bash
# End-to-end test of the agent interface: chora runs nested
# in a headless sway with chora-tiling and two foot windows. An agent
# calls the methods that chora-tiling exports, gets grants, captures the
# output and nodes, types into a terminal, clicks a node and presses a
# shortcut of a plugin. Injected safety keys do nothing, and while the
# session is locked, captures and injections fail. grim and the probe
# capture through both screencopy protocols, and a client of a sandbox
# sees none of the privileged protocols. An MCP client waits for events
# and draws a shader that it hands to the core as an asset. Connections
# reserve space at the edges of the output. An agent starts a program with
# choractl run, keeps one under supervise and restarts it. An MCP client
# scales the output and keeps a HUD in place during a camera move.
#
# Usage: tests/agent.sh BIN_DIR [OUT_DIR]
#   BIN_DIR  holds chora, choractl, chora-tiling and chora-probe
#   OUT_DIR  receives the logs and screenshots (default: a temp dir)
# Needs: sway, foot, grim, jq, python3, pgrep, swaylock and a Mesa driver.
set -euo pipefail

bin=$(realpath "$1")
tests=$(realpath "$(dirname "$0")")
out=$(realpath "${2:-$(mktemp -d)}")
mkdir -p "$out"
# A short path, because a socket path has at most 107 bytes.
XDG_RUNTIME_DIR=$(mktemp -d /tmp/chora-agent-e2e.XXXXXX)
export XDG_RUNTIME_DIR
pids=()
cleanup() {
    kill "${pids[@]}" 2>/dev/null || true
    rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
fail() {
    echo "FAIL: $*" >&2
    grim "$out/failure.png" 2>/dev/null || true
    "$bin/choractl" query "*" >"$out/failure.json" 2>/dev/null || true
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

step "1. chora -- chora-tiling starts nested with two windows"
# The default configuration without gaps.
"$bin/chora-tiling" --print-config | sed 's/^gap: 4$/gap: 0/' >"$out/tiling.yaml"
grep -q '^gap: 0$' "$out/tiling.yaml" || fail "the configuration has no gap option"
"$bin/chora" -- "$bin/chora-tiling" --config "$out/tiling.yaml" >"$out/chora.log" 2>&1 &
chora_pid=$!
pids+=("$chora_pid")
wait_for compgen -G "$XDG_RUNTIME_DIR/chora/wayland-*[0-9].sock"
CHORA_SOCKET=$(compgen -G "$XDG_RUNTIME_DIR/chora/wayland-*[0-9].sock")
export CHORA_SOCKET
inner=$(basename "$CHORA_SOCKET" .sock)
wait_for pgrep -x chora-tiling
ctl() { "$bin/choractl" --name agent "$@"; }
surfaces() { ctl query kind:surface; }
count_is() { test "$(surfaces | jq "[.[] | select($2)] | length")" = "$1"; }
for _ in 1 2; do
    WAYLAND_DISPLAY=$inner foot >>"$out/foot.log" 2>&1 &
    pids+=($!)
done
wait_for count_is 2 '.owner == "tiling"'

# Succeeds when the surfaces stand at the x positions, in any order, for
# example "0 640".
xs_are() {
    test "$(surfaces | jq -r '[.[].transform.matrix[12]] | map(floor) | sort | map(tostring) | join(" ")')" = "$1"
}
ys_are() {
    test "$(surfaces | jq -r '[.[].transform.matrix[13]] | map(floor) | sort | map(tostring) | join(" ")')" = "$1"
}

step "2. tiling.get_layout and tiling.set_layout switch the layout"
ctl call tiling get_layout >"$out/layout.json" || fail "get_layout failed"
jq -e '.layout == "tile" and (.windows | length) == 2' "$out/layout.json" >/dev/null ||
    fail "layout: $(cat "$out/layout.json")"
wait_for xs_are "0 640"
ctl call tiling set_layout '{"layout": "fair_horizontal"}' >"$out/rows.json" || fail "set_layout failed"
jq -e '.layout == "fair_horizontal"' "$out/rows.json" >/dev/null || fail "rows: $(cat "$out/rows.json")"
wait_for xs_are "0 0"
wait_for ys_are "0 400"
ctl call tiling set_layout '{"layout": "fair"}' >/dev/null || fail "set_layout failed"
wait_for xs_are "0 640"

step "3. bad calls fail with a reason"
if ctl call tiling no_such_method 2>"$out/unknown.err"; then fail "an unknown method answered"; fi
grep -q "no exported method" "$out/unknown.err" || fail "unknown: $(cat "$out/unknown.err")"
if ctl call tiling set_layout '{"layout": "cube"}' 2>"$out/bad.err"; then
    fail "a bad layout applied"
fi
grep -q "cube" "$out/bad.err" || fail "bad layout: $(cat "$out/bad.err")"

step "4. a call fails when the exporter gives no answer, or disconnects"
# An exporter that ignores "wait" and exits on "quit".
python3 -I - "$CHORA_SOCKET" <<'EOF' >"$out/exporter.log" 2>&1 &
import json, socket, sys
s = socket.socket(socket.AF_UNIX)
s.connect(sys.argv[1])
f = s.makefile()
def send(i, method, params):
    s.sendall(json.dumps({"jsonrpc": "2.0", "id": i, "method": method,
                          "params": params}).encode() + b"\n")
send(0, "hello", {"name": "silent"})
send(1, "methods.export", {"names": ["wait", "quit"]})
for line in f:
    message = json.loads(line)
    print(line, end="", flush=True)
    if message.get("params", {}).get("method") == "quit":
        sys.exit(0)
EOF
pids+=($!)
wait_for grep -q '"id":1' "$out/exporter.log"
start=$SECONDS
if ctl call silent wait 2>"$out/wait.err"; then fail "a call without answer succeeded"; fi
grep -q "no answer in time" "$out/wait.err" || fail "wait: $(cat "$out/wait.err")"
test $((SECONDS - start)) -ge 9 || fail "the call ended early"
if ctl call silent quit 2>"$out/quit.err"; then fail "a call to a closed client succeeded"; fi
grep -q "disconnected" "$out/quit.err" || fail "quit: $(cat "$out/quit.err")"
kill -0 "$chora_pid" || fail "chora died"

# Prints the color of a pixel of a PNG file as "r g b a".
png_pixel() { python3 -I "$tests/png-pixel.py" "$@"; }
# Prints the color of a pixel of a screenshot of the host as "r g b". The
# header of a 1280x800 PPM file has 16 bytes.
host_pixel() {
    grim -t ppm "$out/host.ppm"
    od -An -tu1 -j $((16 + ($2 * 1280 + $1) * 3)) -N 3 "$out/host.ppm" | awk '{print $1, $2, $3}'
}
# Succeeds when two colors differ by at most 8 in each of r, g and b.
close_to() {
    awk -v a="$1" -v b="$2" 'BEGIN {
        split(a, x, " "); split(b, y, " ")
        for (i = 1; i <= 3; i++) if (x[i] - y[i] > 8 || y[i] - x[i] > 8) exit 1
    }'
}

step "5. screen.capture needs a grant"
if ctl capture "$out/denied.png" 2>"$out/denied.err"; then fail "a capture without grant worked"; fi
grep -q "needs a grant" "$out/denied.err" || fail "denied: $(cat "$out/denied.err")"
"$bin/choractl" grant agent screen.capture || fail "the grant failed"
"$bin/choractl" grants | jq -e '.agent == ["screen.capture"]' >/dev/null ||
    fail "grants: $("$bin/choractl" grants)"

step "6. a capture of the output shows what the host shows"
ctl capture "$out/screen.png" >"$out/screen.json" || fail "the capture failed"
jq -e '.width == 1280 and .height == 800 and .scale == 1' "$out/screen.json" >/dev/null ||
    fail "picture: $(cat "$out/screen.json")"
for point in "320 400" "960 400" "5 5" "1270 790"; do
    read -r x y <<<"$point"
    inner_color=$(png_pixel "$out/screen.png" "$x" "$y")
    host_color=$(host_pixel "$x" "$y")
    close_to "$inner_color" "$host_color" ||
        fail "pixel $point: $inner_color, the host shows $host_color"
done
ctl capture --max-side 640 "$out/small.png" >"$out/small.json" || fail "the small capture failed"
jq -e '.width == 640 and .height == 400 and .scale == 0.5' "$out/small.json" >/dev/null ||
    fail "small picture: $(cat "$out/small.json")"

step "7. a capture of a node shows the node on its own"
echo '[{"create": {"id": "swatch", "kind": "rect", "parent": "root",
        "props": {"size": [100, 50], "color": [1, 0, 0, 1],
                  "transform": {"matrix": [1,0,0,0, 0,1,0,0, 0,0,1,0, 1100,700,0,1]}}}}]' |
    ctl apply || fail "the swatch failed"
ctl capture --node swatch "$out/swatch.png" >"$out/swatch.json" || fail "the node capture failed"
jq -e '.width == 100 and .height == 50 and .origin == [0, 0]' "$out/swatch.json" >/dev/null ||
    fail "swatch: $(cat "$out/swatch.json")"
test "$(png_pixel "$out/swatch.png" 50 25)" = "255 0 0 255" ||
    fail "swatch pixel: $(png_pixel "$out/swatch.png" 50 25)"
first=$(surfaces | jq -r '[.[].id] | sort | first')
size=$(surfaces | jq -c --arg id "$first" '.[] | select(.id == $id) | .surface.size')
ctl capture --node "$first" "$out/window.png" >"$out/window.json" ||
    fail "the window capture failed"
jq -e --argjson size "$size" '[.width, .height] == $size' "$out/window.json" >/dev/null ||
    fail "window: $(cat "$out/window.json"), the surface has $size"

step "8. a revoke ends the grant"
"$bin/choractl" revoke agent || fail "the revoke failed"
if ctl capture "$out/revoked.png" 2>/dev/null; then fail "a capture after the revoke worked"; fi
kill -0 "$chora_pid" || fail "chora died"

step "9. injected text reaches the focused window"
inject() { echo "$1" | ctl inject; }
if inject '[{"text": {"text": "x"}}]' 2>"$out/inject-denied.err"; then
    fail "an injection without grant worked"
fi
grep -q "needs a grant" "$out/inject-denied.err" || fail "denied: $(cat "$out/inject-denied.err")"
"$bin/choractl" grant agent input.inject || fail "the grant failed"
typed="$out/typed.txt"
rm -f "$typed"
inject "[{\"text\": {\"text\": \"echo 'Typed: A-Z!' >$typed\\n\"}}]" || fail "the text failed"
wait_for test -s "$typed"
test "$(cat "$typed")" = "Typed: A-Z!" || fail "typed: $(cat "$typed")"

step "10. an injected click reaches plugins, and a shortcut reaches its grab"
echo '[{"create": {"id": "button", "kind": "rect", "parent": "root",
        "props": {"size": [80, 40], "color": [0, 0, 1, 1], "input": "accept",
                  "transform": {"matrix": [1,0,0,0, 0,1,0,0, 0,0,1,0, 1000,600,0,1]}}}}]' |
    ctl apply || fail "the button failed"
ctl events pointer.pressed pointer.released >"$out/pointer.jsonl" &
pids+=($!)
sleep 0.3
inject '[{"pointer_move": {"position": [1040, 620]}}, {"click": {}}]' || fail "the click failed"
wait_for grep -q '"event":"pointer.released","node":"button"' "$out/pointer.jsonl"
grep -q '"event":"pointer.pressed","node":"button","button":272' "$out/pointer.jsonl" ||
    fail "press: $(cat "$out/pointer.jsonl")"
inject '[{"shortcut": {"keys": "Super+Return"}}]' || fail "the shortcut failed"
wait_for count_is 3 '.owner == "tiling"'
# The bindings of awesome: tags hide and show windows, and the layouts turn.
shown_is() { count_is "$1" '.owner == "tiling" and .visible != false'; }
key() { inject "[{\"shortcut\": {\"keys\": \"$1\"}}]" || fail "the shortcut $1 failed"; }
key Super+Shift+2
wait_for shown_is 2
key Super+2
wait_for shown_is 1
key Super+Ctrl+1
wait_for shown_is 3
key Super+Escape
wait_for shown_is 1
key Super+Shift+1
key Super+1
wait_for shown_is 3
layout_is() { test "$(ctl call tiling get_layout | jq -r .layout)" = "$1"; }
key Super+space
wait_for layout_is fair_horizontal
key Super+Shift+space
wait_for layout_is fair

step "11. injected keys never reach the safety layer"
resets() { grep -c "emergency reset" "$out/chora.log" || true; }
before=$(resets)
inject '[{"shortcut": {"keys": "Ctrl+Alt+Escape"}}, {"shortcut": {"keys": "Ctrl+Alt+BackSpace"}}]' ||
    fail "the safety keys failed"
sleep 1
kill -0 "$chora_pid" || fail "an injected Ctrl+Alt+BackSpace stopped chora"
test "$(resets)" = "$before" || fail "an injected Ctrl+Alt+Escape reset"
count_is 3 '.owner == "tiling"' || fail "the windows lost their plugin"

step "12. while the session is locked, no capture and no injection"
"$bin/choractl" grant agent screen.capture || fail "the grant failed"
WAYLAND_DISPLAY=$inner swaylock -c ff0000 >"$out/swaylock.log" 2>&1 &
swaylock_pid=$!
pids+=("$swaylock_pid")
red() { test "$(host_pixel 640 400)" = "255 0 0"; }
wait_for red
if ctl capture "$out/locked.png" 2>"$out/locked.err"; then fail "a capture while locked worked"; fi
grep -q "locked" "$out/locked.err" || fail "capture: $(cat "$out/locked.err")"
if inject '[{"text": {"text": "x"}}]' 2>"$out/locked-inject.err"; then
    fail "an injection while locked worked"
fi
grep -q "locked" "$out/locked-inject.err" || fail "inject: $(cat "$out/locked-inject.err")"
# A new lock client takes over from the killed one and unlocks.
kill -KILL "$swaylock_pid"
wait "$swaylock_pid" 2>/dev/null || true
WAYLAND_DISPLAY=$inner timeout 20 "$bin/chora-probe" take-lock >"$out/probe.log" 2>&1 ||
    fail "the takeover: $out/probe.log"
ctl capture "$out/unlocked.png" >/dev/null || fail "no capture after the unlock"

step "13. grim captures chora through screencopy"
WAYLAND_DISPLAY=$inner grim -t ppm "$out/grim.ppm" 2>"$out/grim.log" ||
    fail "grim failed: $(cat "$out/grim.log")"
for point in "320 400" "960 400" "1040 620"; do
    read -r x y <<<"$point"
    grim_color=$(od -An -tu1 -j $((16 + (y * 1280 + x) * 3)) -N 3 "$out/grim.ppm" | awk '{print $1, $2, $3}')
    host_color=$(host_pixel "$x" "$y")
    close_to "$grim_color" "$host_color" ||
        fail "grim pixel $point: $grim_color, the host shows $host_color"
done
WAYLAND_DISPLAY=$inner grim -t ppm -g "1000,600 80x40" "$out/grim-region.ppm" 2>>"$out/grim.log" ||
    fail "grim with a region failed: $(cat "$out/grim.log")"
test "$(head -c 9 "$out/grim-region.ppm" | tr '\n' ' ')" = "P6 80 40 " ||
    fail "region: $(head -c 12 "$out/grim-region.ppm" | od -c | head -2)"
region_color=$(od -An -tu1 -j $((13 + (20 * 80 + 40) * 3)) -N 3 "$out/grim-region.ppm" | awk '{print $1, $2, $3}')
close_to "$region_color" "0 0 255" || fail "the region shows $region_color, not the blue button"

step "14. ext-image-copy-capture captures the output and a toplevel, at a pace"
probe() { WAYLAND_DISPLAY=$inner timeout 20 "$bin/chora-probe" "$@"; }
probe capture output >"$out/ext-output.log" 2>&1 || fail "the output capture: $(cat "$out/ext-output.log")"
grep -q "capture: 1280x800, " "$out/ext-output.log" || fail "output: $(cat "$out/ext-output.log")"
colors=$(sed -n 's/.*, \([0-9]*\) colors/\1/p' "$out/ext-output.log")
test "$colors" -gt 3 || fail "the output capture has $colors colors"
probe capture toplevel >"$out/ext-toplevel.log" 2>&1 ||
    fail "the toplevel capture: $(cat "$out/ext-toplevel.log")"
size=$(sed -n 's/capture: \([0-9]*x[0-9]*\), .*/\1/p' "$out/ext-toplevel.log")
surfaces | jq -e --arg size "$size" 'any(.[]; "\(.surface.size[0])x\(.surface.size[1])" == $size)' >/dev/null ||
    fail "the toplevel capture has $size, no surface has that size"

# A client gets four frames per refresh at most; the others fail.
probe capture flood >"$out/ext-flood.log" 2>&1 || fail "the flood: $(cat "$out/ext-flood.log")"
grep -q "capture: 4 served, 6 refused" "$out/ext-flood.log" || fail "flood: $(cat "$out/ext-flood.log")"

step "15. a sandboxed client sees no privileged protocol"
probe sandbox >"$out/sandbox.log" 2>&1 || fail "the sandbox: $(cat "$out/sandbox.log")"
kill -0 "$chora_pid" || fail "chora died"

step "16. a view configures its plugin and adds its nodes"
config="$out/config"
mkdir -p "$config/chora/plugins" "$config/chora/views"
printf 'name: tiling\nexec: %s/chora-tiling --config %s\nconfigure: set_layout\n' "$bin" "$out/tiling.yaml" \
    >"$config/chora/plugins/tiling.yaml"
cat >"$config/chora/views/work.yaml" <<'VIEW'
name: work
plugins:
- name: tiling
  config: {layout: max}
scene:
- id: work-panel
  kind: group
  props: {transform: {translate: [1100, 20]}}
- id: work-badge
  kind: rect
  parent: work-panel
  props: {size: [60, 30], color: [0, 1, 0, 1]}
VIEW
view() { XDG_CONFIG_HOME=$config ctl view "$@"; }
view load work >"$out/view.json" || fail "the view failed to load"
jq -e '.nodes == 2 and .started == []' "$out/view.json" >/dev/null || fail "load: $(cat "$out/view.json")"
wait_for xs_are "0 0 0"
ctl query owner:view-work | jq -e 'length == 2' >/dev/null || fail "the view has no nodes"
view load work >/dev/null || fail "the view failed to load again"
ctl query owner:view-work | jq -e 'length == 2' >/dev/null || fail "a second load doubled the nodes"
if view load "$out/no-such.yaml" 2>/dev/null; then fail "a missing view loaded"; fi
printf 'name: bad\nscene:\n- id: x\n  kind: rect\n  colour: [1, 0, 0, 1]\n' >"$out/bad.yaml"
if view load "$out/bad.yaml" 2>"$out/bad-view.err"; then fail "a view with an unknown key loaded"; fi
grep -q "colour" "$out/bad-view.err" || fail "bad view: $(cat "$out/bad-view.err")"

step "17. a view saves its nodes, unloads, and starts its plugin when it is gone"
echo '[{"update": {"id": "work-badge", "props": {"color": [1, 1, 0, 1]}}}]' |
    "$bin/choractl" --name view-work apply || fail "the update failed"
view save work >/dev/null || fail "the save failed"
grep -q "work-badge" "$config/chora/views/work.yaml" ||
    fail "saved: $(cat "$config/chora/views/work.yaml")"
view load work >/dev/null || fail "the saved view failed to load"
ctl query id:work-badge | jq -e '.[0].color == [1, 1, 0, 1]' >/dev/null ||
    fail "the saved color is gone: $(ctl query id:work-badge)"
view unload work >/dev/null || fail "the unload failed"
ctl query owner:view-work | jq -e 'length == 0' >/dev/null || fail "the unload left nodes"
pkill -x chora-tiling
wait_for count_is 0 '.owner == "tiling"'
view load work >"$out/view-start.json" || fail "the view did not start its plugin"
jq -e '.started == ["tiling"]' "$out/view-start.json" >/dev/null ||
    fail "start: $(cat "$out/view-start.json")"
wait_for count_is 3 '.owner == "tiling"'
wait_for xs_are "0 0 0"

step "18. exit: an MCP client describes the screen, sets the layout, sees it, and waits for events"
# Two windows in one color each instead of the terminals.
pkill -x foot
wait_for count_is 0 'true'
WAYLAND_DISPLAY=$inner "$bin/chora-probe" window 255 0 0 >>"$out/windows.log" 2>&1 &
pids+=($!)
wait_for count_is 1 '.owner == "tiling"'
WAYLAND_DISPLAY=$inner "$bin/chora-probe" window 0 0 255 >>"$out/windows.log" 2>&1 &
pids+=($!)
wait_for count_is 2 '.owner == "tiling"'
ctl call tiling set_layout '{"layout": "fair"}' >/dev/null || fail "set_layout failed"
wait_for xs_are "0 640"
"$bin/choractl" grant agent screen.capture || fail "the grant failed"
"$bin/choractl" grant agent input.inject || fail "the grant failed"
timeout 60 python3 -I "$tests/mcp-client.py" "$bin/choractl" "$out" fair_horizontal >"$out/mcp.log" 2>&1 ||
    fail "the MCP client: $(cat "$out/mcp.log")"
color_at() { png_pixel "$out/$1.png" "$2" "$3" | cut -d' ' -f1-3; }
# The newer window, blue, is the master.
test "$(color_at before 320 400)" = "0 0 255" || fail "before, left: $(color_at before 320 400)"
test "$(color_at before 960 400)" = "255 0 0" || fail "before, right: $(color_at before 960 400)"
test "$(color_at after 640 200)" = "0 0 255" || fail "after, top: $(color_at after 640 200)"
test "$(color_at after 640 600)" = "255 0 0" || fail "after, bottom: $(color_at after 640 600)"
jq -e '[.windows[].position[1]] | sort == [0, 400]' "$out/after.json" >/dev/null ||
    fail "the description after: $(cat "$out/after.json")"
grep -q "screen before and after" "$out/mcp.log" || fail "mcp: $(cat "$out/mcp.log")"
grep -q "its click and its key" "$out/mcp.log" || fail "mcp: $(cat "$out/mcp.log")"
grep -q "its own shader" "$out/mcp.log" || fail "mcp: $(cat "$out/mcp.log")"
test "$(color_at asset 50 50)" = "255 0 255" || fail "the asset: $(color_at asset 50 50)"

step "19. assets.put stores the files of a client until its nodes go"
asset=$(cat "$out/asset-path.txt")
case $asset in "$XDG_RUNTIME_DIR"/chora/*) ;; *) fail "the asset is at $asset" ;; esac
# The agent keeps its nodes, so its files stay after the disconnect.
test -f "$asset" || fail "the asset of the agent went"
printf 'one' >"$out/one.txt"
put=$(ctl asset put one.txt "$out/one.txt") || fail "the put failed"
cmp -s "$put" "$out/one.txt" || fail "the put stored $(cat "$put")"
# The directory of a client is its name in hex: agent.
case $put in */6167656e74/one.txt) ;; *) fail "the put is at $put" ;; esac
printf 'two' | ctl asset put one.txt - >/dev/null || fail "the second put failed"
test "$(cat "$put")" = two || fail "the second put stored $(cat "$put")"
if ctl asset put ../x "$out/one.txt" 2>/dev/null; then fail "a put of ../x worked"; fi
ctl asset remove one.txt || fail "the remove failed"
test ! -e "$put" || fail "the remove left $put"
# The files of a client without keep_nodes go at its disconnect.
python3 -I - "$CHORA_SOCKET" <<'EOF' >"$out/temporary-asset.txt" || fail "the temporary put failed"
import base64, json, socket, sys
stream = socket.socket(socket.AF_UNIX)
stream.connect(sys.argv[1])
lines = stream.makefile("rw")
def call(number, method, params):
    lines.write(json.dumps({"jsonrpc": "2.0", "id": number, "method": method, "params": params}) + "\n")
    lines.flush()
    answer = json.loads(lines.readline())
    if "error" in answer:
        sys.exit(f"{method}: {answer}")
    return answer["result"]
call(1, "hello", {"name": "temporary", "keep_nodes": False})
print(call(2, "assets.put", {"name": "t.png", "data": base64.b64encode(b"x").decode()})["path"])
EOF
temporary=$(cat "$out/temporary-asset.txt")
case $temporary in "$XDG_RUNTIME_DIR"/chora/*) ;; *) fail "the temporary asset is at $temporary" ;; esac
wait_for test ! -e "$temporary"

step "20. output.reserve leaves space out of the work area while its connection lives"
work_area_is() { test "$(ctl outputs | jq -c '.[0].work_area')" = "$1"; }
wait_for work_area_is "[0,0,1280,800]"
"$bin/choractl" --name agent reserve --left 200 >"$out/reserve.json" &
reserve_pid=$!
pids+=("$reserve_pid")
wait_for work_area_is "[200,0,1080,800]"
wait_for xs_are "200 200"
jq -e '.work_area == [200, 0, 1080, 800]' "$out/reserve.json" >/dev/null ||
    fail "the answer: $(cat "$out/reserve.json")"
# The reserves of two connections add up.
"$bin/choractl" --name panel reserve --top 50 --left 100 >/dev/null &
panel_pid=$!
pids+=("$panel_pid")
wait_for work_area_is "[300,50,980,750]"
kill "$reserve_pid" "$panel_pid"
wait_for work_area_is "[0,0,1280,800]"
wait_for xs_are "0 0"
if ctl reserve --output nowhere --top 1 2>/dev/null; then fail "a reserve of no output worked"; fi

step "21. choractl run starts a program, supervise and restart handle one"
# Run needs no grant (D42): the core only hands out an activation token.
# The probe connects to chora, not to the host sway, with the inner display.
WAYLAND_DISPLAY=$inner "$bin/choractl" --name agent run "$bin/chora-probe" window 0 255 0 >"$out/run.json" ||
    fail "the run failed"
pids+=("$(jq -r .pid "$out/run.json")")
wait_for count_is 3 '.owner == "tiling"'
# A program under `choractl supervise` answers `choractl restart`: the
# pid file names the running program, and a new one takes over.
"$bin/choractl" supervise slow -- sleep 60 &
pids+=($!)
sup_file=$XDG_RUNTIME_DIR/chora/supervise/slow.pid
wait_for test -s "$sup_file"
old=$(cat "$sup_file")
"$bin/choractl" restart slow >"$out/restart.json" 2>"$out/restart.err" ||
    fail "the restart failed: $(cat "$out/restart.err")"
jq -e --argjson old "$old" '.restarted == "supervise" and .pid == $old' \
    "$out/restart.json" >/dev/null || fail "the restart: $(cat "$out/restart.json")"
pid_is_new() {
    local now
    now=$(cat "$sup_file" 2>/dev/null) && test -n "$now" && test "$now" != "$old"
}
wait_for pid_is_new
pids+=("$(cat "$sup_file")")
if "$bin/choractl" restart nobody 2>/dev/null; then fail "a restart of no client worked"; fi

step "22. an MCP client draws a panel and moves the camera on a click"
rm -f "$out/panel-ready.txt"
timeout 60 python3 -I "$tests/mcp-client.py" "$bin/choractl" "$out" panel >"$out/panel.log" 2>&1 &
panel_pid=$!
pids+=("$panel_pid")
wait_for test -s "$out/panel-ready.txt"
# The panel is a rounded shader node of the shader that the client put.
test "$(color_at panel 640 30)" = "0 255 255" || fail "the panel: $(color_at panel 640 30)"
test "$(png_pixel "$out/panel.png" 1 1 | cut -d' ' -f4)" = 0 ||
    fail "the round corner: $(png_pixel "$out/panel.png" 1 1)"
# The windows leave the reserved top free.
min_y_is() { test "$(surfaces | jq '[.[].transform.matrix[13]] | min | floor')" = "$1"; }
wait_for min_y_is 60
# The user clicks the left button, then the right one.
inject '[{"pointer_move": {"position": [70, 30]}}, {"click": {}}]' || fail "the click failed"
sleep 0.5
kill -0 "$panel_pid" || fail "the client ended after the left button: $(cat "$out/panel.log")"
inject '[{"pointer_move": {"position": [190, 30]}}, {"click": {}}]' || fail "the click failed"
wait "$panel_pid" || fail "the MCP client: $(cat "$out/panel.log")"
grep -q "moved the camera from its panel" "$out/panel.log" || fail "panel: $(cat "$out/panel.log")"
# The reserve ends with the server.
wait_for work_area_is "[0,0,1280,800]"
wait_for min_y_is 0

step "23. an MCP client reads a probe, scales the output, and keeps a HUD in place"
"$bin/choractl" grant agent screen.capture || fail "the grant failed"
timeout 60 python3 -I "$tests/mcp-client.py" "$bin/choractl" "$out" hud >"$out/hud.log" 2>&1 ||
    fail "the MCP client: $(cat "$out/hud.log")"
grep -q "kept its HUD" "$out/hud.log" || fail "hud: $(cat "$out/hud.log")"
# The HUD stays at the top right during the move and after it, while the
# mark moves from x 500 to x 100.
for shot in hud-moving hud-moved; do
    test "$(color_at "$shot" 1180 40)" = "255 0 255" || fail "$shot: $(color_at "$shot" 1180 40)"
done
test "$(color_at hud-moved 150 350)" = "255 255 255" || fail "the mark: $(color_at hud-moved 150 350)"
test "$(color_at hud-moving 550 350)" != "255 255 255" || fail "the camera did not move yet"
"$bin/choractl" revoke agent || fail "the revoke failed"

echo "PASS (logs and screenshots in $out)"
