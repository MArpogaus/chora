#!/bin/sh
# The volume module of the panel: prints the volume of the default sound
# output as wpctl of WirePlumber reads it within 2 seconds, "vol 40%" or
# "vol muted", or why there is none: "vol: wpctl not found", "vol: no
# output device" or "vol: no answer". German with a German locale.
# Usage: sh volume.sh
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*)
    muted="vol stumm" missing="vol: wpctl nicht gefunden"
    none="vol: kein Ausgabegerät" hung="vol: keine Antwort"
    ;;
*)
    muted="vol muted" missing="vol: wpctl not found"
    none="vol: no output device" hung="vol: no answer"
    ;;
esac
command -v wpctl >/dev/null 2>&1 || {
    echo "$missing"
    exit 0
}
answer=$(timeout 2 wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null)
status=$?
case $status:$answer in
124:*) echo "$hung" ;;
0:"Volume: "*"[MUTED]"*) echo "$muted" ;;
0:"Volume: "*)
    level=${answer#Volume: }
    awk -v level="${level%% *}" 'BEGIN { printf "vol %d%%\n", level * 100 + 0.5 }'
    ;;
*) echo "$none" ;;
esac
