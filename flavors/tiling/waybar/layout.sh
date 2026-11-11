#!/bin/sh
# The layout word of the panel: prints the layout of the current tag as the
# JSON of a custom module of waybar, a word of a person such as "Tile" or
# "Fair", never the internal name. The tooltip names the key that goes to
# the next layout. German with a German locale. jq is needed.
# Usage: sh layout.sh
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*) tip="Nächste Anordnung (Super+Umschalt+Leertaste)" ;;
*) tip="Next layout (Super+Shift+Space)" ;;
esac
layout=$(choractl call tiling get_layout 2>/dev/null | jq -r '.layout // empty' 2>/dev/null)
case ${LC_ALL:-${LC_MESSAGES:-${LANG:-}}} in
de*)
    case $layout in
    tile) word="Kacheln" ;;
    tile_left) word="Kacheln links" ;;
    tile_bottom) word="Kacheln unten" ;;
    tile_top) word="Kacheln oben" ;;
    fair) word="Raster" ;;
    fair_horizontal) word="Raster quer" ;;
    spiral) word="Spirale" ;;
    dwindle) word="Treppe" ;;
    max) word="Voll" ;;
    fullscreen) word="Vollbild" ;;
    magnifier) word="Lupe" ;;
    floating) word="Frei" ;;
    *) word="" ;;
    esac
    ;;
*)
    case $layout in
    tile) word="Tile" ;;
    tile_left) word="Tile left" ;;
    tile_bottom) word="Tile bottom" ;;
    tile_top) word="Tile top" ;;
    fair) word="Fair" ;;
    fair_horizontal) word="Fair rows" ;;
    spiral) word="Spiral" ;;
    dwindle) word="Dwindle" ;;
    max) word="Max" ;;
    fullscreen) word="Fullscreen" ;;
    magnifier) word="Magnifier" ;;
    floating) word="Floating" ;;
    *) word="" ;;
    esac
    ;;
esac
printf '{"text": "%s", "tooltip": "%s"}\n' "$word" "$tip"
