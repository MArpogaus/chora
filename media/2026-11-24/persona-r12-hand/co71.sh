#!/bin/bash
# co-71 (r12): the key sheet opened from a face closes by Close, Esc and a click outside; no click passes through.
# Usage: VARIANT=close|esc|out co71.sh N TAG
set -u
repo=/home/user/chora
export ROUND_DIR=${ROUND_DIR:-/tmp/persona-round}
n=$1 tag=$2 var=${VARIANT:-close}
out=$ROUND_DIR/$tag-co71-$var.log
: >"$out"
one() {
    local i=$1 name=$tag-co71$var-$1 env_file opened closed pos fin
    rm -rf "$ROUND_DIR/keep/$name"; rm -f "$HOME/.cache/fuzzel"
    env_file=$("$repo/tests/persona/round.sh" start "$name" construct 1280x800 | sed -n 's/^PERSONA_ENV=//p')
    [[ -n $env_file ]] || { echo "run $i: no session"; return; }
    set -a; . "$env_file"; set +a
    export PERSONA_ENV=$env_file PERSONA_CHECK=1
    act() { python3 -I "$repo/tests/persona/act.py" "$@" >/dev/null; }
    openl() { choractl state construct | python3 -c "import sys,json; print(json.load(sys.stdin)['open'])"; }
    for _ in $(seq 40); do choractl query '*' | grep -q 'A cube of desktops' && break; sleep 0.5; done
    act click 640 468
    for _ in $(seq 30); do choractl call construct cube.list '{}' >/dev/null 2>&1 && break; sleep 0.5; done
    sleep 5; act key Return; sleep 4; act key Super+F1; sleep 3
    opened=$(openl)
    # logical centre of the Close hit and a point outside the sheet
    pos=$(choractl query '*' | python3 -c "
import sys,json
d=json.load(sys.stdin)
for x in d:
    if x.get('kind')=='text' and x.get('text')=='Close' and x.get('rects') and x['id'].startswith('construct-help'):
        r=list(x['rects'].values())[0]; print(round(r[0]+r[2]/2), round(r[1]+r[3]/2)); break")
    act look
    case $var in
        close) act click ${pos% *} ${pos#* } ;;
        esc) act key Escape ;;
        out) act click 20 780 ;;
    esac
    sleep 3
    closed=$(openl)
    act look
    fin=$(choractl state construct | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin)['focus']))")
    "$repo/tests/persona/round.sh" stop "$name" >/dev/null
    ok=no; [[ $opened == *help* && $closed != *help* ]] && ok=yes
    echo "run $i: sheet open before: $opened, close at [$pos], after $var: $closed, focus $fin; closed: $ok"
}
for i in $(seq "$n"); do ( one "$i" ) 2>&1 | tee -a "$out"; done
echo "co71 $var $tag: $(grep -c 'closed: yes$' "$out")/$n closed" | tee -a "$out"
