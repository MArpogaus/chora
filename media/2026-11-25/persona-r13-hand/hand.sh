#!/bin/bash
# usage: hand.sh co75|co76|co79 N   (fresh compositor per run, fuzzel cache cleared)
set -u
repo=/home/user/chora; export ROUND_DIR=/tmp/persona-round
v=$1 n=$2; out=/tmp/claude-0/r13/hand-$v.log; : >"$out"
one() {
  local i=$1 name=hr13-$v-$1 env_file
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
import sys,json; d=json.load(sys.stdin); print('mode', d['mode'], 'front', d['front'], end='')"
    choractl call construct construct.list '{}' | python3 -c "
import sys,json; d=json.load(sys.stdin); print(', panes', [p['name'] for p in d['panes']], end='')"
    for f in construct xp; do echo -n ", $f $(choractl state $f 2>/dev/null | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin)["focus"]))" 2>/dev/null)"; done
    echo -n ", core focus: $(choractl query '*' | python3 -c "import sys,json; print([x['id'] for x in json.load(sys.stdin) if x.get('focused')])")"; echo
    choractl query 'kind:surface' | python3 -c "
import sys,json
for x in json.load(sys.stdin):
    r=(x.get('rects') or {}).get('HEADLESS-1')
    print('    surface', x['id'], 'owner', x['owner'], 'visible', x.get('visible'), 'rect', r)"; }
  for _ in $(seq 40); do choractl query '*' | grep -q 'A cube of desktops' && break; sleep 0.5; done
  sleep 3
  case $v in
  co75)
    act click 640 364; sleep 6          # Terminal row: a pane
    act key Super+c; sleep 6; act key Return; sleep 5   # cube, step into face 0
    act key Super+t; sleep 6            # a face window
    echo "run $i before Super+x:"; dump
    n0=$(choractl query 'kind:surface' | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")
    act key Super+x; sleep 5
    echo " after Super+x:"; dump
    n1=$(choractl query 'kind:surface' | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")
    panes=$(choractl call construct construct.list '{}' | python3 -c "import sys,json; print(len(json.load(sys.stdin)['panes']))")
    res=FAIL; [[ $n1 -eq $((n0-1)) && $panes -ge 1 ]] && res=pass
    echo "  surfaces $n0 -> $n1, panes $panes: $res" ;;
  co76)
    act click 640 364; sleep 9          # Terminal row: flown into the pane; 3 s hints over
    echo "run $i in the pane after 9 s:"; dump
    sb=$(choractl query id:construct-hint-step-back | python3 -c "import sys,json; d=json.load(sys.stdin); print('yes' if d and d[0].get('rects') else 'no')")
    kk=$(choractl query id:construct-hint-keys | python3 -c "import sys,json; d=json.load(sys.stdin); print('yes' if d and d[0].get('rects') else 'no')")
    echo "  bar [$(bar)] step-back hint: $sb, keys hint: $kk"
    act look
    [[ $sb == yes && $kk == yes ]] && echo "  pass" || echo "  FAIL" ;;
  co79)
    act click 640 364; sleep 7
    act key Alt+Escape; sleep 5
    echo "run $i after Alt+Escape from the pane:"; dump
    b=$(bar); echo "  bar [$b]"
    [[ $b == *"Pull out"* ]] && echo "  told: yes" || echo "  told: NO"
    act key Super+Alt+4; sleep 5
    echo " after Super+Alt+4:"; dump; echo "  bar [$(bar)]"
    act look ;;
  co79b)
    act click 640 468; sleep 6          # cube row: the cube opens, stood back
    act click 154 51; sleep 3; act click 550 294; sleep 7   # Programs, Foot: a window on face 0
    echo "run $i foot started (cube stood back):"; dump; echo "  bar [$(bar)]"
    act key Alt+Escape; sleep 5
    echo " after Alt+Escape:"; dump
    b=$(bar); echo "  bar [$b]"
    [[ $b == *"Pull out"* ]] && echo "  told: yes" || echo "  told: NO"
    act key Super+Alt+4; sleep 5
    echo " after Super+Alt+4:"; dump; echo "  bar [$(bar)]"
    act look ;;
  co79c)
    act click 640 468; sleep 6; act key Return; sleep 5; act key Super+t; sleep 6   # step into face 0, a window in it
    echo "run $i in the face with a window:"; dump
    b=$(bar); echo "  bar [$b]"
    [[ $b == *"Pull out"* ]] && echo "  told: yes" || echo "  told: NO"
    act key Super+Alt+4; sleep 5
    echo " after Super+Alt+4:"; dump
    act look ;;
  esac
  bash $repo/tests/persona/round.sh stop "$name" >/dev/null
}
for i in $(seq "$n"); do ( one "$i" ) 2>&1 | tee -a "$out"; done
