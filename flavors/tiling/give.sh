#!/bin/sh
# Moves the focused window to the desktop of another look: xp, mc or
# construct, as the keys Super+Alt+2 to 4 do; the keyboard stays with
# tiling when that look is on another output (D119, --stay), since the
# person chose the row in a menu of tiling. A refusal shows as a notice
# that says where the window is; the reason of choractl goes to stderr, as
# its words are English and name the tool.
# German with a German locale.
# Usage: sh give.sh xp|mc|construct
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*)
    failed="Fenster nicht verschoben"
    stayed="Das Fenster ist geblieben, wo es war."
    ;;
*)
    failed="Window not moved"
    stayed="The window stayed where it was."
    ;;
esac
case ${1:-} in
xp | mc | construct) ;;
*)
    echo "usage: sh give.sh xp|mc|construct" >&2
    exit 2
    ;;
esac
if ! reason=$(choractl give "$1" --stay 2>&1); then
    command -v notify-send >/dev/null 2>&1 && notify-send -a Desktop "$failed" "$stayed"
    echo "give.sh: $reason" >&2
    exit 1
fi
