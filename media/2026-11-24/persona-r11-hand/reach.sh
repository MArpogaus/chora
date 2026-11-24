#!/bin/bash
# co-reach r3: co-64 (a pane beside the stood-back cube: is its click or the
# pull key named; what a drag on it does) and co-65 (a program started from
# Programs with the cube open: is the top bar hidden, is a way out named,
# does Escape or the named key step out).
# Usage: reach.sh co64|co65|co65in N TAG   (one fresh construct session per
# run, each in a subshell; logical coordinates, no look before the end, as
# act.py scales a click by the last look)
set -u
repo=/home/user/chora
export ROUND_DIR=${ROUND_DIR:-/tmp/persona-round}
what=$1 n=$2 tag=$3
out=$ROUND_DIR/$tag-$what.log
: >"$out"

one() {
    local i=$1 name=$tag-$what-$1 env_file ok words m at before f cam rows bar cur
    rm -rf "$ROUND_DIR/keep/$name"
    rm -f "$HOME/.cache/fuzzel"
    env_file=$("$repo/tests/persona/round.sh" start "$name" construct 1280x800 | sed -n 's/^PERSONA_ENV=//p')
    [[ -n $env_file ]] || { echo "run $i: no session"; return; }
    set -a; . "$env_file"; set +a
    export PERSONA_ENV=$env_file PERSONA_CHECK=1
    act() { python3 -I "$repo/tests/persona/act.py" "$@" >/dev/null; }
    mode() { choractl call construct cube.list '{}' | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['mode'], 'front', d['front'])"; }
    focus() { choractl state "$1" 2>/dev/null | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin)['focus']))"; }
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
    space() { choractl call construct construct.list '{}' | python3 -c "
import sys,json; d=json.load(sys.stdin); c=d['camera']
print('camera', [round(c['x']), round(c['z'])], 'panes', [(p['name'], [round(v) for v in p['at']]) for p in d['panes']])"; }
    pane_at() { choractl query 'kind:surface' | python3 -c "
import sys,json
for x in json.load(sys.stdin):
    r=(x.get('rects') or {}).get('HEADLESS-1')
    if x['owner']=='construct' and x['id']=='$1' and x.get('visible') and r: print(round(r[0]+r[2]/2), round(r[1]+r[3]/2)); break"; }
    if [[ $what == co64 ]]; then
        act click 640 364; sleep 5; act key Super+c; sleep 6
        at=$(pane_at s1)
        words=$(shown construct-hints-)
        ok=no; [[ $words == *"Fly in"* && $words == *"Pull out"* ]] && ok=yes
        echo "run $i: after Super+c: mode $(mode), construct $(focus construct), pane s1 at [$at], hint bar [$words], click and pull named: $ok"
        if [[ -n $at ]]; then
            act drag ${at% *} ${at#* } $(( ${at% *} + 880 )) ${at#* }; sleep 3
            echo "  drag on the pane: mode $(mode), $(space), pane s1 at [$(pane_at s1)], construct $(focus construct)"
        fi
    else
        for _ in $(seq 40); do [[ -n $(shown construct-start | grep -o 'A cube of desktops') ]] && break; sleep 0.5; done
        if [[ ${OPEN:-row} == key ]]; then act key Super+c; else act click 640 468; fi
        for _ in $(seq 30); do choractl call construct cube.list '{}' >/dev/null 2>&1 && break; sleep 0.5; done
        for _ in $(seq 20); do [[ -z $(shown construct-start | grep -o 'A cube of desktops') ]] && break; sleep 0.5; done
        [[ -n $(shown construct-start | grep -o 'A cube of desktops') ]] && echo "  the start card still shows after the row"
        sleep 4
        [[ $what == co65in ]] && { act key Return; sleep 4; }
        act click 154 52
        for _ in $(seq 20); do grep -q '^fuzzel' "$PERSONA_CALLS" && break; sleep 0.5; done
        sleep 3; act click 550 294; sleep 6
        bar=$(shown construct-door-); words=$(shown construct-hints-)
        cur=$(focus construct)
        ok=no; [[ $cur == *surface* && ( -n $bar || $words == *"Step back"* ) ]] && ok=yes
        echo "run $i: after Foot: mode $(mode), construct $cur, xp $(focus xp), $(space), top bar [$bar], hint bar [$words], in the program, way out shown: $ok"
        act key Escape; sleep 2
        echo "  after Escape: mode $(mode), construct $(focus construct), $(space)"
        act key Alt+Escape; sleep 3
        echo "  after Alt+Escape: mode $(mode), construct $(focus construct), $(space), top bar [$(shown construct-door-)]"
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
