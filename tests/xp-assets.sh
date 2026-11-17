#!/usr/bin/env bash
# Fetches the optional Microsoft-owned XP files (fonts, icons, wallpapers,
# the start mark) as get-microsoft-files.sh of
# github.com/MArpogaus/kde-plasma-winxp-launcher does, from
# github.com/rozniak/xfce-winxp-tc, into a folder laid out like a data
# directory. Nothing of it goes into git. chora-xp reads it when
# CHORA_XP_ASSETS names it.
#
# Usage: tests/xp-assets.sh [DEST]   (default ~/.cache/chora/xp-assets)
set -euo pipefail
dest=${1:-$HOME/.cache/chora/xp-assets}
src=$(mktemp -d)
trap 'rm -rf "$src"' EXIT
git clone --quiet --depth 1 --filter=blob:none --sparse https://github.com/rozniak/xfce-winxp-tc "$src/w"
git -C "$src/w" sparse-checkout set fonts/ttf icons/luna wallpapers/common themes/luna/blue/Resources
w=$src/w
rm -rf "$dest"
mkdir -p "$dest/fonts/WindowsXP" "$dest/wallpapers/WindowsXP" "$dest/icons/WindowsXP"
cp "$w"/fonts/ttf/*.ttf "$dest/fonts/WindowsXP/"
cp "$w"/wallpapers/common/*.jpg "$dest/wallpapers/WindowsXP/"
cp "$w/themes/luna/blue/Resources/start_flag.png" "$dest/start-mark.png"
i=$dest/icons/WindowsXP
cp -r "$w/icons/luna/res" "$w/icons/luna/index.theme" "$i/"
while IFS= read -r mapping; do
    [ -n "$mapping" ] || continue
    icon=${mapping%%-->*}
    res=${mapping#*-->}
    if [[ $icon == *-symbolic ]]; then continue; fi
    for size in "$i"/res/*x*; do
        size=${size##*/}
        [ -f "$i/res/$size/$res.png" ] || continue
        mkdir -p "$i/$size/${icon%/*}"
        ln -sf "../../res/$size/$res.png" "$i/$size/$icon.png"
    done
done <"$w/icons/luna/mappings"
echo "$dest"
