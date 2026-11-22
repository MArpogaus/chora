#!/bin/bash
# usage: repro.sh REPO CLICK(0|1)
repo=$1; click=$2
d=/tmp/claude-0/sl/r$click; rm -rf $d; mkdir $d
(cd $repo && CHORA_BIN=${BIN:-$repo/target/debug} CHORA_TEST_FLAGS=--test-injected-press-is-real PERSONA_DIR=$d tests/persona/session.sh tiling+xp 1280x800 >$d.out 2>&1 &)
for i in $(seq 60); do [ -f $d/session.env ] && break; sleep 1; done
sleep 20
set -a; . $d/session.env; set +a; export PATH
if [ "$click" = 1 ]; then
  choractl --name persona inject <<<'[{"pointer_move":{"position":[640,425]}},{"button":{"button":272,"state":"pressed"}},{"button":{"button":272,"state":"released"}}]'; sleep 1
fi
python3 -I $repo/tests/persona/act.py key Super+Return; sleep 4
choractl query kind:surface | python3 -c "
import json,sys
for n in json.load(sys.stdin): print('surface',n['id'],'owner',n['owner'],'parent',n['parent'],'focused',n.get('focused'))"
choractl state tiling | python3 -c "
import json,sys; d=json.load(sys.stdin); print('tiling', d['focus'], [(w.get('floating'),w.get('desks')) for w in d['windows']])"
python3 -I $repo/tests/persona/act.py key Super+2; sleep 2
choractl query kind:surface | python3 -c "
import json,sys
for n in json.load(sys.stdin): print('after Super+2: surface',n['id'],'owner',n['owner'],'visible',n.get('visible'))"
choractl state tiling | python3 -c "
import json,sys; d=json.load(sys.stdin); print('after Super+2 tiling', d['desks'], [{k:w.get(k) for k in ('desks','floating','shown','hidden')} for w in d['windows']])"
choractl query kind:surface | python3 -c "
import json,sys
for n in json.load(sys.stdin): print('rects',n['id'],n.get('rects'),n.get('opacity'))"
$repo/tests/persona/session.sh --stop $d/session.env
