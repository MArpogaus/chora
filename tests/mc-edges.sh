#!/bin/bash
# shellcheck disable=SC2154,SC2016,SC2013 # variables of the sourcing gate; until_ok evaluates its first argument; the ids have no blanks
# Sourced by the gates of chora-mc: the cell that no line of any built-in
# scene is cut at the edge of its page (rpx1-12, rmm1-1). Every panel that a
# helper can fill gets twelve long lines, the scene shows, and the last two
# rows of its page hold no ink (a page edge falls between two lines).
# Needs ctl, until_ok, fail, shot, node_rect, current, $work and $repo of the
# gate that sources it, and the output size in $1 (width and height).
page_edges() {
    local scene id page cut lines
    lines=$(jq -nc '[range(12) | "Line \(. + 1): the water boils, the pasta goes in, the sauce waits and the parsley is cut fine for the top."]')
    for scene in agenda morning mission; do
        for id in $(sed -n 's/^.*id: \([a-z0-9-]*\).*$/\1/p' "$repo/plugins/chora-mc/scenes/$scene.yaml"); do
            ctl call mc panel.set "{\"scene\": \"$scene\", \"panel\": \"$id\", \"lines\": $lines}" >/dev/null || fail "panel.set $scene $id"
        done
    done
    for n in $(seq 30); do
        ctl call mc scene.save "{\"name\": \"edge-user-$n\", \"grid\": [1, 1], \"panels\": [{\"id\": \"p\", \"cell\": [0, 0, 1, 1]}]}" >/dev/null || fail "scene.save edge-user-$n"
    done
    for n in 1 2 3 4; do
        ctl call mc agent.set "{\"name\": \"edge-$n\", \"state\": \"working\", \"task\": \"Cut the parsley fine for the top of the pasta and the sauce\", \"said\": \"The water boils, the pasta goes in, the sauce waits and the parsley is cut fine.\", \"plan\": $(jq -c '.[:8]' <<<"$lines")}" >/dev/null || fail "agent.set edge-$n"
    done
    for scene in agenda morning agents mission overview; do
        ctl call mc scene.show "{\"name\": \"$scene\"}" >/dev/null || fail "scene.show $scene"
        until_ok '[ "$(current)" = '"$scene"' ]' 20 || fail "no page $scene: $(current)"
        until_ok '[ -n "$(node_rect mc-s-'"$scene"'-page)" ]' 20 || fail "no page node of $scene"
        page=$(node_rect "mc-s-$scene-page")
        shot "page-edges-$scene"
        ctl capture "$work/edge-$scene.png" >/dev/null || fail "capture $scene"
        # shellcheck disable=SC2046 # four words, one for each number of the rect
        cut=$(python3 -I "$repo/tests/png-ink.py" "$work/edge-$scene.png" \
            $(jq -r '"\(.[0] | floor) \(.[1] + .[3] - 2 | floor) \(.[2] | floor) 2"' <<<"$page"))
        [ -z "$cut" ] || fail "a line of the page $scene is cut at the edge of the page $page at $1: ink $cut in its last two rows (rmm1-1)"
    done
}
