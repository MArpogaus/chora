#!/bin/bash
# What a reader sees on xp (xp-readable r1: xp-9, xp-10, xp-13, xp-19, xp-39,
# xp-40, xp-41), measured on captures of one headless session at 1280x800:
#
#   pager-contrast     the digits of the workspace buttons 1 to 4 of the
#                      taskbar (our own element, not Luna) have 4.5:1 in
#                      every state: at rest, under the pointer, and dimmed
#                      behind Turn off computer
#   session-labels     the captions of Lock, Log Off and Restart of Turn off
#                      computer (our own buttons) have 4.5:1 as white on their
#                      backing; Stand By and Turn Off are Luna's and lie on its
#                      panel fill, exempt in known-red.toml (white on it 2.1:1)
#   cancel-target      Cancel of Turn off computer is 24 px high or more
#   inactive-title     the title of a window that is not active has 4.5:1
#   bin-clear          the Recycle Bin stands at the bottom right, above the paperclip, and
#                      no balloon (greeting, empty chat) and no notice (one, three) lies on it
#   datetime-metrics   the push buttons of Date and Time are the 75 x 23 of XP
#
# Every cell runs and prints; the gate fails when one is red.
# The captions have a shadow, so the own ones are measured as white (--text=FFFFFF), not as the
# farthest colour, which is the shadow; Stand By and Turn Off lie on the dialog.svg fill #92B1F3 to
# #95B4F5 at their place (2.1:1 white), exempt in known-red.toml, the fill is checked.
# at_least RATIO MIN LABEL is red below the minimum; contrast FILE X Y W H prints the ratio alone;
# pager_digits FILE LABEL measures the digit of each pager button; the title text of a window follows
# the icon of the application and the edge of the frame, so its first 110 px are measured.
# Usage: tests/gate-xp-readable.sh   BIN: directory with the binaries
#   SHOTS: directory for the captures (default: a temp dir)
set -u
. "$(dirname "$0")/gate-lib.sh"
need jq
need_bins chora choractl chora-xp chora-assistant chora-notify
shots=${SHOTS:-$(mktemp -d /tmp/gate-xp-readable.XXXXXX)}
mkdir -p "$shots"
temp_config
export HOME=$XDG_CONFIG_HOME/home
mkdir -p "$HOME"
session_start "1280x800@1" -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
"$bin/chora-assistant" >"$shots/assistant.log" 2>&1 &
pids+=($!)
"$bin/chora-notify" --assets "$repo/flavors/xp/assets/notify" >"$shots/notify.log" 2>&1 &
pids+=($!)
grant screen.capture input.inject
sleep 3

reds=0
red() {
    echo "RED: $*" >&2
    reds=$((reds + 1))
}
inj() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
key() { inj "[{\"shortcut\": {\"keys\": \"$1\"}}]"; }
rect() { ctl query "id:$1" | jq -r '.[0].rects | to_entries[0].value | map(floor) | join(" ")'; }
bin_clear() {
    local label=$1 bx by bw bh id x y w h
    read -r bx by bw bh <<<"$(rect xp-icon-2)"
    [ -n "${bx:-}" ] || { red "bin-clear: $label: no Recycle Bin"; return; }
    while read -r id x y w h; do
        if [ "$w" -gt 0 ] && [ "$h" -gt 0 ] && [ "$bx" -lt $((x + w)) ] && [ "$x" -lt $((bx + bw)) ] &&
            [ "$by" -lt $((y + h)) ] && [ "$y" -lt $((by + bh)) ]; then
            red "bin-clear: $label: $id $x,$y ${w}x$h lies on the Recycle Bin $bx,$by ${bw}x$bh"
        fi
    done < <(ctl query '*' | jq -r '.[] | select(.owner == "assistant" or .owner == "notify") | .id as $id |
        .rects | to_entries[] | "\($id) \(.value | map(floor) | join(" "))"')
}
at_least() {
    awk -v r="$1" -v m="$2" 'BEGIN { exit !(r + 0 >= m + 0) }' || red "$3: $1:1, wants $2:1"
}
contrast() { python3 -I "$repo/tests/png-contrast.py" "$@" | cut -d' ' -f1; }
pager_digits() {
    local file=$1 label=$2 n x y w h
    for n in 0 1 2 3; do
        read -r x y w h <<<"$(rect "xp-pager-$n-HEADLESS-1")"
        at_least "$(contrast "$file" "$((x + 4))" "$((y + 4))" "$((w - 8))" "$((h - 8))")" 4.5 "$label: workspace button $((n + 1))"
    done
}

sleep 1
read -r bx by bw bh <<<"$(rect xp-icon-2)"
{ [ "$((bx + bw))" -ge 1256 ] && [ "$by" -ge 400 ] && [ "$((by + bh))" -le 770 ]; } ||
    red "bin-clear: the Recycle Bin $bx,$by ${bw}x$bh is not at the bottom right of 1280x800 (XP: right edge 8 px from the edge, above the taskbar)"
bin_clear "greeting of the paperclip"
ctl capture "$shots/rest.png" >/dev/null || fail "no capture"
pager_digits "$shots/rest.png" "pager at rest"
read -r x y w h <<<"$(rect xp-pager-1-HEADLESS-1)"
inj "[{\"pointer_move\": {\"position\": [$((x + w / 2)), $((y + h / 2))]}}]"
sleep 0.6
ctl capture "$shots/hover.png" >/dev/null
pager_digits "$shots/hover.png" "pager under the pointer"
inj '[{"pointer_move": {"position": [640, 200]}}]'

key Super+Shift+e
sleep 1.5
ctl capture "$shots/turnoff.png" >/dev/null
pager_digits "$shots/turnoff.png" "pager dimmed behind Turn off computer"
read -r x y w h <<<"$(rect xp-session-button-5)"
[ "${h:-0}" -ge 24 ] || red "cancel-target: Cancel of Turn off computer is ${w:-?} x ${h:-?}, wants 24 px high or more"
for n in 0 1 2 3 4; do
    read -r x y w h <<<"$(rect "xp-session-button-$n")"
    if [ "$n" = 2 ] || [ "$n" = 3 ]; then
        read -r r g b <<<"$(python3 -I "$repo/tests/png-contrast.py" "$shots/turnoff.png" "$x" "$((y + h - 16))" "$w" 15 --text=FFFFFF | sed 's/.* on (\(.*\))/\1/; s/,//g')"
        { [ "${r:-0}" -ge 144 ] && [ "$r" -le 151 ] && [ "${g:-0}" -ge 175 ] && [ "$g" -le 182 ] && [ "${b:-0}" -ge 241 ] && [ "$b" -le 247 ]; } ||
            red "session-labels: caption $n of Turn off computer lies on (${r:-?}, ${g:-?}, ${b:-?}), not the Luna fill (148, 179, 244) of its exemption"
    else
        at_least "$(contrast "$shots/turnoff.png" "$x" "$((y + h - 16))" "$w" 15 --text=FFFFFF)" 4.5 "session-labels: caption $n of Turn off computer"
    fi
done
key Escape
sleep 1

need foot
foot -T "Inactive window" sleep 100 >"$shots/foot1.log" 2>&1 &
foot1=$!
pids+=($!)
sleep 2
foot -T "Active window" sleep 100 >"$shots/foot2.log" 2>&1 &
foot2=$!
pids+=($!)
sleep 2.5
ctl capture "$shots/windows.png" >/dev/null
read -r x y w h <<<"$(ctl state xp | jq -r '.windows[] | select(.title == "Inactive window") | .rect | map(floor) | join(" ")')"
if [ -z "${x:-}" ]; then
    red "inactive-title: no window titled Inactive window"
else
    ratio=$(contrast "$shots/windows.png" "$((x + 24))" "$((y + 6))" 110 15)
    echo "inactive title: $ratio:1"
    at_least "$ratio" 4.5 "inactive-title: the title of a window without the focus"
fi
kill "$foot1" "$foot2" 2>/dev/null
sleep 1

inj '[{"pointer_move": {"position": [1250, 788]}}, {"click": {}}, {"click": {}}]'
sleep 1.5
for b in ok cancel apply; do
    read -r x y w h <<<"$(rect "xp-dlg-datetime-content-c-$b")"
    { [ "${w:-0}" = 75 ] && [ "${h:-0}" = 23 ]; } || red "datetime-metrics: $b is ${w:-?} x ${h:-?}, XP has 75 x 23"
done
read -r x y w h <<<"$(rect xp-dlg-datetime-close)"
inj '[{"pointer_move": {"position": ['"$((x + 10)), $((y + 10))"']}}, {"click": {}}]'
sleep 1

ctl call assistant chat.show '{"name": "atlas"}' >/dev/null || fail "chat.show failed"
sleep 2.5
ctl capture "$shots/chat.png" >/dev/null
bin_clear "empty chat balloon"
ctl call notify notify.show '{"app": "gate", "summary": "Notice 1", "body": "the first notice", "expire_ms": 0}' >/dev/null || fail "notify.show failed"
sleep 1.5
ctl capture "$shots/notice1.png" >/dev/null
bin_clear "one notice"
for n in 2 3; do
    ctl call notify notify.show "{\"app\": \"gate\", \"summary\": \"Notice $n\", \"body\": \"notice number $n\", \"expire_ms\": 0}" >/dev/null || fail "notify.show failed"
done
sleep 1.5
ctl capture "$shots/notice3.png" >/dev/null
bin_clear "three notices"

[ "$reds" = 0 ] || fail "$reds cells red (captures in $shots)"
ok "xp is readable: pager, captions, Cancel, inactive title, Recycle Bin, Date and Time"
