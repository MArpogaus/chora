#!/bin/bash
# Moving the focus in tiling (persona round 5, L1): Alt+Tab moves the focus
# to the next window, as Super+j does. Two foot windows; the text typed
# after the key lands in the window that was not typed to before it. The
# key list names the focus keys first, in the words "Move focus".
#
# Usage: tests/gate-tiling-focus-key.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
need_bins chora choractl chora-tiling
temp_config
session_start 1280x800 -- "$bin/chora-tiling" --no-companions
grant input.inject
for n in 1 2; do
    ctl run foot sh -c "stty raw -echo; cat >$XDG_CONFIG_HOME/typed$n" >/dev/null || fail "the run of foot $n"
    for _ in $(seq 60); do
        seen=$(ctl query 'app_id:foot kind:surface') || { sleep 0.5; continue; }
        [ "$(grep -c '"owner": *"tiling"' <<<"$seen")" = "$n" ] && break
        sleep 0.5
    done
done
sleep 2
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the key $1"; }
type_text() { echo "[{\"text\": {\"text\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the text $1"; }
typed() { cat "$XDG_CONFIG_HOME/typed$1" 2>/dev/null; }

type_text a
sleep 1
first=$(for n in 1 2; do [ "$(typed $n)" = a ] && echo $n; done)
[ "${#first}" = 1 ] || fail "the first text 'a' landed in no single window: '$(typed 1)' '$(typed 2)'"
other=$((3 - first))
key Alt+Tab
sleep 1
type_text b
sleep 1
[ "$(typed "$other")" = b ] || fail "after Alt+Tab the other window did not get the text: '$(typed "$other")'"
[ "$(typed "$first")" = a ] || fail "after Alt+Tab the first window got more text: '$(typed "$first")'"
ok "Alt+Tab moves the focus to the other of two windows"

key Alt+Tab
sleep 1
type_text c
sleep 1
[ "$(typed "$first")" = ac ] || fail "the second Alt+Tab did not move the focus back: '$(typed "$first")'"
ok "Alt+Tab again moves the focus back"

key Super+slash
sleep 2
shown=$(ctl query 'kind:text' 2>/dev/null)
python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
words = {n["id"]: n["text"] for n in nodes if n["id"].startswith("tiling-keys") and n.get("text")}
labels = [t for i, t in words.items() if i.endswith("-l")]
caps = [t for i, t in words.items() if i.endswith("-k")]
if "Move focus" not in labels:
    sys.exit("no row \"Move focus\": %r" % labels)
row = next(i for i, t in words.items() if t == "Move focus")[:-2]
cap = words[row + "-k"]
for key in ("Super+J", "Super+K", "Alt+Tab"):
    if key not in cap:
        sys.exit("the row Move focus has the keys %r, not %s" % (cap, key))
' <<<"$shown" || fail "the key list does not show the focus keys in plain words"
ok "the key list shows Move focus with Super+J, Super+K and Alt+Tab"
