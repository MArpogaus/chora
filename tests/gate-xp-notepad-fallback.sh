#!/bin/bash
# Notepad of the xp start menu when its editor does not open (xp-polish
# r5, j1-2/N1): the start menu tries the setting `editor`, then the
# installed editors with a window, then the editor of the terminal. A
# start that falls back says which editor serves and why the one before
# it did not; when none opens, one notice says Notepad cannot open.
#
# The editors here are fakes on a PATH that has no other editor: a
# gedit that opens a foot, a lowriter that ends without a window.
#
# Usage: tests/gate-xp-notepad-fallback.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the capture xp-notepad-fallback.png
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
need jq
need dbus-daemon
need_bins chora choractl chora-xp
temp_config
work=$XDG_CONFIG_HOME
export HOME=$work/home
export XDG_DATA_HOME=$work/data
export XDG_DATA_DIRS=$work/none
mkdir -p "$HOME" "$XDG_DATA_HOME" "$work/chora" "$work/path" "$work/fakes"
for tool in /usr/bin/*; do
    case ${tool##*/} in
    lowriter | mousepad | gedit | gnome-text-editor | kwrite | kate | xed | pluma | featherpad | leafpad | l3afpad | geany | nano | vim | vi | vim.* | vi.*) ;;
    *) ln -sf "$tool" "$work/path/${tool##*/}" ;;
    esac
done
case_file=$work/case
echo none >"$case_file"
cat >"$work/fakes/gedit" <<FAKE
#!/bin/sh
exec foot -a "notepad-\$(cat $case_file)" sh -c 'sleep 600'
FAKE
chmod +x "$work/fakes/gedit"
sed "s|^editor:.*|editor: chora-no-such-editor|" "$repo/plugins/chora-xp/xp.yaml" >"$work/chora/chora-xp.yaml"
export PATH=$bin:$work/fakes:$work/path
export CHORA_LOG=$work/chora.log
session_start 1280x800 -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
bus="unix:path=$XDG_RUNTIME_DIR/bus"
dbus-daemon --session --nofork --address="$bus" >"$work/dbus.log" 2>&1 &
pids+=($!)
for _ in $(seq 50); do
    [ -S "$XDG_RUNTIME_DIR/bus" ] && break
    sleep 0.2
done
grant screen.capture input.inject methods.call
start_notify "$work/notify.log"
click() {
    echo "[{\"pointer_move\": {\"position\": [$1, $2]}}, {\"click\": {\"button\": 272}}]" | ctl inject - >/dev/null ||
        fail "the click at $1, $2"
}
menu_visible() { ctl query id:xp-menu | jq -e '.[0].visible == true' >/dev/null; }
notice() {
    ctl call notify state.get 2>/dev/null |
        jq -e --arg s "$1" --arg b "$2" '.shown[]? | select(.summary == $s and (.body | contains($b)))' >/dev/null
}
wait_notice() {
    for _ in $(seq 60); do
        notice "$1" "$2" && return 0
        sleep 0.5
    done
    fail "no notice $1 / $2: $(ctl call notify state.get 2>&1)"
}
click_notepad() {
    click 50 785
    for _ in $(seq 40); do
        menu_visible && break
        sleep 0.25
    done
    menu_visible || fail "the start button opens no start menu"
    sleep 1
    item=$(ctl query kind:image | jq -c '[.[] | select(.meta.text == "Notepad" and (.id | startswith("xp-item-")))] | .[0]')
    [ "$item" != null ] || fail "the start menu has no entry Notepad"
    read -r x y w h < <(jq -r '.rects | to_entries[0].value | map(floor) | join(" ")' <<<"$item")
    click $((x + w / 2)) $((y + h / 2))
}
set_editor() { ctl call xp settings.set "{\"editor\": \"$1\"}" >/dev/null || fail "settings.set editor $1"; }

echo missing >"$case_file"
click_notepad
ctl wait --timeout 15 'app_id:notepad-missing kind:surface' >/dev/null || fail "with the editor of the settings missing no window opened: $(ctl query kind:surface | jq -c "[.[].app_id]")"

wait_notice "Notepad opened with gedit" "chora-no-such-editor is not installed"
ok "the editor of the settings is missing: gedit opens and the notice names it and why"

echo exits >"$case_file"
printf '#!/bin/sh\nexit 1\n' >"$work/fakes/lowriter"
chmod +x "$work/fakes/lowriter"
set_editor lowriter
click_notepad
ctl wait --timeout 15 'app_id:notepad-exits kind:surface' >/dev/null || fail "with an editor that ends without a window no window opened"
wait_notice "Notepad opened with gedit" "lowriter ended without a window"
[ -z "${SHOTS:-}" ] || ctl capture --output HEADLESS-1 "$SHOTS/xp-notepad-fallback.png" >/dev/null || fail "the capture"
ok "the editor ends without a window: gedit opens and the notice names both"

echo second >"$case_file"
cat >"$work/fakes/kate" <<FAKE
#!/bin/sh
(sleep 4; exec foot -a notepad-handoff sh -c 'sleep 600') >/dev/null 2>&1 &
exit 0
FAKE
chmod +x "$work/fakes/kate"
rm "$work/fakes/lowriter"
set_editor kate
click_notepad
ctl wait --timeout 15 'app_id:notepad-handoff kind:surface' >/dev/null || fail "a hand-off editor opened no window"
sleep 8
! ctl query 'app_id:notepad-second kind:surface' | jq -e 'length > 0' >/dev/null ||
    fail "a second editor opened over the slow hand-off editor"
! notice "Notepad opened with gedit" "kate" || fail "a notice names a fallback although kate opened"
ok "an editor that hands over and shows its window after 4 s is the only editor"

rm "$work/fakes/gedit" "$work/fakes/kate"
printf '#!/bin/sh\nexit 1\n' >"$work/fakes/lowriter"
printf '#!/bin/sh\nexit 3\n' >"$work/fakes/mousepad"
chmod +x "$work/fakes/mousepad"
set_editor lowriter
click_notepad
wait_notice "Cannot open Notepad" "ended without a window"
! notice "Notepad opened with mousepad" "lowriter ended without a window" || fail "a notice says Notepad opened with mousepad, which ended without a window"
ok "no editor opens: one notice says Notepad cannot open and none says it opened"
