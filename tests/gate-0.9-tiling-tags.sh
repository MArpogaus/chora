#!/bin/bash
# `tiling.get_tags` answers the tags of each output, and the jq filter of
# the tags module of waybar turns the answer into the text of the module:
# a window on tag 1 makes it focused and occupied, a move to tag 3 makes
# tag 3 occupied.
#
# Usage: tests/gate-0.9-tiling-tags.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need jq
need_bins chora choractl chora-tiling chora-probe
temp_config
session_start 640x480,640x480+640+0 -- "$bin/chora-tiling"
tags() {
    ctl call tiling get_tags | WAYBAR_OUTPUT_NAME=$1 \
        jq -c -f "$repo/flavors/tiling/waybar/tags.jq"
}

pid=$(ctl run "$bin/chora-probe" window 255 0 0 |
    sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p')
[ -n "$pid" ] || fail "the run of the window"
run_pid "$pid"
for _ in $(seq 50); do
    surface=$(ctl query 'app_id:chora-probe kind:surface' |
        sed -n 's/^ *"id": *"\([^"]*\)".*/\1/p' | head -n 1)
    [ -n "$surface" ] && break
    sleep 0.2
done
[ -n "$surface" ] || fail "no window"
answer=$(ctl call tiling get_tags)
[ "$(echo "$answer" | jq '.outputs | length')" = 2 ] || fail "not two outputs: $answer"
first=$(echo "$answer" | jq -r '.outputs[] | select(.focused) | .name')
echo "$answer" | jq -e --arg o "$first" '.outputs[] | select(.name == $o) | .tags[0]
    | .shown and .focused and .occupied and (.urgent | not)' >/dev/null ||
    fail "tag 1 of $first: $answer"
[ "$(tags "$first")" = '{"text":"<b><u>1</u></b>"}' ] || fail "the module: $(tags "$first")"
ok "a window makes tag 1 of $first focused and occupied"

ctl call tiling move_to_tag "{\"surface\": \"$surface\", \"tag\": 3}" >/dev/null ||
    fail "move_to_tag"
[ "$(tags "$first")" = '{"text":"<b><u>1</u></b> 3"}' ] || fail "the module: $(tags "$first")"
ok "the window on tag 3 makes it occupied"
