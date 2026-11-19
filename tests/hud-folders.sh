#!/bin/bash
# Prints the HUD folders that the source tree ships, one `NAME DIR` per
# line: the neutral folder of the assistant and the folder of each look in
# the directory of that look.
#
# Usage: tests/hud-folders.sh
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
echo "neutral $repo/plugins/chora-assistant/huds/neutral"
for dir in "$repo"/plugins/chora-*/hud "$repo"/flavors/*/assets/assistant; do
    [ -f "$dir/hud.yaml" ] || continue
    case $dir in
    "$repo"/plugins/*) name=${dir#"$repo"/plugins/chora-} ;;
    *) name=${dir#"$repo"/flavors/} ;;
    esac
    echo "${name%%/*} $dir"
done
