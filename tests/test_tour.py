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

import ast  # noqa: E402
import contextlib  # noqa: E402
import dataclasses  # noqa: E402
import importlib.util  # noqa: E402
import io  # noqa: E402
import pathlib  # noqa: E402

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

    def test_the_tours_own_choractl_is_not_a_disconnect(self):
        """A `choractl` call of the script is not a client of the product: its end is no I14."""
        before = dataclasses.replace(snap("xp"), clients={"xp", "acts", "choractl"})
        row = {"description": "an act"}
        self.assertEqual(tour.Checker(lambda: snap("xp")).judge(row, None, before, snap("xp")), [])

    def test_dismiss_notices_closes_every_shown_card(self):
        calls = []

        class Notify(tour.Session):
            def call(self, client, method, params=None):
                calls.append((client, method, params))
                return {"shown": [{"id": 7}, {"id": 9}]} if method == "state.get" else {}

        Notify({}, [], "").dismiss_notices()
        self.assertEqual([c for c in calls if c[1] == "notify.close"], [("notify", "notify.close", {"id": 7}), ("notify", "notify.close", {"id": 9})])

    def test_notify_config_is_one_helper(self):
        """rt-2: the notify timing lives in one place."""
        text = launch.notify_config()
        self.assertIn("shortest: 600", text)
        self.assertIn("longest: 600", text)
        self.assertIn("position: bottom_right", launch.notify_config(position="bottom_right"))

    def test_check_flag(self):
        self.assertFalse(tour.checking())


CHAPTERS_A = ("xp", "tiling", "handover")


def act_calls(chapter):
    """The `t.act(...)` calls of a chapter file: (line, the keyword names, the keyword nodes)."""
    tree = ast.parse((ROOT / "demo" / "tour" / "chapters" / f"{chapter}.py").read_text())
    return [
        (n.lineno, {k.arg for k in n.keywords}, {k.arg: k.value for k in n.keywords})
        for n in ast.walk(tree)
        if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute) and n.func.attr == "act"
    ]


def check_of(chapter, name):
    """A check function that a chapter exports (design/tour-v9.md: the rows of the chapter use it)."""
    module = importlib.import_module(f"chapters.{chapter}")
    fn = getattr(module, name, None)
    assert fn is not None, f"chapters/{chapter}.py has no check `{name}`"
    return fn


class ChaptersARows(unittest.TestCase):
    """F7 (design/tour-v9.md sections 1, 2, 5; lane CHAPTERS-A): every act row of xp, tiling and
    handover has an `expect` (no bare `inv` row) and a `contract` or `cells`, and an `expect` turns
    red on a wrong state."""

    def test_every_act_row_has_expect_and_contract_or_cells(self):
        for chapter in CHAPTERS_A:
            calls = act_calls(chapter)
            self.assertGreater(len(calls), 5, chapter)
            bare = [line for line, kw, _ in calls if "expect" not in kw or not ({"contract", "cells"} & kw)]
            self.assertEqual(bare, [], f"{chapter}: act rows without expect, or without contract and cells (lines)")

    def test_no_row_is_a_todo_stub(self):
        for chapter in CHAPTERS_A:
            todo = [line for line, kw, nodes in act_calls(chapter) if "todo" in kw and getattr(nodes["todo"], "value", True)]
            self.assertEqual(todo, [], f"{chapter}: todo rows (lines)")

    def test_xp_expect_turns_red_on_a_wrong_volume(self):
        """x7: the speaker sets 75 %; the mixer of the session holds 40 %."""
        volume_is = check_of("xp", "volume_is")
        state = pathlib.Path(tempfile.mkdtemp(prefix="tour-test-")) / "wpctl.state"
        state.write_text("0.40 0\n")
        want = Expect(verify=volume_is(75, state))
        self.assertTrue(want.check(snap(), snap()), "a mixer at 40 % passes for 75 %")
        state.write_text("0.75 0\n")
        self.assertEqual(want.check(snap(), snap()), [])

    def test_tiling_expect_turns_red_on_a_wrong_window_count(self):
        """t3: four terminals tile; the session shows one window."""
        windows_are = check_of("tiling", "windows_are")
        self.assertTrue(Expect(verify=windows_are(4, look="xp")).check(snap(), snap()), "one window passes for four")
        self.assertEqual(Expect(verify=windows_are(1, look="xp")).check(snap(), snap()), [])

    def test_handover_expect_turns_red_on_a_wrong_owner(self):
        """h3: the window is given to mc; the node still says xp owns it."""
        owner_is = check_of("handover", "owner_is")
        want = Expect(verify=owner_is("s1", "mc"))
        self.assertTrue(want.check(snap(), snap()), "owner xp passes for mc")
        mine = snap(nodes=[test_acts.surface("s1", "mc", focused=True)])
        self.assertEqual(want.check(snap(), mine), [])


if __name__ == "__main__":
    unittest.main()
