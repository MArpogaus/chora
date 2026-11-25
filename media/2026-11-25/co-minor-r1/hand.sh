#!/bin/bash
# hand.sh VARIANT TAG : a fresh construct session, then steps per VARIANT; dumps the state.
set -u
repo=/home/user/chora
SCR=/tmp/claude-0/-home-user-chora/b4725ac0-de08-5091-9fb8-43c68be09a0b/scratchpad/h
export ROUND_DIR=$SCR/pr ROUND_KEEP=$SCR/pr/keep
var=$1 tag=$2
act() { python3 -I "$repo/tests/persona/act.py" "$@" >/dev/null; }
dump() {
  choractl call construct cube.list '{}' | python3 -c "
import sys,json; d=json.load(sys.stdin); print('  front', d['front'], 'mode', d['mode'], end='')"
  choractl state construct | python3 -c "import sys,json; d=json.load(sys.stdin); print(', core focus', json.dumps(d['focus']), 'open', d['open'], 'panes', [p.get('name') for p in d.get('panes',[])] if 'panes' in d else '')"
  echo -n "  xp focus "; choractl state xp 2>/dev/null | python3 -c "import sys,json
try: print(json.dumps(json.load(sys.stdin)['focus']))
except Exception: print('n/a')"
  choractl query 'kind:surface' | python3 -c "
import sys,json
for x in json.load(sys.stdin): print('  surface', x['id'], 'owner', x['owner'], 'focused', x.get('focused'), 'rect', (x.get('rects') or {}).get('VIRTUAL-1') or (x.get('rects') or {}).get('HEADLESS-1'))"; }
name=$tag-$var
rm -rf "$ROUND_DIR/keep/$name"; rm -f "$HOME/.cache/fuzzel"
env_file=$("$repo/tests/persona/round.sh" start "$name" construct 1280x800 | sed -n 's/^PERSONA_ENV=//p')
[[ -n $env_file ]] || { echo "no session"; exit 1; }
set -a; . "$env_file"; set +a
export PERSONA_ENV=$env_file PERSONA_CHECK=1
for _ in $(seq 40); do choractl query '*' | grep -q 'A cube of desktops' && break; sleep 0.5; done
[[ $var == maxfull ]] || act click 640 468
for _ in $(seq 30); do choractl call construct cube.list '{}' >/dev/null 2>&1 && break; sleep 0.5; done
sleep 5
case $var in
 maxfull) act click 480 363; sleep 6; act key Super+c; sleep 8; echo before; dump; "$repo/tests/persona/round.sh" begin "$name"; act click 149 51; sleep 3; act look; act click 550 294; sleep 6; echo after start; dump; python3 -I "$repo/tests/persona/check.py" end C3 2>&1 | grep -v '^breach' | head;;
 xs) act key Return; sleep 4; act key Super+t; sleep 6; echo inface; choractl state xp | python3 -c "import sys,json; print(json.load(sys.stdin)['windows'])"; act key Alt+Escape; sleep 4; echo stoodback; choractl state xp | python3 -c "import sys,json; print(json.load(sys.stdin)['windows'])"; dump;;
 max) act key Return; sleep 4; act key Super+t; sleep 6; act key Alt+Escape; sleep 4; echo before foot; dump; "$repo/tests/persona/round.sh" begin "$name"; act key Super+space; sleep 3; act type -- foot; sleep 2; act key Return; sleep 6; echo after foot; dump; python3 -I "$repo/tests/persona/check.py" end C3 2>&1 | grep -v '^breach' | head;;
 back) act key Super+t; sleep 6; echo after Super+t; dump; "$repo/tests/persona/round.sh" begin "$name"; act key Super+space; sleep 3; act type -- foot; sleep 2; act key Return; sleep 6; echo after foot; dump; python3 -I "$repo/tests/persona/check.py" end C3 2>&1 | grep -v '^breach' | head;;
 sheet) act key Return; sleep 4; act key Super+F1; sleep 3; act key Super+t; sleep 8; echo after Super+t; dump; act look; cp "$PERSONA_SHOTS/$(ls "$PERSONA_SHOTS" | tail -1)" "$SCR/$tag-rcr1-1.png"; act key Escape; sleep 3; echo after Escape; dump; act type -- abc; sleep 2; act look; cp "$PERSONA_SHOTS/$(ls "$PERSONA_SHOTS" | tail -1)" "$SCR/$tag-co-78.png";;
 foot) act key Return; sleep 4; act key Super+t; sleep 6; echo after Super+t; dump; "$repo/tests/persona/round.sh" begin "$name"; act key Super+space; sleep 3; act type -- foot; sleep 2; act key Return; sleep 6; echo after foot; dump; python3 -I "$repo/tests/persona/check.py" end C3 2>&1 | grep -v '^breach' | head;;
esac
act look
cp "$PERSONA_SHOTS/$(ls "$PERSONA_SHOTS" | tail -1)" "$SCR/$tag-$var.png"
"$repo/tests/persona/round.sh" stop "$name" >/dev/null
