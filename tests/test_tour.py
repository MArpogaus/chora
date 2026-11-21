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
import json  # noqa: E402
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

    def test_row_id_is_the_letter_of_the_look_and_the_number(self):
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


if __name__ == "__main__":
    unittest.main()
