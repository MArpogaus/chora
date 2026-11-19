import sys, time
sys.path.insert(0, "/home/user/chora/tests/acts"); sys.path.insert(0, "/home/user/chora/tests/acts/looks")
import harness, construct as c
out, build = sys.argv[1], sys.argv[2]
def shot(s, name): open(f"{out}/wm7a-{build}-{name}.png", "wb").write(s.capture(s.outputs()[0]["name"], 1280)[1])
with harness.Session("construct") as s:
    c.open_cube(s)
    s.inject([{"pointer_move": {"position": [640, 400]}}])
    harness.program("chora-probe", "window", "40", "120", "200")(s)
    time.sleep(5); shot(s, "hosted-rest-1to1")
    s.call("construct", "cube.turn", {"by": 1, "ms": 3000}); time.sleep(1.0); shot(s, "hosted-turn-third")
    time.sleep(3); s.call("construct", "view.step_back"); time.sleep(1.5); shot(s, "hosted-stood-back")
