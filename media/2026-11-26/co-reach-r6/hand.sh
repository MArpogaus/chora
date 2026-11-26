#!/bin/bash
# usage: hand.sh CASE N   (fresh compositor per run, fuzzel cache cleared)
set -u
repo=/home/user/chora; export ROUND_DIR=${RD:-/tmp/persona-round}
S=/tmp/claude-0/-home-user-chora/95098445-03b1-5ae2-8de2-50586dd7e8c8/scratchpad
v=$1 n=$2; out=$S/hand-$v${TAG:-}.log; : >"$out"
one() {
  local i=$1 name=hr6-$v-$1-$$ env_file
  rm -rf "$ROUND_DIR/keep/$name"; rm -f "$HOME/.cache/fuzzel"
  env_file=$(bash $repo/tests/persona/round.sh start "$name" construct 1280x800 | sed -n 's/^PERSONA_ENV=//p')
  [[ -n $env_file ]] || { echo "run $i: no session"; return; }
  set -a; . "$env_file"; set +a
  export PERSONA_ENV=$env_file PERSONA_CHECK=1
  act() { python3 -I $repo/tests/persona/act.py "$@" >/dev/null; }
  bar() { choractl query '*' | python3 -c "
import sys,json
d=json.load(sys.stdin); by={x['id']:x for x in d}
def vis(i):
    while i:
        n=by.get(i)
        if not n or n.get('visible') is False: return False
        i=n.get('parent')
    return True
print(' '.join(x['text'] for x in d if x['id'].startswith('construct-hints-') and x.get('kind')=='text' and vis(x['id']) and x.get('rects')))"; }
  dump() { choractl call construct cube.list '{}' | python3 -c "
import sys,json; d=json.load(sys.stdin); print('mode', d.get('mode'), 'front', d.get('front'), 'faces', len(d.get('faces') or []), end='')"
    choractl call construct construct.list '{}' | python3 -c "
import sys,json; d=json.load(sys.stdin); print(', panes', [p['name'] for p in d['panes']], end='')"
    echo -n ", core focus: $(choractl query '*' | python3 -c "import sys,json; print([x['id'] for x in json.load(sys.stdin) if x.get('focused')])")"; echo
    choractl query 'kind:surface' | python3 -c "
import sys,json
for x in json.load(sys.stdin):
    print('    surface', x['id'], 'owner', x['owner'], 'visible', x.get('visible'), 'focused', x.get('focused'))"; }
  shot() { choractl --name persona capture $S/shots/$v${TAG:-}-run$i-$1.png >/dev/null; }
  mkdir -p $S/shots
  for _ in $(seq 40); do choractl query '*' | grep -q 'A cube of desktops' && break; sleep 0.5; done
  sleep 3
  case $v in
  term) act click 640 364; sleep 7 ;;                     # start card Terminal row
  prog) act click 640 468; sleep 6; act click 154 51; sleep 3; act click 550 294; sleep 7 ;;  # cube row, Programs, Foot
  progcard) act click 640 416; sleep 4; act key f; act key o; act key o; act key t; sleep 1; act key Return; sleep 7 ;; # All programs row, foot
  launch) act key Super+space; sleep 3; act key f; act key o; act key o; act key t; sleep 1; act key Return; sleep 7 ;; # launcher, no cube
  launchcube) act key Super+c; sleep 6; act key Super+space; sleep 3; act key f; act key o; act key o; act key t; sleep 1; act key Return; sleep 7 ;;
  supert) act key Super+t; sleep 7 ;;                     # Super+t in the space (walk)
  supertpane) act click 640 364; sleep 7; act key Super+t; sleep 7 ;;  # Super+t in a pane
  supertcube) act key Super+c; sleep 6; act key Super+t; sleep 7 ;;  # Super+t stood back
  pull) act key Super+c; sleep 6; act key Return; sleep 5; act key Super+t; sleep 6; act key Super+Alt+4; sleep 6 ;;  # pull out of a face
  face) act click 640 468; sleep 6; act key Return; sleep 5; act key Super+t; sleep 6
    echo "run $i in the face with a window:"; dump; b=$(bar); echo "  bar [$b]"
    [[ $i == 1 ]] && shot face
    [[ $b == *"Super Alt 4 Pull out"* ]] && told=yes || told=NO; echo "  told: $told"
    act key Super+Alt+4; sleep 6
    echo " after Super+Alt+4:"; dump; echo "  bar [$(bar)]"
    p=$(choractl call construct construct.list '{}' | python3 -c "import sys,json; print(len(json.load(sys.stdin)['panes']))")
    [[ $told == yes && $p -ge 1 ]] && echo "  RESULT pass" || echo "  RESULT FAIL"
    bash $repo/tests/persona/round.sh stop "$name" >/dev/null; return ;;
  esac
  echo "run $i in the pane:"; dump; echo "  bar [$(bar)]"
  [[ $i == 1 ]] && shot in-pane
  act key Alt+Escape; sleep 5
  echo " after Alt+Escape:"; dump; b=$(bar); echo "  bar [$b]"
  [[ $i == 1 ]] && shot stood-back
  told=NO
  if [[ $b == *"Super C Cube"* || -n ${CUBE:-} ]]; then
    [[ $b == *"Super C Cube"* ]] && echo "  told: Super C Cube" || echo "  told: NO (Super C pressed untold)"
    act key Super+c; sleep 9
    echo " after Super+C:"; dump; b=$(bar); echo "  bar [$b]"
    [[ $i == 1 ]] && shot cube
    [[ $b == *"Super C Cube"* ]] || told=NO
  fi
  [[ $b == *"Super Alt 2 Into face"* ]] && told=yes
  key=Super+Alt+2
  echo "  told: $told"
  act key $key; sleep 6
  echo " after $key:"; dump; b2=$(bar); echo "  bar [$b2]"
  [[ $i == 1 ]] && shot after-key
  st=$(choractl state xp 2>/dev/null | python3 -c "import sys,json; w=[w for w in json.load(sys.stdin).get('windows',[]) if w.get('app_id')=='foot']; print(w[-1]['output'] if w else '-', w[-1]['shown'] if w else '-')" 2>/dev/null)
  mode=$(choractl call construct cube.list '{}' | python3 -c "import sys,json; print(json.load(sys.stdin).get('mode'))")
  echo "  foot on xp: $st, mode $mode"
  [[ $told == yes && $st == "VIRTUAL-1 True" && $mode == use && $b2 == *"Pull out"* ]] && echo "  RESULT pass" || echo "  RESULT FAIL"
  bash $repo/tests/persona/round.sh stop "$name" >/dev/null
}
for i in $(seq "$n"); do ( one "$i" ) 2>&1 | tee -a "$out"; done
