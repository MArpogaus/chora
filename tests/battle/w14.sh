#!/bin/bash
# W14 hand-over: the construct flavor, xp, tiling and mc, each on its own
# output (add-output). Title-bar drags and Super drags, `window.give`
# between all of them, a give to a client without a claim (must be
# refused), a give to a client that is not there, and a kill of the target
# right after a give. No window is lost: no_lost runs after every step.
WF=w14; STOP_ON_DEATH=1; . "$(dirname "$0")/lib.sh"; . "$(dirname "$0")/lib-wm.sh"
config_home
trap 'end_foots 2>/dev/null; session_stop' EXIT
wm_session 1280x800 || exit 1
c0=$(rss chora)
theme=$repo/flavors/xp/theme
# the construct flavor takes the only output, then the others come
bg "$bin/chora-construct"
ctl wait --timeout 15 id:construct-look > /dev/null || finding "W14: the flavor shows no space" breaks "chora-construct"
for x in 1280 2560 3840; do ctl add-output "1280x800+$x+0" > /dev/null || finding "W14: add-output +$x failed" breaks "add-output"; done
mapfile -t outs < <(ctl outputs | py 'import json, sys; print(*[o["name"] for o in sorted(json.load(sys.stdin), key=lambda o: o["position"][0])], sep="\n")')
[ "${#outs[@]}" = 4 ] || { finding "W14: ${#outs[@]} outputs, not 4" breaks "add-output x3"; exit 1; }
c_out=${outs[0]} xp_out=${outs[1]} tiling_out=${outs[2]} mc_out=${outs[3]}
start_xp() { bg "$bin/chora-xp" --theme "$theme" --output "$xp_out"; xp_pid=$bgpid; }
start_tiling() { bg "$bin/chora-tiling" --output "$tiling_out"; tiling_pid=$bgpid; }
start_mc() { bg "$bin/chora-mc" --output "$mc_out"; mc_pid=$bgpid; }
start_xp; start_tiling; start_mc
for c in xp tiling mc; do ctl wait "owner:$c" --timeout 20 > /dev/null || finding "W14: $c does not start" breaks "chora-$c --output"; done
inj() { echo "$1" | ctl inject > /dev/null; }
# settled <owner> [window]: the window belongs to owner, nothing is lost
settled() {
  local w=${2:-$w}
  expect "$w is with $(owner "$w"), not $1, after: $step_name" breaks "test \"\$(owner $w)\" = $1" 15 || return 1
  no_lost "$step_name"
}
# new_foot <output> <name>: the pointer on that output, so the new window goes to its window manager
new_foot() {
  inj "[{\"pointer_move\": {\"position\": [640, 300], \"output\": \"$1\"}}]"
  sleep 0.3
  foot_bg "$2"
  expect "no window $2" breaks "test -n \"\$(surface $2)\"" 15
  sleep 0.5
}
super_drag() { # window x y output
  inj "[{\"pointer_move\": {\"node\": \"$1\", \"point\": [100, 60]}},
    {\"key\": {\"key\": \"Super_L\", \"state\": \"pressed\"}},
    {\"button\": {\"state\": \"pressed\", \"hold\": true}},
    {\"pointer_move\": {\"position\": [$2, $3], \"output\": \"$4\"}},
    {\"button\": {\"state\": \"released\"}},
    {\"key\": {\"key\": \"Super_L\", \"state\": \"released\"}}]"
}
give() { # from to window
  ctl call "$1" window.give "{\"surface\": \"$3\", \"to\": \"$2\"}" 2>&1
}
shots_all() { local o; for o in "${outs[@]}"; do shot_wm "$1-$o" "$o"; done; }

new_foot "$xp_out" a
w=$(surface a)
step_name="start"; settled xp
shots_all start

# 1. title bar of xp to the tiling output
step_name="title-bar drag xp -> tiling"
inj "[{\"pointer_move\": {\"node\": \"xp-title-$w\", \"point\": [60, 10]}},
  {\"button\": {\"state\": \"pressed\", \"hold\": true}},
  {\"pointer_move\": {\"position\": [300, 300], \"output\": \"$xp_out\"}},
  {\"pointer_move\": {\"position\": [640, 400], \"output\": \"$tiling_out\"}},
  {\"button\": {\"state\": \"released\"}}]"
settled tiling

# 2. a round of gives between all window managers (with the flavor too)
for pair in "tiling xp" "xp mc" "mc tiling" "tiling construct" "construct xp" "xp tiling" "tiling mc" "mc construct" "construct mc" "mc xp"; do
  set -- $pair
  step_name="window.give $1 -> $2"
  out=$(give "$1" "$2" "$w") || { finding "W14: window.give $1 -> $2 refused: ${out:0:140}" endangers "$step_name"; no_lost "$step_name refused"; continue; }
  settled "$2" || break
done
shots_all gives
alive "give round"

# 3. choractl give and Super drags
cur=$(owner "$w")
step_name="choractl give (focused window)"
if [ "$cur" != tiling ]; then give "$cur" tiling "$w" > /dev/null; settled tiling; fi
ctl give mc > /dev/null 2>&1 || finding "W14: choractl give mc failed" breaks "choractl give mc"
settled mc
step_name="mc -> tiling"
give mc tiling "$w" > /dev/null; settled tiling
super_drag "$w" 640 400 "$xp_out"
step_name="Super drag tiling -> xp"; settled xp
step_name="Super drag xp -> mc output"
super_drag "$w" 640 400 "$mc_out"
sleep 1
no_lost "$step_name"
step "owner after the drag to the mc output: $(owner "$w")"
step_name="Super drag to the construct output"
super_drag "$w" 640 400 "$c_out"
sleep 1
no_lost "$step_name"
step "owner after the drag to the construct output: $(owner "$w")"
shots_all superdrag
# bring it back to xp for the next steps
cur=$(owner "$w")
[ "$cur" != xp ] && give "$cur" xp "$w" > /dev/null
step_name="back to xp"; settled xp

# 4. refusals: nobody, a client without a claim, a client that is gone
python3 -I - "$CHORA_SOCKET" > "$work/noclaim.log" 2>&1 <<'EOF' &
import json, socket, sys, time
s = socket.socket(socket.AF_UNIX); s.connect(sys.argv[1])
s.sendall(json.dumps({"jsonrpc": "2.0", "id": 1, "method": "hello", "params": {"name": "noclaim"}}).encode() + b"\n")
f = s.makefile(); print(f.readline())
time.sleep(120)
EOF
track $!
sleep 1
for target in nobody noclaim agent; do
  step_name="give to $target"
  if out=$(give xp "$target" "$w"); then
    finding "W14: window.give to '$target' (no claim or no such client) was accepted: ${out:0:100}" endangers "ctl call xp window.give to $target"
  fi
  sleep 1
  [ "$(owner "$w")" = xp ] || finding "W14: a refused give to $target moved the window to $(owner "$w")" breaks "$step_name"
  no_lost "$step_name"
done
# the same through choractl give
step_name="choractl give noclaim"
ctl give noclaim > /dev/null 2>&1 && finding "W14: choractl give to a client without a claim was accepted" endangers "choractl give noclaim"
no_lost "$step_name"
alive "refused gives"

# 5. a kill of the target right after a give; no window is lost
for round in tiling mc xp; do
  step_name="give to $round, kill -9 of $round"
  cur=$(owner "$w")
  if [ "$cur" = "$round" ]; then
    give "$cur" construct "$w" > /dev/null
    settled construct
    cur=construct
  fi
  give "$cur" "$round" "$w" > /dev/null || finding "W14: give $cur -> $round refused" endangers "$step_name"
  eval "kill -9 \$${round}_pid"
  expect "the window is still with the dead $round after 25 s: $(owner "$w")" breaks "o=\$(owner $w); [ \"\$o\" != $round ] && [ \"\$o\" != none ] && [ \"\$o\" != gone ]" 25
  sleep 1
  no_lost "$step_name"
  shot_wm "after-kill-$round" "$c_out"
  step "after the kill of $round the window is with $(owner "$w")"
  "start_$round"
  ctl wait "owner:$round" --timeout 20 > /dev/null || finding "W14: $round does not come back" breaks "restart $round"
  sleep 1
  no_lost "restart $round"
done

# 6. many windows, one per output, Super drags over all outputs
new_foot "$tiling_out" b
new_foot "$mc_out" c
new_foot "$xp_out" d
no_lost "four windows"
for x in b c d; do
  id=$(surface $x)
  for o in "$tiling_out" "$xp_out" "$mc_out"; do
    super_drag "$id" 500 300 "$o"
    sleep 0.7
  done
  no_lost "super drags of $x"
done
shots_all tour
# kill every window manager at once: the windows end with the flavor or a restarted manager
step_name="kill -9 of xp, tiling and mc together"
kill -9 "$xp_pid" "$tiling_pid" "$mc_pid" 2>/dev/null
sleep 12
expect "windows are still with a dead window manager" breaks 'ctl query kind:surface | py "import json,sys; sys.exit(any(n.get(\"owner\") in (None,\"xp\",\"tiling\",\"mc\") for n in json.load(sys.stdin)))"' 25
step "owners after the kill of all three: $(ctl query kind:surface | py 'import json,sys; print([(n["surface"].get("app_id"), n.get("owner"), n.get("to"), n["visible"], bool(n["rects"])) for n in json.load(sys.stdin)])')"
no_lost "$step_name"
shots_all allkilled
alive "kill all"
rss_growth "$c0" "$(rss chora)" chora
log_scan
