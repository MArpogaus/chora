import json,sys,importlib.util,tomllib,collections
def load(p):
    s=importlib.util.spec_from_file_location("td",p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
d=json.load(open('/tmp/r7/tour-diff-r7.json'))
def run(code,toml):
    m=load(code);rows=tomllib.load(open(toml,'rb'))["change"];out={}
    for r in d:
        if r['verdict']=='none' or 'scene' not in r: out[(r['act'],r['capture'])]=(r['verdict'],None);continue
        v=m.verdict(r['act'],r['scene'],not r['blobs'],rows,None,r['blobs'])
        out[(r['act'],r['capture'])]=(v['verdict'],v.get('cause'))
    return out
a=run('/tmp/old/tour_diff_old.py','/tmp/old/intended_old.toml')
b=run('tests/tour-diff.py','demo/tour/intended.toml')
for n,x in (("before",a),("after",b)): print(n,dict(collections.Counter(v[0] for v in x.values())))
for k in a:
    if a[k][0]!=b[k][0]: print("moved",k,a[k][0],'->',b[k][0])
