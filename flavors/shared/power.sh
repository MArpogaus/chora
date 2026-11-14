#!/bin/sh
# The power menu of every look: the launcher in menu mode with the
# prompt Power and four words, Lock, Log out, Restart and Shut down
# (German with a German locale), and the action of the one that you
# choose. A word as the argument skips the menu: lock, logout, restart or
# shutdown. Log out ends the session in XDG_SESSION_ID, else all sessions
# of the user. Lock runs the locker and says so in a notice when it does
# not start.
# Usage: sh power.sh [lock|logout|restart|shutdown]
#   CHORA_MENU    the menu command (default fuzzel); it gets --dmenu
#   CHORA_LOCKER  the lock command (default swaylock -f)
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*) lock="Sperren" logout="Abmelden" restart="Neustart" shutdown="Herunterfahren" prompt="Ausschalten  " nolock="Sperren fehlgeschlagen" ;;
*) lock="Lock" logout="Log out" restart="Restart" shutdown="Shut down" prompt="Power  " nolock="Cannot lock the screen" ;;
esac
choice=$1
if [ -z "$choice" ]; then
    choice=$(printf '%s\n%s\n%s\n%s\n' "$lock" "$logout" "$restart" "$shutdown" |
        ${CHORA_MENU:-fuzzel} --dmenu --prompt "$prompt" --lines 4 --width 20) || exit 0
fi
case $choice in
"$lock" | lock)
    locker=${CHORA_LOCKER:-swaylock -f}
    $locker && exit 0
    echo "power.sh: $locker did not lock the screen" >&2
    command -v notify-send >/dev/null 2>&1 && notify-send -a Power -u critical "$nolock" "$locker"
    exit 1
    ;;
"$logout" | logout)
    if [ -n "${XDG_SESSION_ID:-}" ]; then
        exec loginctl terminate-session "$XDG_SESSION_ID"
    fi
    exec loginctl terminate-user "${USER:-$(id -un)}"
    ;;
"$restart" | restart) exec systemctl reboot ;;
"$shutdown" | shutdown) exec systemctl poweroff ;;
esac
