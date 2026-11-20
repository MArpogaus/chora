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


if __name__ == "__main__":
    unittest.main()
