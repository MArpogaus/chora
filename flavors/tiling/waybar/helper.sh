#!/bin/sh
# The helper module of the panel: prints the word of the helper door, as
# chora-chat --door says it ("No helper", or the name and the state of a
# helper), as the JSON of a custom module of waybar. The tooltip names
# the key of the chat. German with a German locale.
# Usage: sh helper.sh
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*) tip="Mit einem Helfer chatten (Super+a)" ;;
*) tip="Chat with a helper (Super+a)" ;;
esac
word=$(chora-chat --door 2>/dev/null)
jq -cn --arg text "${word:-$tip}" --arg tooltip "$tip" '{$text, $tooltip}'
