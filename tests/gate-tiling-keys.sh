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
    seen=$(ctl query 'app_id:foot kind:surface') || { sleep 0.5; continue; }
    grep -q '"owner": *"tiling"' <<<"$seen" && break
    sleep 0.5
done
sleep 2
keys() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the key $1"; }
list() { ctl query 'kind:text' 2>/dev/null; }

mkdir -p "$XDG_CONFIG_HOME/chora"
"$bin/chora-tiling" --print-config |
    sed 's|^  MOD+Return: {run: [^}]*}|&\n  MOD+x: {run: "echo hi"}|; s|^  MOD+a: Chat with a helper|  MOD+a: Chat with a helper\n  MOD+x: Say hello|' >"$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
ctl call tiling settings.reload >/dev/null || fail "settings.reload"

keys Super+slash
sleep 2
shown=$(list)
for word in "Next window" "Super+j" "Say hello" "Super+x" "Closes it" ; do
    case $word in "Closes it") word="Esc closes it" ;; esac
    grep -q "$word" <<<"$shown" || fail "the key list does not show '$word'"
done
ok "Super+/ shows the keys with their words, and a new binding with its label"

list | python3 -I -c '
import json, sys
nodes = [n for n in json.load(sys.stdin) if n["id"].startswith("tiling-keys")]
rect = lambda n: next(iter(n["rects"].values()))
def fail(why):
    sys.exit(why)
boxes = [(n["text"], rect(n)) for n in nodes if n.get("rects")]
for i, (a, [ax, ay, aw, ah]) in enumerate(boxes):
    for b, [bx, by, bw, bh] in boxes[i + 1:]:
        if ax < bx + bw - 1 and bx < ax + aw - 1 and ay < by + bh - 1 and by < ay + ah - 1:
            fail(f"the texts {a!r} and {b!r} overlap")
mono = [n for n in nodes if n["id"].endswith("-k")]
if not mono:
    fail("no keycap text")
for n in mono:
    got, want = rect(n)[2], len(n["text"]) * 7.8
    if abs(got - want) > 1.5:
        fail("the keycap %r is %s wide, a monospace face has %s" % (n["text"], got, want))
' || fail "the key list is drawn badly"
ok "no two texts of the key list overlap and the keycaps are monospace"
rects=$(ctl query 'kind:rect') || fail "ctl query failed"
python3 -I -c '
import json, sys
panel = next(next(iter(n["rects"].values())) for n in json.load(sys.stdin) if n["id"] == "tiling-keys-panel")
if panel[1] < 52 or panel[1] + panel[3] > 748:
    sys.exit("the panel at y %s, %s high touches the bar or the edge" % (panel[1], panel[3]))
' <<<"$rects" || fail "the key list touches the bar"
ok "the key list stays clear of the bar"

keys Escape
sleep 1
grep -q "Next window" <<<"$(list)" && fail "Esc did not close the key list"
[ ! -s "$log" ] || fail "Esc reached the window while the list was open"
keys Escape
sleep 1
[ "$(od -An -c "$log" | tr -d ' ')" = '033' ] || fail "Esc after the list did not reach the window: $(od -An -c "$log")"
ok "Esc closes the list, and then the keys are the window's again"

for _ in 1 2; do
    started=$(ctl run "$bin/chora-probe" window 255 255 255) || fail "the run of a window"
    pid=$(sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p' <<<"$started")
    [ -n "$pid" ] || fail "the run of a window"
    run_pid "$pid"
done
for _ in $(seq 60); do
    [ "$(ctl query 'app_id:chora-probe kind:surface' | grep -c '"owner": *"tiling"')" = 2 ] && break
    sleep 0.5
done
sleep 2
keys Super+slash
sleep 2
grant screen.capture
shot=$XDG_CONFIG_HOME/blur.png
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
ctl query 'kind:rect' >"$XDG_CONFIG_HOME/rects.json"
python3 -I - "$repo/tests/png-pixel.py" "$shot" "$XDG_CONFIG_HOME/rects.json" <<'PY' || fail "no blur behind the key list"
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
import json
panel = next(next(iter(n["rects"].values())) for n in json.load(open(sys.argv[3])) if n["id"] == "tiling-keys-panel")
row = int(panel[1] + panel[3] - 10)
line = [rows[row][x * 4] for x in range(600, 640)]
steps = [abs(a - b) for a, b in zip(line, line[1:])]
assert max(steps) <= 2, f"the gap between two white windows is sharp under the panel: {line}"
assert max(line) - min(line) >= 3, "the panel does not show the windows behind it at all"
PY
ok "the key list blurs the white windows and the gap behind it"
keys Escape
