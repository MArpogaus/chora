#!/bin/sh
# A screenshot in every look: the whole output, or with `area` the
# part that slurp lets you select. The file goes to Screenshots in the
# pictures folder (`xdg-user-dir PICTURES` when that is installed, else
# XDG_PICTURES_DIR, else ~/Pictures), a copy goes to the clipboard when wl-copy is there, and
# a notice names the file, or the reason when grim fails. German with a German locale. grim is needed,
# and slurp for an area. The notice is a short balloon through `choractl call notify
# notify.show` to chora-notify on the live API; when that fails, through
# notify-send to the notification server of the session bus. Without
# either, the result goes to stderr.
# Usage: sh screenshot.sh [full|area]
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*) saved="Bildschirmfoto gespeichert" failed="Bildschirmfoto fehlgeschlagen" ;;
*) saved="Screenshot saved" failed="Screenshot failed" ;;
esac
json() {
    printf '"%s"' "$(printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g')"
}
say() {
    if command -v choractl >/dev/null 2>&1 &&
        choractl call notify notify.show \
            "{\"app\": \"Screenshot\", \"summary\": $(json "$1"), \"body\": $(json "$2"), \"expire_ms\": 6000}" \
            >/dev/null 2>&1; then
        return 0
    fi
    if command -v notify-send >/dev/null 2>&1 && notify-send -a Screenshot "$1" "$2" 2>/dev/null; then
        return 0
    fi
    echo "screenshot.sh: $1 $2" >&2
    return 0
}
pictures=""
if command -v xdg-user-dir >/dev/null 2>&1; then
    pictures=$(xdg-user-dir PICTURES 2>/dev/null)
    [ "$pictures" = "$HOME" ] && pictures=""
fi
dir=${pictures:-${XDG_PICTURES_DIR:-$HOME/Pictures}}/Screenshots
# mkdir-ok: the Screenshots folder of the user pictures, not a runtime directory
mkdir -p "$dir" || exit 1
file=$dir/$(date +%Y-%m-%d_%H-%M-%S).png
if [ "${1:-full}" = area ]; then
    geometry=$(slurp) || exit 0
    reason=$(grim -g "$geometry" "$file" 2>&1)
else
    reason=$(grim "$file" 2>&1)
fi || {
    rm -f "$file"
    say "$failed" "${reason:-grim failed}"
    exit 1
}
if command -v wl-copy >/dev/null 2>&1; then
    wl-copy <"$file" 2>/dev/null
fi
say "$saved" "$file"
