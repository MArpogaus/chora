#!/bin/sh
# Starts waybar with the configuration of the tiling flavor on the outputs
# of chora-tiling only: an output of another window manager, such as
# chora-xp beside tiling, gets no panel. The outputs come from
# `choractl call tiling get_tags`; every other output at the start is left
# out, and an output that comes later gets a panel. Without an answer of
# tiling within 10 seconds, the panel shows on every output.
# Usage: sh waybar.sh DIR   DIR: the waybar folder of the flavor
dir=$1
tiling=""
for _ in 1 2 3 4 5 6 7 8 9 10; do
    tiling=$(choractl call tiling get_tags 2>/dev/null | jq -c '[.outputs[].name]') && [ -n "$tiling" ] && break
    sleep 1
done
outputs=$(choractl outputs 2>/dev/null | jq -c --argjson tiling "${tiling:-null}" '
    if $tiling == null then ["*"]
    else [.[].name | select(. as $n | $tiling | index($n) | not) | "!" + .] + ["*"] end')
config=${XDG_RUNTIME_DIR:-/tmp}/chora-tiling-waybar.json
jq -n --arg include "$dir/config.jsonc" --argjson outputs "${outputs:-[\"*\"]}" \
    '{include: [$include], output: $outputs}' >"$config"
exec waybar --config "$config"
