#!/bin/bash
# co-78 hand repro: cube row, Return (step in), Super+F1 (key sheet), [Super+t], Escape.
# Usage: co78.sh sheet|control N TAG   (sheet = with Super+t, control = without)
set -u
repo=/home/user/chora
SCR=/tmp/claude-0/-home-user-chora/80d14a8c-e6e2-583a-9723-62e303384623/scratchpad
export ROUND_DIR=$SCR/pr ROUND_KEEP=$SCR/pr/keep
var=$1 n=$2 tag=$3
out=$SCR/co-repro/$tag-co78-$var.log
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
for x in json.load(sys.stdin): print('  surface', x['id'], 'owner', x['owner'])"; }
one() {
  local i=$1 name=$tag-co78$var-$1 env_file
  rm -rf "$ROUND_DIR/keep/$name"; rm -f "$HOME/.cache/fuzzel"
  env_file=$("$repo/tests/persona/round.sh" start "$name" construct 1280x800 | sed -n 's/^PERSONA_ENV=//p')
  [[ -n $env_file ]] || { echo "run $i: no session"; return; }
  set -a; . "$env_file"; set +a
  export PERSONA_ENV=$env_file PERSONA_CHECK=1
  for _ in $(seq 40); do choractl query '*' | grep -q 'A cube of desktops' && break; sleep 0.5; done
  act click 640 468
  for _ in $(seq 30); do choractl call construct cube.list '{}' >/dev/null 2>&1 && break; sleep 0.5; done
  sleep 5; act key Return; sleep 4
  act key Super+F1; sleep 3
  echo "run $i sheet open:"; dump
  if [[ $var == sheet ]]; then act key Super+t; sleep 6; echo "run $i after Super+t:"; dump; fi
  act key Escape; sleep 3
  echo "run $i after Escape:"; dump
  act look
  cp "$PERSONA_SHOTS/$(ls "$PERSONA_SHOTS" | tail -1)" "$SCR/co-repro/$tag-co78-$var-run$i-final.png"
  "$repo/tests/persona/round.sh" stop "$name" >/dev/null
  echo "run $i done"
}
for i in $(seq "$n"); do ( one "$i" ) 2>&1 | tee -a "$out"; done
