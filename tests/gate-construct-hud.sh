#!/bin/bash
# Gate of step K3 of the construct track: the voice HUD (F28), the pane
# label and the restyled window list (docs/construct-style.md). A fake
# voice agent (tests/fake-voice.py) emits the events of the real one.
#
# For each theme and for 1920x1080 and 1280x720:
# 1. The construct claims the indicator of voice once.
# 2. Each state shows its label in construct-hud-state.
# 3. voice.say of 800 ms shows its sentence; 3.1 s later it is gone.
# 4. A question with a detail of 6 KiB shows it whole; a done of another
#    id keeps it, of its id closes it.
# 5. A restart of the fake voice gets a second claim, and the HUD hides
#    while voice is away.
# 6. Flown into a pane, the HUD sits at the bottom right; flown out, at
#    the bottom centre again.
# Then, with CHORA_FRAME_BUDGET_US=1, the watchdog takes the shaders: the
# HUD panel and its label stay, and the glow comes back with no shader.
# Last, with a pane that fills the view, a new window waits in the window
# list with a notice in the HUD, and window.show makes its pane.
#
# Usage: tests/gate-construct-hud.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the captures as
#          construct-hud-THEME-SIZE-CASE.png
#   SIZES  the outputs (default "1920x1080 1280x720")
#   THEMES the themes (default "light dark")
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct
need foot
need mkfifo
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
# The fake voice agent reads its events from a fifo on fd 3.
voice_start() {
    rm -f "$work/voice.in" "$work/claims"
    mkfifo "$work/voice.in"
    python3 -I "$repo/tests/fake-voice.py" "$work/claims" <"$work/voice.in" &
    voice_pid=$!
    pids+=("$voice_pid")
    exec 3>"$work/voice.in"
}
voice_stop() {
    exec 3>&-
    # The programs that the gate started hold the fifo too: no EOF.
    kill "$voice_pid" 2>/dev/null
    wait "$voice_pid" 2>/dev/null
}
emit() { echo "$1 $2" >&3; }
claims() { cat "$work/claims" 2>/dev/null || echo 0; }
state() {
    emit voice.state "{\"state\": \"$1\", \"task\": \"Look up the train to Basel\", \"said\": \"\"}"
}
say() { emit voice.say "{\"state\": \"speaking\", \"text\": \"$1\", \"duration_ms\": $2}"; }
label() { node id:construct-hud-state 'n[0]["text"]'; }
# The left edge and the width of the HUD panel on the output.
hud_x() { node id:construct-hud-glass 'round(list(n[0]["rects"].values())[0][0])'; }
foot_pane() {
    foot -a "$1" -f "$font" -o "colors-dark.background=$2" -o "colors-dark.foreground=$3" \
        sh -c "printf '%s\n' '$4'; sleep 600" &
    pids+=($!)
    until_ok "ctl call construct construct.list | grep -q '\"name\": \"$1\"'" ||
        fail "$1 became no pane"
}
sentence='The next train to Basel leaves at 14:32 from platform 7. It takes 54 minutes, and there is a dining car on board.'
detail=$(python3 -I -c '
for k in range(96):
    print(f"{k:03}  rm -rf /home/user/projects/build-cache/artifact-{k:03}/objects/deps  # 4.2 MiB")' | head -c 6144)

round() {
    local theme=$1 size=$2 w
    # shellcheck disable=SC2034 # until_ok reads it
    w=${size%x*}
    local tag=$theme-$size
    printf 'theme: %s\n' "$theme" >"$work/chora/chora-construct.yaml"
    CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000} session_start "$size" -- "$bin/chora-construct"
    local session=${pids[-1]}
    grant screen.capture input.inject
    ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"

    voice_start
    until_ok 'test "$(claims)" = 1' || fail "$tag: no claim of the indicator"
    for s in asleep listening thinking working question speaking dreaming; do
        state "$s"
        # shellcheck disable=SC2034 # until_ok reads it
        want=${s^^}
        until_ok 'test "$(label)" = "$want"' || fail "$tag: $s shows $(label)"
        case $s in listening | thinking) sleep 0.4 && shot "$shots/construct-hud-$tag-$s.png" ;; esac
    done
    sleep 0.5
    [ "$(claims)" = 1 ] || fail "$tag: $(claims) claims, not one"
    ok "$tag: one claim, and each state names itself"

    state speaking
    say "$sentence" 800
    until_ok 'test "$(node id:construct-hud-say "n[0][\"visible\"]")" = True' ||
        fail "$tag: the sentence does not show"
    shot "$shots/construct-hud-$tag-speaking.png"
    sleep 3.1 # 0.8 s of sound, 2 s more, a fade of 0.12 s
    gone=$(node id:construct-hud-say 'n[0]["opacity"] == 0 or not n[0]["visible"]')
    [ "$gone" = True ] || fail "$tag: the sentence still shows 3.1 s after its start: $(node id:construct-hud-say "n[0][\"text\"], n[0][\"opacity\"], n[0][\"visible\"]")"
    ok "$tag: the sentence showed and went"

    foot_pane white ffffff 14181e 'a white pane'
    call pane.move '{"name": "white", "ahead": -1000}'
    say "$sentence" 6000
    sleep 0.6
    shot "$shots/construct-hud-$tag-speaking-white.png"
    call pane.move '{"name": "white", "ahead": 1000, "right": 3000}'
    foot_pane black 000000 eceef2 'a black pane'
    call pane.move '{"name": "black", "ahead": -1000}'
    say "$sentence" 6000
    sleep 0.6
    shot "$shots/construct-hud-$tag-speaking-black.png"

    state question
    emit voice.confirm "$(python3 -I -c 'import json, sys; print(json.dumps({"id": 41, "prompt": "Delete the build cache of 96 projects?", "detail": sys.argv[1]}))' "$detail")"
    until_ok 'test -n "$(node id:construct-ask-hint "n[0][\"id\"]")"' || fail "$tag: no question"
    joined=$(ctl query owner:construct | python3 -I -c '
import json, sys
nodes = {n["id"]: n["text"] for n in json.load(sys.stdin) if n["id"].startswith("construct-ask-detail-")}
print("\n".join(nodes[f"construct-ask-detail-{k}"] for k in range(len(nodes))))')
    [ "$joined" = "$detail" ] || fail "$tag: the detail lost bytes"
    sleep 0.4
    shot "$shots/construct-hud-$tag-question.png"
    emit voice.confirm '{"id": 40, "done": true}'
    sleep 0.5
    [ -n "$(node id:construct-ask 'n[0]["id"]')" ] || fail "$tag: a done of another id closed the question"
    emit voice.confirm '{"id": 41, "done": true}'
    until_ok 'test -z "$(node id:construct-ask "n[0][\"id\"]")"' || fail "$tag: the done did not close it"
    ok "$tag: the question showed its whole detail until its done"

    state listening
    key Tab
    until_ok 'test "$(node id:construct-list "n[0][\"opacity\"]")" = 1.0' || fail "$tag: Tab opened no list"
    shot "$shots/construct-hud-$tag-list.png"
    key Escape
    ok "$tag: the window list shows over a pane"

    # Two small panes side by side: the label of the pane ahead, then of
    # the pane under the pointer.
    call pane.move '{"name": "black", "ahead": 1000}'
    call pane.resize '{"name": "black", "scale": 0.5}'
    call pane.resize '{"name": "white", "scale": 0.5}'
    call pane.move '{"name": "white", "right": -2150}'
    sleep 0.8
    [ "$(node id:construct-label-text 'n[0]["text"]')" = BLACK ] || fail "$tag: the pane ahead has no label"
    shot "$shots/construct-hud-$tag-label-ahead.png"
    # The core tells the pointer only from a press on: a short drag that
    # turns there and back ends over the white pane.
    read -r px py < <(node "id:construct-pane-$(node 'kind:surface' 'next(x["id"] for x in n if x["surface"].get("app_id") == "white")')" \
        '"%d %d" % (lambda r: (r[0] + r[2] / 2, r[1] + r[3] / 2))(list(n[0]["rects"].values())[0])')
    echo "[{\"pointer_move\": {\"position\": [$px, $py]}}, {\"button\": {\"state\": \"pressed\"}},
      {\"pointer_move\": {\"position\": [$((px + 20)), $py]}}, {\"pointer_move\": {\"position\": [$px, $py]}},
      {\"button\": {\"state\": \"released\"}}]" | ctl inject || fail "inject drag"
    until_ok 'test "$(node id:construct-label-text "n[0][\"text\"]")" = WHITE' ||
        fail "$tag: the pane under the pointer has no label"
    sleep 0.3
    shot "$shots/construct-hud-$tag-label-pointer.png"
    ok "$tag: the label names the pane ahead, then the pane under the pointer"

    centre=$(hud_x)
    call pane.fly '{"name": "black"}'
    say "Done. The cache is gone." 6000
    until_ok 'test "$(hud_x)" -gt "$((w / 2))"' || fail "$tag: flown in, the HUD stays at $(hud_x)"
    sleep 0.5
    shot "$shots/construct-hud-$tag-flown.png"
    key Alt+Escape
    until_ok 'test "$(hud_x)" -lt "$((w / 2))"' || fail "$tag: flown out, the HUD stays at $(hud_x)"
    ok "$tag: flown in, the HUD moved from $centre to the bottom right, and back"

    voice_stop
    until_ok 'test "$(node id:construct-hud "n[0][\"visible\"]")" = False' || fail "$tag: the HUD stays without voice"
    voice_start
    until_ok 'test "$(claims)" = 1' || fail "$tag: no claim after the restart of voice"
    ok "$tag: the restarted voice got its claim"
    voice_stop
    kill "$session"
    wait "$session" 2>/dev/null
}

for theme in ${THEMES:-light dark}; do
    for size in ${SIZES:-1920x1080 1280x720}; do
        round "$theme" "$size"
    done
done

# The watchdog takes the shaders of a client that makes the frames slow.
for theme in ${THEMES:-light dark}; do
    printf 'theme: %s\n' "$theme" >"$work/chora/chora-construct.yaml"
    CHORA_FRAME_BUDGET_US=1 session_start 1280x720 -- "$bin/chora-construct"
    session=${pids[-1]}
    grant screen.capture input.inject
    ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
    voice_start
    state thinking
    until_ok 'test "$(node id:construct-fog "n[0].get(\"shader\")")" = None' 80 ||
        fail "$theme: the watchdog took no shader"
    state working
    until_ok 'test "$(label)" = WORKING' || fail "$theme: plain, the label does not follow"
    [ -n "$(node id:construct-hud-glass 'n[0]["rects"] or ""')" ] || fail "$theme: plain, no HUD panel"
    [ "$(node id:construct-hud-glow 'n[0].get("shader")')" = None ] || fail "$theme: a glass.frag came back"
    sleep 0.5
    shot "$shots/construct-hud-$theme-1280x720-plain.png"
    ok "$theme: after the watchdog, the HUD stays plain and names its state"
    voice_stop
    kill "$session"
    wait "$session" 2>/dev/null
done

# The focused pane (rule 16 of the guide): the pane ahead shows clear,
# with an accent hairline and a soft glow under it; two others beside.
for theme in ${THEMES:-light dark}; do
    printf 'theme: %s\n' "$theme" >"$work/chora/chora-construct.yaml"
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
printf 'theme: light\n' >"$work/chora/chora-construct.yaml"
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
[ "$(label)" = NOTICE ] || fail "the HUD gives no notice: $(label)"
sleep 0.3
shot "$shots/construct-hud-light-1280x720-noroom.png"
call window.show '{"name": "third"}'
until_ok 'ctl call construct construct.list | grep -q "\"name\": \"third\""' || fail "window.show made no pane"
ok "with the view full, a new window waited in the list, and window.show made its pane"
kill "$session"
wait "$session" 2>/dev/null
