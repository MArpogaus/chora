#!/bin/bash
# Screenshot and power menu of tiling (tiling-v2, item 10): Super+Print
# writes a screenshot of the output to Screenshots in the pictures folder
# and a notice names the file; Super+Shift+e opens the power menu; each of
# its four words does its thing (stand-ins of loginctl, systemctl and
# swaylock leave a marker), and a lock that does not start says so. Print
# alone needs a grant that the core does not give yet.
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
for what in loginctl systemctl notify-send swaylock; do
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

for pair in "Lock|swaylock -f" "Log out|loginctl terminate-session 7" "Restart|systemctl reboot" "Shut down|systemctl poweroff"; do
    word=${pair%%|*}
    want=${pair#*|}
    : >"$marks"
    CHOICE=$word XDG_SESSION_ID=7 sh "$repo/flavors/shared/power.sh" || fail "power.sh $word"
    grep -q "^$want" "$marks" || fail "$word did not run $want: $(cat "$marks")"
done
ok "Lock, Log out, Restart and Shut down each do their thing"

: >"$marks"
CHORA_LOCKER=false sh "$repo/flavors/shared/power.sh" lock 2>/dev/null && fail "a lock that did not start exited 0"
grep -q "^notify-send.*Cannot lock the screen" "$marks" || fail "a lock that did not start showed no notice: $(cat "$marks")"
ok "a lock that does not start is an error and a notice"

: >"$marks"
env -u XDG_SESSION_ID USER=zed CHOICE=Log\ out sh "$repo/flavors/shared/power.sh" || fail "power.sh without a session"
grep -q "^loginctl terminate-user zed" "$marks" || fail "Log out without a session did not end the user: $(cat "$marks")"
grep -q "terminate-session" "$marks" && fail "Log out without a session named an empty session"
ok "Log out without XDG_SESSION_ID ends the sessions of the user"

printf '#!/bin/sh\necho "%s/Bilder"\n' "$home" >"$home/xdg-user-dir"
chmod +x "$home/xdg-user-dir"
sh "$repo/flavors/shared/screenshot.sh" || fail "screenshot.sh with xdg-user-dir"
ls "$home/Bilder/Screenshots/"*.png >/dev/null 2>&1 || fail "screenshot.sh ignored xdg-user-dir PICTURES"
ok "the screenshot goes to the folder that xdg-user-dir names"
