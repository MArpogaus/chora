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
    """A task left open by an earlier goal cannot leak into the next: `end`
    leaves the end state as the begin of what follows, so a driver that
    forgets `begin` still holds the next goal against the state it found."""

    def test_the_end_of_a_task_is_the_begin_of_the_next(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, one_window("s1"))
            run.save("start", snap(), {})
            run_end(run, "T7")
            saved = run.load("begin")
            self.assertIsNotNone(saved, "no begin after the end")
            self.assertEqual(saved[0].toplevels, {"s1"})

    def test_a_window_left_open_by_goal_one_is_no_new_window_of_goal_two(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, one_window("s1"))
            run.save("start", snap(), {})
            status, out = run_end(run, "T1")
            self.assertEqual(status, 0, out)
            status, out = run_end(run, "T1")
            self.assertEqual(status, 1, "the window of goal one counts for goal two")
            self.assertIn("no new focused window", out)

    def test_an_explicit_begin_still_wins(self):
        with tempfile.TemporaryDirectory() as folder:
            run = FakeRun(folder, one_window("s1"))
            run.save("start", snap(), {})
            run_end(run, "T7")
            run.scene = one_window("s2")
            run.save("begin", snap(), {})
            status, out = run_end(run, "T1")
            self.assertEqual(status, 0, out)


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
            run.save("start", cube_snap("navigate", front=3), {})
            _, out = run_end(run, "T7")
            line = next(x for x in out.splitlines() if x.startswith("task T7"))
            self.assertIn("scene: cube stood back, face 3, pane room no, walk no", line)


if __name__ == "__main__":
    unittest.main()
