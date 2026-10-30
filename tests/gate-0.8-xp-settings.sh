#!/bin/bash
# Gate check 4 of 0.8 (D62): `settings.set` changes the wallpaper of
# chora-xp, a capture shows it at once, and after the method `restart`
# the settings file holds it. A wrong value gets an error. The scripts
# gate-0.8-N.sh are checks of the 0.7 gate, so this one has a name of its
# own.
#
# Usage: tests/gate-0.8-xp-settings.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl chora-xp; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
config=$(mktemp -d /tmp/gate-xp-settings.XXXXXX)
export XDG_CONFIG_HOME=$config
session_start 1280x800 -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
file=$config/chora/chora-xp.yaml
out=$config/out
mkdir -p "$out"
grant screen.capture
# Answers when chora-xp exports its methods.
ready() { ctl call xp settings.get >/dev/null 2>&1; }
for _ in $(seq 50); do ready && break; sleep 0.2; done
ready || fail "chora-xp does not answer: $(ctl call xp settings.get 2>&1)"

red=$repo/plugins/chora-xp/testdata/red.jpg
# The red picture over the whole screen.
wallpaper() {
    printf '{"wallpaper": {"picture": {"file": "%s"}, "position": "stretch", "color": [0, 78, 152]}}' "$1"
}
is_red() { pixel "$1" 640 400 | awk '{exit !($1 > 150 && $2 < 80 && $3 < 80)}'; }

if ctl call xp settings.set "$(wallpaper red.jpg)" 2>/dev/null; then
    fail "settings.set took a relative path"
fi
test -e "$file" && fail "a wrong value wrote $file"
ctl capture "$out/before.png" >/dev/null || fail "the capture before"
is_red "$out/before.png" && fail "the wallpaper is red before the change"

ctl call xp settings.set "$(wallpaper "$red")" >/dev/null || fail "settings.set of the wallpaper"
ctl capture "$out/after.png" >/dev/null || fail "the capture after"
is_red "$out/after.png" || fail "no red wallpaper at once: $(pixel "$out/after.png" 640 400)"

ctl call xp restart >/dev/null || fail "the restart"
sleep 1
for _ in $(seq 50); do ready && break; sleep 0.2; done
ready || fail "chora-xp does not answer after the restart"
grep -qF "$red" "$file" || fail "the file has no red wallpaper: $(cat "$file")"
ctl capture "$out/restart.png" >/dev/null || fail "the capture after the restart"
is_red "$out/restart.png" || fail "no red wallpaper after the restart"
rm -rf "$config"
ok "settings.set changed the XP wallpaper at once, and the file keeps it over a restart"
