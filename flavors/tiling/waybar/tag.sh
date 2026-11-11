#!/bin/sh
# One tag of the panel: prints the JSON of a custom module of waybar for
# tag N. The nine tag modules ask at once, so the answer of
# `tiling.get_tags` is kept in a file for the current second and read by
# all of them. A tag that tiling does not know, or no answer, prints
# nothing, and the module hides. jq is needed.
# Usage: sh tag.sh N
dir=$(dirname "$0")
file=${XDG_RUNTIME_DIR:-/tmp}/chora-tiling-tags.json
[ "$(stat -c %Y "$file" 2>/dev/null)" = "$(date +%s)" ] || {
    choractl call tiling get_tags >"$file.$$" 2>/dev/null && mv "$file.$$" "$file"
    rm -f "$file.$$"
}
jq -c --argjson n "$1" -f "$dir/tags.jq" "$file" 2>/dev/null
