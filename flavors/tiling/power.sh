#!/bin/sh
# The power menu of the tiling flavor: the launcher in menu mode with the
# prompt Power and four words, Lock, Log out, Restart and Shut down
# (German with a German locale), and the action of the one that you
# choose. A word as the argument skips the menu: lock, logout, restart or
# shutdown. Log out ends the session in XDG_SESSION_ID, else all sessions
# of the user.
# Usage: sh power.sh [lock|logout|restart|shutdown]
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*) lock="Sperren" logout="Abmelden" restart="Neustart" shutdown="Herunterfahren" prompt="Ausschalten  " ;;
*) lock="Lock" logout="Log out" restart="Restart" shutdown="Shut down" prompt="Power  " ;;
esac
choice=$1
if [ -z "$choice" ]; then
    if command -v chora-tiling >/dev/null 2>&1; then
        menu="chora-tiling --launch fuzzel"
    else
        menu=fuzzel
    fi
    choice=$(printf '%s\n%s\n%s\n%s\n' "$lock" "$logout" "$restart" "$shutdown" |
        $menu --dmenu --prompt "$prompt" --lines 4 --width 20) || exit 0
fi
case $choice in
"$lock" | lock) exec loginctl lock-session ;;
"$logout" | logout)
    if [ -n "${XDG_SESSION_ID:-}" ]; then
        exec loginctl terminate-session "$XDG_SESSION_ID"
    fi
    exec loginctl terminate-user "${USER:-$(id -un)}"
    ;;
"$restart" | restart) exec systemctl reboot ;;
"$shutdown" | shutdown) exec systemctl poweroff ;;
esac
