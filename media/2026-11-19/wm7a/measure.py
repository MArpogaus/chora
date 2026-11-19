"""WM7a measurements of the hosted xp session (construct hosts, one xp on four faces)."""
import json, os, signal, sys, time
sys.path.insert(0, "/home/user/chora/tests/acts"); sys.path.insert(0, "/home/user/chora/tests/acts/looks")
import harness, construct as c
OUT = sys.argv[1]
def stats(s):
    return s.core.call("render.stats")["outputs"]
def rest_cost(s, label, secs=5.0):
    a = stats(s); t = time.monotonic(); time.sleep(secs); b = stats(s); dt = time.monotonic() - t
    row = {k: {f: round((b[k][f] - a[k].get(f, 0)) / dt, 2) for f in ("frames", "damaged_frames", "passes")} for k in b if k in a}
    print(label, json.dumps(row)); return row
def frame_ms(s):
    lines = [l for p in s.logs() if p.name.startswith("chora") for l in open(p, errors="replace") if "frame" in l.lower() and "ms" in l]
    return lines[-3:]
res = {}
with harness.Session("construct") as s:
    res["flat_rest"] = rest_cost(s, "flat construct at rest (/s)")
    c.open_cube(s)
    s.inject([{"pointer_move": {"position": [640, 400]}}])
    harness.program("chora-probe", "window", "40", "120", "200", "--report", log="probe.log")(s)
    time.sleep(2)
    res["hosted_rest"] = rest_cost(s, "hosted cube at rest, use mode (/s)")
    s.core.call("methods.call", {"client": "construct", "method": "view.step_back", "params": {}}); time.sleep(1.5)
    res["hosted_navigate_rest"] = rest_cost(s, "hosted cube stood back (/s)")
    s.core.call("methods.call", {"client": "construct", "method": "view.step_in", "params": {}}); time.sleep(4)
    claims = [(n["id"], n.get("owner"), n.get("claim")) for n in s.core.call("scene.query", {"selector": "*"}) if n.get("claim")]
    res["claims"] = claims; print("claims", claims)
    probe_log = s.work / "probe.log"
    def keys_seen(): return [l.strip() for l in open(probe_log) if l.startswith("key")]
    win = next(n for n in s.core.call("scene.query", {"selector": "kind:surface"}) if (n.get("surface") or {}).get("app_id") == "chora-probe")
    print("probe window", win["id"], win.get("owner"), win.get("rects"))
    s.inject([{"pointer_move": {"position": [100, 180]}}, {"click": {}}]); time.sleep(0.5)
    out = {}
    for where, pos in (("inside the claim (on the face)", [100, 180]),):
        s.inject([{"pointer_move": {"position": pos}}]); time.sleep(0.3)
        before = len(keys_seen()); s.inject([{"text": {"text": "a"}}]); time.sleep(0.6)
        out[where] = len(keys_seen()) - before
    res["keys_on_face"] = out; print("keys reach the window", out)
    before = len(keys_seen()); t0 = time.monotonic()
    s.inject([{"shortcut": {"keys": "Ctrl+Alt+Right"}}]); time.sleep(0.6)
    res["turn_key_to_window"] = len(keys_seen()) - before
    lst = s.cube(); res["after_turn_key"] = {k: lst[k] for k in ("front", "turning", "mode")}; print("after turn key", res["after_turn_key"])
    s.call("construct", "cube.turn", {"to": 0}); time.sleep(1)
    grab = harness.Core(str(s.runtime / "chora.sock") if False else s.env["CHORA_SOCKET"], name="grab-measure", events=["input.button"])
    try:
        grab.call("input.grab", {"binding": "Super+BTN_LEFT"})
        presses_before = sum(1 for l in open(probe_log) if l.startswith("press"))
        s.inject([{"pointer_move": {"position": [300, 300]}}, {"shortcut": {"keys": "Super+BTN_LEFT"}}]) if False else None
        s.inject([{"pointer_move": {"position": [300, 300]}}, {"key": {"key": "Super_L", "state": "pressed"}}, {"click": {}}, {"key": {"key": "Super_L", "state": "released"}}])
        time.sleep(0.8)
        ev = grab.drain(); res["grab_events"] = ev[:4]
        res["grab_press_to_window"] = sum(1 for l in open(probe_log) if l.startswith("press")) - presses_before
        print("lone Super+BTN_LEFT grab on a face: events", ev[:4], "presses to window", res["grab_press_to_window"])
    except RuntimeError as e:
        res["grab_error"] = str(e); print("grab", e)
    grab.close()
    s.call("construct", "cube.turn", {"by": 1, "ms": 3000}); time.sleep(0.4)
    a = stats(s); time.sleep(1.0); b = stats(s)
    res["turn_frames_per_s"] = {k: b[k]["frames"] - a[k]["frames"] for k in b if k in a}; print("frames in 1 s of a turn", res["turn_frames_per_s"])
    time.sleep(0.2)
    pid = c.pid_of(s, "chora-construct"); xpid = c.pid_of(s, "chora-xp")
    t0 = time.monotonic(); os.kill(pid, signal.SIGKILL)
    marks = {}
    def surf(): return [(n["id"], n.get("owner"), n.get("parent")) for n in s.core.call("scene.query", {"selector": "kind:surface"})]
    restarted = False
    while time.monotonic() - t0 < 25:
        t = round(time.monotonic() - t0, 2); cl = s.clients(); outs = [o["name"] for o in s.outputs()]
        if "xp" not in cl: marks.setdefault("xp_gone", t)
        if not os.path.exists(f"/proc/{xpid}"): marks.setdefault("xp_process_gone", t)
        if len(outs) == 1: marks.setdefault("virtual_outputs_gone", t)
        if any(o is None or o == "xp" for _, o, _ in surf()) and "xp" not in cl: marks.setdefault("window_without_live_owner", t)
        if not restarted and t >= 3.0:
            cmd, extra = harness.LOOKS["construct"]; s.spawn([str(s.bin / cmd[0]), *cmd[1:]], "construct-again.log", **extra); restarted = True; marks["construct_restarted"] = t
        if any(o == "construct" for _, o, _ in surf()): marks.setdefault("window_back_with_construct", t); break
        time.sleep(0.1)
    res["kill_marks"] = marks; res["surfaces_after"] = surf(); print("kill marks", marks, surf())
    shot = s.capture(s.outputs()[0]["name"], 1280)[1]; open(f"{OUT}/after-kill-restart.png", "wb").write(shot)
json.dump(res, open(f"{OUT}/measure.json", "w"), indent=1)
