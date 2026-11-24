#!/bin/bash
# co-67: a program started while the cube stands back. Usage: co67.sh key|row N TAG
set -u
repo=/home/user/chora
export ROUND_DIR=${ROUND_DIR:-/tmp/persona-round}
how=$1 n=$2 tag=$3
out=$ROUND_DIR/$tag-$how.log
: >"$out"
one() {
    local i=$1 name=$tag-$how-$1 env_file
    rm -rf "$ROUND_DIR/keep/$name"; rm -f "$HOME/.cache/fuzzel"
    env_file=$("$repo/tests/persona/round.sh" start "$name" construct 1280x800 | sed -n 's/^PERSONA_ENV=//p')
    [[ -n $env_file ]] || { echo "run $i: no session"; return; }
    set -a; . "$env_file"; set +a
    export PERSONA_ENV=$env_file PERSONA_CHECK=1
    act() { python3 -I "$repo/tests/persona/act.py" "$@" >/dev/null; }
    dump() { choractl call construct cube.list '{}' | python3 -c "
import sys,json; d=json.load(sys.stdin)
print('mode', d['mode'], 'front', d['front'], 'keys', sorted(d)[:12]); print('  faces', json.dumps(d.get('faces'))[:300])"
      choractl call construct construct.list '{}' | python3 -c "
import sys,json; d=json.load(sys.stdin); c=d['camera']
print('  camera', [round(c['x']), round(c['z'])], 'panes', [(p['name'], [round(v) for v in p['at']]) for p in d['panes']])"
      choractl query 'kind:surface' | python3 -c "
import sys,json
for x in json.load(sys.stdin):
    if x['owner'] not in ('xp',) : print('  surface', x['id'], x['owner'], 'visible', x.get('visible'), 'rect', (x.get('rects') or {}).get('HEADLESS-1'))"
      for f in construct xp; do echo "  focus $f: $(choractl state $f 2>/dev/null | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin)['focus']))")"; done; }
    for _ in $(seq 40); do choractl query '*' | grep -q 'A cube of desktops' && break; sleep 0.5; done
    act click 640 468
    for _ in $(seq 30); do choractl call construct cube.list '{}' >/dev/null 2>&1 && break; sleep 0.5; done
    sleep 5; act key Super+c; sleep 6
    echo "run $i before:"; dump
    if [[ $how == key ]]; then act key Super+t; else
        act click 154 52
        for _ in $(seq 20); do grep -q '^fuzzel' "$PERSONA_CALLS" && break; sleep 0.5; done
        sleep 3; act click 550 294
    fi
    sleep 8
    echo " after start:"; dump
    act key Alt+Escape; sleep 3
    echo " after Alt+Escape:"; dump
    act key Return; sleep 4
    echo " after step in (Return):"; dump
    act look
    "$repo/tests/persona/round.sh" stop "$name" >/dev/null
}
for i in $(seq "$n"); do ( one "$i" ) 2>&1 | tee -a "$out"; done
