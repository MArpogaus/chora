#!/bin/bash
# End-to-end test of the XP flavor: chora runs nested in a headless sway
# with chora-xp. The taskbar and the start button show, two foot windows
# get title bars, a drag of a title bar moves its window, the minimize
# button and the task button hide and show a window, the start menu opens
# and starts a terminal, and the buttons of the title bar maximize and
# close a window.
#
# A virtual pointer of the host drives chora, as in tests/nested.sh. Its
# coordinates are those of the 1280x800 output, which shows the scene
# from its origin.
#
# Usage: tests/xp.sh BIN_DIR [OUT_DIR]
#   BIN_DIR  holds chora, choractl, chora-xp and chora-probe
#   OUT_DIR  receives the logs and screenshots (default: a temp dir)
# Needs: sway, foot, grim, jq, pgrep and a Mesa driver.
set -euo pipefail

bin=$(realpath "$1")
out=$(realpath "${2:-$(mktemp -d)}")
theme=$(realpath "$(dirname "$0")/../flavors/xp/theme")
mkdir -p "$out"
# A short path, because a socket path has at most 107 bytes.
XDG_RUNTIME_DIR=$(mktemp -d /tmp/chora-xp-e2e.XXXXXX)
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

step "1. chora -- chora-xp starts nested"
"$bin/chora" -- "$bin/chora-xp" --theme "$theme" >"$out/chora.log" 2>&1 &
chora_pid=$!
pids+=("$chora_pid")
wait_for compgen -G "$XDG_RUNTIME_DIR/chora/*.sock"
CHORA_SOCKET=$(compgen -G "$XDG_RUNTIME_DIR/chora/*.sock")
export CHORA_SOCKET
inner=$(basename "$CHORA_SOCKET" .sock)
query() { "$bin/choractl" query "$@"; }
has_node() { test "$(query "$1" | jq length)" = 1; }
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
# Prints "x y w" of the frame of a surface: the translation of its group,
# and the width of the client plus the borders.
frame() {
    query "id:xp-win-$1" | jq -r '.[0].transform.matrix | "\(.[12] | floor) \(.[13] | floor)"'
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

step "4. a drag of a title bar moves the window"
top=${ids[1]}
read -r x y <<<"$(frame "$top")"
timeout 20 "$bin/chora-probe" drag $((x + 60)) $((y + 15)) $((x + 260)) $((y + 115)) \
    >>"$out/probe.log" 2>&1 || fail "the drag failed: $out/probe.log"
moved() { test "$(frame "$top")" = "$((x + 200)) $((y + 100))"; }
wait_for moved
read -r x y <<<"$(frame "$top")"
w=$(frame_width "$top")
wait_for pixel_is $((x + w / 2)) $((y + 3)) "$blue"
grim "$out/dragged.png"

step "5. the minimize button hides the window, its task button shows it"
click() { timeout 20 "$bin/chora-probe" drag "$1" "$2" "$1" "$2" >>"$out/probe.log" 2>&1; }
# Clicks button N of the title bar of a frame at X Y with the width W:
# 0 close, 1 maximize, 2 minimize. The 22 pixel buttons sit 6 pixels from
# the right edge, 2 pixels apart, 5 pixels from the top.
title_button() { click $(($2 + $4 - 17 - 24 * $1)) $(($3 + 16)) || fail "the click failed"; }
visible() { test "$(query "id:xp-win-$top" | jq '.[0].visible')" = "$1"; }
title_button 2 "$x" "$y" "$w"
wait_for visible false
# The task buttons follow the start button, 163 pixels apart.
click $((106 + 163 + 80)) 785 || fail "the click failed"
wait_for visible true

step "6. the start button opens the start menu"
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

step "7. the Terminal entry of the start menu starts a terminal"
count_is() { test "$(query "kind:surface owner:xp" | jq length)" = "$1"; }
click 50 785 || fail "the click failed"
wait_for menu_visible true
# The first program sits below the 64 pixel header, 36 pixels high.
click 100 $((menu_y + 88)) || fail "the click failed"
wait_for menu_visible false
wait_for count_is 3

step "8. the maximize button fills the area above the taskbar"
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

step "9. the close button closes the window"
title_button 0 0 0 1280
wait_for count_is 2
kill -0 "$chora_pid" || fail "chora died"
pgrep -x chora-xp >/dev/null || fail "chora-xp died: $out/chora.log"

echo "PASS (logs and screenshots in $out)"
