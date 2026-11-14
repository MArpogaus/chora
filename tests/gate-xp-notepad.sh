#!/bin/bash
# Notepad of the xp start menu (xp-4): the start menu has the entry
# Notepad, the first program below the pinned rows, and a click on it
# starts the plain text editor of the setting `editor`. Here that
# editor is a foot with the app id notepad.
#
# Usage: tests/gate-xp-notepad.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the capture xp-notepad-menu.png
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
need jq
need_bins chora choractl chora-xp
temp_config
work=$XDG_CONFIG_HOME
export HOME=$work/home
export XDG_DATA_HOME=$work/data
export XDG_DATA_DIRS=$work/none
mkdir -p "$HOME" "$XDG_DATA_HOME" "$work/chora"
sed "s|^editor:.*|editor: foot -a notepad sh -c 'sleep 600'|" "$repo/plugins/chora-xp/xp.yaml" >"$work/chora/chora-xp.yaml"
grep -q '^editor: foot' "$work/chora/chora-xp.yaml" || fail "the settings of chora-xp have no editor"
export PATH=$bin:$PATH
session_start 1280x800 -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
grant screen.capture input.inject
click() {
    echo "[{\"pointer_move\": {\"position\": [$1, $2]}}, {\"click\": {\"button\": 272}}]" | ctl inject - >/dev/null ||
        fail "the click at $1, $2"
}
menu_visible() { ctl query id:xp-menu | jq -e '.[0].visible == true' >/dev/null; }

click 50 785
for _ in $(seq 40); do
    menu_visible && break
    sleep 0.25
done
menu_visible || fail "the start button opens no start menu"
sleep 1
item=$(ctl query kind:image | jq -c '[.[] | select(.meta.text == "Notepad" and (.id | startswith("xp-item-")))] | .[0]')
[ "$item" != null ] || fail "the start menu has no entry Notepad: $(ctl query kind:image | jq -c '[.[] | select(.id | startswith("xp-item-")) | .meta.text]')"
[ "$(jq -r .id <<<"$item")" = xp-item-2 ] || fail "Notepad is $(jq -r .id <<<"$item"), not the first program below the pinned rows"
[ -z "${SHOTS:-}" ] || ctl capture --output HEADLESS-1 "$SHOTS/xp-notepad-menu.png" >/dev/null || fail "the capture"
ok "the start menu has Notepad as the first program below the pinned rows"

read -r x y w h < <(jq -r '.rects | to_entries[0].value | map(floor) | join(" ")' <<<"$item")
click $((x + w / 2)) $((y + h / 2))
ctl wait --timeout 15 'app_id:notepad kind:surface' >/dev/null || fail "a click on Notepad did not start the editor of the settings"
ok "a click on Notepad starts the editor of the settings"
