#!/bin/bash
# Gate of the handover between window managers (D55). A headless session
# gets three outputs: xp on the first, tiling on the second, mc on the
# third, each with `--output`. A foot window moves between them:
#
# 1. a drag of its title bar from xp to the tiling output: tiling owns it;
# 2. `window.give` of tiling to xp: xp owns it;
# 3. `choractl give mc`: mc owns it;
# 4. `window.give` of mc to tiling, then a drag with Super and the left
#    button from tiling to the xp output: xp owns it;
# 5. a give to a client that does not exist is refused, and the window
#    stays with xp;
# 6. tiling ends right after a give to it: the window is not lost.
#
# After each step every surface is attached, visible and owned, and the
# window that moved has the keyboard focus. Captures go to SHOTS as
# handover-*.png.
#
# Usage: tests/gate-handover.sh   BIN: directory with the binaries
#   SHOTS: directory for the captures (default: a temp dir)
set -u
. "$(dirname "$0")/gate-lib.sh"
need jq
need foot
need_bins chora choractl chora-tiling chora-xp chora-mc
temp_config
shots=${SHOTS:-$XDG_CONFIG_HOME}
mkdir -p "$shots"
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
theme=$repo/flavors/xp/theme

session_start 1280x800
grant input.inject screen.capture methods.call
ctl add-output 1280x800+1280+0 >/dev/null || fail "add-output"
ctl add-output 1280x800+2560+0 >/dev/null || fail "add-output"
mapfile -t outs < <(ctl outputs | jq -r 'sort_by(.position[0]) | .[].name')
[ "${#outs[@]}" = 3 ] || fail "three outputs: ${outs[*]}"
xp_out=${outs[0]} tiling_out=${outs[1]} mc_out=${outs[2]}

# plugin NAME ARGS: a plugin in the background, its pid in pid_NAME.
plugin() {
    local name=$1
    shift
    "$bin/chora-$name" "$@" >"$shots/$name.log" 2>&1 &
    pids+=($!)
    printf -v "pid_$name" %s $!
}
# Tiling first: it grabs Super+BTN_LEFT for its drag before the cube of xp.
plugin tiling --output "$tiling_out"
plugin xp --theme "$theme" --output "$xp_out"
plugin mc --output "$mc_out"
for c in tiling xp mc; do
    ctl wait "owner:$c" --timeout 20 >/dev/null || fail "$c does not start: $(tail -n 3 "$shots/$c.log")"
done

# The pointer on the xp output, so that the new window goes to xp.
inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
inject "[{\"pointer_move\": {\"position\": [640, 300], \"output\": \"$xp_out\"}}]"
foot -f "DejaVu Sans Mono:size=10" sh -c 'echo handover; sleep 600' >"$shots/foot.log" 2>&1 &
run_pid $!
w=$(ctl wait "app_id:foot kind:surface owner:xp" --timeout 20 | jq -r '.[0].id') ||
    fail "foot does not show in xp"

# settled OWNER: within 20 s every surface is attached, visible and owned,
# and $w belongs to OWNER and has the focus.
settled() {
    local all
    for _ in $(seq 100); do
        all=$(ctl query kind:surface)
        echo "$all" | jq -e --arg w "$w" --arg o "$1" '
            all(.[]; .owner != null and .parent != null and ((.rects // {}) | length) > 0)
            and any(.[]; .id == $w and .owner == $o and .focused == true)' >/dev/null &&
            return 0
        sleep 0.2
    done
    fail "$w is not settled with $1: $(echo "$all" | jq -c 'map({id, owner, parent, focused, rects})')"
}
shot() {
    local o
    for o in "${outs[@]}"; do
        ctl capture "$shots/handover-$1-$o.png" --output "$o" >/dev/null || fail "capture $1 $o"
    done
}
settled xp
shot 0-start
ok "foot $w starts in xp"

# 1. The title bar to the middle of the tiling output.
inject "[{\"pointer_move\": {\"node\": \"xp-title-$w\", \"point\": [60, 10]}},
  {\"button\": {\"state\": \"pressed\", \"hold\": true}},
  {\"pointer_move\": {\"position\": [300, 300], \"output\": \"$xp_out\"}},
  {\"pointer_move\": {\"position\": [640, 400], \"output\": \"$tiling_out\"}},
  {\"button\": {\"state\": \"released\"}}]"
settled tiling
shot 1-drag
ok "1. a drag from xp to the tiling output: tiling owns $w"

# 2. window.give of tiling, for the focused window.
ctl call tiling window.give '{"to": "xp"}' >/dev/null || fail "tiling window.give"
settled xp
shot 2-give
ok "2. window.give of tiling to xp: xp owns $w"

# 3. choractl give finds the owner of the focused window.
ctl give mc >/dev/null || fail "choractl give mc"
settled mc
shot 3-mc
ok "3. choractl give mc: mc owns $w"

# 4. mc gives it to tiling; a drag with Super to the xp output.
ctl call mc window.give "{\"surface\": \"$w\", \"to\": \"tiling\"}" >/dev/null ||
    fail "mc window.give"
settled tiling
inject "[{\"pointer_move\": {\"node\": \"$w\", \"point\": [100, 100]}},
  {\"key\": {\"key\": \"Super_L\", \"state\": \"pressed\"}},
  {\"button\": {\"state\": \"pressed\", \"hold\": true}},
  {\"pointer_move\": {\"position\": [640, 400], \"output\": \"$xp_out\"}},
  {\"button\": {\"state\": \"released\"}},
  {\"key\": {\"key\": \"Super_L\", \"state\": \"released\"}}]"
settled xp
shot 4-super-drag
ok "4. mc to tiling by window.give, tiling to xp by a Super drag: xp owns $w"

# 5. No such client: refused, and the window stays.
if ctl call xp window.give "{\"surface\": \"$w\", \"to\": \"nobody\"}" >/dev/null 2>&1; then
    fail "a give to nobody was accepted"
fi
settled xp
ok "5. a give to a client that does not exist is refused; xp keeps $w"

# 6. The target ends right after the give. With resume its grace time of
# 10 s keeps the window, then another window manager takes it.
ctl call xp window.give "{\"surface\": \"$w\", \"to\": \"tiling\"}" >/dev/null ||
    fail "xp window.give to tiling"
kill "$pid_tiling"
for _ in $(seq 100); do
    owner=$(ctl query "id:$w" | jq -r '.[0].owner')
    [ "$owner" = xp ] || [ "$owner" = mc ] && break
    sleep 0.2
done
[ "$owner" = xp ] || [ "$owner" = mc ] || fail "$w is with ${owner:-nobody} 20 s after tiling ended"
settled "$owner"
shot 6-killed
ok "6. tiling ended after the give: $owner adopted $w"
echo "ok: gate-handover; captures in $shots"
