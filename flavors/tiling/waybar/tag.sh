#!/bin/sh
# One tag of the panel: prints the JSON of a custom module of waybar for
# tag N from the answer of `tiling.get_tags`. A tag that tiling does not
# know, or no answer, prints nothing, and the module hides. jq is needed.
# Usage: sh tag.sh N
dir=$(dirname "$0")
choractl call tiling get_tags 2>/dev/null | jq -c --argjson n "$1" -f "$dir/tags.jq" 2>/dev/null
