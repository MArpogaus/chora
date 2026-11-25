#!/bin/bash
# The bar and the windows of tiling speak one colour language. In each
# theme, at the scale 1 and 2, a capture is measured: the current tag is the
# accent, the hairline of both islands is the line colour, as the border of
# the other window is, and the border of the focused window starts in the
# accent at its top left and ends in accent-2 at its bottom right.
#
# Usage: tests/gate-tiling-bar-palette.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need waybar
need jq
need_bins chora choractl chora-tiling chora-probe
flavor=$repo/flavors/tiling

near() {
    awk -v a="$1" -v b="$2" -v t="$3" 'BEGIN {
        split(a, x, " "); split(b, y, " ")
        for (i = 1; i <= 3; i++) { d = x[i] - y[i]; if (d < 0) d = -d; if (d > t) exit 1 }
    }'
}
palette() {
    case $1 in
    ink) echo "167 139 250 94 234 212 46 51 68" ;;
    paper) echo "109 74 255 14 159 142 213 216 225" ;;
    esac
}

window() {
    local started
    started=$(ctl run "$bin/chora-probe" window "$@") || fail "the run of the window"
    run_pid "$(sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p' <<<"$started")"
}

round() {
    local theme=$1 size=$2 scale=1
    [[ $size == *@* ]] && scale=${size##*@}
    local tag=$theme-$size
    read -r ar ag ab br bg bb lr lg lb <<<"$(palette "$theme")"
    accent="$ar $ag $ab"
    accent_2="$br $bg $bb"
    line="$lr $lg $lb"
    "$bin/chora-tiling" --print-config >"$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
    sed -i "s/^  theme: system/  theme: $theme/" "$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
    env_file=$XDG_CONFIG_HOME/$tag.env session_start "$size" -- "$bin/chora-tiling" --no-companions
    grant screen.capture
    PATH=$bin:$PATH CHORA_FLAVOR_DIR=$flavor setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >"$XDG_CONFIG_HOME/wb-$tag.log" 2>&1 &
    run_pid $!
    window 255 255 255
    window 0 0 0
    shot=$XDG_CONFIG_HOME/$tag.png
    px() { pixel "$shot" $(($1 * scale)) $(($2 * scale)); }
    for _ in $(seq 40); do
        sleep 1
        ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || continue
        [ "$(px 640 24)" != "$(px 640 300)" ] && near "$(px 130 50)" "$accent" 12 &&
            near "$(px 30 52)" "$accent" 14 && near "$(px 616 786)" "$accent_2" 14 && break
    done
    pill=$(px 173 16)
    near "$pill" "$accent" 2 || fail "$tag: the current tag is $pill, not the accent $accent"
    left_edge=$(px 30 8)
    near "$left_edge" "$line" 2 || fail "$tag: the left island edge is $left_edge, not the line $line"
    right_edge=$(px 1250 8)
    near "$right_edge" "$line" 2 || fail "$tag: the right island edge is $right_edge, not the line $line"
    focus_start=$(px 30 52)
    near "$focus_start" "$accent" 14 || fail "$tag: the focus border starts in $focus_start, not the accent $accent"
    focus_end=$(px 616 786)
    near "$focus_end" "$accent_2" 14 || fail "$tag: the focus border ends in $focus_end, not accent-2 $accent_2"
    unfocused=$(px 660 52)
    near "$unfocused" "$line" 8 || fail "$tag: the border of the other window is $unfocused, not the line $line"
    ok "$tag: the tag and the focus border share accent and accent-2, the island hairlines and the other border are the line"
    kill "${pids[@]}" "${run_pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
    run_pids=()
}

temp_config
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
mkdir -p "$XDG_CONFIG_HOME/chora"
for theme in ink paper; do
    for size in 1280x800 2560x1600@2; do
        round "$theme" "$size"
    done
done
