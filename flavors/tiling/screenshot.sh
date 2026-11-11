#!/bin/sh
# A screenshot of the tiling flavor: the whole output, or with `area` the
# part that slurp lets you select. The file goes to Screenshots in the
# pictures folder, a copy goes to the clipboard when wl-copy is there, and
# a notice names the file. German with a German locale. grim is needed,
# and slurp for an area.
# Usage: sh screenshot.sh [full|area]
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*) saved="Bildschirmfoto gespeichert" failed="Bildschirmfoto fehlgeschlagen" ;;
*) saved="Screenshot saved" failed="Screenshot failed" ;;
esac
say() {
    command -v notify-send >/dev/null 2>&1 && notify-send -a Screenshot "$1" "$2"
    return 0
}
dir=${XDG_PICTURES_DIR:-$HOME/Pictures}/Screenshots
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
