#!/bin/bash
# Shadows and motion of tiling (tiling-v2, items 8 and 9): a window has a
# shadow on the wallpaper; a layout change slides the windows in 200 ms
# and a tag switch brings them in from the side, each ending at its place
# with the opacity 1; look.animations false makes both instant; an idle
# scene draws no frame.
#
# Usage: tests/gate-tiling-motion.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need swaybg
need_bins chora choractl chora-tiling chora-probe
temp_config
session_start 1280x800 -- "$bin/chora-tiling" --no-companions
grant screen.capture
shot=$XDG_CONFIG_HOME/motion.png
start() {
    started=$(ctl run "$@") || fail "the run of $1"
    pid=$(sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p' <<<"$started")
    [ -n "$pid" ] || fail "the run of $1"
    run_pid "$pid"
}
start swaybg -c '#808080' -m solid_color
start "$bin/chora-probe" window 40 44 60
start "$bin/chora-probe" window 70 70 90
for _ in $(seq 60); do
    seen=$(ctl query 'app_id:chora-probe kind:surface') || { sleep 0.5; continue; }
    [ "$(grep -c '"owner": *"tiling"' <<<"$seen")" = 2 ] && break
    sleep 0.5
done
sleep 3
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
grey=$(pixel "$shot" 1279 400 | cut -d" " -f2)
for spot in "10 400 6" "1270 400 6"; do
    read -r x y least <<<"$spot"
    got=$(pixel "$shot" "$x" "$y" | cut -d' ' -f2)
    [ $((grey - got)) -ge "$least" ] || fail "no shadow at $x,$y: $got against the wallpaper $grey"
done
ok "the focused window has a deep shadow, the other a light one"

groups() {
    local all
    all=$(ctl query 'kind:group') || return 1
    python3 -I -c '
import json, sys
for n in json.load(sys.stdin):
    if n["id"].startswith("tw-"):
        print(n["id"], round(n["transform"]["matrix"][12]), round(n["opacity"], 3), int(n["visible"]))
' <<<"$all" | sort
}
shown=$(groups) || fail "the query of the groups"
[ "$(awk '$3 != 1' <<<"$shown" | wc -l)" = 0 ] || fail "a window stays transparent: $shown"
ok "every window is opaque once it stands"

settled=$(groups) || fail "the query of the groups"
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
cp "$shot" "$XDG_CONFIG_HOME/settled.png"
"$bin/choractl" grant choractl methods.call >/dev/null || fail "the grant"
slide=$XDG_CONFIG_HOME/slide
mkdir -m 700 "$slide"
printf '{"frame": 1, "method": "methods.call", "params": {"client": "tiling", "method": "set_layout", "params": {"layout": "tile_left"}}}\n' >"$slide/script"
timeout 60 "$bin/choractl" record --frames 24 --out "$slide" --script "$slide/script" >/dev/null || fail "the recording"
for _ in $(seq 60); do
    end=$(groups) && [ "$end" != "$settled" ] && [ "$end" = "$(groups)" ] && break
    sleep 0.25
done
[ "$end" != "$settled" ] || fail "the windows did not move"
[ "$(awk '$3 != 1' <<<"$end" | wc -l)" = 0 ] || fail "a window is transparent after its move: $end"
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
gap=640
between=0
for n in $(seq 2 12); do
    got=$(pixel "$slide/$(printf 'frame-%05d.png' "$n")" "$gap" 400)
    [ "$got" != "$(pixel "$XDG_CONFIG_HOME/settled.png" "$gap" 400)" ] && [ "$got" != "$(pixel "$shot" "$gap" 400)" ] && between=$((between + 1))
done
[ "$between" -gt 0 ] || fail "the slide is not between the places: no frame of the 200 ms shows a window in the gap at $gap,400"
ok "a layout change slides the windows and they end at their places"

ctl call tiling view_tag '{"tag": 2}' >/dev/null || fail "view_tag 2"
for _ in $(seq 60); do
    left=$(groups) && [ -n "$left" ] && [ "$(awk '$4 != 0' <<<"$left" | wc -l)" = 0 ] && break
    sleep 0.25
done
[ "$(awk '$4 != 0' <<<"$left" | wc -l)" = 0 ] || fail "the windows of tag 1 stay shown on tag 2: $left"
frames=$XDG_CONFIG_HOME/frames
mkdir -m 700 "$frames"
printf '{"frame": 1, "method": "methods.call", "params": {"client": "tiling", "method": "view_tag", "params": {"tag": 1}}}\n' >"$frames/script"
timeout 60 "$bin/choractl" record --frames 24 --out "$frames" --script "$frames/script" >/dev/null || fail "the recording"
for _ in $(seq 60); do
    there=$(groups) && [ "$(awk '$4 != 1' <<<"$there" | wc -l)" = 0 ] && break
    sleep 0.25
done
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
for x in 330 960; do
    [ "$(pixel "$frames/frame-00024.png" "$x" 400)" = "$(pixel "$shot" "$x" 400)" ] || fail "the last frame does not show the window at $x,400 as it stands"
    [ "$(pixel "$frames/frame-00002.png" "$x" 400)" != "$(pixel "$shot" "$x" 400)" ] || fail "a tag switch does not bring the windows in from the side: the window at $x,400 stands there in the second frame"
done
ok "a tag switch brings the windows in from the side"

mkdir -p "$XDG_CONFIG_HOME/chora"
"$bin/chora-tiling" --print-config | sed 's/^  animations: true/  animations: false/' >"$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
ctl call tiling settings.reload >/dev/null || fail "settings.reload"
sleep 1
settled=$(groups) || fail "the query of the groups"
ctl call tiling set_layout '{"layout": "tile"}' >/dev/null || fail "set_layout"
now=$(groups) || fail "the query of the groups"
sleep 1
again=$(groups) || fail "the query of the groups"
[ "$now" = "$again" ] || fail "with animations false the windows still moved over time"
[ "$now" != "$settled" ] || fail "the layout did not change"
ok "animations false makes the change at once"

frames() {
    local stats
    stats=$(ctl render-stats) || return 1
    python3 -I -c 'import json, sys; print(json.load(sys.stdin)["outputs"]["HEADLESS-1"]["damaged_frames"])' <<<"$stats"
}
before=$(frames) || fail "render-stats"
sleep 5
after=$(frames) || fail "render-stats"
[ "$before" = "$after" ] || fail "an idle scene drew $((after - before)) frames in 5 s"
ok "an idle scene draws no frame"
