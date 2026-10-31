#!/bin/bash
# Helpers of W13-W15 (construct, hand-over, voice HUD). Source lib.sh
# first. Where the gates (tests/gate-construct-windows.sh) stop at the
# first failure, these helpers write a finding and go on. Severities:
# breaks (a feature or a window is gone), endangers (a refusal or a
# safety rule fails, or data may get lost), nit.
#
# Programs that a workflow starts go through `track`: their pid reaches
# /tmp/bt-$WF.pids, and kill-session.sh ends them.
track() { echo "$1" >> "/tmp/bt-$WF.pids"; }
# bg CMD...: a program in the background, tracked; the pid is in $bgpid.
bg() {
  "$@" &
  bgpid=$!
  track "$bgpid"
}
# wm_session <outputs> [startup...]: a session with a tight frame budget
# (the software renderer is slow), a work dir, and the grants of agent.
wm_session() {
  export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
  work=/tmp/bt-$WF-work
  rm -rf "$work"
  mkdir -p "$work" "$XDG_CONFIG_HOME/chora/constructs"
  session_start "$@" || return 1
  admin grant agent screen.capture input.inject methods.call >> "$RUN" 2>&1
  foot=$(command -v foot)
  # no foot on the host (the sandbox): the one of ~/.local/opt/clients needs
  # its libraries, which a wrapper adds for foot alone
  clients=$HOME/.local/opt/clients/usr
  if [ -z "$foot" ] && [ -x "$clients/bin/foot" ]; then
    printf '#!/bin/sh\nLD_LIBRARY_PATH=%s${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH} exec %s "$@"\n' "$clients/lib64" "$clients/bin/foot" > "$work/foot-bin"
    chmod +x "$work/foot-bin"
    foot=$work/foot-bin
  fi
  # sh foot.sh NAME: a foot with the app id NAME that writes its pid to
  # $work/NAME.pid and a typed line to $work/NAME.typed
  cat > "$work/foot.sh" <<EOF
echo \$\$ >"$work/\$1.pid"
exec "$foot" -a "\$1" -f "DejaVu Sans Mono:size=10" sh -c 'echo \$0; read l; printf %s "\$l" >"$work/\$0.typed"; sleep 600' "\$1"
EOF
}
# expect <title> <severity> <code> [secs]: waits for the shell code to
# succeed, else a finding.
expect() {
  local title=$1 sev=$2 n=$(( ${4:-5} * 4 ))
  for _ in $(seq "$n"); do
    eval "$3" && return 0
    sleep 0.25
  done
  finding "$WF: $title" "$sev" "$WF: $title; code: $3"
  return 1
}
until_ok() {
  for _ in $(seq "${2:-40}"); do
    eval "$1" && return 0
    sleep 0.25
  done
  return 1
}
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null; }
# shellcheck disable=SC2028 # \n is the JSON escape, for the core
type_text() { echo "[{\"text\": {\"text\": \"$1\\n\"}}]" | ctl inject >/dev/null; }
call() { ctl call construct "$@"; }
foot_bg() { bg sh "$work/foot.sh" "$1"; }
kill9() { kill -9 "$(cat "$work/$1.pid")" 2>/dev/null; }
end_foots() {
  local f
  for f in "$work"/*.pid; do
    [ -s "$f" ] && kill "$(cat "$f")" 2>/dev/null
  done
}
py() { python3 -I -c "$@"; }
# the surface id of the window with the app id $1
surface() { ctl query kind:surface | py "import json, sys; print(next((n['id'] for n in json.load(sys.stdin) if n['surface'].get('app_id') == '$1'), ''))"; }
owner() { ctl query "id:$1" | py 'import json, sys; n = json.load(sys.stdin); print(n[0].get("owner") if n else "gone")'; }
panes() { call construct.list 2>/dev/null | py 'import json, sys; print(" ".join(sorted(p["name"] for p in json.load(sys.stdin)["panes"])))'; }
# the pane state of the window $1 in window.list
wstate() { call window.list 2>/dev/null | py "import json, sys; print(next((w['pane'] for w in json.load(sys.stdin)['windows'] if w['name'] == '$1'), 'missing'))"; }
focused() { ctl query "$1" | py 'import json, sys; n = json.load(sys.stdin); print(n[0].get("focused") if n else "")'; }
geometry() {
  ctl query kind:surface | py '
import json, sys
for n in sorted(json.load(sys.stdin), key=lambda n: n["surface"].get("app_id") or ""):
    print(n["surface"].get("app_id"), n.get("owner"), n["rects"], n["opacity"], n["visible"], n["transform"])'
}
# open_at NAME: Tab opens the window list; Down to the row of NAME
open_at() {
  local row
  row=$(call window.list | py "
import json, sys
rows = [w['name'] for w in json.load(sys.stdin)['windows'] if w['pane'] != 'shown']
print(rows.index('$1'))") || { finding "$WF: $1 is not in the window list" endangers "window.list"; return 1; }
  key Tab
  ctl wait --timeout 5 "id:construct-list meta:state=open" >/dev/null || { finding "$WF: Tab opened no list" breaks "Tab"; return 1; }
  for _ in $(seq "$row"); do key Down; done
}
# no_lost <step>: no surface is lost. A
# surface without an owner that the core does not show, one reserved for
# a client that did not attach it, one of the construct without a pane,
# and a pane that neither shows nor is listed as not shown are lost (the no_lost of tests/gate-construct-windows.sh). Also
# fails when a window that the workflow knows is gone ($WINDOWS holds
# app ids that must still exist unless the step killed them).
no_lost() {
  # The pane of a killed program goes with the destroy event, a moment
  # after the surface: look again before a pane counts as lost (W13).
  local out
  out=$(no_lost_once "$1") && { step "no_lost ok: $1"; return 0; }
  sleep 1
  out=$(no_lost_once "$1") || { finding "$WF: a window is lost after '$1': ${out:0:300}" breaks "no_lost after $1"; return 1; }
  step "no_lost ok after a second look: $1"
}
no_lost_once() {
  ctl query kind:surface > "$work/surfaces.json" || { echo "query failed"; return 1; }
  ctl query owner:construct > "$work/nodes.json" 2>/dev/null || echo '[]' > "$work/nodes.json"
  call construct.list > "$work/panes.json" 2>/dev/null || echo '{"panes": []}' > "$work/panes.json"
  call window.list > "$work/windows.json" 2>/dev/null || echo '{"windows": []}' > "$work/windows.json"
  python3 -I - "$work" "$1" 2>&1 <<'EOF'
import json, sys
work, step = sys.argv[1], sys.argv[2]
load = lambda f: json.load(open(f"{work}/{f}.json"))
nodes = {n["id"]: n for n in load("nodes")}
panes = {p["surface"] for p in load("panes")["panes"] if "surface" in p}
listed = {w["surface"] for w in load("windows")["windows"] if w["pane"] != "shown"}
bad = []
for n in load("surfaces"):
    sid, size = n["id"], n["surface"].get("size") or [0, 0]
    owner = n.get("owner")
    app = n["surface"].get("app_id")
    if min(size) <= 0:
        bad.append(f"{sid} ({app}): size {size}")
    elif n.get("to"):
        bad.append(f"{sid} ({app}): reserved for {n['to']}, which did not attach it")
    elif owner is None:
        # the core shows a window that no window manager takes
        if not shows:
            bad.append(f"{sid} ({app}): no owner, not shown by the core")
    elif owner == "construct":
        pane = nodes.get(f"construct-pane-{sid}", {})
        if sid not in panes:
            bad.append(f"{sid} ({app}): owned by the construct without a pane")
        elif not (pane.get("visible") and pane.get("rects")) and sid not in listed:
            bad.append(f"{sid} ({app}): a pane that neither shows nor is listed")
    elif n["opacity"] <= 0 or not n["visible"]:
        bad.append(f"{sid} ({app}) of {owner}: opacity {n['opacity']}, visible {n['visible']}")
if bad:
    print(f"{step}: " + "; ".join(bad))
    sys.exit(1)
EOF
}
# alive <what>: the core answers, no panic so far
alive() { health "$1"; }
# windows_exist <step> <app id>...: each app id still has a surface
windows_exist() {
  local s=$1 a
  shift
  for a in "$@"; do
    [ -n "$(surface "$a")" ] || finding "$WF: the window $a is gone after '$s'" breaks "windows_exist $s $a"
  done
}
# shot_wm <name> [output]: a capture into $shots, errors ignored
shot_wm() { shot "$@" > /dev/null; }
