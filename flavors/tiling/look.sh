#!/bin/sh
# The look of the tiling flavor: light (Paper), dark (Ink) or system (the
# colour scheme of the system, the default). It sets look.theme in
# chora-tiling.yaml and applies it at once. German with a German locale.
# Usage: sh look.sh light|dark|system
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*) failed="Aussehen nicht geändert" ;;
*) failed="Look not changed" ;;
esac
case ${1:-} in
light) theme=paper ;;
dark) theme=ink ;;
system) theme=system ;;
*)
    echo "usage: sh look.sh light|dark|system" >&2
    exit 2
    ;;
esac
if ! choractl call tiling settings.set "{\"look\": {\"theme\": \"$theme\"}}" >/dev/null; then
    command -v notify-send >/dev/null 2>&1 && notify-send -a Look "$failed" "choractl call tiling settings.set"
    exit 1
fi
