#!/usr/bin/python3
"""Selftest of the tour recorder (demo/tour), without a session: an act whose `expect` fails is a
red row of the timeline (and run.py would exit 1), an act whose expect holds is green, a `todo`
stub is neither, and every row carries `contract`, `cells` and `capture`.

Usage: python3 -I tests/test_tour.py [-v]
"""

import pathlib
import subprocess
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "demo" / "tour"))
sys.path.insert(0, str(ROOT / "tests" / "acts"))
sys.path.insert(0, str(ROOT / "tests"))

import ast  # noqa: E402
import json  # noqa: E402
from unittest import mock  # noqa: E402
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

    def capture(self, path, output=None):
        pass


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


class Shooter(FakeSession):
    """A session whose `capture` writes a small PNG."""

    def __init__(self):
        self.shots = []

    def capture(self, path, output=None):
        import test_tour_diff as ttd

        pathlib.Path(path).write_bytes(ttd.png(64, 36))
        self.shots.append((path, output))


def boxes_with(*rects):
    return {"HEADLESS-1": {"size": [1280, 720], "boxes": [{"node": f"t{i}", "kind": "text", "rect": list(r), "text": "hi", "font_size": 14} for i, r in enumerate(rects)]}}


class CaptureDumps(unittest.TestCase):
    """F7 r1: per Q capture the tour writes `<act>.<capture>.png` and `.json` where tests/tour-diff.py reads them."""

    def tour(self, slug="01-xp", boxes=None, snaps=None):
        work = tempfile.mkdtemp(prefix="tour-dump-")
        feed = iter(snaps or [snap(), test_acts.snap(boxes=boxes) if boxes else snap()])
        self.session = Shooter()
        t = tour.Tour(self.session, work, work + "/w", checker=tour.Checker(lambda: next(feed)))
        t.begin("1 test", slug)
        t.run_segment = lambda label="": None
        self.dir = pathlib.Path(work) / "captures"
        return t

    def act(self, t, capture="Q"):
        t.act("look", "an act", "something", capture=capture)
        t.done()
        t.flush()

    def diff(self):
        import test_tour_diff as ttd

        return ttd.diff

    def test_row_id_without_a_scene_is_the_letter_of_the_look_and_the_number(self):
        for slug, want in (("01-xp", "x1"), ("02-tiling", "t1"), ("03-mc-light", "m1"), ("03b-mc-dark", "md1"), ("04-construct", "c1"), ("05-handover", "h1")):
            t = self.tour(slug)
            self.act(t)
            self.assertEqual(t.rows[0]["id"], want, slug)

    def test_a_q_capture_writes_png_and_json(self):
        t = self.tour(boxes=boxes_with((10, 10, 50, 14), (100, 200, 30, 12)))
        self.act(t)
        self.assertTrue((self.dir / "x1.Q.png").read_bytes().startswith(b"\x89PNG"))
        data = json.loads((self.dir / "x1.Q.json").read_text())
        self.assertEqual([n["id"] for n in data["nodes"]], ["s1"])
        self.assertEqual(data["masks"], [[10, 10, 50, 14], [100, 200, 30, 12]])

    def test_tour_diff_reads_the_dump(self):
        t = self.tour(boxes=boxes_with((10, 10, 50, 14)))
        self.act(t)
        rows = self.diff().compare(self.dir, self.dir, [])
        self.assertEqual([(r["act"], r["capture"], r["verdict"]) for r in rows], [("x1", "Q", "same")])

    def test_a_capture_with_a_dot_keeps_its_act(self):
        t = self.tour()
        self.act(t, capture="Q+0.5")
        rows = self.diff().compare(self.dir, self.dir, [])
        self.assertEqual([(r["act"], r["capture"]) for r in rows], [("x1", "Q+0.5")])

    def test_a_build_without_scene_boxes_has_no_masks_and_still_a_dump(self):
        t = self.tour(snaps=[snap(), test_acts.snap(boxes=None)])
        self.act(t)
        self.assertEqual(json.loads((self.dir / "x1.Q.json").read_text())["masks"], [])
        self.assertTrue((self.dir / "x1.Q.png").exists())

    def test_a_baseline_without_boxes_is_not_an_i6_flip(self):
        """TOUR-BASE: a snapshot without `scene.boxes` (733967aa) reads no text box, so I6 is not green there, it
        is not measured; the row says "I6 not measured". rtb2-2: a candidate that measures it red is a regression unless a check
        row names I6."""
        import shutil

        t = self.tour(snaps=[snap(), test_acts.snap(boxes=None)])
        self.act(t)
        base = self.dir
        cand = pathlib.Path(tempfile.mkdtemp(prefix="tour-cand-"))
        for path in base.iterdir():
            shutil.copy(path, cand / path.name)
        data = json.loads((cand / "x1.Q.json").read_text())
        data["checks"]["I6"] = "red"
        (cand / "x1.Q.json").write_text(json.dumps(data))
        [row] = self.diff().compare(base, cand, [])
        self.assertEqual((row["verdict"], row["flipped"]), ("regression", ["I6"]), row)
        self.assertIn("I6", row["unmeasured"], row)
        [row] = self.diff().compare(base, cand, [{"act": "x1", "kind": "check", "check": "I6", "cause": "xp-39"}])
        self.assertEqual(row["verdict"], "change", row)

    def test_a_baseline_without_boxes_marks_the_box_checks_from_one_constant(self):
        """TOUR-BASE r2 (rtb-4, check 5): the checks that read `scene.boxes` are "not measured" on a snapshot without
        them, all from one constant; the reported-run part of I6 reads no box, so I6 stays measured where a run is reported."""
        t = self.tour(snaps=[snap(), test_acts.snap(boxes=None)])
        self.act(t)
        checks = json.loads((self.dir / "x1.Q.json").read_text())["checks"]
        want = ["I12", "I13", "I2", "I5", "I6", "I9"]
        self.assertEqual(sorted(tour.BOX_CHECKS), want)
        self.assertEqual(sorted(n for n, c in checks.items() if c == "not measured"), want)
        run = {"text": "hi", "rect": [0, 0, 10, 10], "node": "s1"}
        states = {"xp": {"windows": [], "focus": {"holder": None}, "open": [], "busy": [], "texts": [run]}}
        t = self.tour(snaps=[snap(), test_acts.snap(boxes=None, states=states, nodes=[], toplevels=set())])
        self.act(t)
        checks = json.loads((self.dir / "x1.Q.json").read_text())["checks"]
        self.assertNotEqual(checks["I6"], "not measured", checks)
        self.assertEqual(sorted(n for n, c in checks.items() if c == "not measured"), [n for n in want if n != "I6"])

    def test_a_new_take_keeps_no_stale_picture(self):
        """rw-1: a run folder with an old `x1.Q.png` (and an old H50 one the take did not copy) gets the new pictures."""
        t = self.tour(snaps=[snap() for _ in range(6)])
        self.dir.mkdir(parents=True)
        stale = b"\x89PNG stale"
        (self.dir / "x1.Q.png").write_bytes(stale)
        (self.dir / "x2.H50.png").write_bytes(stale)
        self.act(t)
        self.assertNotEqual((self.dir / "x1.Q.png").read_bytes(), stale)
        t.act("look", "another", "something", capture="H50")
        t.done()
        t.flush()
        self.assertNotEqual((self.dir / "x2.H50.png").read_bytes(), stale)

    def test_a_failed_capture_leaves_no_old_picture_behind(self):
        t = self.tour()
        self.dir.mkdir(parents=True)
        (self.dir / "x1.Q.png").write_bytes(b"old")
        self.session.capture = lambda path, output=None: (_ for _ in ()).throw(RuntimeError("no screen"))
        self.act(t)
        self.assertFalse((self.dir / "x1.Q.png").exists())

    def test_the_check_run_writes_no_dump(self):
        import os

        os.environ["TOUR_CHECK"] = "1"
        try:
            t = self.tour()
            self.act(t)
        finally:
            del os.environ["TOUR_CHECK"]
        self.assertFalse(self.dir.exists())

    def test_h50_takes_the_frame_in_the_middle_of_the_act(self):
        """The held frame is half way through the act, not the quiet frame after it; the pointer is masked."""
        import unittest.mock as mock

        class Frames(Shooter):
            def ctl(self, *args, **kw):
                import test_tour_diff as ttd

                out = pathlib.Path(args[args.index("--out") + 1])
                for n in range(1, int(args[args.index("--frames") + 1]) + 1):
                    (out / f"frame-{n:05d}.png").write_bytes(ttd.png(8, 8, lambda x, y, n=n: (n, 0, 0)))
                return ""

        work = tempfile.mkdtemp(prefix="tour-dump-")
        feed = iter([snap(), snap()])
        t = tour.Tour(Frames(), work, work + "/w", checker=tour.Checker(lambda: next(feed)))
        t.begin("1 test", "01-xp")
        t.px, t.py = 300.0, 200.0
        with mock.patch.object(tour.subprocess, "run"):
            t.act("pointer", "a drag", "it moves", settle=0.0, lead=0.0, tail=0.0, capture="H50")
            start = t.rows[0]["frame"]
            t.wait(2.0)
            end = t.frame
            t.done()
            t.flush()
        import test_tour_diff as ttd

        got = (pathlib.Path(work) / "captures" / "x1.H50.png").read_bytes()
        middle = (start + end) // 2
        self.assertEqual(got, ttd.png(8, 8, lambda x, y: (middle, 0, 0)))
        masks = json.loads((pathlib.Path(work) / "captures" / "x1.H50.json").read_text())["masks"]
        self.assertTrue(any(m[0] <= 300 <= m[0] + m[2] and m[1] <= 200 <= m[1] + m[3] for m in masks), masks)


class SceneIds(unittest.TestCase):
    """rw-2: dumps and intended.toml name an act by the scene id of design/tour-v9.md (`scene=` of the
    act), not by its number in the chapter; a later act of the same scene takes a letter (`h1b`)."""

    CHAPTERS = ("xp", "tiling", "mc", "construct", "handover")

    def acts(self, chapter):
        """(description source, scene id or None) of every `t.act(...)` of a chapter file, in file order."""
        tree = ast.parse((ROOT / "demo" / "tour" / "chapters" / f"{chapter}.py").read_text())
        calls = sorted((n for n in ast.walk(tree) if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute) and n.func.attr == "act"), key=lambda c: c.lineno)
        out = []
        for c in calls:
            scene = next((k.value.value for k in c.keywords if k.arg == "scene" and isinstance(k.value, ast.Constant)), None)
            out.append((ast.unparse(c.args[1]), scene))
        return out

    def test_every_act_names_its_scene(self):
        import re

        for chapter in self.CHAPTERS:
            bare = [d for d, scene in self.acts(chapter) if not (scene and re.fullmatch(r"[xtmch]\d+", scene))]
            self.assertEqual(bare, [], f"{chapter}: acts without a scene id like x3")

    @staticmethod
    def design_scenes():
        """The scene ids of the tables of design/tour-v9.md: the file of the checkout, else the one of
        origin/claude/session-log (the design lives on that branch)."""
        import re
        import subprocess

        path = ROOT / "design" / "tour-v9.md"
        if path.exists():
            text = path.read_text()
        else:
            run = subprocess.run(["git", "show", "origin/claude/session-log:design/tour-v9.md"], cwd=ROOT, capture_output=True, text=True)
            if run.returncode:
                raise unittest.SkipTest("design/tour-v9.md is neither in the checkout nor on origin/claude/session-log")
            text = run.stdout
        return set(re.findall(r"^\| ([xtmch]\d+) \|", text, re.M))

    def test_every_scene_id_is_one_of_design_tour_v9(self):
        known = self.design_scenes()
        self.assertTrue({"x1", "t1", "m1", "c1", "h1"} <= known, known)
        for chapter in self.CHAPTERS:
            odd = sorted({scene for d, scene in self.acts(chapter) if scene not in known and scene not in self.OFF_DESIGN.get(chapter, ())})
            self.assertEqual(odd, [], f"{chapter}: scene ids that design/tour-v9.md does not name")

    OFF_DESIGN = {"xp": ("x12",)}
    """rv-1: the three cube acts of xp (Ctrl+Alt+Right, Ctrl+Alt+Left, Super drag) keep `x12`, an id the design
    does not name: the one that fits, c3 (turn by key, by drag), is taken by 8 acts of the construct chapter, whose
    ids `c3`, `c3b` .. would be the same files in `captures/` (a take of the tour writes all chapters to one folder).
    The design says "Not in xp: the cube"; the PM decides whether it names x12 or the acts move."""

    def test_the_off_design_ids_are_three_acts_of_xp_and_the_design_has_not_caught_up(self):
        known = self.design_scenes()
        self.assertEqual(known & {"x12"}, set(), "design/tour-v9.md names x12 now: empty OFF_DESIGN")
        self.assertEqual([d[:20] for d, scene in self.acts("xp") if scene == "x12"], ["'Ctrl+Alt+Right: tur", "'Ctrl+Alt+Left: back", "'Super held, drags o"])

    def test_the_send_to_act_is_h2_and_start_is_h1(self):
        scene = {d: s for d, s in self.acts("handover")}
        send = [s for d, s in scene.items() if "Send To" in d]
        start = [s for d, s in scene.items() if d.startswith("'Click Start")]
        self.assertEqual((send, start), (["h2"], ["h1"]))

    def test_a_second_act_of_a_scene_has_its_letter(self):
        work = tempfile.mkdtemp(prefix="tour-scene-")
        t = tour.Tour(FakeSession(), work, work + "/w")
        t.begin("5 handover", "05-handover")
        t.run_segment = lambda label="": None
        for n, scene in enumerate(["h1", "h1", "h1", "h2", "h1"]):
            t.act("look", f"act {n}", "x", lead=0.0, tail=0.0, scene=scene)
        self.assertEqual([r["id"] for r in t.rows], ["h1", "h1b", "h1c", "h2", "h1d"])

    def test_the_chapters_use_each_scene_of_intended_toml(self):
        """intended.toml names a row by the id of the take (tour.act_id): the scene id, and for a later
        act of the same scene the scene id and a letter (`c2b`); one id scheme for take and table."""
        import tomllib

        table = tomllib.loads((ROOT / "demo" / "tour" / "intended.toml").read_text())
        ids = set()
        for chapter in self.CHAPTERS:
            seen = {}
            for number, (_, scene) in enumerate(self.acts(chapter), 1):
                ids.add(tour.act_id(chapter, number, scene, seen))
        for chapter in self.CHAPTERS:
            tree = ast.parse((ROOT / "demo" / "tour" / "chapters" / f"{chapter}.py").read_text())
            for loop in (n for n in ast.walk(tree) if isinstance(n, ast.For)):
                for call in (n for n in ast.walk(loop) if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute) and n.func.attr == "act"):
                    scene = next((k.value.value for k in call.keywords if k.arg == "scene" and isinstance(k.value, ast.Constant)), None)
                    ids |= {f"{scene}{letter}" for letter in "bcdefghij"} if scene else set()
        self.assertEqual([c["act"] for c in table["change"] if c["act"] not in ids], [])

    def test_the_first_row_of_h2_is_the_send_to_act(self):
        """The row that intended.toml calls h2 carries the Send To act, not the Start click."""
        work = tempfile.mkdtemp(prefix="tour-scene-")
        t = tour.Tour(FakeSession(), work, work + "/w")
        t.begin("5 handover", "05-handover")
        t.run_segment = lambda label="": None
        for text, scene in (("Click Start", "h1"), ("Click Notepad", "h1"), ("Title-bar right click, Send To, Tiling desktop", "h2")):
            t.act("look", text, "x", lead=0.0, tail=0.0, scene=scene)
        self.assertEqual({r["id"]: r["description"] for r in t.rows}["h2"], "Title-bar right click, Send To, Tiling desktop")


class OldBuild(unittest.TestCase):
    """F7 r1: the v9 harness replays on 733967aa, which has no `scene.boxes` and no `state.get` of
    chora-notify (design/tour-v9.md section 6): it degrades, it does not break."""

    class Core:
        def __init__(self, error=None):
            self.error = error
            self.calls = []

        def call(self, method, params=None, timeout=10):
            self.calls.append(method)
            if self.error and method in ("scene.boxes", "scene.query"):
                raise RuntimeError(f"{method}: {self.error}")
            return {"ok": method}

    def test_a_core_without_scene_boxes_answers_none(self):
        core = tour.OldCore(self.Core("no method"))
        self.assertIsNone(core.call("scene.boxes"))
        self.assertEqual(core.call("screen.capture"), {"ok": "screen.capture"})
        self.assertEqual(core.missing, {"scene.boxes"})

    def test_a_real_error_of_scene_boxes_is_not_hidden(self):
        core = tour.OldCore(self.Core("the output is gone"))
        with self.assertRaises(RuntimeError):
            core.call("scene.boxes")

    def test_only_scene_boxes_may_be_missing(self):
        core = tour.OldCore(self.Core("no method"))
        with self.assertRaises(RuntimeError):
            core.call("scene.query", {"selector": "*"})

    def notify(self, error):
        closed = []

        class Notify(tour.Session):
            def call(self, client, method, params=None):
                raise RuntimeError(error)

            def close_on_bus(self, ids):
                closed.append(list(ids))

        return Notify({}, [], ""), closed

    def test_dismiss_notices_without_state_get_closes_on_the_bus(self):
        s, closed = self.notify("methods.call: notify has no method state.get")
        s.dismiss_notices()
        self.assertEqual(len(closed), 1)
        self.assertEqual(closed[0][0], 1)

    def test_the_words_of_choractl_for_a_missing_method_count(self):
        s, closed = self.notify("choractl call notify state.get {}: choractl: notify.state.get is no exported method (code 1)")
        s.dismiss_notices()
        self.assertEqual(len(closed), 1)

    def test_dismiss_notices_does_not_hide_another_error(self):
        s, closed = self.notify("the core closed the connection during methods.call")
        with self.assertRaises(RuntimeError):
            s.dismiss_notices()
        self.assertEqual(closed, [])

    def test_a_build_without_boxes_is_not_red_for_missing_boxes(self):
        class Missing(tour.Checker):
            pass

        before = test_acts.snap(boxes=None)
        helper = test_acts.snap(boxes=None, nodes=[{"id": "hud-chat", "kind": "node", "owner": "assistant", "meta": {"surface": "chat"}, "rects": {"HEADLESS-1": [0, 0, 10, 10]}}])
        got = tour.Checker(lambda: before, baseline=True).judge({"description": "an act"}, None, before, helper)
        self.assertEqual([line for line in got if "missing boxes" in line], [])

    def test_the_candidate_keeps_missing_boxes_red(self):
        """rw-3: without TOUR_BASELINE a build that lost `scene.boxes` is red, never green on nothing."""
        before = test_acts.snap(boxes=None)
        helper = test_acts.snap(boxes=None, nodes=[{"id": "hud-chat", "kind": "node", "owner": "assistant", "meta": {"surface": "chat"}, "rects": {"HEADLESS-1": [0, 0, 10, 10]}}])
        got = tour.Checker(lambda: before).judge({"description": "an act"}, None, before, helper)
        self.assertEqual([line for line in got if "missing boxes" in line], ["I8: missing boxes: no scene.boxes answer for the helper surfaces"])

    def test_the_candidate_prints_every_box_invariant(self):
        import invariants

        planted = {key: (lambda before, after, act, key=key: [f"{key} planted"]) for key in ("I5", "I6", "I8", "I9")}
        with mock.patch.dict(invariants.ALL, planted, clear=True):
            got = tour.Checker(lambda: snap()).judge({"description": "an act"}, None, snap(), snap())
        self.assertEqual(got, [f"{key}: {key} planted" for key in planted])

    def attached(self, baseline):
        """The core that `Checker.attach` leaves in the harness, with TOUR_BASELINE set or not."""
        import os

        import harness

        session = mock.MagicMock()
        session.core = object()
        if baseline:
            os.environ["TOUR_BASELINE"] = "1"
        try:
            with mock.patch.object(harness, "Session", return_value=session):
                tour.Checker.attach({"XDG_RUNTIME_DIR": "/tmp", "CHORA_SOCKET": "/tmp/s"}, ["xp"], "/tmp/evidence")
        finally:
            os.environ.pop("TOUR_BASELINE", None)
        return session.core

    def test_only_the_baseline_core_may_lack_boxes(self):
        self.assertIsInstance(self.attached(True), tour.OldCore)
        self.assertNotIsInstance(self.attached(False), tour.OldCore)

    def test_the_exemption_of_the_list_holds_in_the_tour_too(self):
        """The one design exemption of tests/acts/known-red.toml (the Luna tray, down to 3.0:1) is no red line of a row; known-red rows are not applied."""
        import invariants

        tray = "on O '6:34 PM' (xp-tray-O) has the contrast {}:1 ((255, 255, 255) on (13, 149, 238)), the guide wants 4.5:1"
        pager = "on O '2' (xp-pager-1-O) has the contrast 3.63:1 ((248, 251, 254) on (57, 128, 244)), the guide wants 4.5:1"
        lines = [tray.format("3.21"), tray.format("2.90"), pager]
        shown = {"I6": lambda before, after, act: list(lines)}
        with mock.patch.dict(invariants.ALL, shown, clear=True):
            got = tour.Checker(lambda: snap()).judge({"description": "an act"}, None, snap(), snap())
        self.assertEqual(got, ["I6: " + tray.format("2.90"), "I6: " + pager])

    def test_a_chapter_that_cannot_run_on_the_baseline_is_none_with_its_reason(self):
        import os
        import types

        module = types.ModuleType("chapters.xp")

        def boom(ctx):
            raise RuntimeError("choractl call xp-give: no method window.give")

        module.run = boom
        sys.modules["chapters.xp"] = module
        os.environ["TOUR_BASELINE"] = "1"
        out = io.StringIO()
        try:
            with contextlib.redirect_stdout(out):
                code = run.main(["--check", "xp"])
        finally:
            del os.environ["TOUR_BASELINE"]
            del sys.modules["chapters.xp"]
            os.environ.pop("TOUR_CHECK", None)
        text = out.getvalue()
        self.assertEqual(code, 0, text)
        self.assertIn("baseline: none", text)
        self.assertIn("no method window.give", text)
        self.assertNotIn("BROKE", text)


    def baseline_take(self, error, chapter="handover"):
        """run.py on the baseline (not `--check`) with a chapter that dumps h1.Q and then raises `error`;
        (exit code, output, the baseline-none.json it wrote or None)."""
        import os
        import types

        out_dir = tempfile.mkdtemp(prefix="tour-none-")

        def stops(ctx):
            work = tempfile.mkdtemp(prefix="tour-none-w-")
            t = tour.Tour(Shooter(), ctx.out, work + "/w", checker=tour.Checker(lambda: snap()))
            t.begin("5 handover", "05-handover")
            t.run_segment = lambda label="": None
            t.act("look", "Click Start", "x", lead=0.0, tail=0.0, scene="h1", capture="Q")
            t.done()
            t.flush()
            raise error

        module = types.ModuleType(f"chapters.{chapter}")
        module.run = stops
        sys.modules[f"chapters.{chapter}"] = module
        os.environ.update(TOUR_BASELINE="1", TOUR_OUT=out_dir)
        out = io.StringIO()
        try:
            with contextlib.redirect_stdout(out):
                code = run.main([chapter])
        finally:
            for key in ("TOUR_BASELINE", "TOUR_OUT", "TOUR_CHECK"):
                os.environ.pop(key, None)
            del sys.modules[f"chapters.{chapter}"]
        path = pathlib.Path(out_dir) / "baseline-none.json"
        return code, out.getvalue(), json.loads(path.read_text()) if path.exists() else None

    def take(self, args, out_dir, chapter="handover"):
        """run.py with a chapter that does nothing, on `out_dir`; the exit code."""
        import os
        import types

        module = types.ModuleType(f"chapters.{chapter}")
        module.run = lambda ctx: None
        sys.modules[f"chapters.{chapter}"] = module
        os.environ["TOUR_OUT"] = out_dir
        try:
            with contextlib.redirect_stdout(io.StringIO()), mock.patch.object(assemble, "main"):
                return run.main(args)
        finally:
            for key in ("TOUR_BASELINE", "TOUR_OUT", "TOUR_CHECK"):
                os.environ.pop(key, None)
            del sys.modules[f"chapters.{chapter}"]

    def test_a_take_starts_with_an_empty_captures_folder(self):
        """rv-2: the png and json of an act that was renamed, or whose snapshot failed, of an earlier take
        in the same TOUR_OUT are gone, or tests/tour-diff.py compares a picture of no act."""
        out_dir = tempfile.mkdtemp(prefix="tour-old-")
        old = pathlib.Path(out_dir) / "captures"
        old.mkdir()
        for name in ("h5.Q.png", "h5.Q.json"):
            (old / name).write_bytes(b"old")
        self.assertEqual(self.take(["handover"], out_dir), 0)
        self.assertEqual(sorted(p.name for p in old.glob("*")), [])
        self.assertEqual(out_dir, str(old.parent))

    def test_a_check_leaves_the_folder_of_the_record_alone(self):
        out_dir = tempfile.mkdtemp(prefix="tour-keep-")
        keep = pathlib.Path(out_dir) / "captures"
        keep.mkdir()
        (keep / "h5.Q.png").write_bytes(b"old")
        self.assertEqual(self.take(["--check", "handover"], out_dir), 0)
        self.assertTrue((keep / "h5.Q.png").exists())

    def test_another_error_on_the_baseline_is_broke(self):
        """rw-4: a TypeError of the harness, a dead core: the take fails loud, nothing is `baseline: none`."""
        for error in (TypeError("act() got an unexpected keyword argument"), RuntimeError("the core closed the connection")):
            code, text, none = self.baseline_take(error)
            self.assertEqual(code, 1, text)
            self.assertIn("BROKE", text)
            self.assertNotIn("baseline: none from", text)
            self.assertIsNone(none)

    def test_baseline_none_json_names_the_captures_that_did_not_run(self):
        code, text, none = self.baseline_take(RuntimeError("choractl call xp-give: no method window.give"))
        self.assertEqual(code, 0, text)
        (entry,) = none
        self.assertEqual(entry["chapter"], "handover")
        self.assertNotIn("h1.Q", entry["names"])
        self.assertIn("h2.Q", entry["names"])
        self.assertIn("h7.Q", entry["names"])
        self.assertTrue(all(name.partition(".")[2] for name in entry["names"]), entry["names"])

    def test_the_names_of_the_plan_follow_the_loops_of_a_chapter(self):
        planned = tour.planned("tiling")
        self.assertEqual(len(planned), len(set(planned)))
        self.assertEqual([n for n in planned if n.startswith("t3")], [f"t3{letter}.Q" for letter in ("", *"bcdefgh")])


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
        one = snap("xp")
        self.assertTrue(Expect(verify=windows_are(4, look="xp")).check(one, one), "one window passes for four")
        self.assertEqual(Expect(verify=windows_are(1, look="xp")).check(one, one), [])

    def test_handover_expect_turns_red_on_a_wrong_owner(self):
        """h3: the window is given to mc; the node still says xp owns it."""
        owner_is = check_of("handover", "owner_is")
        want = Expect(verify=owner_is("s1", "mc"))
        sound = test_acts.snap()
        self.assertTrue(want.check(sound, sound), "owner xp passes for mc")
        mine = test_acts.snap(nodes=[test_acts.surface("s1", "mc", focused=True)])
        self.assertEqual(want.check(sound, mine), [])

    def test_xp_letter_and_run_box_turn_red_on_a_wrong_state(self):
        """x3: the letter holds other text; x9: the Run box never said Nothing found."""
        letter_is, box_says = check_of("xp", "letter_is"), check_of("xp", "box_says")
        d = pathlib.Path(tempfile.mkdtemp(prefix="tour-test-"))
        (d / "letter.txt").write_text("Dear Anna,\n")
        self.assertTrue(letter_is("Dear Anna,\nLove, Erika", d / "letter.txt")(snap()))
        (d / "letter.txt").write_text("Dear Anna,\nLove, Erika\n")
        self.assertEqual(letter_is("Dear Anna,\nLove, Erika", d / "letter.txt")(snap()), [])
        (d / "fuzzel.rows.1").write_text("Zebra\nalpha\n")
        check = box_says(str(d), "nosuch")
        (d / "fuzzel.rows.2").write_text('Nothing found for "nosuch"\nZebra\n')
        self.assertEqual(check(snap()), [])
        self.assertTrue(box_says(str(d), "nosuch")(snap()), "a box of an act before is no answer of this act")

    def test_logged_reads_only_what_the_act_added(self):
        """rca-4: a line of an earlier act does not pass a later one."""
        logged = check_of("xp", "logged")
        log = pathlib.Path(tempfile.mkdtemp(prefix="tour-test-")) / "claims.chat"
        log.write_text("How do I make the sound louder?\n")
        check = logged(str(log), "louder")
        self.assertTrue(check(snap()), "the line of the act before passes")
        log.write_text("How do I make the sound louder?\nlouder again\n")
        self.assertEqual(check(snap()), [])

    def test_stand_in_wpctl_steps_by_the_percent(self):
        """rca-3: `set-volume -l 1.0 SINK 5%+` adds five points; `75%` sets 0.75."""
        import subprocess

        d = tempfile.mkdtemp(prefix="tour-test-")
        launch.wpctl_stub(d)
        wpctl = lambda *a: subprocess.run([f"{d}/wpctl", *a], capture_output=True, text=True).stdout
        wpctl("set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", "5%+")
        self.assertEqual(open(f"{d}/wpctl.state").read().split()[0], "0.45")
        wpctl("set-volume", "@DEFAULT_AUDIO_SINK@", "10%-")
        self.assertEqual(open(f"{d}/wpctl.state").read().split()[0], "0.35")
        wpctl("set-volume", "@DEFAULT_AUDIO_SINK@", "75%")
        self.assertEqual(open(f"{d}/wpctl.state").read().split()[0], "0.75")

    def test_tiling_tiles_turn_red_when_they_overlap_or_the_gaps_differ(self):
        tiles_hold = check_of("tiling", "tiles_hold")

        def tiled(*rects):
            wins = [{"surface": f"s{i}", "shown": True, "rect": r, "floating": False} for i, r in enumerate(rects)]
            return test_acts.snap(states={"tiling": {"windows": wins, "focus": {}, "open": [], "busy": []}})

        good = tiled([14, 54, 619, 652], [647, 54, 619, 652])
        self.assertEqual(tiles_hold(2)(good), [])
        self.assertTrue(tiles_hold(2)(tiled([14, 54, 700, 652], [647, 54, 619, 652])), "overlap")
        self.assertTrue(tiles_hold(3)(tiled([14, 54, 600, 200], [647, 54, 619, 200], [14, 300, 619, 200])), "gaps 47 and 46 differ")

    def test_handover_lock_and_windows_turn_red(self):
        lock_called, windows_only = check_of("handover", "lock_called"), check_of("handover", "windows_only")
        calls = pathlib.Path(tempfile.mkdtemp(prefix="tour-test-")) / "calls.log"
        self.assertTrue(lock_called(str(calls))(snap()), "no call yet")
        calls.write_text("swaylock -f\n")
        self.assertEqual(lock_called(str(calls))(snap()), [])
        self.assertTrue(windows_only("s1", "s2")(test_acts.snap()), "a window is lost")
        self.assertEqual(windows_only("s1")(test_acts.snap()), [])

    def test_notice_check_reads_the_text_of_the_notice(self):
        """rca-1: act 3 names the missing editor; a stale notice of act 2 or another text is red."""
        notice_says, notice_is_new = check_of("xp", "notice_says"), check_of("xp", "notice_is_new")
        block = lambda text: {"notify": {"shown": [{"id": 2, "summary": "Notepad", "body": text}], "waiting": [], "closed": [], "accepted": []}}
        stale = test_acts.snap(blocks=block("Saved"))
        self.assertTrue(notice_says("mousepad")(stale), "a notice without the editor passes")
        right = test_acts.snap(blocks=block("Install mousepad"))
        self.assertEqual(notice_says("mousepad")(right), [])
        self.assertTrue(notice_is_new(right, right), "the notice of the read before is no new notice")
        self.assertEqual(notice_is_new(snap(), right), [])

    def test_shared_checks_live_in_checks_py(self):
        """rca-5: tiling and handover import from chapters/checks.py, not from the xp chapter."""
        for chapter in ("tiling", "handover"):
            src = (ROOT / "demo" / "tour" / "chapters" / f"{chapter}.py").read_text()
            self.assertNotIn("from chapters.xp", src, chapter)


CHAPTERS_B = ("mc", "construct")


class ChaptersB(unittest.TestCase):
    """F7 (design/tour-v9.md section 8, lane 5): every act row of mc and construct says what it
    expects, so `run.py --check` counts no `inv` row for these chapters."""

    def test_every_row_has_an_expect(self):
        for chapter in CHAPTERS_B:
            rows = act_calls(chapter)
            self.assertTrue(rows, chapter)
            bare = [line for line, keys, _ in rows if "expect" not in keys and "todo" not in keys]
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

    def test_mc_starts_chora_notify_as_the_mc_session_does(self):
        """tests/gate-mc-sofa.sh: `chora-notify --assets plugins/chora-mc/notify` (card text 19 px, bottom left); the plain default drew the card at 11 px and the tour read it as a breach of the mc."""
        text = (ROOT / "demo" / "tour" / "chapters" / "mc.py").read_text()
        self.assertIn('f"{BIN}/chora-notify", "--assets", f"{REPO}/plugins/chora-mc/notify"', text)
        self.assertIn('notify_config(position="bottom_left")', text)

class TilingArgv(unittest.TestCase):
    """RV9-2/3: chora-tiling starts its own bar and wallpaper (ccf7ebab) unless it gets `--no-companions`,
    and the tiling chapter starts waybar itself: two bars on one output, tiles 40 px lower and a bar
    without its modules. A build that does not know the flag (733967aa) must not get it."""

    def binary(self, tmp, knows):
        path = pathlib.Path(tmp, "chora-tiling")
        path.write_text("#!/bin/sh\n" + ('echo "      --no-companions    Starts no panel"\n' if knows else 'echo "      --output <NAME>"\n'))
        path.chmod(0o755)
        return str(path)

    def test_a_build_with_companions_gets_no_companions(self):
        with tempfile.TemporaryDirectory() as tmp:
            binary = self.binary(tmp, True)
            self.assertEqual(launch.tiling_argv(binary, "--output", "A"), [binary, "--no-companions", "--output", "A"])

    def test_a_build_without_the_flag_gets_none(self):
        with tempfile.TemporaryDirectory() as tmp:
            binary = self.binary(tmp, False)
            self.assertEqual(launch.tiling_argv(binary), [binary])

    def test_the_chapters_start_tiling_by_tiling_argv(self):
        for chapter in ("tiling", "handover"):
            text = (ROOT / "demo" / "tour" / "chapters" / f"{chapter}.py").read_text()
            self.assertIn("tiling_argv(", text, chapter)
            self.assertNotIn('[f"{BIN}/chora-tiling"', text, chapter)


class HandoverCompanions(unittest.TestCase):
    """rv9j-4: the handover chapter starts chora-tiling with `--no-companions` (tiling_argv), so the act
    "tiling desktop empty" needs the bar and the wallpaper from the chapter, as tiling.py starts them
    (`companions=` of the tiling chapter); without them the tiling screen of 05-01..05-03b is black."""

    def commands(self):
        module = importlib.import_module("chapters.handover")
        fn = getattr(module, "tiling_companions", None)
        self.assertIsNotNone(fn, "chapters/handover.py builds no companion command for the tiling output")
        return fn("HEADLESS-3")

    def test_a_bar_and_a_wallpaper_start_as_the_tiling_chapter_starts_them(self):
        commands = self.commands()
        tiling = (ROOT / "demo" / "tour" / "chapters" / "tiling.py").read_text()
        wallpaper = [c for c in commands if c[0] == "swaybg"]
        self.assertEqual(len(wallpaper), 1, commands)
        self.assertIn("#2e3440", wallpaper[0])
        self.assertIn('["swaybg", "--color", "#2e3440"]', tiling)
        bar = [c for c in commands if any(a.endswith("flavors/tiling/waybar/waybar.sh") for a in c)]
        self.assertEqual(len(bar), 1, commands)
        self.assertEqual(bar[0][:3], ["dbus-run-session", "--", "sh"], bar)
        self.assertTrue(bar[0][-1].endswith("flavors/tiling/waybar"), bar)

    def test_the_wallpaper_covers_the_tiling_output_only(self):
        """The other three outputs draw their own desktops (xp, mc, construct): a swaybg on every output paints under them."""
        [wallpaper] = [c for c in self.commands() if c[0] == "swaybg"]
        self.assertEqual(wallpaper[wallpaper.index("-o") + 1], "HEADLESS-3", wallpaper)

    def test_the_chapter_starts_them_after_tiling(self):
        text = (ROOT / "demo" / "tour" / "chapters" / "handover.py").read_text()
        start = text.index("def start_multi")
        body = text[start : text.index("\ndef run(", start)]
        self.assertLess(body.index('tiling_argv(f"{BIN}/chora-tiling"'), body.index("tiling_companions("), body)


class HandoverCompanionsRun(unittest.TestCase):
    """rtb5-3: `choractl run setsid` for the bar and the wallpaper ran with `check=False, capture_output=True` and the
    tests read only the command lines, so a missing swaybg or a failed waybar left the tiling screen black and the act
    green. The chapter waits for the two processes and raises on a miss, as it does for `owner:*`; the first act, which
    shows that screen, checks that they run."""

    def module(self):
        return importlib.import_module("chapters.handover")

    def test_a_companion_that_does_not_run_is_named(self):
        missing = self.module().companions_missing("HEADLESS-3", alive=lambda pattern: False)
        self.assertEqual(len(missing), 2, missing)
        self.assertTrue(any("swaybg" in m for m in missing), missing)
        self.assertTrue(any("waybar" in m for m in missing), missing)

    def test_companions_that_run_are_not_named(self):
        self.assertEqual(self.module().companions_missing("HEADLESS-3", alive=lambda pattern: True), [])

    def test_one_missing_companion_is_the_one_named(self):
        missing = self.module().companions_missing("HEADLESS-3", alive=lambda pattern: "swaybg" in pattern)
        self.assertEqual(len(missing), 1, missing)
        self.assertIn("waybar", missing[0])

    def test_the_wallpaper_is_looked_for_on_the_tiling_output(self):
        seen = []
        self.module().companions_missing("HEADLESS-3", alive=lambda pattern: seen.append(pattern) or True)
        self.assertTrue(any("swaybg" in p and "HEADLESS-3" in p for p in seen), seen)

    def test_start_multi_raises_on_a_miss_after_starting_them(self):
        text = (ROOT / "demo" / "tour" / "chapters" / "handover.py").read_text()
        start = text.index("def start_multi")
        body = text[start : text.index("\ndef run(", start)]
        self.assertIn("companions_missing(", body)
        self.assertLess(body.index("tiling_companions("), body.index("companions_missing("), body)
        self.assertIn("raise RuntimeError", body[body.index("companions_missing(") :], body)

    def test_the_act_that_shows_the_empty_tiling_screen_checks_that_they_run(self):
        module = self.module()
        self.assertEqual(module.companions_running("HEADLESS-3", alive=lambda pattern: True)(None), [])
        failed = module.companions_running("HEADLESS-3", alive=lambda pattern: False)(None)
        self.assertEqual(len(failed), 2, failed)
        text = (ROOT / "demo" / "tour" / "chapters" / "handover.py").read_text()
        act = next(line for line in text.splitlines() if 't.act("look", "Two screens' in line)
        self.assertIn("companions_running(", act)


class CodeWindow(unittest.TestCase):
    """RV9-3: the code window of the tiling chapter showed crates/chora-kit/src/wobbly.rs of the
    checkout, and that file differs between the baseline and the candidate: a pixel diff of the
    window, not of the product. It shows a file of the tour."""

    def test_the_code_window_shows_a_file_of_the_tour(self):
        text = (ROOT / "demo" / "tour" / "launch.py").read_text()
        self.assertNotIn("crates/", text.split("args=")[1].split("\n")[0])
        self.assertIn("args={sample}", text)
        self.assertIn("SAMPLE)", (ROOT / "demo" / "tour" / "chapters" / "tiling.py").read_text())
        self.assertTrue((ROOT / "demo" / "tour" / "sample.rs").exists())


class StubTerminal(unittest.TestCase):
    """RV9-r4: the terminals of the tiling chapter showed other text in the two builds: the shell
    prompt came from the environment, and a content window (code, notes, page) read its size at the
    start, while the tile was still animating to its place, so the code window of the baseline was
    scrolled to line 110 and the candidate's to 118. One stub: a fixed prompt and directory, and a
    content window that waits for its tile before it prints."""

    def script(self):
        return launch.foot_stub("/tmp/x", "/tmp/x/history", "/tmp/x/sample.rs")

    def test_the_shell_has_a_fixed_prompt_and_directory(self):
        text = self.script()
        self.assertIn("PS1='$ '", text)
        self.assertIn("cd /home/user/notes", text)

    def test_a_content_window_waits_for_its_tile(self):
        text = self.script()
        self.assertRegex(text, r"sleep \d+(\.\d+)?; python3 -I")

    def test_the_tiling_chapter_uses_the_stub(self):
        text = (ROOT / "demo" / "tour" / "chapters" / "tiling.py").read_text()
        self.assertIn("foot_stub(", text)


class FixedClock(unittest.TestCase):
    """RV9-r4: mc reads the time by running `date` (plugins/chora-mc/src/region.rs `date`: the clocks
    and the `since` of the agent), so a `date` first on PATH pins it: one time for both takes."""

    def run_date(self, tmp, *args):
        launch.date_stub(tmp)
        out = subprocess.run([f"{tmp}/date", *args], capture_output=True, text=True, env={"PATH": "/usr/bin:/bin"})
        return out.stdout.strip()

    def test_the_stub_date_gives_one_time_to_every_call(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.assertEqual(self.run_date(tmp, "+%H:%M"), "18:51")
            self.assertEqual(self.run_date(tmp, "+%H:%M"), "18:51")

    def test_the_stub_date_takes_the_formats_of_the_clocks(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.assertEqual(self.run_date(tmp, "+%a %d %b %H:%M"), "Mon 23 Nov 18:51")

    def test_every_session_gets_the_stub(self):
        text = (ROOT / "demo" / "tour" / "launch.py").read_text()
        self.assertEqual(text.count("date_stub(f"), 2)


class LauncherEnv(unittest.TestCase):
    """TOUR-LAUNCH: the launchers of the xp start menu (xp-item-N) come from PATH and XDG_DATA_DIRS;
    both runs of a comparison must get one fixed environment, not the one of the shell that ran them."""

    def env_of(self, path, data_dirs):
        old = dict(launch.os.environ)
        try:
            launch.os.environ["PATH"] = path
            launch.os.environ["XDG_DATA_DIRS"] = data_dirs
            launch.os.environ["XDG_CONFIG_DIRS"] = data_dirs
            with mock.patch.object(launch.Launcher, "path_without_writer", lambda self, p: "FARM"):
                with tempfile.TemporaryDirectory() as scratch, mock.patch.object(launch, "SCRATCH", scratch):
                    return launch.Launcher("t").base_env()
        finally:
            launch.os.environ.clear()
            launch.os.environ.update(old)

    def test_the_launcher_environment_does_not_follow_the_shell(self):
        a = self.env_of("/usr/bin:/opt/a/bin", "/opt/a/share")
        b = self.env_of("/usr/local/bin:/opt/b/bin", "/opt/b/share")
        for key in ("PATH", "XDG_DATA_DIRS", "XDG_CONFIG_DIRS"):
            self.assertIn(key, a)
            self.assertEqual(a[key].replace(a["XDG_DATA_HOME"][: -len("/data")], "D"), b[key].replace(b["XDG_DATA_HOME"][: -len("/data")], "D"), key)
        self.assertNotIn("/opt/", a["PATH"] + a["XDG_DATA_DIRS"])


class LauncherSupported(unittest.TestCase):
    """TOUR-LAUNCH r1 (rml1-1): the tour judges a supported setup: the fuzzel of its sessions is
    >= 1.11 (the package minimum of tests/gate-package-deps.sh) or the tour stops with one line, and
    every look has chora-notify serving."""

    @staticmethod
    def version(binary):
        import re

        out = subprocess.run([binary, "--version"], capture_output=True, text=True).stdout
        found = re.match(r"^[^0-9]*([0-9][0-9.]*)", out)
        return tuple(int(x) for x in found.group(1).split(".")) if found else ()

    def test_the_fuzzel_of_a_session_is_supported_or_the_tour_stops(self):
        with tempfile.TemporaryDirectory() as scratch, mock.patch.object(launch, "SCRATCH", scratch):
            try:
                env = launch.Launcher("t").base_env()
            except RuntimeError as err:
                self.assertEqual(len(str(err).splitlines()), 1, "one clear line")
                self.assertIn("fuzzel", str(err))
                return
            found = launch.shutil.which("fuzzel", path=env["PATH"])
            self.assertIsNotNone(found, "no fuzzel on the PATH of the session")
            self.assertGreaterEqual(self.version(found), (1, 11), found)

    def test_every_look_has_chora_notify_serving(self):
        calls = []

        def fake_run(argv, *a, **kw):
            calls.append(list(argv))
            return subprocess.CompletedProcess(argv, 1, "", "")

        with tempfile.TemporaryDirectory() as scratch, mock.patch.object(launch, "SCRATCH", scratch):
            L = launch.Launcher("t")
            out = f"{L.dir}/headless.out"

            def fake_spawn(argv, env, log, **kw):
                if log == "headless.out":
                    open(out, "w").write("CHORA_SOCKET=/s\nXDG_RUNTIME_DIR=/r\nWAYLAND_DISPLAY=w\n")
                return mock.Mock(pid=1)

            with mock.patch.object(launch.subprocess, "run", fake_run), mock.patch.object(L, "spawn", fake_spawn), \
                 mock.patch.object(launch.time, "sleep", lambda s: None):
                L.start(None, voice=False)
        started = [c for c in calls if any(str(x).endswith("chora-notify") for x in c) and "call" not in c]
        self.assertTrue(started, f"no chora-notify started: {calls}")


if __name__ == "__main__":
    unittest.main()
