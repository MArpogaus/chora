#!/bin/bash
# W13 construct: chora-construct as an overlay over tiling and as a
# flavor. Panes, fly in and out, the window list (Tab), F1 help, adopt
# and give back, a nested `chora -- chora-tiling` pane, and kill -9 of a
# program, of a flown-in program and of chora-construct. The hard rule:
# no window is ever lost; no_lost runs after every step.
WF=w13; STOP_ON_DEATH=1; . "$(dirname "$0")/lib.sh"; . "$(dirname "$0")/lib-wm.sh"
config_home
trap 'end_foots 2>/dev/null; session_stop' EXIT
wm_session 1280x800 || exit 1
rss_note start
c0=$(rss chora)
cat > "$XDG_CONFIG_HOME/chora/constructs/empty.yaml" <<EOF
panes: []
EOF
cat > "$XDG_CONFIG_HOME/chora/constructs/own.yaml" <<EOF
panes:
- kind: program
  command: sh $work/foot.sh o1
  size: [640, 400]
- kind: program
  command: sh $work/foot.sh o2
EOF
cat > "$XDG_CONFIG_HOME/chora/constructs/flav.yaml" <<EOF
radius: 6000
panes:
$(for f in f1 f2 f3 f4 f5; do printf -- '- kind: program\n  command: sh %s/foot.sh %s\n  size: [640, 400]\n' "$work" "$f"; done)
EOF

# --- 1. overlay over tiling -------------------------------------------
bg "$bin/chora-tiling"
ctl wait --timeout 10 owner:tiling > /dev/null || finding "W13: tiling does not start" breaks "chora-tiling"
foot_bg t1
expect "t1 did not show" breaks 'test -n "$(surface t1)"' 15
foot_bg t2
expect "tiling took not both foots" breaks 'test "$(ctl query "kind:surface owner:tiling" | grep -c app_id)" = 2' 15
sleep 1
t1=$(surface t1); t2=$(surface t2)
before=$(geometry)
no_lost "tiling alone"
bg "$bin/chora-construct" --overlay
construct_pid=$bgpid
expect "the overlay does not answer" breaks 'call construct.list >/dev/null 2>&1' 15
call construct.load '{"name": "empty"}' > /dev/null || finding "W13: construct.load empty refused" breaks "construct.load"
ctl wait --timeout 10 id:construct-look > /dev/null || finding "W13: overlay shows no space" breaks "construct.load empty"
sleep 1
[ -z "$(panes)" ] || finding "W13: the overlay load made panes for tiling windows: $(panes)" endangers "overlay empty load"
no_lost "overlay load"
shot_wm overlay-loaded

# window list with Tab
key Tab
expect "Tab opened no window list" breaks 'ctl query "id:construct-list meta:state=open" | grep -q construct-list' 5 && key Down
sleep 0.5
shot_wm overlay-list
key Escape
expect "Escape left the window list open" nit '! ctl query "id:construct-list meta:state=open" | grep -q construct-list'
no_lost "overlay list"

# F1 help
key F1
expect "F1 shows no help in the overlay" breaks 'test "$(ctl query id:construct-help | py "import json, sys; n = json.load(sys.stdin); print(n[0][\"opacity\"] if n else 0)")" = 1.0' 5
shot_wm overlay-help
call help.toggle > /dev/null
no_lost "overlay help"

# fly in (window.show) and out
call window.show '{"name": "t2"}' > /dev/null || finding "W13: window.show t2 refused" breaks "window.show"
expect "t2 became no pane" breaks 'test "$(panes)" = t2'
call pane.move '{"name": "t2", "ahead": -4000}' > /dev/null
call window.show '{"name": "t1"}' > /dev/null || finding "W13: window.show t1 refused" breaks "window.show"
sleep 1.5
type_text "hello t1"
expect "t1 got no text after the fly-in: $(cat "$work/t1.typed" 2>/dev/null)" breaks 'test "$(cat "$work/t1.typed" 2>/dev/null)" = "hello t1"'
no_lost "overlay flown in"
shot_wm overlay-flown
key Alt+Escape
sleep 1
no_lost "overlay flown out"

# adopt and give back
call construct.leave > /dev/null || finding "W13: construct.leave refused" breaks "construct.leave"
ctl wait --gone --timeout 10 owner:construct > /dev/null || finding "W13: construct nodes stay after leave" endangers "construct.leave"
sleep 1
after=$(geometry)
[ "$after" = "$before" ] || finding "W13: leave changed the windows: before [$before] after [$after]" endangers "construct.leave"
no_lost "overlay leave"
alive "overlay leave"

# own windows, kill -9 of programs
foot_bg k1
foot_bg k2
expect "no k2" breaks 'test -n "$(surface k2)"' 15
call construct.load '{"name": "own"}' > /dev/null
expect "own panes: $(panes)" breaks 'test "$(panes)" = "o1 o2"' 20
no_lost "overlay own"
call window.show '{"name": "k1"}' > /dev/null
sleep 1.5
kill9 k1
expect "the pane of the killed flown-in k1 stays: $(panes)" endangers 'test "$(panes)" = "o1 o2"'
sleep 1
echo '[{"pointer_move": {"position": [4, 4]}}, {"click": {}}]' | ctl inject > /dev/null
no_lost "overlay kill -9 flown-in program"
call window.show '{"name": "k2"}' > /dev/null
call pane.move '{"name": "k2", "ahead": -4000}' > /dev/null
sleep 1
kill9 k2
expect "the pane of the killed hidden k2 stays: $(panes)" endangers 'test "$(panes)" = "o1 o2"'
no_lost "overlay kill -9 hidden program"
kill9 o1
expect "the pane of the killed o1 stays: $(panes)" endangers 'test "$(panes)" = o2'
no_lost "overlay kill -9 own program"

# give back by pane.remove and by window.give
call window.show '{"name": "t1"}' > /dev/null
sleep 1.5
call pane.remove '{"name": "t1"}' > /dev/null
expect "pane.remove of the taken t1 did not give it back: $(owner "$t1")" breaks 'test "$(owner "$t1")" = tiling'
no_lost "overlay pane.remove taken"
call window.show '{"name": "t2"}' > /dev/null
expect "t2 became no pane" breaks 'panes | grep -qw t2'
expect "t2 is a pane but still owned by $(owner "$t2")" endangers 'test "$(owner "$t2")" = construct'
out=$(call window.give "{\"surface\": \"$t2\", \"to\": \"tiling\"}" 2>&1) || finding "W13: window.give t2 to tiling refused by the overlay: ${out:0:150}; owner $(owner "$t2"), panes: $(panes)" endangers "window.give"
expect "t2 is not with tiling after the give: $(owner "$t2")" breaks 'test "$(owner "$t2")" = tiling'
no_lost "overlay window.give"

# kill -9 of chora-construct with windows flown in and hidden
call window.show '{"name": "t1"}' > /dev/null
call window.show '{"name": "t2"}' > /dev/null
key Tab
sleep 1
kill -9 "$construct_pid"
wait "$construct_pid" 2>/dev/null
expect "construct nodes stay 20 s after kill -9" endangers 'test "$(ctl query owner:construct)" = "[]"' 20
expect "after kill -9 of the overlay: $(owner "$t1") $(owner "$t2")" breaks 'test "$(owner "$t1")/$(owner "$t2")" = tiling/tiling' 20
no_lost "overlay kill -9 construct"
shot_wm overlay-killed
alive "overlay kill -9"
rss_note overlay
end_foots
session_stop

# --- 2. flavor, with tiling beside -------------------------------------
wm_session 1280x800 || exit 1
bg "$bin/chora-construct"
construct_pid=$bgpid
ctl wait --timeout 15 id:construct-look > /dev/null || finding "W13: the flavor shows no space" breaks "chora-construct"
call construct.load '{"name": "flav"}' > /dev/null || finding "W13: construct.load flav refused" breaks "construct.load"
expect "flav panes: $(panes)" breaks 'test "$(panes)" = "f1 f2 f3 f4 f5"' 20
sleep 1
no_lost "flavor load"
shot_wm flavor-loaded

# fly in and out; the pane far behind the camera is listed
call pane.fly '{"name": "f1"}' > /dev/null || finding "W13: pane.fly f1 refused" breaks "pane.fly"
sleep 2
type_text "hello f1"
expect "f1 got: $(cat "$work/f1.typed" 2>/dev/null)" breaks 'test "$(cat "$work/f1.typed" 2>/dev/null)" = "hello f1"'
shot_wm flavor-flown
no_lost "flavor flown in"
# kill -9 of the program that is flown in
kill9 f1
expect "the pane of the killed flown-in f1 stays: $(panes)" endangers '! panes | grep -qw f1'
sleep 1
expect "no keyboard for the space after f1 died" nit 'test "$(focused id:construct-look)" = True'
no_lost "flavor kill -9 flown-in"
key Alt+Escape
call pane.move '{"name": "f2", "ahead": -6000}' > /dev/null
sleep 0.5
[ "$(wstate f2)" = behind ] || finding "W13: f2 far behind the camera is '$(wstate f2)', not behind" endangers "pane.move ahead -6000"
open_at f2 && { key Return; sleep 1.5; }
[ "$(wstate f2)" = shown ] || finding "W13: the list did not fly to f2: $(wstate f2)" breaks "Tab, Down, Return"
key F1
sleep 0.5
shot_wm flavor-help
key F1
key Alt+Escape
no_lost "flavor list and help"
kill9 f3
expect "the pane of the killed f3 stays: $(panes)" endangers '! panes | grep -qw f3'
no_lost "flavor kill -9 shown program"
call pane.move '{"name": "f4", "ahead": -6000}' > /dev/null
sleep 0.5
kill9 f4
expect "the pane of the killed hidden f4 stays: $(panes)" endangers '! panes | grep -qw f4'
no_lost "flavor kill -9 hidden program"

# a nested chora with its own tiling becomes a pane
nested_n=2
sock=$XDG_RUNTIME_DIR/chora/wayland-$nested_n.sock
outer=$CHORA_SOCKET
ctl run -- chora -- chora-tiling > "$work/run-nested.json" 2>> "$RUN" || finding "W13: choractl run chora -- chora-tiling failed" breaks "choractl run -- chora -- chora-tiling"
npid=$(py 'import json, sys; print(json.load(sys.stdin)["pid"])' < "$work/run-nested.json" 2>/dev/null)
[ -n "$npid" ] && track "$npid"
expect "the nested chora opened no IPC socket" breaks 'test -S "$sock"' 20
expect "the nested window became no pane" breaks 'panes | grep -qF "chora: "' 20 || { step "window.list: $(call window.list | tr -d '\n ' | cut -c1-700)"; step "panes: $(panes)"; }
# the name of the pane has a blank ("chora: chora-tiling")
npane=$(call construct.list | py 'import json, sys; print(next((p["name"] for p in json.load(sys.stdin)["panes"] if p["name"].startswith("chora")), ""))')
step "nested pane: $npane"
WAYLAND_DISPLAY=$XDG_RUNTIME_DIR/wayland-$nested_n CHORA_SOCKET=$sock bg "$foot" -a inner -f "DejaVu Sans Mono:size=10" sh -c 'read l; printf %s "$l" >"$0"; sleep 600' "$work/inner.typed"
expect "the nested tiling took no foot" breaks 'CHORA_SOCKET=$sock ctl query "kind:surface owner:tiling" | grep -q inner' 15
[ -n "$npane" ] && call pane.fly "{\"name\": \"$npane\"}" > /dev/null
sleep 2
type_text "hello nested"
expect "the foot in the nested session got: $(cat "$work/inner.typed" 2>/dev/null)" breaks 'test "$(cat "$work/inner.typed" 2>/dev/null)" = "hello nested"'
shot_wm nested-flown
no_lost "nested flown in"
key Alt+Escape
sleep 1
no_lost "nested flown out"
# the outer session still answers while the nested one runs
CHORA_SOCKET=$outer ctl query id:construct-look > /dev/null || finding "W13: the outer session does not answer with a nested chora" breaks "nested chora"
# kill -9 of the nested chora: its pane goes, nothing else is lost
[ -n "$npid" ] && kill -9 "$npid" 2>/dev/null
[ -n "$npane" ] && expect "the pane of the killed nested chora stays: $(panes)" endangers '! panes | grep -qF "$npane"' 20
no_lost "nested kill -9"

# tiling joins (after the nested pane, or it takes that window); the give
# rows and window.give hand panes to it. The list comes first: a window
# that tiling takes has the keyboard.
bg "$bin/chora-tiling"
ctl wait --timeout 10 owner:tiling > /dev/null || finding "W13: tiling does not start beside the flavor" breaks "chora-tiling"
f5=$(surface f5)
call pane.move '{"name": "f5", "ahead": -6000}' > /dev/null
open_at f5 && { key g; sleep 0.5; key Return; }
expect "the give row did not hand f5 to tiling: $(owner "$f5")" breaks 'test "$(owner "$f5")" = tiling'
no_lost "flavor give row"
f2=$(surface f2)
out=$(call window.give "{\"surface\": \"$f2\", \"to\": \"tiling\"}" 2>&1) || finding "W13: window.give f2 refused: ${out:0:150}" endangers "window.give"
expect "f2 is not with tiling: $(owner "$f2")" breaks 'test "$(owner "$f2")" = tiling'
no_lost "flavor give f2"

# kill -9 of the flavor: the windows go to tiling
f_left=$(call construct.list | py 'import json, sys; print(*[p["surface"] for p in json.load(sys.stdin)["panes"] if "surface" in p])')
kill -9 "$construct_pid"
wait "$construct_pid" 2>/dev/null
for s in $f_left; do
  expect "$s is not with tiling 20 s after kill -9 of the flavor: $(owner "$s")" breaks 'test "$(owner "$s")" = tiling' 25
done
no_lost "flavor kill -9 construct"
shot_wm flavor-killed
alive "flavor kill -9"
rss_note end
rss_growth "$c0" "$(rss chora)" chora
log_scan
