#!/bin/bash
# The key list of tiling (tiling-v2, item 6): Super+/ shows every key with
# its words, built from the bindings, so a binding added to the file shows
# after a reload, with its label. Esc closes the list and does not reach
# the window; afterwards Esc reaches the window again.
#
# Usage: tests/gate-tiling-keys.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
need_bins chora choractl chora-tiling
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
grant input.inject
log=$XDG_CONFIG_HOME/typed
ctl run foot sh -c "stty raw -echo; cat >$log" >/dev/null || fail "the run of foot"
for _ in $(seq 60); do
    ctl query 'app_id:foot kind:surface' | grep -q '"owner": *"tiling"' && break
    sleep 0.5
done
sleep 2
keys() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the key $1"; }
list() { ctl query 'kind:text' 2>/dev/null; }

mkdir -p "$XDG_CONFIG_HOME/chora"
"$bin/chora-tiling" --print-config |
    sed 's|^  MOD+Return: {run: foot}|  MOD+Return: {run: foot}\n  MOD+x: {run: "echo hi"}|; s|^  MOD+a: Chat with a helper|  MOD+a: Chat with a helper\n  MOD+x: Say hello|' >"$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
ctl call tiling settings.reload >/dev/null || fail "settings.reload"

keys Super+slash
sleep 2
shown=$(list)
for word in "Next window" "Super+j" "Say hello" "Super+x" "Closes it" ; do
    case $word in "Closes it") word="Esc closes it" ;; esac
    grep -q "$word" <<<"$shown" || fail "the key list does not show '$word'"
done
ok "Super+/ shows the keys with their words, and a new binding with its label"

keys Escape
sleep 1
grep -q "Next window" <<<"$(list)" && fail "Esc did not close the key list"
[ ! -s "$log" ] || fail "Esc reached the window while the list was open"
keys Escape
sleep 1
[ "$(od -An -c "$log" | tr -d ' ')" = '033' ] || fail "Esc after the list did not reach the window: $(od -An -c "$log")"
ok "Esc closes the list, and then the keys are the window's again"
