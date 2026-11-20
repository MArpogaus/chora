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


if __name__ == "__main__":
    unittest.main()
