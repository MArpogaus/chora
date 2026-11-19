#!/bin/sh
# The helper module of the panel: prints the word of the helper door as
# chora-chat --door reads it from the presenter chora-assistant ("No
# helper", or the name and the state of a helper), as the JSON of a
# custom module of waybar. The word is the one
# door of the chat in the tiling flavor, and it has no tooltip. German with
# a German locale.
# Usage: sh helper.sh
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*) none="Kein Helfer" ;;
*) none="No helper" ;;
esac
word=$(chora-chat --door 2>/dev/null)
jq -cn --arg text "${word:-$none}" '{$text}'
