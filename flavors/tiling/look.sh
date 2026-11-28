#!/bin/sh
# The look of the tiling flavor: light (Paper), dark (Ink) or system (the
# colour scheme of the system, the default). It sets look.theme in
# chora-tiling.yaml and applies it at once. A refusal shows as a notice that
# says what stayed; the reason of choractl goes to stderr, as its words are
# English and name the tool. German with a German locale.
# Usage: sh look.sh light|dark|system
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*)
    failed="Design nicht geändert"
    stayed="Das Design ist geblieben, wie es war."
    ;;
*)
    failed="Theme not changed"
    stayed="The look stayed as it was."
    ;;
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
if ! reason=$(choractl call tiling settings.set "{\"look\": {\"theme\": \"$theme\"}}" 2>&1 >/dev/null); then
    command -v notify-send >/dev/null 2>&1 && notify-send -a Look "$failed" "$stayed"
    echo "look.sh: $reason" >&2
    exit 1
fi
