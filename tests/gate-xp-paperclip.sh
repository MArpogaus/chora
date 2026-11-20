#!/bin/bash
# The paperclip of xp (xp-23): on a fresh xp session, with the shipped
# settings of chora-assistant and with no voice agent, the paperclip shows
# at its place of xp.md: its box 8 px from the right edge of the output
# and 8 px above the taskbar, its art drawn, and the tray door beside the
# clock. A second cell runs the same with a voice agent that listens.
# BIN: directory with the binaries, SHOTS: directory for the capture.
set -u
. "$(dirname "$0")/gate-lib.sh"
need jq
need_bins chora choractl chora-xp chora-assistant
temp_config
export HOME=$XDG_CONFIG_HOME/home
mkdir -p "$HOME"
shots=${SHOTS:-$(mktemp -d /tmp/gate-xp-paperclip.XXXXXX)}
mkdir -p "$shots"
session_start 1280x800 -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
"$bin/chora-assistant" >"$shots/assistant.log" 2>&1 &
pids+=($!)
grant screen.capture
rect() { ctl query id:hud-figure | jq -r '.[0].rects | to_entries[0].value | map(floor) | join(" ")'; }
has_figure() { [ "$(ctl query id:hud-figure | jq length)" = 1 ]; }
near() { test $(($1 - $2)) -le 3 && test $(($2 - $1)) -le 3; }
at_its_place() {
    local x y w h
    read -r x y w h <<<"$(rect)"
    near $((x + w)) 1272 || fail "$1: the paperclip is not 8 px from the right edge: $x $w"
    near $((y + h)) 762 || fail "$1: the paperclip is not 8 px above the taskbar: $y $h"
    [ "$(ctl query id:hud-figure | jq '.[0].visible')" = true ] || fail "$1: the paperclip node is hidden"
    ok "$1: the paperclip shows at its place: $x,$y ${w}x$h"
}
drawn() {
    local art
    art=$(ctl query id:hud-figure | jq -r '.[0].image')
    python3 -I - "$art" "$1" "$2" "$3" <<'PY' || fail "$4: the paperclip art is not on screen"
import sys
from PIL import Image
art, shot, x, y = Image.open(sys.argv[1]).convert("RGBA"), Image.open(sys.argv[2]).convert("RGB"), int(sys.argv[3]), int(sys.argv[4])
opaque = [(i, j) for j in range(art.height) for i in range(art.width) if art.getpixel((i, j))[3] == 255]
assert len(opaque) > art.width * art.height // 10, f"the art is blank: {len(opaque)} opaque pixels"
same = sum(1 for i, j in opaque if sum(abs(a - b) for a, b in zip(art.getpixel((i, j))[:3], shot.getpixel((x + i, y + j)))) <= 12)
assert same * 2 >= len(opaque), f"only {same} of {len(opaque)} art pixels show at {x},{y}"
PY
}
wait_figure() {
    for _ in $(seq 50); do
        has_figure && return 0
        sleep 0.2
    done
    return 1
}
wait_figure || fail "no helper: a fresh xp session shows no paperclip: $(tail -n 3 "$shots/assistant.log")"
at_its_place "no helper"
sleep 1
ctl capture "$shots/xp-paperclip-no-helper.png" >/dev/null || fail "no helper: the capture failed"
read -r x y _ <<<"$(rect)"
drawn "$shots/xp-paperclip-no-helper.png" "$x" "$y" "no helper"
ok "no helper: the paperclip art is drawn"
mkfifo "$shots/voice.in"
python3 -I "$repo/tests/fake-voice.py" "$shots/claims" <"$shots/voice.in" >"$shots/voice.log" 2>&1 &
pids+=($!)
exec 5>"$shots/voice.in"
echo 'voice.state {"state": "listening"}' >&5
sleep 1.5
wait_figure || fail "a helper: a fresh xp session shows no paperclip"
at_its_place "a voice agent"
sleep 1
ctl capture "$shots/xp-paperclip-voice.png" >/dev/null || fail "a voice agent: the capture failed"
read -r x y _ <<<"$(rect)"
drawn "$shots/xp-paperclip-voice.png" "$x" "$y" "a voice agent"
ok "a voice agent: the paperclip art is drawn"
