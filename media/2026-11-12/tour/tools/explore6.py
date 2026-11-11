import sys, os, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
L = SceneScript("cube", "scene-cube.sh", "light")
print(L.dir, L.pids)
