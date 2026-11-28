#!/bin/bash
# Notepad of the xp start menu (xp-4): the start menu has the entry
# Notepad, the first program below the pinned rows, and a click on it
# starts the plain text editor of the setting `editor`. Here that
# editor is a foot with the app id notepad. Last (xp-46, rpx1-1): a foot
# that keeps what it is typed holds the keyboard; Super+x, Super+k, Super+m
# and Super+q, which xp binds to nothing, type nothing into it and the
# release of Super opens no Start menu.
#
# Usage: tests/gate-xp-notepad.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the capture xp-notepad-menu.png
# shellcheck disable=SC2016 # until_ok evaluates its first argument
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

log=$work/typed
ctl run foot -a strayxp sh -c "stty raw -echo; cat >$log" >/dev/null || fail "the run of the typing foot"
ctl wait --timeout 15 'app_id:strayxp kind:surface owner:xp' >/dev/null || fail "the typing foot is not a window of xp"
until_ok 'ctl query "app_id:strayxp kind:surface" | jq -e ".[0].focused == true" >/dev/null' 40 || fail "the typing foot does not hold the keyboard"
if menu_visible; then click 50 785; fi
until_ok '! menu_visible' 20 || fail "the Start menu stays open"
typed_text() { echo "[{\"text\": {\"text\": \"$1\"}}]" | ctl inject - >/dev/null || fail "typing $1"; }
chord() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the key $1"; }
typed_text start:
for key in Super+x Super+k Super+m Super+q; do
    chord "$key"
done
typed_text :end
until_ok '[ "$(cat "$log" 2>/dev/null)" = "start::end" ]' 20
typed=$(cat "$log" 2>/dev/null)
[ "$typed" = "start::end" ] || fail "xp-46: the client typed '$typed' and not start::end; surfaces: $(ctl query 'kind:surface' | jq -c '[.[] | {app: .app_id, focused}]'); $(core_diag)"
ok "Super+x, Super+k, Super+m and Super+q type nothing into a client of xp"
until_ok menu_visible 4 && fail "xp-46: an unbound Super chord opened the Start menu"
ok "an unbound Super chord opens no Start menu"
