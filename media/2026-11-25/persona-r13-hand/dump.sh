#!/bin/bash
# usage: dump.sh NAME  -> state dump line
set -a; . $(cat /tmp/claude-0/r13/$1.env); set +a
echo "mode/front: $(choractl call construct cube.list '{}' | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['mode'],d['front'])")"
choractl call construct construct.list '{}' | python3 -c "import sys,json; d=json.load(sys.stdin); print('panes',[(p['name']) for p in d['panes']])"
for f in construct xp; do echo "$f state: $(choractl state $f 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d.get('focus')), json.dumps(d.get('holder')))" 2>&1)"; done
choractl query '*' | python3 -c "
import sys,json
d=json.load(sys.stdin); by={x['id']:x for x in d}
def vis(i):
    while i:
        n=by.get(i)
        if not n or n.get('visible') is False: return False
        i=n.get('parent')
    return True
print('hints:',' | '.join(x['text'] for x in d if x['id'].startswith('construct-hints-') and x.get('kind')=='text' and vis(x['id']) and x.get('rects')))"
