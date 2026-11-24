#!/bin/bash
# co-reach r2: co-61 (way out of a stepped-in face named in the hint bar)
# and co-62 (a click on the pane beside the stood-back cube flies into it).
# Usage: reach.sh co61|co62 N TAG   (one fresh construct session per run;
# each run in a subshell, so no PERSONA_* of one run reaches the next)
set -u
repo=/home/user/chora
export ROUND_DIR=${ROUND_DIR:-/tmp/persona-round}
what=$1 n=$2 tag=$3
out=$ROUND_DIR/$tag-$what.log
: >"$out"

one() {
    local i=$1 name=$tag-$what-$1 env_file ok words m at before f cam rows
    rm -rf "$ROUND_DIR/keep/$name"
    env_file=$("$repo/tests/persona/round.sh" start "$name" construct 1280x800 | sed -n 's/^PERSONA_ENV=//p')
    [[ -n $env_file ]] || { echo "run $i: no session"; return; }
    set -a; . "$env_file"; set +a
    export PERSONA_ENV=$env_file PERSONA_CHECK=1
    act() { python3 -I "$repo/tests/persona/act.py" "$@" >/dev/null; }
    mode() { choractl call construct cube.list '{}' | python3 -c "import sys,json; print(json.load(sys.stdin)['mode'])"; }
    focus() { choractl state "$1" 2>/dev/null | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin)['focus']))"; }
    if [[ $what == co61 ]]; then
        act click 640 468; sleep 6; act key Return; sleep 5
        words=$(choractl query '*' | python3 -c "
import sys,json
d=json.load(sys.stdin); by={x['id']:x for x in d}
def vis(i):
    while i:
        n=by.get(i)
        if not n or n.get('visible') is False: return False
        i=n.get('parent')
    return True
print(' | '.join(x['text'] for x in d if x['id'].startswith('construct-hints-') and x.get('kind')=='text' and vis(x['id']) and x.get('rects')))")
        m=$(mode)
        ok=no; [[ $m == use && $words == *"Step back"* ]] && ok=yes
        echo "run $i: mode $m, construct $(focus construct), xp $(focus xp), hint bar [$words], way out named: $ok"
    else
        act click 640 364; sleep 5; act key Super+c; sleep 6
        at=$(choractl query 'kind:surface' | python3 -c "
import sys,json
for x in json.load(sys.stdin):
    r=(x.get('rects') or {}).get('HEADLESS-1')
    if x['owner']=='construct' and x.get('visible') and r: print(round(r[0]+r[2]/2), round(r[1]+r[3]/2)); break")
        before="mode $(mode), construct $(focus construct), pane at [$at]"
        if [[ -n $at ]]; then act click $at; sleep 3; fi
        f=$(focus construct)
        cam=$(choractl call construct construct.list '{}' | python3 -c "import sys,json; c=json.load(sys.stdin)['camera']; print(round(c['x']), round(c['z']))")
        ok=no; [[ $f == *'"surface": "s1"'* ]] && ok=yes
        if [[ $ok == yes ]]; then act type "echo reach"; sleep 1; fi
        echo "run $i: before click: $before; after: mode $(mode), construct $f, xp $(focus xp), camera [$cam], flown in with the keys: $ok"
    fi
    act look
    "$repo/tests/persona/round.sh" stop "$name" >/dev/null
    rows=$(grep -v '"I6"' "$ROUND_DIR/keep/$name/breaches.jsonl" 2>/dev/null | grep -c . )
    echo "  non-I6 breach rows: $rows $(grep -v '"I6"' "$ROUND_DIR/keep/$name/breaches.jsonl" 2>/dev/null | python3 -c "
import sys,json,collections
c=collections.Counter()
for l in sys.stdin:
    try: r=json.loads(l)
    except Exception: continue
    c[r.get('invariant','?')+': '+('co-44 gap' if 'are 9 apart' in r.get('breach','') else r.get('breach','')[:90])]+=1
print('; '.join(f'{k} x{v}' for k,v in c.items()))")"
}

for i in $(seq "$n"); do
    ( one "$i" ) 2>&1 | tee -a "$out"
done
echo "$what $tag: $(grep -c ': yes$' "$out")/$n pass" | tee -a "$out"
