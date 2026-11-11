#!/bin/sh
# The volume module of the panel: prints the volume of the default sound
# output, as wpctl of WirePlumber reads it within 2 seconds, as the JSON
# of a custom module of waybar: "Volume 40%" or "Muted", or "Sound
# unavailable" with the reason in the tooltip: "wpctl not found", "No
# output device" or "No answer". German with a German locale.
# Usage: sh volume.sh
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*)
    volume="Lautstärke" muted="Stumm" unavailable="Ton nicht verfügbar"
    missing="wpctl nicht gefunden" none="Kein Ausgabegerät" hung="Keine Antwort"
    ;;
*)
    volume="Volume" muted="Muted" unavailable="Sound unavailable"
    missing="wpctl not found" none="No output device" hung="No answer"
    ;;
esac
say() { printf '{"text": "%s", "tooltip": "%s"}\n' "$1" "$2"; }
command -v wpctl >/dev/null 2>&1 || {
    say "$unavailable" "$missing"
    exit 0
}
answer=$(timeout 2 wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null)
status=$?
case $status:$answer in
124:*) say "$unavailable" "$hung" ;;
0:"Volume: "*"[MUTED]"*) say "$muted" "$muted" ;;
0:"Volume: "*)
    level=${answer#Volume: }
    shown=$(awk -v level="${level%% *}" -v word="$volume" 'BEGIN { printf "%s %d%%", word, level * 100 + 0.5 }')
    say "$shown" "$shown"
    ;;
*) say "$unavailable" "$none" ;;
esac
