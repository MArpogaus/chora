#!/bin/bash
# The doors of the tiling bar (tiling-v2, item 2): with the mouse alone,
# Apps runs the launcher, a click on tag 2 views tag 2, a click on the
# layout word goes to the next layout, and the power icon opens the power menu, whose Lock locks. The clicks
# are injected pointer clicks on the waybar of a headless session; the
# launcher and `loginctl` are stand-ins that leave a marker.
#
# Usage: tests/gate-tiling-doors.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need waybar
need jq
need_bins chora choractl chora-tiling
flavor=$repo/flavors/tiling
fake=$(mktemp -d /tmp/gate-doors.XXXXXX)
marks=$fake/marks
: >"$marks"
for what in fuzzel loginctl; do
    printf '#!/bin/sh\necho %s "$@" >>"%s"\n' "$what" "$marks" >"$fake/$what"
    chmod +x "$fake/$what"
done
printf '#!/bin/sh\necho fuzzel "$@" >>"%s"\ncase " $* " in *" --dmenu "*) echo Lock ;; esac\n' "$marks" >"$fake/fuzzel"
export PATH=$fake:$bin:$PATH
export CHORA_FLAVOR_DIR=$flavor
temp_config
tmp_dirs+=("$fake")
session_start 1280x800 -- "$bin/chora-tiling"
CHORA_FLAVOR_DIR=$flavor setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/dev/null 2>&1 &
run_pid $!
grant screen.capture input.inject
shot=$fake/bar.png
for _ in $(seq 40); do
    sleep 0.5
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || continue
    [ "$(pixel "$shot" 640 24)" != "$(pixel "$shot" 640 300)" ] && break
done

pill=
for x in $(seq 14 2 200); do
    read -r r g b _ < <(pixel "$shot" "$x" 24)
    [ "$r $g $b" = "167 139 250" ] && {
        pill=$x
        break
    }
done
[ -n "$pill" ] || fail "no accent pill of tag 1 in the bar"

click() {
    echo "[{\"pointer_move\": {\"position\": [$1, 24]}}, {\"click\": {\"button\": 272}}]" | ctl inject - >/dev/null ||
        fail "the click at $1"
    sleep 1.5
}
marked() { grep -q "^$1" "$marks"; }

tag2=$((pill + 26 + 12))
click $tag2
ctl call tiling get_tags | jq -e '.outputs[0].tags[1].shown' >/dev/null || fail "a click on tag 2 did not view it"
ok "a click on the number 2 views tag 2"

before=$(ctl call tiling get_layout | jq -r .layout)
layout=$((pill + 26 * 8 + 25 + 9 + 48))
click $layout
after=$(ctl call tiling get_layout | jq -r .layout)
[ "$before" != "$after" ] || fail "a click on the layout word kept $before"
ok "a click on the layout word goes from $before to $after"

click $((pill - 40))
marked fuzzel || fail "Apps did not run the launcher"
ok "Apps runs the launcher"

click 1250
marked loginctl || fail "the power icon did not lock"
ok "the power icon opens the menu, and Lock locks"
