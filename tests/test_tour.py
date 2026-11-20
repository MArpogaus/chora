#!/usr/bin/python3
"""Selftest of the tour recorder (demo/tour), without a session: an act whose `expect` fails is a
red row of the timeline (and run.py would exit 1), an act whose expect holds is green, a `todo`
stub is neither, and every row carries `contract`, `cells` and `capture`.

Usage: python3 -I tests/test_tour.py [-v]
"""

import pathlib
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "demo" / "tour"))
sys.path.insert(0, str(ROOT / "tests" / "acts"))
sys.path.insert(0, str(ROOT / "tests"))

import importlib.util  # noqa: E402

import assemble  # noqa: E402
import test_acts  # noqa: E402
import tour  # noqa: E402
from harness import Expect  # noqa: E402

_spec = importlib.util.spec_from_file_location("tour_run", ROOT / "demo" / "tour" / "run.py")
run = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(run)


def snap(*owners):
    """A sound snapshot of the look xp: one shown window per owner (here only xp), or none."""
    if not owners:
        return test_acts.snap(states={"xp": {"windows": [], "focus": {"holder": None}, "open": [], "busy": []}}, nodes=[], toplevels=set())
    return test_acts.snap()


class FakeSession:
    env = {}

    def ctl(self, *args, **kw):
        return ""


class TourRows(unittest.TestCase):
    def tour(self, snaps):
        feed = iter(snaps)
        work = tempfile.mkdtemp(prefix="tour-test-")
        t = tour.Tour(FakeSession(), work, work + "/w", checker=tour.Checker(lambda: next(feed)))
        t.begin("1 test", "01-test")
        t.run_segment = lambda label="": None
        return t

    def act(self, t, **kw):
        t.act("look", "an act", "something", **kw)
        t.done()
        t.flush()

    def test_failing_expect_is_a_red_row(self):
        t = self.tour([snap(), snap()])
        self.act(t, expect=Expect(new_window={"owner": "xp"}), contract="T1", cells=["doors launcher-door"], capture="Q")
        row = t.rows[0]
        self.assertEqual(row["status"], "RED")
        self.assertTrue(row["red"])
        line = assemble.row_line("1 test", row)
        self.assertIn("RED:", line)
        self.assertEqual(run.tally(t.rows)["red"], 1)

    def test_holding_expect_is_green(self):
        t = self.tour([snap(), snap("xp")])
        self.act(t, expect=Expect(new_window={"owner": "xp"}))
        self.assertEqual(t.rows[0]["status"], "green")
        self.assertEqual(t.rows[0]["red"], [])
        self.assertTrue(assemble.row_line("1 test", t.rows[0]).rstrip().endswith("green |"))

    def test_todo_stub_is_not_judged(self):
        t = self.tour([snap(), snap()])
        self.act(t, expect=Expect(new_window={"owner": "xp"}), todo=True)
        self.assertEqual(t.rows[0]["status"], "todo")
        self.assertEqual(run.tally(t.rows)["todo"], 1)
        self.assertEqual(t.armed, [])

    def test_row_fields(self):
        t = self.tour([snap(), snap()])
        self.act(t, contract="T3", cells=["gate-xp-question.sh"], capture="Q+1")
        row = t.rows[0]
        self.assertEqual((row["contract"], row["cells"], row["capture"]), ("T3", ["gate-xp-question.sh"], "Q+1"))

    def test_no_snapshot_is_red(self):
        def boom():
            raise OSError("no socket")

        work = tempfile.mkdtemp(prefix="tour-test-")
        t = tour.Tour(FakeSession(), work, work + "/w", checker=tour.Checker(boom))
        t.begin("1 test", "01-test")
        t.run_segment = lambda label="": None
        self.act(t, expect=Expect(gone=0))
        self.assertEqual(t.rows[0]["status"], "RED")

    def test_known_red_does_not_fail(self):
        t = self.tour([snap(), snap()])
        self.act(t, expect=Expect(new_window={"owner": "xp"}), known="listed")
        row = t.rows[0]
        self.assertEqual(row["status"], "known")
        count = run.tally(t.rows)
        self.assertEqual((count["red"], count["known"]), (0, 1))
        self.assertIn("known: listed", assemble.row_line("1 test", row))

    def test_check_flag(self):
        self.assertFalse(tour.checking())


if __name__ == "__main__":
    unittest.main()
