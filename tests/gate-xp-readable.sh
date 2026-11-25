#!/bin/bash
# What a reader sees on xp (xp-readable r1: xp-9, xp-10, xp-13, xp-19, xp-39,
# xp-40, xp-41), measured on captures of one headless session at 1280x800:
#
#   pager-contrast     the digits of the workspace buttons 1 to 4 of the
#                      taskbar (our own element, not Luna) have 4.5:1 in
#                      every state: at rest, under the pointer, and dimmed
#                      behind Turn off computer
#   pager-current      the current workspace button differs from each other one by 3:1 at rest and under
#                      the pointer (rv9j-1), measured on the fill pixels at the left edge of the buttons
#   session-labels     the captions of Lock, Log Off and Restart of Turn off
#                      computer (our own buttons) have 7:1, the bar of a button, as white on their
#                      backing; Stand By and Turn Off are Luna's and lie on its
#                      panel fill, exempt in known-red.toml (white on it 2.1:1)
#   caption-size       the captions of Lock and Restart of Turn off computer stand 12 px high or more in ink,
#                      shadow included (11 px type gives 10, 14 px type 12 to 13): the 14 px of body text (xp-9)
#   caption-gap        neighbouring captions of the big buttons of Turn off computer keep 4 px of
#                      clear columns between their white ink (rxm1-1)
#   cancel-target      Cancel of Turn off computer is 24 px high or more
#   inactive-title     the title of a window that is not active has 4.5:1
#   bin-clear          the Recycle Bin stands at the bottom right, above the paperclip, and
#                      no balloon (greeting, empty chat) and no notice (one, three) lies on it or on
#                      any other desktop icon (xp-13)
#   datetime-click     one click on the clock of the tray opens Date and Time, which shows a taskbar button (xp-10)
#   datetime-metrics   the push buttons of Date and Time are the 75 x 23 of XP
#
# Every cell runs and prints; the gate fails when one is red.
# The captions have a shadow, so the own ones are measured as white (--text=FFFFFF), not as the
# farthest colour, which is the shadow; Stand By and Turn Off lie on the dialog.svg fill #92B1F3 to
# #95B4F5 at their place (2.1:1 white), exempt in known-red.toml, the fill is checked on the two rows
# under the caption (the 14 px caption covers most of its own rows, so they have no common colour).
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
    local label=$1 icon bx by bw bh id x y w h seen=0
    while read -r icon bx by bw bh; do
        seen=$((seen + 1))
        while read -r id x y w h; do
            if [ "$w" -gt 0 ] && [ "$h" -gt 0 ] && [ "$bx" -lt $((x + w)) ] && [ "$x" -lt $((bx + bw)) ] &&
                [ "$by" -lt $((y + h)) ] && [ "$y" -lt $((by + bh)) ]; then
                red "bin-clear: $label: $id $x,$y ${w}x$h lies on the desktop icon $icon $bx,$by ${bw}x$bh"
            fi
        done < <(ctl query '*' | jq -r '.[] | select(.owner == "assistant" or .owner == "notify") | .id as $id |
            .rects | to_entries[] | "\($id) \(.value | map(floor) | join(" "))"')
    done < <(ctl query kind:image | jq -r '.[] | select(.id | test("^xp-icon-[0-9]+$")) | .id as $id |
        .rects | to_entries[0] | "\($id) \(.value | map(floor) | join(" "))"')
    [ "$seen" -ge 2 ] || red "bin-clear: $label: only $seen desktop icons"
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

pager_apart() {
    local file=$1 label=$2 n x y w h cur=
    read -r x y w h <<<"$(rect xp-pager-0-HEADLESS-1)"
    cur=$(python3 -I "$repo/tests/png-pixel.py" "$file" "$((x + 3))" "$((y + h / 2))")
    for n in 1 2 3; do
        read -r x y w h <<<"$(rect "xp-pager-$n-HEADLESS-1")"
        at_least "$(python3 -I - "$cur" "$(python3 -I "$repo/tests/png-pixel.py" "$file" "$((x + 3))" "$((y + h / 2))")" <<'PY'
import sys


def lum(p):
    lin = [c / 255 / 12.92 if c / 255 <= 0.04045 else ((c / 255 + 0.055) / 1.055) ** 2.4 for c in map(int, p.split()[:3])]
    return 0.2126 * lin[0] + 0.7152 * lin[1] + 0.0722 * lin[2]


a, b = sorted((lum(sys.argv[1]), lum(sys.argv[2])), reverse=True)
print(f"{(a + 0.05) / (b + 0.05):.2f}")
PY
)" 3.0 "pager-current: $label: workspace button $((n + 1)) beside the current one"
    done
}

sleep 1
read -r bx by bw bh <<<"$(rect xp-icon-2)"
{ [ "$((bx + bw))" -ge 1256 ] && [ "$by" -ge 400 ] && [ "$((by + bh))" -le 770 ]; } ||
    red "bin-clear: the Recycle Bin $bx,$by ${bw}x$bh is not at the bottom right of 1280x800 (XP: right edge 8 px from the edge, above the taskbar)"
bin_clear "greeting of the paperclip"
ctl capture "$shots/rest.png" >/dev/null || fail "no capture"
pager_digits "$shots/rest.png" "pager at rest"
pager_apart "$shots/rest.png" "pager at rest"
read -r x y w h <<<"$(rect xp-pager-1-HEADLESS-1)"
inj "[{\"pointer_move\": {\"position\": [$((x + w / 2)), $((y + h / 2))]}}]"
sleep 0.6
ctl capture "$shots/hover.png" >/dev/null
pager_digits "$shots/hover.png" "pager under the pointer"
pager_apart "$shots/hover.png" "pager under the pointer"
inj '[{"pointer_move": {"position": [640, 200]}}]'

key Super+Shift+e
sleep 1.5
ctl capture "$shots/turnoff.png" >/dev/null
pager_digits "$shots/turnoff.png" "pager dimmed behind Turn off computer"
read -r x y w h <<<"$(rect xp-session-button-5)"
[ "${h:-0}" -ge 24 ] || red "cancel-target: Cancel of Turn off computer is ${w:-?} x ${h:-?}, wants 24 px high or more"
for n in 0 4; do
    read -r x y w h <<<"$(rect "xp-session-button-$n")"
    ink=$(python3 -I "$repo/tests/png-ink.py" "$shots/turnoff.png" "$((x + 8))" "$((y + h - 17))" "$((w - 16))" 16 | tr ' ' '\n' | sort -n | tail -n 1)
    echo "caption $n ink: ${ink:-?} px"
    [ "${ink:-0}" -ge 12 ] || red "caption-size: caption $n of Turn off computer is ${ink:-?} px of ink high, 14 px type gives 12 or more"
done
prev=
for n in 0 1 2 3 4; do
    read -r x y w h <<<"$(rect "xp-session-button-$n")"
    read -r left right <<<"$(python3 -I "$repo/tests/png-extent.py" "$shots/turnoff.png" "$x" "$((y + h - 17))" "$w" 16)"
    if [ "$left" = none ] || [ -z "${left:-}" ]; then
        red "caption-gap: caption $n of Turn off computer shows no white ink"
    else
        [ -z "$prev" ] || [ "$((left - prev - 1))" -ge 4 ] ||
            red "caption-gap: caption $n of Turn off computer starts $((left - prev - 1)) px after the caption before it (ink ends at x $prev, starts at x $left), wants 4 or more (rxm1-1)"
        prev=$right
    fi
done
for n in 0 1 2 3 4; do
    read -r x y w h <<<"$(rect "xp-session-button-$n")"
    if [ "$n" = 2 ] || [ "$n" = 3 ]; then
        read -r r g b <<<"$(python3 -I "$repo/tests/png-contrast.py" "$shots/turnoff.png" "$x" "$((y + h - 2))" "$w" 2 --text=FFFFFF | sed 's/.* on (\(.*\))/\1/; s/,//g')"
        { [ "${r:-0}" -ge 144 ] && [ "$r" -le 151 ] && [ "${g:-0}" -ge 175 ] && [ "$g" -le 182 ] && [ "${b:-0}" -ge 241 ] && [ "$b" -le 247 ]; } ||
            red "session-labels: caption $n of Turn off computer lies on (${r:-?}, ${g:-?}, ${b:-?}), not the Luna fill (148, 179, 244) of its exemption"
    else
        at_least "$(contrast "$shots/turnoff.png" "$x" "$((y + h - 16))" "$w" 15 --text=FFFFFF)" 7.0 "session-labels: caption $n of Turn off computer"
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

inj '[{"pointer_move": {"position": [1250, 788]}}, {"click": {}}]'
sleep 1.5
[ "$(ctl query 'id:xp-dlg-datetime-content-c-ok' | jq length)" = 1 ] ||
    red "datetime-click: one click on the clock of the tray opens no Date and Time (xp-10)"
[ "$(ctl query kind:image | jq '[.[] | select(.id | startswith("xp-task-datetimepanel-"))] | length')" -ge 1 ] ||
    red "datetime-click: Date and Time shows no taskbar button (xp-10)"
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
