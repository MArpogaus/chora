#!/bin/sh
# The settings door of the tiling flavor: opens chora-tiling.yaml in the
# terminal editor, $EDITOR or else vi, in a foot window of the palette. A
# file that does not exist yet is written first with all options and their
# defaults (chora-tiling --print-config). The change takes effect on
# Super+Ctrl+r, which reads the file again.
# Usage: sh settings.sh
file=${XDG_CONFIG_HOME:-$HOME/.config}/chora/chora-tiling.yaml
if [ ! -s "$file" ]; then
    # mkdir-ok: the config folder of the user, as chora-tiling itself writes it
    mkdir -p "$(dirname "$file")" && chora-tiling --print-config >"$file" || exit 1
fi
# shellcheck disable=SC2016 # the inner shell expands $EDITOR and $1
exec chora-tiling --launch foot -e sh -c 'exec ${EDITOR:-vi} "$1"' sh "$file"
