#!/bin/bash
# co-70 (r12, same steps as r11 co68.sh PANE=0): a face window with the keyboard, the cube stood back: is Pull out told, does Super+Alt+4 pull it.
# Usage: co68.sh N TAG
set -u
repo=/home/user/chora
export ROUND_DIR=${ROUND_DIR:-/tmp/persona-round}
n=$1 tag=$2
out=$ROUND_DIR/$tag-co70${PANE:-0}.log
: >"$out"
one() {
    local i=$1 name=$tag-co70p${PANE:-0}-$1 env_file bar1 bar2 bar3
    rm -rf "$ROUND_DIR/keep/$name"; rm -f "$HOME/.cache/fuzzel"
    env_file=$("$repo/tests/persona/round.sh" start "$name" construct 1280x800 | sed -n 's/^PERSONA_ENV=//p')
    [[ -n $env_file ]] || { echo "run $i: no session"; return; }
    set -a; . "$env_file"; set +a
    export PERSONA_ENV=$env_file PERSONA_CHECK=1
    act() { python3 -I "$repo/tests/persona/act.py" "$@" >/dev/null; }
    shown() { choractl query '*' | python3 -c "
import sys,json
d=json.load(sys.stdin); by={x['id']:x for x in d}
def vis(i):
    while i:
        n=by.get(i)
        if not n or n.get('visible') is False: return False
        i=n.get('parent')
    return True
print(' | '.join(x['text'] for x in d if x['id'].startswith('$1') and x.get('kind')=='text' and vis(x['id']) and x.get('rects')))"; }
    dump() { choractl call construct cube.list '{}' | python3 -c "
import sys,json; d=json.load(sys.stdin); print('mode', d['mode'], 'front', d['front'], end='')"
      choractl call construct construct.list '{}' | python3 -c "
import sys,json; d=json.load(sys.stdin); c=d['camera']
print(', camera', [round(c['x']), round(c['z'])], 'panes', [(p['name'], [round(v) for v in p['at']]) for p in d['panes']], end='')"
      for f in construct xp; do echo -n ", focus $f $(choractl state $f 2>/dev/null | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin)['focus']))")"; done; echo
      choractl query 'kind:surface' | python3 -c "
import sys,json
for x in json.load(sys.stdin):
    r=(x.get('rects') or {}).get('HEADLESS-1')
    print('    surface', x['id'], 'owner', x['owner'], 'visible', x.get('visible'), 'rect', r)"; }
    for _ in $(seq 40); do choractl query '*' | grep -q 'A cube of desktops' && break; sleep 0.5; done
    if [[ ${PANE:-0} == 1 ]]; then act click 640 364; sleep 5; act key Super+c; sleep 6; else act click 640 468; fi
    for _ in $(seq 30); do choractl call construct cube.list '{}' >/dev/null 2>&1 && break; sleep 0.5; done
    sleep 5; act key Return; sleep 4; act key Super+t; sleep 6
    echo "run $i in the face after Super+t:"; dump
    act key Alt+Escape; sleep 4
    bar1=$(shown construct-hints-)
    echo " stood back, hint bar [$bar1]"; dump
    pull=no; [[ $bar1 == *"Pull out"* ]] && pull=yes
    act key Super+Alt+4; sleep 5
    bar2=$(shown construct-hints-)
    echo " after Super+Alt+4, hint bar [$bar2]"; dump
    act look
    "$repo/tests/persona/round.sh" stop "$name" >/dev/null
    echo "  told before: $pull"
    echo "  non-I6 breaches: $(grep -v '"I6"' "$ROUND_DIR/keep/$name/breaches.jsonl" 2>/dev/null | grep -vc 'are 9 apart')"
}
for i in $(seq "$n"); do ( one "$i" ) 2>&1 | tee -a "$out"; done
