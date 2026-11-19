#!/bin/bash
# Gate of step K3 of the construct track: the pane label, the restyled
# window list, the notices and the help of the construct
# (docs/construct-style.md). The HUD host chora-assistant draws the voice
# HUD (F28) and the questions; tests/gate-hud.sh checks that the
# construct draws none.
#
# For each theme and for 1920x1080 and 1280x720, with the flag
# --test-injected-press-is-real of the core, so that the injected drag
# counts as a drag of the person:
# 1. Tab opens the window list over a pane, and Escape closes it.
# 2. The label names the pane ahead, then the pane under the pointer.
# 3. Long German words stay inside their panels: the rows of the window
#    list and the pane label.
# Then, with CHORA_FRAME_BUDGET_US=1, the watchdog takes the shaders and
# the space stays plain. The focused pane has its edge and glow.
# Last, with a pane that fills the view, a new window waits in the window
# list with a notice under the island, and window.show makes its pane.
# Then Super+F1 shows the help panel with the bindings; settings.set of the
# modifier changes it at once, and help.toggle hides it.
#
# Usage: tests/gate-construct-hud.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the captures as
#          construct-hud-THEME-SIZE-CASE.png
#   SIZES  the outputs (default "1920x1080 1280x720 1920x1080@1.5
#          1280x720@1.5")
#   THEMES the themes (default "light dark")
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct
need foot
temp_config
work=$XDG_CONFIG_HOME
shots=${SHOTS:-$work}
mkdir -p "$work/chora" "$shots"
export PATH=$bin:$PATH
font='DejaVu Sans Mono:size=11'

shot() { ctl capture "$@" >/dev/null || fail "capture $*"; }
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject || fail "inject $1"; }
call() { ctl call construct "$@" >/dev/null || fail "construct $*"; }
# node SELECTOR EXPR: EXPR of the first node n[0], as python sees it.
node() { ctl query "$1" | python3 -I -c "import json, sys; n = json.load(sys.stdin); print($2 if n else '')"; }
until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
# fits TEXT GLASS: the rect of the text node lies inside the rect of the
# glass node, on the output.
fits() {
    ctl query owner:construct | python3 -I -c "
import json, sys
nodes = {n['id']: n for n in json.load(sys.stdin)}
def rect(i):
    r = list(nodes[i]['rects'].values())
    return r[0] if r else None
t, g = rect('$1'), rect('$2')
ok = t and g and t[0] >= g[0] - 0.5 and t[1] >= g[1] - 0.5 and t[0] + t[2] <= g[0] + g[2] + 0.5 and t[1] + t[3] <= g[1] + g[3] + 0.5
sys.exit(not ok and print(f'$1 {t} not inside $2 {g}', file=sys.stderr) is None)"
}
long=Donaudampfschifffahrtsgesellschaftskapitänsmützenabzeichen
foot_pane() {
    foot -a "$1" -T "$4" -f "$font" -o "colors-dark.background=$2" -o "colors-dark.foreground=$3" \
        sh -c "printf '%s\n' '$4'; sleep 600" &
    pids+=($!)
    until_ok "ctl call construct construct.list | grep -q '\"name\": \"$1\"'" ||
        fail "$1 became no pane"
}

round() {
    local theme=$1 size=$2
    local tag=$theme-$size
    printf 'theme: %s\nvoice_hud: host\n' "$theme" >"$work/chora/chora-construct.yaml"
    CHORA_TEST_FLAGS=--test-injected-press-is-real \
        CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000} session_start "$size" -- "$bin/chora-construct"
    local session=${pids[-1]}
    grant screen.capture input.inject
    ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"

    foot_pane white ffffff 14181e 'a white pane'
    call pane.move '{"name": "white", "ahead": 1000, "right": 3000}'
    foot_pane black 000000 eceef2 'a black pane'
    call pane.move '{"name": "black", "ahead": -1000}'

    key Tab
    until_ok 'test "$(node id:construct-list "n[0][\"opacity\"]")" = 1.0' || fail "$tag: Tab opened no list"
    shot "$shots/construct-hud-$tag-list.png"
    key Escape
    until_ok 'test "$(node id:construct-list "n[0][\"opacity\"]")" = 0.0' || fail "$tag: Escape did not close the list"
    ok "$tag: the window list shows over a pane"

    # Two small panes side by side: the label of the pane ahead, then of
    # the pane under the pointer.
    call pane.move '{"name": "black", "ahead": 1000}'
    call pane.resize '{"name": "black", "scale": 0.5}'
    call pane.resize '{"name": "white", "scale": 0.5}'
    call pane.move '{"name": "white", "right": -2150}'
    until_ok 'test "$(node id:construct-label-body-text "n[0][\"text\"]")" = "a black pane"' ||
        fail "$tag: the pane ahead has no label: $(node id:construct-label-body-text 'n[0]["text"]')"
    shot "$shots/construct-hud-$tag-label-ahead.png"
    # The core tells the pointer only from a press on: a short drag that
    # turns there and back ends over the white pane.
    read -r px py < <(node "id:construct-pane-$(node 'kind:surface' 'next(x["id"] for x in n if x["surface"].get("app_id") == "white")')" \
        '"%d %d" % (lambda r: (r[0] + r[2] / 2, r[1] + r[3] / 2))(list(n[0]["rects"].values())[0])')
    echo "[{\"pointer_move\": {\"position\": [$px, $py]}}, {\"button\": {\"state\": \"pressed\"}},
      {\"pointer_move\": {\"position\": [$((px + 20)), $py]}}, {\"pointer_move\": {\"position\": [$px, $py]}},
      {\"button\": {\"state\": \"released\"}}]" | ctl inject || fail "inject drag"
    until_ok 'test "$(node id:construct-label-body-text "n[0][\"text\"]")" = "a white pane"' ||
        fail "$tag: the pane under the pointer has no label: $(node id:construct-label-body-text 'n[0]["text"]')"
    sleep 0.3
    shot "$shots/construct-hud-$tag-label-pointer.png"
    ok "$tag: the label names the pane ahead, then the pane under the pointer"

    call pane.fly '{"name": "black"}'
    until_ok 'ctl call construct construct.list | python3 -I -c "import json, sys; c = json.load(sys.stdin)[\"camera\"]; sys.exit(c[\"z\"] == 0.0 and c[\"x\"] == 0.0)"' ||
        fail "$tag: no flight into the black pane"
    sleep 0.5
    key Alt+Escape
    until_ok 'ctl call construct construct.list | python3 -I -c "import json, sys; c = json.load(sys.stdin)[\"camera\"]; sys.exit(not (c[\"z\"] == 0.0 and c[\"x\"] == 0.0))"' ||
        fail "$tag: no flight back out of the black pane"

    foot -a "$long-$long" -f "$font" sh -c 'sleep 600' &
    pids+=($!)
    until_ok "ctl call construct window.list | grep -q Donaudampf" || fail "$tag: no long window"
    sleep 0.5
    fits construct-label-body-text construct-label-body-glass || fail "$tag: the label runs out of its pill"
    shot "$shots/construct-hud-overflow-$tag-label.png"
    # Behind the camera, its pane is a row of the list.
    long_pane=$(ctl call construct construct.list | python3 -I -c 'import json, sys; print(next(p["name"] for p in json.load(sys.stdin)["panes"] if p["name"].startswith("Donau")))')
    call pane.move "{\"name\": \"$long_pane\", \"ahead\": -4000}"
    sleep 2
    echo '[{"pointer_move": {"position": [4, 300]}}]' | ctl inject || fail "inject pointer"
    sleep 0.5
    key Tab
    until_ok 'test "$(node id:construct-list "n[0][\"opacity\"]")" = 1.0' || fail "$tag: Tab opened no list: $(ctl call construct construct.list | tr -d "\n " | cut -c1-900) $(ctl call construct window.list "{}" | tr -d "\n " | cut -c1-500)"
    for row in $(ctl query owner:construct | python3 -I -c 'import json, sys; print(*[n["id"] for n in json.load(sys.stdin) if n["id"].startswith("construct-list-row-") and n["id"].endswith(("-name", "-note"))])'); do
        fits "$row" construct-list-body-glass || fail "$tag: $row runs out of the list"
    done
    shot "$shots/construct-hud-overflow-$tag-list.png"
    key Escape
    ok "$tag: long words stay inside the list and the label"
    kill "$session"
    wait "$session" 2>/dev/null
}

for theme in ${THEMES:-light dark}; do
    for size in ${SIZES:-1920x1080 1280x720 1920x1080@1.5 1280x720@1.5}; do
        round "$theme" "$size"
    done
done

# The watchdog takes the shaders of a client that makes the frames slow.
for theme in ${THEMES:-light dark}; do
    printf 'theme: %s\nvoice_hud: host\n' "$theme" >"$work/chora/chora-construct.yaml"
    CHORA_FRAME_BUDGET_US=1 session_start 1280x720 -- "$bin/chora-construct"
    session=${pids[-1]}
    grant screen.capture input.inject
    ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
    until_ok 'test "$(node id:construct-fog "n[0].get(\"shader\")")" = None' 80 ||
        fail "$theme: the watchdog took no shader"
    sleep 0.5
    shot "$shots/construct-hud-$theme-1280x720-plain.png"
    ok "$theme: after the watchdog, the space stays plain"
    kill "$session"
    wait "$session" 2>/dev/null
done

# The focused pane (rule 16 of the guide): the pane ahead shows clear,
# with an accent hairline and a soft glow under it; two others beside.
for theme in ${THEMES:-light dark}; do
    printf 'theme: %s\nvoice_hud: host\n' "$theme" >"$work/chora/chora-construct.yaml"
    CHORA_FRAME_BUDGET_US=500000 session_start 1280x720 -- "$bin/chora-construct"
    session=${pids[-1]}
    grant screen.capture input.inject
    ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
    for f in one two three; do
        foot_pane "$f" 1e2430 eceef2 "pane $f"
        call pane.resize "{\"name\": \"$f\", \"scale\": 0.5}"
    done
    # The camera looks along +z: right is +x, ahead +z.
    for spot in 'one 0 2000' 'two -800 2700' 'three 800 2700'; do
        read -r f x z <<<"$spot"
        ctl call construct construct.list | python3 -I -c '
import json, sys
f, x, z = sys.argv[1], float(sys.argv[2]), float(sys.argv[3])
at = next(p["at"] for p in json.load(sys.stdin)["panes"] if p["name"] == f)
print(json.dumps({"name": f, "right": x - at[0], "ahead": z - at[1]}))' "$f" "$x" "$z" >"$work/move.json"
        call pane.move "$(cat "$work/move.json")"
    done
    sleep 1
    focused=$(ctl query id:construct-focus | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(n[0]["visible"] if n else "")')
    [ "$focused" = True ] || fail "$theme: no glow under the focused pane"
    shot "$shots/construct-hud-focus-$theme.png"
    ok "$theme: the focused pane has its edge and glow"
    kill "$session"
    wait "$session" 2>/dev/null
done

# No room in view (C7): a pane fills the view, so a new window becomes no
# pane; it waits in the window list, the HUD says so, and window.show
# makes its pane. No window gets lost.
printf 'theme: light\nvoice_hud: host\n' >"$work/chora/chora-construct.yaml"
CHORA_FRAME_BUDGET_US=500000 session_start 1280x720 -- "$bin/chora-construct"
session=${pids[-1]}
grant screen.capture input.inject
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
foot_pane first ffffff 14181e 'the first pane'
foot_pane second 000000 eceef2 'the second pane'
call pane.move '{"name": "first", "ahead": -1300}'
foot -a third -f "$font" sh -c 'sleep 600' &
pids+=($!)
listed() { ctl call construct window.list | python3 -I -c 'import json, sys; print(next((w["pane"] for w in json.load(sys.stdin)["windows"] if w["name"] == "third"), "missing"))'; }
until_ok 'test "$(listed)" = none' || fail "the third window is not in the list: $(listed)"
until_ok 'node id:construct-notice-text '"'"'n[0]["text"]'"'"' | grep -q "No room in view"' || fail "no notice: $(node id:construct-notice-text 'n[0]["text"]')"
sleep 0.3
shot "$shots/construct-hud-light-1280x720-noroom.png"
call window.show '{"name": "third"}'
until_ok 'ctl call construct construct.list | grep -q "\"name\": \"third\""' || fail "window.show made no pane"
ok "with the view full, a new window waited in the list, and window.show made its pane"
kill "$session"
wait "$session" 2>/dev/null || true

helped() {
    ctl query owner:construct | python3 -I -c '
import json, sys
nodes = {n["id"]: n for n in json.load(sys.stdin)}
body = "construct-help-body-row-"
row = next((i[len(body):-len("-what")] for i, n in nodes.items() if i.startswith(body) and i.endswith("-what") and n.get("text") == sys.argv[1]), None)
caps = sorted((i for i in nodes if row is not None and i.startswith(f"{body}{row}-keys-") and i.endswith("-text")), key=lambda i: int(i.split("-")[-2]))
print("+".join(nodes[i]["text"] for i in caps))' "$1"
}
for theme in ${THEMES:-light dark}; do
    printf 'theme: %s\nvoice_hud: host\n' "$theme" >"$work/chora/chora-construct.yaml"
    CHORA_FRAME_BUDGET_US=500000 session_start 1280x720 -- "$bin/chora-construct"
    session=${pids[-1]}
    grant screen.capture input.inject
    ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
    foot_pane white ffffff 14181e 'a white pane'
    key Super+F1
    until_ok 'test "$(node id:construct-help "n[0][\"opacity\"]")" = 1.0' || fail "$theme: Super+F1 shows no help"
    [ "$(helped 'Move or resize a pane')" = Super ] || fail "$theme: the row of the modifier is $(helped 'Move or resize a pane')"
    [ "$(helped 'Show these keys')" = Super+F1 ] || fail "$theme: the key of the help is $(helped 'Show these keys')"
    for t in $(ctl query owner:construct | python3 -I -c 'import json, sys; print(*[n["id"] for n in json.load(sys.stdin) if n["id"].startswith("construct-help-body-row-") and n["id"].endswith("-what")])'); do
        fits "$t" construct-help-body-glass || fail "$theme: $t runs out of the help"
    done
    shot "$shots/construct-help-$theme.png"
    ctl call construct settings.set '{"modifier": "Alt"}' >/dev/null || fail "settings.set modifier"
    until_ok 'test "$(helped "Move or resize a pane")" = Alt' || fail "$theme: the help keeps $(helped "Move or resize a pane")"
    shot "$shots/construct-help-$theme-alt.png"
    [ "$(ctl call construct help.toggle | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["shown"])')" = False ] ||
        fail "$theme: help.toggle did not hide the help"
    until_ok 'test "$(node id:construct-help "n[0][\"visible\"]")" = False' || fail "$theme: the help stays"
    ok "$theme: Super+F1 shows the bindings, a new modifier shows at once, and help.toggle hides them"
    kill "$session"
    wait "$session" 2>/dev/null || true
done
