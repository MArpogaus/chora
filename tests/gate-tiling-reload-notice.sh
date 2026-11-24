#!/bin/bash
# A reload that fails says so (ti-53, honest failure states): with a broken
# line in chora-tiling.yaml (`  gap_inner 20`, no colon) and with a wrong
# action name, MOD+Ctrl+r shows a notice of chora-notify that names the
# file (and the line, where the parser knows it), the card stands at the
# bottom right, and the settings of the last good reload stay in force:
# gap_inner is still 30. A valid file still reloads (gap_inner 30).
#
# Usage: tests/gate-tiling-reload-notice.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need jq
need dbus-run-session
need_bins chora choractl chora-tiling chora-notify
export PATH=$bin:$PATH
temp_config
home=$XDG_CONFIG_HOME/home
mkdir -p "$home" "$XDG_CONFIG_HOME/chora"
file=$XDG_CONFIG_HOME/chora/chora-tiling.yaml
CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x800 -- "$bin/chora-tiling" --no-companions
grant screen.capture methods.call input.inject
HOME=$home LC_ALL=C setsid dbus-run-session -- sh -c 'chora-notify' >"$XDG_CONFIG_HOME/notify.log" 2>&1 &
run_pid $!
until_ok 'ctl call notify state.get >/dev/null 2>&1' 40 || fail "chora-notify does not answer: $(tail -n 3 "$XDG_CONFIG_HOME/notify.log")"
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null; }
gap_inner() { ctl call tiling get_layout 2>/dev/null | jq -r .gap_inner; }
shot=$XDG_CONFIG_HOME/notice.png
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
before=$(pixel "$shot" 1000 700)

"$bin/chora-tiling" --print-config >"$file"
sed -i 's/^  gap_inner: 12/  gap_inner: 30/' "$file"
key Super+Ctrl+r
gap_is_30() { [ "$(gap_inner)" = 30 ]; }
until_ok gap_is_30 || fail "a valid file did not reload: gap_inner is $(gap_inner)"
ok "a valid file reloads: gap_inner 30"
[ -z "$(shown_notices)" ] || fail "a valid reload shows a notice: $(shown_notices)"

broken() {
    local name=$1 line=$2 pattern=$3
    dismiss_notices
    cp "$file" "$file.good"
    case $name in
    line) sed -i "s/^  radius: 10/  radius: 10\n$line/" "$file" ;;
    action) sed -i "s/^  MOD+Ctrl+r: reload/  MOD+Ctrl+r: reload\n  $line/" "$file" ;;
    esac
    key Super+Ctrl+r
    until_ok 'shown_notices | grep -q chora-tiling.yaml' 24 || fail "$name: no notice names the file: [$(shown_notices)]; $(ctl call notify state.get 2>&1 | head -c 300)"
    shown_notices | grep -Eq "$pattern" || fail "$name: the notice does not say $pattern: $(shown_notices)"
    ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
    [ "$(pixel "$shot" 1000 700)" != "$before" ] || fail "$name: no card at the bottom right"
    [ "$(gap_inner)" = 30 ] || fail "$name: the old settings are not in force: gap_inner is $(gap_inner)"
    ok "$name: the notice names the file and the reason, the old settings stay"
    mv "$file.good" "$file"
}
broken line '  gap_inner 20' 'line [0-9]+'
broken action 'MOD+Shift+c: frobnicate' 'frobnicate|unknown'
