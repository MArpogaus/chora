#!/bin/sh
# A screenshot in every look: the whole output, or with `area` the
# part that slurp lets you select. The file goes to Screenshots in the
# pictures folder (`xdg-user-dir PICTURES` when that is installed, else
# XDG_PICTURES_DIR, else ~/Pictures), a copy goes to the clipboard when wl-copy is there, and
# a notice names the file. German with a German locale. grim is needed,
# and slurp for an area. When no notification daemon answers and
# CHORA_NOTIFIER names one (a command), the script starts it and sends the
# notice, so that the person always sees the result in the colours of the
# look and not in those of a daemon that the bus starts by itself. A session with
# no session bus at all gets a bus of its own for 7 s, with the daemon and
# the notice on it; without a way to show it, the result goes to stderr.
# Environment:
#   CHORA_NOTIFIER  the command that starts a notification daemon
# Usage: sh screenshot.sh [full|area]
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*) saved="Bildschirmfoto gespeichert" failed="Bildschirmfoto fehlgeschlagen" ;;
*) saved="Screenshot saved" failed="Screenshot failed" ;;
esac
notify() {
    command -v notify-send >/dev/null 2>&1 && notify-send -a Screenshot "$1" "$2" 2>/dev/null
}
has_bus() {
    [ -n "${DBUS_SESSION_BUS_ADDRESS:-}" ] || [ -S "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/bus" ]
}
say_on_own_bus() {
    # shellcheck disable=SC2016
    setsid dbus-run-session -- sh -c '
        sh -c "$CHORA_NOTIFIER" >/dev/null 2>&1 &
        for _ in 1 2 3 4 5 6 7 8 9 10; do
            sleep 0.3
            notify-send -a Screenshot "$1" "$2" 2>/dev/null && break
        done
        sleep 6' sh "$1" "$2" >/dev/null 2>&1 &
}
daemon_runs() {
    command -v dbus-send >/dev/null 2>&1 || return 0
    dbus-send --session --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus \
        org.freedesktop.DBus.NameHasOwner string:org.freedesktop.Notifications 2>/dev/null | grep -q true
}
say() {
    if [ -n "${CHORA_NOTIFIER:-}" ]; then
        if ! has_bus && command -v dbus-run-session >/dev/null 2>&1; then
            say_on_own_bus "$1" "$2"
            return 0
        fi
        if ! daemon_runs; then
            setsid sh -c "$CHORA_NOTIFIER" >/dev/null 2>&1 &
            for _ in 1 2 3 4 5 6 7 8 9 10; do
                sleep 0.3
                daemon_runs && break
            done
        fi
    fi
    notify "$1" "$2" && return 0
    echo "screenshot.sh: $1 $2" >&2
    return 0
}
pictures=""
if command -v xdg-user-dir >/dev/null 2>&1; then
    pictures=$(xdg-user-dir PICTURES 2>/dev/null)
    [ "$pictures" = "$HOME" ] && pictures=""
fi
dir=${pictures:-${XDG_PICTURES_DIR:-$HOME/Pictures}}/Screenshots
mkdir -p "$dir" || exit 1
file=$dir/$(date +%Y-%m-%d_%H-%M-%S).png
if [ "${1:-full}" = area ]; then
    geometry=$(slurp) || exit 0
    grim -g "$geometry" "$file"
else
    grim "$file"
fi || {
    say "$failed" ""
    exit 1
}
if command -v wl-copy >/dev/null 2>&1; then
    wl-copy <"$file" 2>/dev/null
fi
say "$saved" "$file"
