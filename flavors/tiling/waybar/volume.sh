#!/bin/sh
# The volume module of the panel: prints the volume of the default sound
# output as wpctl of WirePlumber reads it, "vol 40%" or "vol muted", and
# "vol none" without wpctl or an output, so that the module always has a
# label. Usage: sh volume.sh
answer=$(wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null) || answer=""
case $answer in
"Volume: "*"[MUTED]"*) echo "vol muted" ;;
"Volume: "*)
    level=${answer#Volume: }
    awk -v level="${level%% *}" 'BEGIN { printf "vol %d%%\n", level * 100 + 0.5 }'
    ;;
*) echo "vol none" ;;
esac
