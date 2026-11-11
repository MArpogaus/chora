import sys, os, time, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from multi import *
L, S, out_of = start_multi()
print(L.dir, out_of, L.pids)
