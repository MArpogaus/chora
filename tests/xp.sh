#!/bin/bash
# End-to-end test of the XP flavor: chora runs nested in a headless sway
# with chora-xp. The taskbar and the start button show, two foot windows
# get title bars, a drag of a title bar moves its window and bends its
# mesh, the minimize button sends the window through the magic lamp and
# the task button brings it back, the start menu opens and starts a
# terminal, the buttons of the title bar maximize and close a window, and
# the desktop buttons turn the cube to another desktop and back, My
# Documents of the start menu starts the file manager, and the exported
# methods do what the clicks do. Buttons light up under the pointer.
#
# A virtual pointer of the host drives chora, as in tests/nested.sh. Its
# coordinates are those of the 1280x800 output, which shows the scene
# from its origin. The effects run five times slower, so that screenshots
# catch them half way; they go to OUT_DIR as wobbly.png, lamp.png,
# restore.png, cube.png and cube-back.png. The software renderer of the
# host is slow, so a frame budget keeps the watchdog of the core calm.
#
# Usage: tests/xp.sh BIN_DIR [OUT_DIR]
#   BIN_DIR  holds chora, choractl, chora-xp and chora-probe
#   OUT_DIR  receives the logs and screenshots (default: a temp dir)
# Needs: sway, foot, grim, jq, pgrep and a Mesa driver. The file manager
# is thunar when it is installed, else foot in the folder.
set -euo pipefail

bin=$(realpath "$1")
# The plugins start apps with `choractl run` from PATH (D42).
export PATH="$bin:$PATH"
out=$(realpath "${2:-$(mktemp -d)}")
theme=$(realpath "$(dirname "$0")/../flavors/xp/theme")
mkdir -p "$out"
# A short path, because a socket path has at most 107 bytes.
XDG_RUNTIME_DIR=$(mktemp -d /tmp/chora-xp-e2e.XXXXXX)
export XDG_RUNTIME_DIR
# No user manager of the host: apps of the start menu run as children
# of choractl, which the cleanup ends.
unset DBUS_SESSION_BUS_ADDRESS
pids=()
cleanup() {
    kill "${pids[@]}" 2>/dev/null || true
    rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
fail() {
    echo "FAIL: $*" >&2
    grim "$out/failure.png" 2>/dev/null || true
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
# A home of its own: step 13 opens it in the file manager.
export HOME=$XDG_RUNTIME_DIR
unset XDG_CONFIG_HOME XDG_DATA_HOME

if command -v thunar >/dev/null; then
    file_manager=thunar
else
    file_manager="foot --working-directory"
fi

sed "s|^file_manager: .*|file_manager: $file_manager|" \
    "$(dirname "$0")/../plugins/chora-xp/xp.yaml" >"$out/chora-xp.yaml"

step "1. chora -- chora-xp starts nested"
CHORA_FRAME_BUDGET_US=500000 "$bin/chora" -- "$bin/chora-xp" --theme "$theme" --slow-motion 5 \
    --config "$out/chora-xp.yaml" >"$out/chora.log" 2>&1 &
chora_pid=$!
pids+=("$chora_pid")
wait_for compgen -G "$XDG_RUNTIME_DIR/chora/wayland-*[0-9].sock"
CHORA_SOCKET=$(compgen -G "$XDG_RUNTIME_DIR/chora/wayland-*[0-9].sock")
export CHORA_SOCKET
inner=$(basename "$CHORA_SOCKET" .sock)
query() { "$bin/choractl" query "$@"; }
has_node() { test "$(query "$1" | jq length)" = 1; }
has_node_count() { test "$(query "$2" | jq length)" = "$1"; }
wait_for has_node id:xp-tray-winit

# Prints the color of a pixel of a new screenshot as "r g b". The header
# of a 1280x800 PPM file has 16 bytes. The screenshot goes to a file: a
# pipe that ends early would fail under pipefail.
pixel() {
    grim -t ppm "$out/pixel.ppm"
    od -An -tu1 -j $((16 + ($2 * 1280 + $1) * 3)) -N 3 "$out/pixel.ppm" |
        awk '{print $1, $2, $3}'
}
# Succeeds when the pixel at X Y passes an awk condition on r, g and b.
pixel_is() {
    pixel "$1" "$2" | awk "{r = \$1; g = \$2; b = \$3; exit !($3)}"
}
blue='b > 180 && b > r + 80'
green='g > r + 40 && g > b + 40'
white='r > 240 && g > 240 && b > 240'

step "2. the taskbar is blue, and the start button at its left is green"
wait_for pixel_is 640 790 "$blue"
wait_for pixel_is 60 795 "$green"
grim "$out/desktop.png"

step "3. two foot windows get title bars"
for _ in 1 2; do
    WAYLAND_DISPLAY=$inner foot >>"$out/foot.log" 2>&1 &
    pids+=($!)
done
# The windows show once their groups are visible.
shown() { test "$(query meta:xp=window | jq '[.[] | select(.visible)] | length')" = 2; }
wait_for shown
sleep 0.5
query kind:surface >"$out/surfaces.json"
query meta:xp=window >"$out/windows.json"
grim "$out/windows.png"
# Prints "x y" of the frame of a surface: the translation of its group,
# which starts at the 10 pixel shadow. frame_width prints the width of the
# client plus the borders.
frame() {
    query "id:xp-win-$1" |
        jq -r '.[0].transform.matrix | "\(.[12] + 10 | floor) \(.[13] + 10 | floor)"'
}
frame_width() { query "id:$1" | jq -r '.[0].surface.size[0] + 8'; }
mapfile -t ids < <(jq -r '[.[] | select(.owner == "xp") | .id] | sort_by(.[1:] | tonumber) | .[]' \
    "$out/surfaces.json")
test "${#ids[@]}" = 2 || fail "surfaces: $out/surfaces.json"
for id in "${ids[@]}"; do
    read -r x y <<<"$(frame "$id")"
    w=$(frame_width "$id")
    # The top of the title bar, right of the title text, is blue in both
    # the active and the inactive look.
    wait_for pixel_is $((x + w / 2)) $((y + 3)) "$blue"
done

# Succeeds when a point of the mesh of a window group lies more than N
# pixels from its place in the flat grid.
bent() {
    query "id:xp-win-$1" | jq -e --argjson n "$2" '.[0] | .mesh as $m | .size as $s
      | $m != null and ([range(0; $m.points | length) as $i
          | ($m.points[$i][0] - $s[0] * ($i % ($m.cols + 1)) / $m.cols | fabs)
          + ($m.points[$i][1] - $s[1] * (($i / ($m.cols + 1)) | floor) / $m.rows | fabs)]
          | max > $n)' >/dev/null
}
flat() { query "id:xp-win-$1" | jq -e '.[0].mesh == null' >/dev/null; }

step "4. the close button and the start button light up under the pointer"
image_of() { query "id:$1" | jq -r '.[0].image'; }
lit() { test "$(image_of "$1")" != "$2"; }
read -r x y <<<"$(frame "${ids[1]}")"
w=$(frame_width "${ids[1]}")
close=$(image_of "xp-close-${ids[1]}")
start=$(image_of xp-start-winit)
timeout 20 "$bin/chora-probe" point $((x + w - 17)) $((y + 16)) 1500 >>"$out/probe.log" 2>&1 &
job=$!
wait_for lit "xp-close-${ids[1]}" "$close"
grim "$out/hover.png"
wait "$job" || fail "the pointer failed: $out/probe.log"
timeout 20 "$bin/chora-probe" point 50 785 1500 >>"$out/probe.log" 2>&1 &
job=$!
wait_for lit xp-start-winit "$start"
# The close button is back to its look without the pointer.
test "$(image_of "xp-close-${ids[1]}")" = "$close" || fail "the close button stayed lit"
wait "$job" || fail "the pointer failed: $out/probe.log"

step "5. a drag of a title bar moves the window and bends it like jelly"
top=${ids[1]}
read -r x y <<<"$(frame "$top")"
# The button stays down for 1.5 s while the pointer circles.
timeout 20 "$bin/chora-probe" wiggle $((x + 60)) $((y + 15)) $((x + 260)) $((y + 115)) 1500 \
    >>"$out/probe.log" 2>&1 &
wiggle=$!
pids+=("$wiggle")
wait_for bent "$top" 5
grim "$out/wobbly.png"
query "id:xp-win-$top" >"$out/wobbly.json"
wait "$wiggle" || fail "the drag failed: $out/probe.log"
moved() { test "$(frame "$top")" = "$((x + 200)) $((y + 100))"; }
wait_for moved
# The window settles and loses its mesh.
wait_for flat "$top"
read -r x y <<<"$(frame "$top")"
w=$(frame_width "$top")
wait_for pixel_is $((x + w / 2)) $((y + 3)) "$blue"
grim "$out/dragged.png"

step "6. the minimize button hides the window, its task button shows it"
click() { timeout 20 "$bin/chora-probe" drag "$1" "$2" "$1" "$2" >>"$out/probe.log" 2>&1; }
# Clicks button N of the title bar of a frame at X Y with the width W:
# 0 close, 1 maximize, 2 minimize. The 22 pixel buttons sit 6 pixels from
# the right edge, 2 pixels apart, 5 pixels from the top.
title_button() { click $(($2 + $4 - 17 - 24 * $1)) $(($3 + 16)) || fail "the click failed"; }
visible() { test "$(query "id:xp-win-$top" | jq '.[0].visible')" = "$1"; }
title_button 2 "$x" "$y" "$w" &
job=$!
wait_for bent "$top" 50
# The lamp takes 2 s in slow motion; this is about half way.
sleep 0.7
grim "$out/lamp.png"
query "id:xp-win-$top" >"$out/lamp.json"
wait "$job" || fail "the click failed"
wait_for visible false
wait_for flat "$top"
# The task buttons follow the start button, 163 pixels apart.
click $((106 + 163 + 80)) 785 &
job=$!
wait_for bent "$top" 50
sleep 0.7
grim "$out/restore.png"
wait "$job" || fail "the click failed"
wait_for visible true
wait_for flat "$top"

step "7. the start button opens the start menu"
click 50 785 || fail "the click failed"
menu_visible() { test "$(query id:xp-menu | jq '.[0].visible')" = "$1"; }
wait_for menu_visible true
# The menu sits on the taskbar; its left column is white.
menu_y=$(query id:xp-menu | jq -r '.[0].transform.matrix[13] | floor')
wait_for pixel_is 100 $((menu_y + 250)) "$white"
pixel_is 200 $((menu_y + 30)) "$blue" || fail "header: $(pixel 200 $((menu_y + 30)))"
grim "$out/menu.png"
click 50 785 || fail "the click failed"
wait_for menu_visible false

step "8. the Terminal entry of the start menu starts a terminal"
count_is() { test "$(query "kind:surface owner:xp" | jq length)" = "$1"; }
click 50 785 || fail "the click failed"
wait_for menu_visible true
# The first program sits below the 64 pixel header, 36 pixels high.
click 100 $((menu_y + 88)) || fail "the click failed"
wait_for menu_visible false
wait_for count_is 3

step "9. the maximize button fills the area above the taskbar"
top=$(query "kind:surface owner:xp" | jq -r '[.[].id] | sort_by(.[1:] | tonumber) | last')
wait_for visible true
read -r x y <<<"$(frame "$top")"
w=$(frame_width "$top")
title_button 1 "$x" "$y" "$w"
size_is() { test "$(query "id:$top" | jq -c '.[0].surface.size')" = "$1"; }
# 800 pixels less the taskbar and the title bar, both 30 pixels high.
wait_for size_is "[1280,740]"
test "$(frame "$top")" = "0 0" || fail "maximized at $(frame "$top")"
grim "$out/maximized.png"

step "10. the close button closes the window"
title_button 0 0 0 1280
wait_for count_is 2

step "11. the desktop buttons turn the cube to the second desktop and back"
desk_visible() { test "$(query "id:xp-desk-$1" | jq '.[0].visible')" = "$2"; }
# Succeeds while two faces show and one of them turned far enough that
# its matrix divides by the depth along x: about 30 degrees.
turning() {
    query "id:xp-face-$1" | jq -e '.[0] | .visible and
        (.transform.matrix | (.[3] / .[15]) | fabs > 2.5e-4)' >/dev/null &&
        query "id:xp-face-$2" | jq -e '.[0].visible' >/dev/null
}
background='r > 45 && r < 70 && g > 98 && g < 122 && b > 152 && b < 178'
# The four desktop buttons sit left of the 82 pixel tray, 24 pixels apart.
pager() { click $((1280 - 82 - 100 + 24 * ($1 - 1) + 11)) 785 || fail "the click failed"; }
pager 2 &
job=$!
wait_for turning 0 1
# Half way the cube moved back: the corner shows the background, no
# wallpaper.
wait_for pixel_is 5 5 "$background"
grim "$out/cube.png"
query "id:xp-face-0" >"$out/cube.json"
wait "$job" || fail "the click failed"
wait_for desk_visible 1 true
wait_for has_node_count 0 id:xp-face-0
desk_visible 0 false || fail "the first desktop still shows"
grim "$out/desktop2.png"
# The second desktop has no windows; the wallpaper shows at the corner.
pixel_is 5 5 "$background" && fail "the cube stayed: $(pixel 5 5)"
pager 1 &
job=$!
wait_for turning 1 0
grim "$out/cube-back.png"
wait "$job" || fail "the click failed"
wait_for desk_visible 0 true
wait_for has_node_count 0 id:xp-face-1

step "12. Ctrl+Alt+Right turns the cube, and Ctrl+Alt+Left turns it back"
# A virtual keyboard of the host with its keymap: left Ctrl 29, left Alt
# 56, Right 106, Left 105.
keys() { timeout 20 "$bin/chora-probe" keys "$1" >>"$out/probe.log" 2>&1; }
keys 29+56+106 || fail "the keys failed: $out/probe.log"
wait_for desk_visible 1 true
wait_for has_node_count 0 id:xp-face-0
desk_visible 0 false || fail "the first desktop still shows"
keys 29+56+105 || fail "the keys failed: $out/probe.log"
wait_for desk_visible 0 true

step "13. My Documents of the start menu starts the file manager"
# Only its window is checked here. tests/gate-0.8-xp-token.sh checks the
# activation token of its surface.created (gate check 3 of 0.8).
"$bin/choractl" events surface.created >"$out/fm-created.jsonl" 2>/dev/null &
fm_job=$!
sleep 0.5
click 50 785 || fail "the click failed"
wait_for menu_visible true
# The first place sits right of the 189 pixel column, below the header.
click 280 $((menu_y + 85)) || fail "the click failed"
wait_for menu_visible false
wait_for count_is 3
# The event can reach the file after the window shows.
# shellcheck disable=SC2016 # a jq program, $app is a jq variable
created='select(.event == "surface.created" and (.app_id // "" | ascii_downcase) == $app)'
wait_for jq -e --arg app "${file_manager%% *}" "$created" "$out/fm-created.jsonl"
kill "$fm_job" 2>/dev/null || true
app=$(query "kind:surface owner:xp" | jq -r '[.[] | .surface.app_id // ""] | join(" ")')
grep -qiw "${file_manager%% *}" <<<"$app" || fail "no ${file_manager%% *} window: $app"
grim "$out/file-manager.png"

step "14. the exported methods do what the clicks do"
xp() { "$bin/choractl" call xp "$1" "${2:-null}"; }
# The start menu opens and closes without a click.
xp start_menu '{"open": true}' >/dev/null
wait_for menu_visible true
xp start_menu '{"open": false}' >/dev/null
wait_for menu_visible false
# Minimize pours a window into its task button, restore and activate
# pour it back.
win=$(query "kind:surface owner:xp" | jq -r '.[0].id')
poured() { test "$(query "id:xp-win-$win" | jq '.[0].visible')" = "$1"; }
xp minimize "{\"surface\": \"$win\"}" >/dev/null
wait_for poured false
xp restore "{\"surface\": \"$win\"}" >/dev/null
wait_for poured true
xp minimize "{\"surface\": \"$win\"}" >/dev/null
wait_for poured false
xp activate "{\"surface\": \"$win\"}" >/dev/null
wait_for poured true
# The cube turns to a desktop and back.
xp turn_to '{"desktop": 1}' >/dev/null
wait_for desk_visible 1 true
xp turn_to '{"desktop": 0}' >/dev/null
wait_for desk_visible 0 true
# Restart runs the plugin again in the same process: the name xp says
# hello again, and its windows come back.
"$bin/choractl" events client.disconnected client.connected \
    >"$out/restart-events.jsonl" 2>/dev/null &
restart_job=$!
sleep 0.5
xp restart >/dev/null
wait_for jq -e 'select(.event == "client.disconnected" and .name == "xp")' \
    "$out/restart-events.jsonl"
wait_for jq -e 'select(.event == "client.connected" and .name == "xp")' \
    "$out/restart-events.jsonl"
kill "$restart_job" 2>/dev/null || true
wait_for count_is 3
grim "$out/after-restart.png"

# A token given before or with the first commit reaches plugins
# through surface.created.
"$bin/choractl" events surface.created >"$out/created.jsonl" 2>/dev/null &
events_job=$!
sleep 0.5
activated=$(WAYLAND_DISPLAY=$inner timeout 20 "$bin/chora-probe" activated)
sleep 0.5
kill "$events_job" 2>/dev/null || true
token=${activated##* }
test -n "$token" || fail "the probe found no token"
jq -e --arg t "$token" \
    'select(.event == "surface.created" and .token == $t and .pid >= 1)' \
    "$out/created.jsonl" >/dev/null ||
    fail "no surface.created with the token and the pid: $out/created.jsonl"

kill -0 "$chora_pid" || fail "chora died"
pgrep -x chora-xp >/dev/null || fail "chora-xp died: $out/chora.log"

echo "PASS (logs and screenshots in $out)"
