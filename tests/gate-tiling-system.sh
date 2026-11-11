#!/bin/bash
# Screenshot and power menu of tiling (tiling-v2, item 10): Super+Print
# writes a screenshot of the output to Screenshots in the pictures folder
# and a notice names the file; Super+Shift+e opens the power menu; each of
# its four words does its thing (stand-ins of loginctl and systemctl leave
# a marker). Print alone needs a grant that the core does not give yet.
#
# Usage: tests/gate-tiling-system.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need grim
need_bins chora choractl chora-tiling
home=$(mktemp -d /tmp/gate-system.XXXXXX)
tmp_dirs+=("$home")
marks=$home/marks
: >"$marks"
for what in loginctl systemctl notify-send; do
    printf '#!/bin/sh\necho %s "$@" >>"%s"\n' "$what" "$marks" >"$home/$what"
    chmod +x "$home/$what"
done
# shellcheck disable=SC2016 # the stand-in expands its own arguments
printf '#!/bin/sh\necho fuzzel "$@" >>"%s"\ncase " $* " in *" --dmenu "*) echo "$CHOICE" ;; esac\n' "$marks" >"$home/fuzzel"
chmod +x "$home/fuzzel"
export HOME=$home PATH=$home:$bin:$PATH CHORA_FLAVOR_DIR=$repo/flavors/tiling
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
grant input.inject
keys() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the key $1"; }

keys Super+Print
sleep 3
shot=$(find "$home/Pictures/Screenshots" -name "*.png" 2>/dev/null | head -n 1)
[ -n "$shot" ] || fail "Super+Print wrote no screenshot"
[ "$(python3 -I -c 'import struct,sys; d=open(sys.argv[1],"rb").read(24); print(*struct.unpack(">II", d[16:24]))' "$shot")" = "1280 800" ] || fail "the screenshot is not 1280x800"
grep -q "^notify-send.*$shot" "$marks" || fail "no notice names $shot: $(cat "$marks")"
ok "Super+Print writes a screenshot of the output, and a notice names the file"

: >"$marks"
keys Super+Shift+e
sleep 2
grep -q -- "--dmenu" "$marks" || fail "Super+Shift+e did not open the menu: $(cat "$marks")"
ok "Super+Shift+e opens the power menu"

for pair in "Lock|loginctl lock-session" "Log out|loginctl terminate-session" "Restart|systemctl reboot" "Shut down|systemctl poweroff"; do
    word=${pair%%|*}
    want=${pair#*|}
    : >"$marks"
    CHOICE=$word sh "$repo/flavors/tiling/power.sh" || fail "power.sh $word"
    grep -q "^$want" "$marks" || fail "$word did not run $want: $(cat "$marks")"
done
ok "Lock, Log out, Restart and Shut down each do their thing"
