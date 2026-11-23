"""Selftest of tests/persona/check.py without a session: the contracts and
the reads of the checker on planted snapshots (run by tests/persona-selftest.sh).

Usage: python3 -I tests/persona/test_check.py
"""

import contextlib
import io
import json
import pathlib
import sys
import tempfile
import time
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import check  # noqa: E402
from harness import Snapshot  # noqa: E402


def surface(sid, owner, focused=False):
    node = {"id": sid, "kind": "surface", "owner": owner, "parent": "root", "meta": {}}
    if focused:
        node["focused"] = True
    return node


def snap(states=None, nodes=None, toplevels=(), cube=None, clients=("xp",)):
    return Snapshot(
        states={} if states is None else states,
        nodes=[] if nodes is None else nodes,
        toplevels=set(toplevels),
        colours={},
        wallpaper=set(),
        clients=set(clients),
        cube=cube,
    )


def ctx(look="xp", **more):
    return check.Ctx(look, [], [], None, [], [], **more)


def verdict(task, before, after, look="xp"):
    return check.contracts(task, ctx(look), before).check(before, after)


class Clock(unittest.TestCase):
    """T4:clock reads the clock as gate-doors clock-door does: the text of
    the tray of the look is a time."""

    def tray(self, text):
        return {"id": "xp-tray-HEADLESS-1", "kind": "group", "owner": "xp", "meta": {"text": text}}

    def test_a_tray_that_shows_a_time_is_a_clock(self):
        for text in ("3:27 PM", "10:05", "Volume 12:40 PM"):
            with self.subTest(text):
                self.assertEqual(verdict("T4:clock", snap(), snap(nodes=[self.tray(text)])), [])

    def test_a_tray_without_a_time_is_no_clock(self):
        self.assertIn("no clock", verdict("T4:clock", snap(), snap(nodes=[self.tray("")]))[0])
        self.assertIn("no clock", verdict("T4:clock", snap(), snap(nodes=[self.tray("Ready")]))[0])

    def test_no_node_is_no_clock(self):
        self.assertIn("no clock", verdict("T4:clock", snap(), snap())[0])

    def test_a_node_named_clock_still_counts(self):
        node = {"id": "mc-clock", "kind": "text", "owner": "mc", "text": "11:11"}
        self.assertEqual(verdict("T4:clock", snap(), snap(nodes=[node])), [])


class Reader:
    """The session as read_states sees it: a look, the clients, the states."""

    def __init__(self, label, clients, states):
        self.label, self._clients, self._states = label, set(clients), states

    def states(self):
        return {k: v for k, v in self._states.items() if k == self.label}

    def clients(self):
        return self._clients

    def state(self, look):
        return self._states.get(look)


def construct_state():
    """The construct with one window on a face: the window is the view of the
    xp that manages that face, and xp holds the focus."""
    return {
        "windows": [],
        "views": [{"surface": "s2", "owner": "xp", "pane": "face-1"}],
        "focus": {"holder": None},
        "open": [],
        "busy": [],
    }


def xp_state():
    return {
        "windows": [{"surface": "s2", "shown": True}],
        "focus": {"holder": "xp", "surface": "s2"},
        "open": [],
        "busy": [],
    }


class ManagerChild(unittest.TestCase):
    """The xp that a face of the cube starts owns its windows: I1 and I3 hold
    for them with the state of xp read, which the construct session lacks:
    state.get of the construct lists the window as a view of xp and the core
    focuses it for xp. Without xp's state both invariants read an owner that
    is no look (round p2r2: I1 x23, I3 x21 on demo)."""

    def invariants(self, states):
        nodes = [surface("s2", "xp", focused=True)]
        after = snap(states=states, nodes=nodes, toplevels={"s2"}, clients=("construct", "xp"))
        out = []
        for key in ("I1", "I3"):
            out += check.invariants.ALL[key](after, after, check.Act("x"))
        return out

    def test_the_construct_alone_breaks_i1_and_i3_on_the_window_of_xp(self):
        self.assertTrue(self.invariants({"construct": construct_state()}))

    def test_the_state_of_the_xp_child_is_read_in_a_construct_session(self):
        both = {"construct": construct_state(), "xp": xp_state()}
        got = check.read_states(Reader("construct", ("construct", "xp"), both))
        self.assertEqual(sorted(got), ["construct", "xp"])
        self.assertEqual(self.invariants(got), [])

    def test_no_xp_yet_no_state_of_it(self):
        both = {"construct": construct_state(), "xp": xp_state()}
        got = check.read_states(Reader("construct", ("construct",), both))
        self.assertEqual(sorted(got), ["construct"])
        none = {"construct": construct_state(), "xp": None}
        got = check.read_states(Reader("construct", ("construct", "xp"), none))
        self.assertEqual(sorted(got), ["construct"])

    def test_the_other_looks_read_only_themselves(self):
        both = {"xp": xp_state(), "construct": construct_state()}
        got = check.read_states(Reader("xp", ("xp", "construct"), both))
        self.assertEqual(sorted(got), ["xp"])

    def test_a_window_of_xp_in_no_windows_list_stays_a_breach(self):
        lost = xp_state() | {"windows": []}
        out = self.invariants({"construct": construct_state(), "xp": lost})
        self.assertTrue(any("s2" in line for line in out), out)


class FakeSession:
    label = "xp"

    def __init__(self, env):
        self.env = env


class FakeRun(check.Run):
    """A run in a temp folder whose scene is the snapshot `scene`."""

    def __init__(self, folder, scene):
        self.dir = pathlib.Path(folder)
        self.state_dir = self.dir / "check"
        self.state_dir.mkdir()
        self.calls = self.dir / "calls.log"
        self.calls.write_text("")
        self.scene = scene
        env = {"PERSONA_LOG": str(self.dir / "actions.jsonl"), "PERSONA_VOICE": str(self.dir)}
        self.s = FakeSession(env)

    def snapshot(self, marks=None):
        return self.scene

    def marks(self):
        return {}


def one_window(sid="s1"):
    nodes = [surface(sid, "xp", focused=True)]
    return snap(nodes=nodes, toplevels={sid})


def run_end(run, *args):
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        status = check.end(run, list(args))
    return status, out.getvalue()


class BeginPerTask(unittest.TestCase):
    """Every goal is held against the `begin` taken right before it: `end`
    uses that begin up, so the next goal never compares with the begin or the
    end of the one before (r3: a repeat `end` compared with the end state of
    the previous one and meant nothing)."""

    def test_end_without_a_begin_is_no_verdict(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, one_window("s1"))
            run.save("start", snap(), {})
            with self.assertRaises(SystemExit) as raised:
                run_end(run, "T7")
            self.assertIn("begin", str(raised.exception))

    def test_the_end_of_a_goal_leaves_no_begin_for_the_next(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, one_window("s1"))
            run.save("start", snap(), {})
            run.save("begin", snap(), {})
            run_end(run, "T7")
            self.assertIsNone(run.load("begin"), "the begin of goal one is left for goal two")

    def test_a_window_left_open_by_goal_one_is_no_new_window_of_goal_two(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, one_window("s1"))
            run.save("start", snap(), {})
            run.save("begin", snap(), {})
            status, out = run_end(run, "T1")
            self.assertEqual(status, 0, out)
            run.save("begin", run.scene, {})
            status, out = run_end(run, "T1")
            self.assertEqual(status, 1, "the window of goal one counts for goal two")
            self.assertIn("no new focused window", out)

    def test_a_second_end_without_a_begin_does_not_pass_on_the_first_end(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, one_window("s1"))
            run.save("start", snap(), {})
            run.save("begin", snap(), {})
            run_end(run, "T1")
            with self.assertRaises(SystemExit):
                run_end(run, "T1")


def win(sid, owner, rect=(14, 54, 1252, 702), shown=True):
    return {"surface": sid, "owner": owner, "shown": shown, "rect": list(rect)}


def look_state(holder_surface, windows, holder="self"):
    focus = {"holder": holder, "surface": holder_surface}
    return {"windows": windows, "focus": focus, "open": [], "busy": []}


def goal(run, before, after, task, hist=None, look=None):
    """One goal: `begin` is the snapshot `before`, the scene at `end` is
    `after`, `hist` the toplevels the persona's actions saw."""
    run.save("begin", before, {})
    run.scene = after
    hist = [[]] if hist is None else hist
    with open(run.state_dir / "history.jsonl", "a", encoding="utf-8") as f:
        for i, tops in enumerate(hist):
            row = {"t": time.time() + 1 + i, "action": "x", "open": [], "helper": {}}
            row |= {"page": None, "toplevels": list(tops), "keysheet": None}
            f.write(json.dumps(row) + "\n")
    if look:
        run.s.label = look
    return run_end(run, task)


def task_line(out, task):
    return next((x for x in out.splitlines() if x.startswith(f"task {task}")), "")


class Closed(unittest.TestCase):
    """L1 holds the goal to its own begin and says "closed" for a window the
    persona closed, whether it was open at the begin or came during the goal."""

    def after(self):
        state = look_state("s1", [win("s1", "xp")], holder="xp")
        return snap(states={"xp": state}, nodes=[surface("s1", "xp", True)], toplevels={"s1"})

    def test_a_window_opened_and_closed_in_the_goal_is_closed(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            status, out = goal(run, snap(), self.after(), "L1", hist=[{"s1", "s2"}])
            self.assertEqual(status, 0, out)
            self.assertIn("task L1 pass", out)

    def test_a_window_of_the_begin_that_the_persona_closed_is_closed(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            status, out = goal(run, one_window("s0"), self.after(), "L1")
            self.assertEqual(status, 0, out)

    def test_no_window_closed_is_red_and_never_says_gone(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            status, out = goal(run, snap(), self.after(), "L1", hist=[{"s1"}])
            self.assertEqual(status, 1, out)
            line = task_line(out, "L1")
            self.assertIn("closed", line)
            self.assertNotIn("gone", line)


class NoConfig(unittest.TestCase):
    """L2 measures a reload of the config; with no config it is "not
    measured (no config)", neither red nor green."""

    def after(self):
        state = look_state("s1", [win("s1", "xp")], holder="xp")
        return snap(states={"xp": state}, nodes=[surface("s1", "xp", True)], toplevels={"s1"})

    def test_a_missing_config_is_not_measured(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            status, out = goal(run, snap(), snap(), "L2")
            self.assertEqual(status, 0, out)
            self.assertIn("task L2 not measured (no config)", out)
            self.assertNotIn("pass", task_line(out, "L2"))

    def test_an_empty_config_is_not_measured(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            (pathlib.Path(folder) / "config").mkdir()
            (pathlib.Path(folder) / "config" / "chora.yaml").write_text("")
            status, out = goal(run, snap(), snap(), "L2")
            self.assertEqual(status, 0, out)
            self.assertIn("not measured (no config)", out)

    def test_a_config_is_measured(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            (pathlib.Path(folder) / "xp.yaml").write_text("editor: mousepad\n")
            run.save("begin", snap(), {})
            (pathlib.Path(folder) / "xp.yaml").write_text("editor: notepad\n")
            run.scene = self.after()
            status, out = run_end(run, "L2")
            self.assertEqual(status, 0, out)
            self.assertIn("task L2 pass", out)
            status, out = goal(run, snap(), snap(), "L2")
            self.assertEqual(status, 1, out)

    def test_a_run_that_changed_nothing_is_red(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            (pathlib.Path(folder) / "xp.yaml").write_text("editor: mousepad\n")
            status, out = goal(run, snap(), self.after(), "L2")
            self.assertEqual(status, 1, out)
            self.assertIn("the config value did not change", task_line(out, "L2"))

    def test_a_change_the_look_did_not_take_is_red(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            run.save("begin", snap(), {})
            (pathlib.Path(folder) / "xp.yaml").write_text("editor: notepad\n")
            run.scene = snap()
            status, out = run_end(run, "L2")
            self.assertEqual(status, 1, out)
            self.assertIn("the look is not back with the focus", task_line(out, "L2"))


class GivenAway(unittest.TestCase):
    """L3 does not count against the look the windows that a later goal gave
    to another look."""

    def scene(self, holders, extra=()):
        tiling = look_state("s1", [win("s1", "tiling")], holder=holders.get("tiling"))
        xp = look_state("s2", [win("s2", "xp")], holder=holders.get("xp"))
        nodes = [surface("s1", "tiling", True), surface("s2", "xp")] + list(extra)
        tops = {"s1", "s2"} | {n["id"] for n in extra}
        return snap(states={"tiling": tiling, "xp": xp}, nodes=nodes, toplevels=tops)

    def before(self):
        nodes = [surface("s1", "tiling", True), surface("s2", "tiling")]
        return snap(nodes=nodes, toplevels={"s1", "s2"})

    def test_a_holder_that_holds_a_given_window_does_not_count(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            after = self.scene({"tiling": "tiling", "xp": "xp"})
            status, out = goal(run, self.before(), after, "L3", look="tiling")
            self.assertEqual(status, 0, out)

    def test_only_the_given_window_holds_the_focus(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            after = self.scene({"xp": "xp"})
            status, out = goal(run, self.before(), after, "L3", look="tiling")
            self.assertEqual(status, 0, out)

    def test_a_holder_of_a_window_that_was_not_given_still_counts(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            xp = look_state("s3", [win("s3", "xp")], holder="xp")
            tiling = look_state("s1", [win("s1", "tiling")], holder="tiling")
            nodes = [surface("s1", "tiling", True), surface("s3", "xp")]
            after = snap(states={"tiling": tiling, "xp": xp}, nodes=nodes, toplevels={"s1", "s3"})
            before = snap(nodes=[surface("s1", "tiling", True)], toplevels={"s1"})
            status, out = goal(run, before, after, "L3", look="tiling")
            self.assertEqual(status, 1, out)


class NoEvent(unittest.TestCase):
    """A contract that holds state ("no window lost") is not reached when no
    observation came since the begin: neither pass nor FAIL."""

    def test_t7_without_an_event_is_not_reached(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, one_window("s1"))
            status, out = goal(run, one_window("s1"), one_window("s1"), "T7", hist=())
            self.assertEqual(status, 0, out)
            self.assertIn("task T7 not reached (no event since begin)", out)
            self.assertNotIn("pass", task_line(out, "T7"))

    def test_t7_with_an_event_is_judged(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, one_window("s1"))
            status, out = goal(run, one_window("s1"), one_window("s1"), "T7", hist=[["s1"]])
            self.assertIn("task T7 pass", out)

    def test_c2_and_l3_without_an_event_are_not_reached(self):
        for task in ("C2", "L3"):
            with self.subTest(task), tempfile.TemporaryDirectory() as folder:
                run = FakeRun(folder, one_window("s1"))
                _, out = goal(run, one_window("s1"), one_window("s1"), task, hist=())
                self.assertIn(f"task {task} not reached", out)


class Covered(unittest.TestCase):
    """T7: a given window that is shown but drawn under another window is not
    lost; it is a separate MINOR line (root order, known)."""

    def before(self):
        nodes = [surface("s1", "tiling", True), surface("s2", "tiling")]
        return snap(nodes=nodes, toplevels={"s1", "s2"})

    def after(self, order, shown=True):
        nodes = {"s1": surface("s1", "tiling", True), "s2": surface("s2", "xp")}
        states = {
            "tiling": look_state("s1", [win("s1", "tiling")], holder="tiling"),
            "xp": look_state("s2", [win("s2", "xp", shown=shown)], holder=None),
        }
        return snap(states=states, nodes=[nodes[i] for i in order], toplevels={"s1", "s2"})

    def test_a_covered_window_is_not_lost_and_is_a_minor_line(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            status, out = goal(run, self.before(), self.after(["s2", "s1"]), "T7")
            self.assertEqual(status, 0, out)
            self.assertIn("task T7 pass", out)
            self.assertIn("minor T7 covered: s2 under s1", out)

    def test_a_window_on_top_has_no_minor_line(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            status, out = goal(run, self.before(), self.after(["s1", "s2"]), "T7")
            self.assertEqual(status, 0, out)
            self.assertNotIn("minor", out)

    def test_a_window_that_is_gone_is_lost(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, snap())
            status, out = goal(run, self.before(), one_window("s1"), "T7")
            self.assertEqual(status, 1, out)
            self.assertIn("windows lost ['s2']", out)


class KnownRedNamesBreaches(unittest.TestCase):
    """A known-red row hides only the breaches it names (capture, match); an
    unlisted breach of the same look and invariant prints."""

    ROWS = [{"look": "xp", "id": "I6", "capture": "xp-a", "match": "wants 14 px", "finding": "f", "slice": "s"}]

    def lines(self, *texts):
        rows = [{"time": 1.0, "action": "x", "look": "xp", "invariant": "I6", "breach": t} for t in texts]
        return "".join(json.dumps(r) + "\n" for r in rows)

    def found(self, *texts):
        original = check.harness.known_red
        check.harness.known_red = lambda: self.ROWS
        try:
            with tempfile.TemporaryDirectory() as folder:
                run = FakeRun(folder, snap())
                (run.dir / "breaches.jsonl").write_text(self.lines(*texts))
                return [b["breach"] for b in run.breaches()]
        finally:
            check.harness.known_red = original

    def test_a_listed_breach_is_hidden(self):
        self.assertEqual(self.found("(xp-a) guide wants 14 px"), [])

    def test_another_element_of_the_same_pair_prints(self):
        text = "(xp-b) guide wants 14 px"
        self.assertEqual(self.found("(xp-a) guide wants 14 px", text), [text])

    def test_another_text_of_the_same_pair_prints(self):
        text = "(xp-a) overlaps (xp-c)"
        self.assertEqual(self.found(text), [text])


def pane_node(visible=True):
    return {"id": "construct-pane-s1", "kind": "group", "owner": "construct", "visible": visible}


def cube_snap(mode="navigate", front=2, faces=4, nodes=(), busy=()):
    cube = {"faces": [{"output": f"V-{i}"} for i in range(faces)], "front": front, "mode": mode}
    state = construct_state() | {"busy": list(busy)}
    return snap(states={"construct": state}, nodes=list(nodes), cube=cube, clients=("construct",))


class SceneState(unittest.TestCase):
    """Each contract line says the look state that the scene shows, so a
    report does not infer it from the steps of the persona (r2: "cube open"
    where the shot shows the pane room)."""

    def test_the_cube_stood_back_on_a_face(self):
        got = check.scene_state(cube_snap("navigate", front=2), "construct")
        self.assertEqual(got["cube"], "stood back")
        self.assertEqual(got["face"], 2)
        self.assertEqual(got["pane room"], "no")
        self.assertEqual(got["walk"], "no")

    def test_the_cube_in_a_face(self):
        got = check.scene_state(cube_snap("use", front=1), "construct")
        self.assertEqual((got["cube"], got["face"]), ("in a face", 1))

    def test_no_cube_open_names_no_face(self):
        got = check.scene_state(cube_snap("use", faces=0, front=0), "construct")
        self.assertEqual(got["cube"], "closed")
        self.assertNotIn("face", got)

    def test_the_pane_room_is_read_from_the_pane_nodes(self):
        room = check.scene_state(cube_snap("use", faces=0, nodes=[pane_node()]), "construct")
        self.assertEqual(room["pane room"], "yes")
        hidden = check.scene_state(cube_snap("use", faces=0, nodes=[pane_node(False)]), "construct")
        self.assertEqual(hidden["pane room"], "no")

    def test_the_walk_is_read_from_the_busy_word(self):
        got = check.scene_state(cube_snap("use", faces=0, busy=["walk"]), "construct")
        self.assertEqual(got["walk"], "yes")

    def test_a_look_without_a_cube_says_only_what_it_has(self):
        self.assertEqual(check.scene_state(snap(), "xp"), {})

    def test_the_line_of_a_task_carries_the_scene(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, cube_snap("navigate", front=3))
            run.s.label = "construct"
            run.save("begin", cube_snap("navigate", front=3), {})
            _, out = goal(run, cube_snap("navigate", front=3), cube_snap("navigate", front=3), "T7")
            line = next(x for x in out.splitlines() if x.startswith("task T7"))
            self.assertIn("scene: cube stood back, face 3, pane room no, walk no", line)


class WithChildren(unittest.TestCase):
    """with_children counts a window of a face manager as shown only while
    its face is at the front (RPM-1)."""

    def cube(self, front=0):
        faces = [{"manager": "xp", "output": "face-0"}, {"manager": None, "output": "face-1"}]
        return {"faces": faces, "front": front}

    def child(self, output, shown=True):
        return {"windows": [{"surface": "s2", "shown": shown, "output": output}]}

    def run_it(self, cube, state, label="construct", clients=("construct", "xp")):
        snapshot = snap(states={"construct": {}}, cube=cube, clients=clients)
        out = check.with_children(Reader(label, clients, {"xp": state}), snapshot)
        return out.states.get("xp")

    def test_a_child_window_on_the_front_face_is_shown(self):
        got = self.run_it(self.cube(0), self.child("face-0"))
        self.assertTrue(got["windows"][0]["shown"])

    def test_a_child_window_on_another_face_is_not_shown(self):
        got = self.run_it(self.cube(1), self.child("face-0"))
        self.assertFalse(got["windows"][0]["shown"])

    def test_a_window_the_child_does_not_show_stays_hidden(self):
        got = self.run_it(self.cube(0), self.child("face-0", shown=False))
        self.assertFalse(got["windows"][0]["shown"])

    def test_a_client_that_manages_no_face_is_not_read(self):
        cube = {"faces": [{"manager": None, "output": "face-0"}], "front": 0}
        self.assertIsNone(self.run_it(cube, self.child("face-0")))

    def test_another_look_reads_no_children(self):
        self.assertIsNone(self.run_it(self.cube(0), self.child("face-0"), label="xp"))


class WaitForApp(unittest.TestCase):
    """ph-1: round.sh probe waits for the window of the probe in the scene,
    not a fixed time."""

    def nodes(self, app):
        return [{"id": "s1", "kind": "surface", "surface": {"app_id": app}}]

    def test_the_window_is_there_at_once(self):
        self.assertTrue(check.wait_for_app(lambda: self.nodes("chora-probe"), "chora-probe", 20))

    def test_the_window_comes_after_a_few_reads(self):
        reads = iter([[], [], self.nodes("chora-probe")])
        got = check.wait_for_app(lambda: next(reads), "chora-probe", 20, sleep=lambda _: None)
        self.assertTrue(got)

    def test_another_window_is_no_probe(self):
        ticks = iter(range(0, 100, 5))
        got = check.wait_for_app(
            lambda: self.nodes("foot"),
            "chora-probe",
            20,
            sleep=lambda _: None,
            clock=lambda: next(ticks),
        )
        self.assertFalse(got)


def face_snap(mode="use", shown=True, holder="xp", new=("s2",), owner="xp"):
    """The construct with its cube open and xp on its faces: xp holds `s2`
    (shown on the front face, or on another one) after a window of the face
    came; the construct holds nothing."""
    cube = {
        "faces": [{"output": f"V-{i}", "manager": "xp"} for i in range(4)],
        "front": 0,
        "mode": mode,
    }
    xp = {
        "windows": [{"surface": "s2", "shown": shown, "output": "V-0" if shown else "V-1"}],
        "focus": {"holder": holder, "surface": "s2" if holder else None},
    }
    states = {"construct": construct_state(), "xp": xp}
    nodes = [surface(sid, owner, focused=True) for sid in new]
    return snap(states=states, nodes=nodes, toplevels=new, cube=cube, clients=("construct", "xp"))


class FaceManager(unittest.TestCase):
    """co-48, co-49 (p2-r6-co r6): in use mode the pointer and the keys of
    a face reach its window manager and its windows (docs/cube-modes.md,
    States). A window that the xp of the front face owns and focuses is a
    focused window of the construct session, not a red of C3, T1 or T3."""

    def contract(self, task, after, hist=(), entries=()):
        before = face_snap(new=())
        c = check.Ctx("construct", list(hist), [], None, [], list(entries))
        return check.contracts(task, c, before).check(before, after)

    def test_c3_takes_the_manager_of_the_front_face_as_the_holder(self):
        self.assertEqual(self.contract("C3", face_snap()), [])

    def test_t1_takes_a_new_window_of_the_front_face(self):
        self.assertEqual(self.contract("T1", face_snap()), [])

    def test_t3_takes_the_manager_of_the_front_face_back_with_the_focus(self):
        hist = [{"helper": {"construct": w}} for w in ("idle", "working", "idle")]
        entries = [{"role": "agent", "text": "Hello"}]
        got = self.contract("T3", face_snap(), hist=hist, entries=entries)
        self.assertEqual(got, [])

    def test_a_window_of_a_face_that_does_not_show_is_no_holder(self):
        self.assertNotEqual(self.contract("C3", face_snap(shown=False)), [])

    def test_in_navigate_mode_the_manager_is_no_holder(self):
        self.assertNotEqual(self.contract("C3", face_snap(mode="navigate")), [])


if __name__ == "__main__":
    unittest.main()
