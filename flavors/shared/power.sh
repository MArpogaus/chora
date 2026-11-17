#!/bin/sh
# The power menu of every look: the launcher in menu mode with the
# prompt Power and five words, Lock, Log out, Restart, Shut down and
# Screenshot (German with a German locale), and the action of the one that
# you choose. The row Screenshot is the way to a screenshot for a person
# with a mouse. A word as the argument skips the menu: lock, logout,
# restart, shutdown or screenshot. Log out ends the session in XDG_SESSION_ID, else all sessions
# of the user. Lock runs the locker and says so in a notice when it does
# not start.
# Usage: sh power.sh [lock|logout|restart|shutdown]
#   CHORA_MENU    the menu command (default fuzzel); it gets --dmenu
#   CHORA_LOCKER  the lock command (default swaylock -f)
#   CHORA_SCREENSHOT  the screenshot command (default screenshot.sh
#                 beside this file)
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*) lock="Sperren" logout="Abmelden" restart="Neustart" shutdown="Herunterfahren" shot="Bildschirmfoto" prompt="Ausschalten  " nolock="Sperren fehlgeschlagen" ;;
*) lock="Lock" logout="Log out" restart="Restart" shutdown="Shut down" shot="Screenshot" prompt="Power  " nolock="Cannot lock the screen" ;;
esac
choice=$1
if [ -z "$choice" ]; then
    choice=$(printf '%s\n%s\n%s\n%s\n%s\n' "$lock" "$logout" "$restart" "$shutdown" "$shot" |
        ${CHORA_MENU:-fuzzel} --dmenu --prompt "$prompt" --lines 5 --width 20) || exit 0
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
"$shot" | screenshot)
    sleep 0.4
    exec ${CHORA_SCREENSHOT:-sh "$(dirname "$0")/screenshot.sh"}
    ;;
esac
