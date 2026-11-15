#!/bin/bash
# Puts the CJK and emoji fonts of the e2e job into the directory that
# the text checks of tests/gate-0.8-9.sh and gate-0.8-10.sh read. A font
# that this machine lacks is left out, and the text checks then skip.
#
# Usage: tests/e2e-fonts.sh
#   CHORA_FONT_DIRS  the directory (default: ~/.local/share/fonts/chora-test)
set -eu
target=${CHORA_FONT_DIRS:-$HOME/.local/share/fonts/chora-test}
mkdir -p "$target"
for font in /usr/share/fonts/google-noto-sans-cjk-vf-fonts/NotoSansCJK-VF.ttc \
    /usr/share/fonts/twemoji/Twemoji.ttf; do
    if [ -e "$font" ]; then ln -sf "$font" "$target/"; fi
done
