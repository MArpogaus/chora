import json, sys, time
sys.path.insert(0, "/home/user/chora/tests/acts"); sys.path.insert(0, "/home/user/chora/tests/acts/looks")
import harness, construct as c
res = {}
with harness.Session("construct") as s:
    c.open_cube(s)
    s.inject([{"pointer_move": {"position": [640, 400]}}])
    harness.program("chora-probe", "window", "40", "120", "200", "--report", log="probe.log")(s)
    time.sleep(2)
    probe = s.work / "probe.log"
    def presses(): return sum(1 for l in open(probe) if l.startswith("press"))
    def minimized(): return [w.get("minimized") for w in s.state("xp")["windows"]]
    def super_d(pos):
        s.inject([{"pointer_move": {"position": pos}}]); time.sleep(0.3)
        m0 = minimized(); s.inject([{"shortcut": {"keys": "Super+d"}}]); time.sleep(1.0); m1 = minimized()
        if m1 != m0:
            s.inject([{"shortcut": {"keys": "Super+d"}}]); time.sleep(1.0)
        return m0 != m1
    res["xp_key_pointer_on_face_1to1"] = super_d([640, 400])
    s.call("construct", "view.step_back"); time.sleep(1.5)
    res["xp_key_pointer_on_cube_navigate"] = super_d([640, 400])
    res["xp_key_pointer_beside_cube_navigate"] = super_d([20, 20])
    s.call("construct", "view.step_in"); time.sleep(4)
    win = next(n for n in s.core.call("scene.query", {"selector": "kind:surface"}) if (n.get("surface") or {}).get("app_id") == "chora-probe")
    p0 = presses(); l0 = s.cube()
    s.inject([{"pointer_move": {"position": [600, 400]}}, {"key": {"key": "Super_L", "state": "pressed"}},
              {"button": {"state": "pressed", "hold": True}}, {"pointer_move": {"position": [400, 400]}},
              {"pointer_move": {"position": [200, 400]}}, {"button": {"state": "released"}},
              {"key": {"key": "Super_L", "state": "released"}}])
    time.sleep(1.5)
    res["super_drag_on_face"] = {"presses_to_window": presses() - p0, "front_before": l0["front"], "front_after": s.cube()["front"],
        "construct_busy": s.state("construct")["busy"]}
    logs = s.new_log_text() if s.log_marks else {}
    res["grabs_note"] = [l for p in s.logs() for l in open(p, errors="replace") if "BTN_LEFT" in l][-5:]
print(json.dumps(res, indent=1))
