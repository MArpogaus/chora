#!/bin/bash
# Gate of the keys that create and close panes in chora-construct:
# t starts the terminal, and its window becomes a pane; space starts the
# launcher, whose own window shows without a pane, and the program that
# it starts becomes a pane; after the launcher the walk has the keyboard
# again; x asks the window of the pane ahead to close, and its pane goes;
# pane.remove asks as well, and a window that refuses stays a pane; a
# radius of settings.set leashes the panes at once.
# The launcher here is a foot window that starts a second foot; where
# fuzzel is installed, the programs that it starts become panes too.
#
# Usage: tests/gate-construct-keys.sh   BIN: directory with the binaries
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-probe
need foot
need setsid
temp_config
work=$XDG_CONFIG_HOME
mkdir -p "$work/chora"
# chora-construct starts its programs with `choractl run`.
export PATH=$bin:$PATH
font='DejaVu Sans Mono:size=10'
cat >"$work/chora/chora-construct.yaml" <<EOF
terminal: foot -a term -f '$font' sh -c 'sleep 600'
launcher: foot -a launcher -f '$font' sh -c 'sleep 1; setsid foot -a launched -f "$font" sh -c "sleep 600" & sleep 2'
EOF
# fuzzel, where it is installed, finds these entries. For one of them it
# asks for no token of its own, and the program inherits the token of
# fuzzel. fuzzel 1.14 reads StartupNotify inverted, so the gate has both.
export XDG_DATA_HOME=$work/data
mkdir -p "$XDG_DATA_HOME/applications" "$work/fuzzel"
for notify in true false; do
    cat >"$XDG_DATA_HOME/applications/notify-$notify.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Notify$notify
Exec=foot -a notify-$notify -f '$font' sh -c 'sleep 600'
StartupNotify=$notify
EOF
done
printf '[main]\nfont=%s\n' "$font" >"$work/fuzzel/fuzzel.ini"
# A debug core on the software renderer is slow for the space (as K1).
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
session_start 1280x800 -- "$bin/chora-construct"
grant input.inject
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject || fail "inject $1"; }
panes() {
    ctl call construct construct.list |
        python3 -I -c 'import json, sys; print(" ".join(sorted(p["name"] for p in json.load(sys.stdin)["panes"])))'
}
focused() { ctl query "$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(n[0].get("focused") if n else "")'; }
# until_ok CODE: waits up to 10 s for the shell code to succeed.
until_ok() {
    for _ in $(seq 40); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}

ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
key t
until_ok 'test "$(panes)" = term' || fail "t: the panes are '$(panes)'"
ok "t made the terminal the pane term"

key space
until_ok 'ctl query kind:surface | grep -q \"launcher\"' ||
    fail "the launcher showed no window"
[ "$(panes)" = term ] || fail "the launcher became a pane: $(panes)"
until_ok 'test "$(panes)" = "launched term"' || fail "the program of the launcher is no pane: $(panes)"
ok "the launcher showed without a pane, and its program became the pane launched"
until_ok 'test "$(focused id:construct-look)" = True' || fail "the walk has no keyboard after the launcher"
ok "the walk has the keyboard again"

before=$(panes)
key x
until_ok 'test "$(panes)" != "$before"' || fail "x closed no pane: $(panes)"
ok "x closed a pane: $before -> $(panes)"

# pane.remove only asks the window to close, as x does: a window that
# refuses, as chora-probe does, stays a pane, and shows.
"$bin/chora-probe" window 200 40 40 &
pids+=($!)
until_ok 'panes | grep -qw chora-probe' || fail "chora-probe became no pane: $(panes)"
ctl call construct pane.remove '{"name": "chora-probe"}' >/dev/null || fail "pane.remove"
sleep 1
panes | grep -qw chora-probe || fail "pane.remove dropped the pane of a window that stays: $(panes)"
ok "pane.remove asked chora-probe to close; it refused and stays a pane"

# A radius of settings.set leashes at once, without a construct file.
ctl call construct settings.set '{"radius": 1000}' >/dev/null || fail "settings.set radius"
ctl call construct pane.move '{"name": "chora-probe", "ahead": 50000}' >/dev/null || fail "pane.move"
far=$(ctl call construct construct.list | python3 -I -c '
import json, math, sys
print(round(next(math.hypot(*p["at"]) for p in json.load(sys.stdin)["panes"] if p["name"] == "chora-probe")))')
[ "$far" -le 1000 ] || fail "settings.set radius 1000 did not leash: a pane at $far"
ok "settings.set radius 1000 leashed the pane at $far"

# fuzzel, a layer-shell launcher, may pass its own token on to the
# program that it starts; that program's window is a pane all the same.
if ! command -v fuzzel >/dev/null; then
    echo "skip: fuzzel is not installed here"
    exit 0
fi
ctl call construct settings.set '{"launcher": "fuzzel"}' >/dev/null || fail "settings.set launcher"
for notify in true false; do
    key space
    sleep 2
    echo "[{\"text\": {\"text\": \"Notify$notify\"}}]" | ctl inject || fail "inject Notify$notify"
    sleep 0.5
    key Return
    until_ok "panes | grep -qw notify-$notify" || fail "the program notify-$notify of fuzzel is no pane: $(panes)"
    until_ok 'test "$(focused id:construct-look)" = True' || fail "the walk has no keyboard after fuzzel"
done
ok "the programs that fuzzel started became panes, with or without the token of fuzzel"
