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

import contextlib  # noqa: E402
import importlib.util  # noqa: E402
import io  # noqa: E402

import assemble  # noqa: E402
import launch  # noqa: E402
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

    def rect(self, selector, output=None):
        return None


class Fixed(tour.Checker):
    """A checker whose judgement is a fixed list of red lines (the row logic, not the invariants)."""

    def __init__(self, source, red):
        super().__init__(source)
        self.red = red

    def judge(self, row, expect, before, after):
        return list(self.red)


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
        self.act(t, expect=Expect(new_window={"owner": "xp"}), known="expect: listed")
        row = t.rows[0]
        self.assertEqual(row["status"], "known")
        count = run.tally(t.rows)
        self.assertEqual((count["red"], count["known"]), (0, 1))
        self.assertIn("known: listed", assemble.row_line("1 test", row))

    def test_row_without_expect_is_inv_not_green(self):
        """rt-1: no `expect` is not a green row."""
        t = self.tour([snap(), snap()])
        self.act(t)
        row = t.rows[0]
        self.assertEqual(row["status"], "inv")
        count = run.tally(t.rows)
        self.assertEqual((count["inv"], count["green"], count["red"]), (1, 0, 0))
        self.assertTrue(assemble.row_line("1 test", row).rstrip().endswith("inv |"))

    def test_inv_row_with_broken_invariant_is_red(self):
        t = self.tour([snap(), snap()])
        t.checker = Fixed(lambda: snap(), ["I5: overprint"])
        self.act(t)
        self.assertEqual(t.rows[0]["status"], "RED")

    def known_tour(self, red):
        """rt-4: `known` names the invariant it hides; the checker answers `red`."""
        t = self.tour([snap(), snap()])
        t.checker = Fixed(lambda: snap(), red)
        return t

    def test_known_hides_only_its_invariant(self):
        t = self.known_tour(["I5: overprint"])
        self.act(t, known="I4: the Super release")
        row = t.rows[0]
        self.assertEqual(row["status"], "RED")
        self.assertEqual(row["red"], ["I5: overprint"])
        self.assertEqual(run.tally(t.rows)["red"], 1)

    def test_known_keeps_its_own_red_known(self):
        t = self.known_tour(["I4: the Super release"])
        self.act(t, known="I4: the Super release")
        self.assertEqual(t.rows[0]["status"], "known")

    def test_known_and_another_red_is_red(self):
        t = self.known_tour(["I4: a", "I5: b"])
        self.act(t, known="I4")
        row = t.rows[0]
        self.assertEqual(row["status"], "RED")
        self.assertEqual(row["red"], ["I5: b"])

    def test_known_act_that_is_green_says_drop_known(self):
        t = self.known_tour([])
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            self.act(t, known="I4: the Super release")
        self.assertIn("drop known", out.getvalue())
        self.assertIn("known act 1 is green", out.getvalue())
        self.assertNotEqual(t.rows[0]["status"], "known")

    def test_known_must_name_an_invariant(self):
        t = self.known_tour([])
        with self.assertRaises(ValueError):
            t.act("look", "an act", "something", known="listed")

    def test_missing_pane_raises(self):
        """rt-5: a missing pane is an error, not a skipped act."""
        from chapters import construct

        with self.assertRaisesRegex(RuntimeError, "no pane"):
            construct.pane_and_face(FakeSession(), "HEADLESS-1")

    def test_timeline_shows_contract_cells_capture(self):
        """rt-6: contract, cells and capture are in the timeline."""
        for column in ("contract", "cells", "capture"):
            self.assertIn(column, assemble.HEAD)
        t = self.tour([snap(), snap()])
        self.act(t, contract="T3", cells=["gate-xp-question.sh", "act: the helper asks"], capture="Q+1")
        line = assemble.row_line("1 test", t.rows[0])
        self.assertIn("T3", line)
        self.assertIn("gate-xp-question.sh", line)
        self.assertIn("Q+1", line)
        table = assemble.table([("1 test", t.rows, 0.0)], "test")
        self.assertEqual(table[4].count("|"), table[-1].count("|"))
        self.assertEqual(len(assemble.HEAD), line.count(" | ") + 1)

    def test_read_waits_for_busy_empty(self):
        """rt-3: an act is read at a quiet frame (every busy empty)."""
        busy = test_acts.snap(states={"xp": {"windows": [], "focus": {"holder": None}, "open": [], "busy": ["anim"]}}, nodes=[], toplevels=set())
        t = self.tour([snap(), busy, busy, snap("xp")])
        t.quiet_gap = 0
        self.act(t, expect=Expect(new_window={"owner": "xp"}))
        self.assertEqual(t.rows[0]["status"], "green")

    def test_notify_config_is_one_helper(self):
        """rt-2: the notify timing lives in one place."""
        text = launch.notify_config()
        self.assertIn("shortest: 600", text)
        self.assertIn("longest: 600", text)
        self.assertIn("position: bottom_right", launch.notify_config(position="bottom_right"))

    def test_check_flag(self):
        self.assertFalse(tour.checking())


if __name__ == "__main__":
    unittest.main()
