#!/bin/bash
# co-77 hand repro (co-repro r1). Variant a: cube row, Return (step into face 0), Super+t (terminal in the face),
# Super+Space (Programs list), click Foot, type "echo rN". Variant b: same without Super+t (empty face).
# Usage: co77.sh a|b N TAG
set -u
repo=/home/user/chora
SCR=/tmp/claude-0/-home-user-chora/80d14a8c-e6e2-583a-9723-62e303384623/scratchpad
export ROUND_DIR=$SCR/pr ROUND_KEEP=$SCR/pr/keep
var=$1 n=$2 tag=$3
out=$SCR/co-repro/$tag-co77-$var.log
: >"$out"
act() { python3 -I "$repo/tests/persona/act.py" "$@" >/dev/null; }
dump() {
  choractl call construct cube.list '{}' | python3 -c "
import sys,json; d=json.load(sys.stdin); print('  front', d['front'], 'mode', d['mode'], end='')"
  choractl state construct | python3 -c "import sys,json; d=json.load(sys.stdin); print(', core focus', json.dumps(d['focus']), 'open', d['open'])"
  echo -n "  xp focus "; choractl state xp 2>/dev/null | python3 -c "import sys,json
try: print(json.dumps(json.load(sys.stdin)['focus']))
except Exception: print('n/a')"
  choractl query 'kind:surface' | python3 -c "
import sys,json
for x in json.load(sys.stdin): print('  surface', x['id'], 'owner', x['owner'], 'visible', x.get('visible'), 'rect', (x.get('rects') or {}).get('HEADLESS-1'))"; }
one() {
  local i=$1 name=$tag-co77$var-$1 env_file
  rm -rf "$ROUND_DIR/keep/$name"; rm -f "$HOME/.cache/fuzzel"
  env_file=$("$repo/tests/persona/round.sh" start "$name" construct 1280x800 | sed -n 's/^PERSONA_ENV=//p')
  [[ -n $env_file ]] || { echo "run $i: no session"; return; }
  set -a; . "$env_file"; set +a
  export PERSONA_ENV=$env_file PERSONA_CHECK=1
  for _ in $(seq 40); do choractl query '*' | grep -q 'A cube of desktops' && break; sleep 0.5; done
  act click 640 468
  for _ in $(seq 30); do choractl call construct cube.list '{}' >/dev/null 2>&1 && break; sleep 0.5; done
  sleep 5; act key Return; sleep 4
  echo "run $i step in:"; dump
  if [[ $var == a ]]; then
    act key Super+t; sleep 6
    echo "run $i after Super+t (terminal 1):"; dump
  fi
  act key Super+space; sleep 3
  act look
  act type -- foot; sleep 2; act look; act key Return; sleep 6
  echo "run $i after Foot from Programs:"; dump
  act type -- "echo r$i"; sleep 2
  act look
  cp "$PERSONA_SHOTS/$(ls "$PERSONA_SHOTS" | tail -1)" "$SCR/co-repro/$tag-co77-$var-run$i-final.png"
  "$repo/tests/persona/round.sh" stop "$name" >/dev/null
  echo "run $i done"
}
for i in $(seq "$n"); do ( one "$i" ) 2>&1 | tee -a "$out"; done
