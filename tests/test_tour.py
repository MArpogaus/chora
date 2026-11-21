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
import dataclasses  # noqa: E402
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


CHAPTERS_B = ("mc", "construct")


def act_calls(chapter):
    """The `t.act(...)` calls of a chapter file, as {keyword names}: the rows of the chapter, read
    from the source (no session needed)."""
    import ast

    tree = ast.parse((ROOT / "demo" / "tour" / "chapters" / f"{chapter}.py").read_text())
    return [
        (call.lineno, {k.arg for k in call.keywords})
        for call in ast.walk(tree)
        if isinstance(call, ast.Call) and isinstance(call.func, ast.Attribute) and call.func.attr == "act"
    ]


class ChaptersB(unittest.TestCase):
    """F7 (design/tour-v9.md section 8, lane 5): every act row of mc and construct says what it
    expects, so `run.py --check` counts no `inv` row for these chapters."""

    def test_every_row_has_an_expect(self):
        for chapter in CHAPTERS_B:
            rows = act_calls(chapter)
            self.assertTrue(rows, chapter)
            bare = [line for line, keys in rows if "expect" not in keys and "todo" not in keys]
            self.assertEqual(bare, [], f"{chapter}: rows without `expect` (would be `inv`), by line")

    def test_rows_name_their_contract(self):
        """Each chapter holds the contracts of the design rows it plays (sections 3 and 4). Not here: T6
        (m11, the launcher list is a layer surface of fuzzel that the scene does not list) and T1, T3
        of construct (c1, c7: the walk room and the strip helper are not in the cube scene)."""
        want = {"mc": {"M1", "T3", "T2", "T1", "T4"}, "construct": {"C1", "C2", "T2"}}
        for chapter, contracts in want.items():
            text = (ROOT / "demo" / "tour" / "chapters" / f"{chapter}.py").read_text()
            missing = sorted(c for c in contracts if f'contract="{c}' not in text)
            self.assertEqual(missing, [], f"{chapter}: contracts not on any row")

    def test_mc_expects_turn_red_on_a_planted_state(self):
        """Planted wrong state: each mc check says red where the state is not the one the act names."""
        from chapters import mc

        after = snap("xp")
        wrong_page = FakeSession()
        wrong_page.call = lambda client, method, params=None: {"current": "ingredients"}
        self.assertTrue(mc.page_is(wrong_page, "boil")(after))
        self.assertEqual(mc.page_is(wrong_page, "ingredients")(after), [])
        hud = dataclasses.replace(after, nodes=[{"id": "mc-r0-footer-hud", "kind": "text", "visible": True, "text": "next step"}])
        self.assertTrue(mc.hud_says("how much garlic")(hud))
        self.assertEqual(mc.hud_says("next step")(hud), [])
        none = dataclasses.replace(after, blocks={"notify": {"shown": []}})
        self.assertTrue(mc.notice_shown("Timer: spaghetti, 9 minutes")(none))
        card = dataclasses.replace(after, blocks={"notify": {"shown": [{"id": 1, "summary": "Timer: spaghetti, 9 minutes"}]}})
        self.assertEqual(mc.notice_shown("Timer: spaghetti, 9 minutes")(card), [])
        self.assertTrue(mc.clock_shown()(after))
        chat = dataclasses.replace(after, blocks={"assistant": {"shown": []}})
        self.assertTrue(mc.chat_is(True)(chat))
        self.assertEqual(mc.chat_is(False)(chat), [])

    def test_mc_column_counts_the_texts_that_are_drawn(self):
        """mc-54 (cb-4): the closed list of all keys (`mc-r1-right-*`) and a hidden column keep
        `visible: true` but no rect on any output, so nothing is drawn. The count of the column
        reads the texts with a rect: 24 drawn before Super+n, none while it hides, 24 again after."""
        from chapters import mc

        def text(region, name, rects):
            return {"id": f"mc-{region}-right-{name}", "kind": "text", "visible": True, "rects": rects}

        drawn = {"HEADLESS-1": [888.0, 110.0, 204.0, 28.0]}
        column = lambda rects: [text("r0", f"keys-{i}", rects) for i in range(24)]
        stale = [text("r1", f"keys-{i}", {}) for i in range(48)]
        before = column(drawn) + stale
        hidden = column({})
        back = column(drawn)
        self.assertEqual(mc.column_count(before), 24)
        self.assertEqual(mc.column_count(hidden), 0)
        self.assertEqual(mc.column_count(back), 24)
        self.assertEqual(mc.column_fewer_than(24)(dataclasses.replace(snap("xp"), nodes=hidden)), [])
        self.assertTrue(mc.column_fewer_than(24)(dataclasses.replace(snap("xp"), nodes=back)))
        self.assertEqual(mc.column_back_to(24)(dataclasses.replace(snap("xp"), nodes=back)), [])
        self.assertTrue(mc.column_back_to(24)(dataclasses.replace(snap("xp"), nodes=hidden)))

    def test_construct_expects_turn_red_on_a_planted_state(self):
        """Planted wrong state: the cube is on face 0 where the act says face 1; s1 is still a pane."""
        from chapters import construct

        cube = {"faces": [{}, {}, {}, {}], "front": 0, "mode": "navigate", "turning": False}
        after = dataclasses.replace(snap("xp"), cube=cube)
        self.assertTrue(construct.cube_is(front=1).check(after, after))
        self.assertEqual(construct.cube_is(front=0, mode="navigate").check(after, after), [])
        self.assertTrue(construct.cube_is(mode="use").check(after, after))
        pane = dataclasses.replace(after, nodes=[{"id": "s1", "kind": "surface", "owner": "xp"}])
        self.assertTrue(construct.window_is_pane(pane))
        self.assertEqual(construct.window_back_on_face(pane), [])


if __name__ == "__main__":
    unittest.main()
