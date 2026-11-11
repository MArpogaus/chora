#!/bin/sh
# Starts waybar with the configuration of the tiling flavor on the outputs
# of chora-tiling only: an output of another window manager, such as
# chora-xp beside tiling, gets no panel. The outputs come from
# `choractl call tiling get_tags`; every other output is left out. Every
# 2 seconds the script reads the outputs again, and when the left-out
# ones changed it starts waybar again with them. It ends when waybar
# ends, and passes SIGUSR2, the reload of the unit, on to waybar.
# Without an answer of tiling within 10 seconds, the panel shows on every
# output, with a line on stderr. jq is needed.
# Usage: sh waybar.sh DIR   DIR: the waybar folder of the flavor
command -v jq >/dev/null 2>&1 || {
    echo "waybar.sh: jq is not installed; the panel of the tiling flavor needs it" >&2
    exit 1
}
dir=$1
config=${XDG_RUNTIME_DIR:-/tmp}/chora-tiling-waybar.json

tiling_outputs() {
    choractl call tiling get_tags 2>/dev/null | jq -c '[.outputs[].name]' 2>/dev/null
}

panel_outputs() {
    choractl outputs 2>/dev/null | jq -c --argjson tiling "$1" \
        '[.[].name | select(. as $n | $tiling | index($n) | not) | "!" + .] + ["*"]'
}

tiling=""
for _ in 1 2 3 4 5 6 7 8 9 10; do
    tiling=$(tiling_outputs) && [ -n "$tiling" ] && break
    sleep 1
done
if [ -z "$tiling" ]; then
    echo "waybar.sh: chora-tiling did not name its outputs; the panel shows on every output" >&2
fi

pid=""
trap '[ -z "$pid" ] || kill "$pid" 2>/dev/null; exit 0' TERM INT
trap '[ -z "$pid" ] || kill -USR2 "$pid" 2>/dev/null' USR2
while :; do
    if [ -n "$tiling" ]; then
        outputs=$(panel_outputs "$tiling")
    else
        outputs='["*"]'
    fi
    jq -n --arg include "$dir/config.jsonc" --argjson outputs "${outputs:-[\"*\"]}" \
        '{include: [$include], output: $outputs}' >"$config"
    waybar --config "$config" &
    pid=$!
    while kill -0 "$pid" 2>/dev/null; do
        sleep 2
        now=$(tiling_outputs) || now=""
        [ -n "$now" ] || continue
        next=$(panel_outputs "$now")
        if [ -n "$next" ] && [ "$next" != "$outputs" ]; then
            tiling=$now
            kill "$pid" 2>/dev/null
            wait "$pid"
            continue 2
        fi
    done
    wait "$pid"
    exit $?
done
