#!/usr/bin/python3
"""Selftest of the act harness, part 2: the known-red list (`KnownRed`) and the
size rule of the guide (`GuideSize`, I6). They share the sound snapshot and the
helpers of tests/test_acts.py.

Usage: python3 -I tests/test_acts_guide.py [-v]
"""

import pathlib
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "acts"))

import harness  # noqa: E402
import invariants  # noqa: E402
from test_acts import HELD, breaches, panel, run, shot, snap, surf, text_node, with_texts  # noqa: E402,F401


class KnownRed(unittest.TestCase):
    rows = [{"look": "xp", "id": "I1", "finding": "net2-1", "slice": "NET2 adapters"}]

    def test_listed_failure_does_not_block(self):
        verdict = harness.judge({("xp", "I1"): ["no state"]}, self.rows)
        self.assertEqual(verdict.blocking, [])
        self.assertEqual(verdict.red, [("xp", "I1")])

    def test_listed_pass_blocks(self):
        verdict = harness.judge({("xp", "I1"): []}, self.rows)
        self.assertTrue(verdict.blocking)

    def test_unlisted_failure_blocks(self):
        verdict = harness.judge({("xp", "I3"): ["focus"], ("xp", "I1"): ["no state"]}, self.rows)
        self.assertTrue(verdict.blocking)

    def test_a_row_needs_finding_and_slice(self):
        with self.assertRaises(ValueError):
            harness.judge({}, [{"look": "xp", "id": "I1"}])

    def test_match_limits_a_row_to_its_breach(self):
        rows = [dict(self.rows[0], match="missing key open")]
        listed = harness.judge({("xp", "I1"): ["missing key open"]}, rows)
        self.assertEqual(listed.blocking, [])
        other = harness.judge({("xp", "I1"): ["the toplevel s1 is detached"]}, rows)
        self.assertTrue(other.blocking)

    def test_a_second_overlap_of_a_listed_row_blocks(self):
        row = {
            "look": "syn",
            "id": "I5",
            "finding": "syn-1",
            "slice": "synthetic",
            "match": "(syn-left)",
        }
        named = "in act 'a': on HEADLESS-1 'One' (syn-left) overlaps 'Two' (syn-label) at []"
        other = "in act 'a': on HEADLESS-1 'Three' (syn-right) overlaps 'Four' (syn-right) at []"
        self.assertEqual(harness.judge({("syn", "I5"): [named]}, [row]).blocking, [])
        self.assertTrue(harness.judge({("syn", "I5"): [named, other]}, [row]).blocking)
        self.assertTrue(harness.judge({("syn", "I5"): [other]}, [row]).blocking)

    def test_one_row_per_fault_of_a_cell(self):
        rows = [
            dict(self.rows[0], act="a", capture="l1"),
            dict(self.rows[0], act="b", capture="l2"),
        ]
        one = "in act 'a': on O 'One' (l1) is small"
        two = "in act 'b': on O 'Two' (l2) is small"
        three = "in act 'b': on O 'Three' (l3) is small"
        self.assertEqual(harness.judge({("xp", "I1"): [one, two]}, rows).blocking, [])
        self.assertTrue(harness.judge({("xp", "I1"): [one, three]}, rows).blocking)
        self.assertTrue(harness.judge({("xp", "I1"): [one]}, rows).blocking, "row two is stale")

    def test_a_flaky_row_may_stay_unseen(self):
        rows = [dict(self.rows[0], capture="l1"), dict(self.rows[0], act="a", capture="l2", flaky=True, evidence="run 1 with, run 2 without, in act 'a'")]
        one = "in act 'a': on O 'One' (l1) is small"
        self.assertEqual(harness.judge({("xp", "I1"): [one]}, rows).blocking, [])

    def test_the_scene_is_read_again_while_a_text_changes(self):
        s = object.__new__(harness.Session)
        clock = iter(["12:00", "12:01", "12:01", "12:01"])
        reads = []

        def call(method, _params=None):
            if method == "scene.query":
                reads.append(1)
                return [{"id": "c", "text": next(clock)}]
            return {}

        s.core = type("C", (), {"call": staticmethod(call)})()
        s.label, s.looks = "xp", ["xp"]
        s.shots = lambda outputs: {}
        s.settled_scene([], sleep=lambda _t: None)
        self.assertEqual(len(reads), 4)

    def test_the_scene_is_read_again_while_a_box_moves(self):
        """The capture and the boxes are one moment: a sheet that still slides in (mc, Super+F1) read
        its boxes at the end of the slide on the capture of its start, and the text measured 1.00:1."""
        s = object.__new__(harness.Session)
        moving = iter([{"O": {"boxes": [{"node": "n", "rect": [0, 28, 5, 5]}]}}] + [{"O": {"boxes": [{"node": "n", "rect": [0, 56, 5, 5]}]}}] * 6)
        shots = []

        def call(method, _params=None):
            if method == "scene.boxes":
                return next(moving)
            return [{"id": "n", "text": "x"}] if method == "scene.query" else {}

        s.core = type("C", (), {"call": staticmethod(call)})()
        s.label, s.looks = "xp", ["xp"]
        s.shots = lambda outputs: shots.append(1) or {}
        _, boxes, _ = s.settled_scene([], sleep=lambda _t: None)
        self.assertEqual(len(shots), 2)
        self.assertEqual(boxes["O"]["boxes"][0]["rect"], [0, 56, 5, 5])

    def test_a_scene_that_never_settles_says_so(self):
        """rw-7: after the last try the last read is returned with a line in the log, never silently."""
        import contextlib
        import io

        s = object.__new__(harness.Session)
        step = iter(range(100))

        def call(method, _params=None):
            if method == "scene.boxes":
                return {"O": {"boxes": [{"node": "n", "rect": [0, next(step), 5, 5]}]}}
            return [{"id": "n", "text": "x"}] if method == "scene.query" else {}

        s.core = type("C", (), {"call": staticmethod(call)})()
        s.label, s.looks = "xp", ["xp"]
        s.shots = lambda outputs: {}
        err = io.StringIO()
        with contextlib.redirect_stderr(err):
            nodes, boxes, _ = s.settled_scene([], tries=3, sleep=lambda _t: None)
        self.assertIn("did not settle", err.getvalue())
        self.assertIn("3 reads", err.getvalue())
        self.assertEqual(nodes[0]["id"], "n")
        self.assertIsNotNone(boxes)

    def test_a_card_that_is_not_yet_on_the_capture_is_captured_again(self):
        """xp-38: the box of the notify card is in the scene while the first capture still shows the
        wallpaper there (text 1.04:1, green on green); the capture is repeated until it holds still."""
        s = object.__new__(harness.Session)
        frames = iter([{"O": (1, [b"wallpaper"])}] + [{"O": (1, [b"card"])}] * 8)
        taken = []

        def call(method, _params=None):
            if method == "scene.boxes":
                return {"O": {"boxes": [{"node": "notify-card-body", "rect": [0, 0, 5, 5]}]}}
            return [{"id": "notify-card-body", "text": "x"}] if method == "scene.query" else {}

        s.core = type("C", (), {"call": staticmethod(call)})()
        s.label, s.looks = "xp", ["xp"]
        s.shots = lambda outputs: taken.append(1) or next(frames)
        _, _, shots = s.settled_scene([], sleep=lambda _t: None)
        self.assertEqual(shots["O"][1], [b"card"])

    def test_a_card_that_lands_while_the_snapshot_reads_the_clients_is_read_again(self):
        """xp-38 r2: the nodes and the pixels of a snapshot are of the settled read, its boxes are
        read last; a notify card that lands between them has a box on the wallpaper capture and no
        node (11 px, 1.04:1). The scene is read again until its boxes are those of the capture."""
        s = object.__new__(harness.Session)
        card = [False]

        def call(method, _params=None):
            if method == "scene.boxes":
                boxes = [{"node": "notify-card-body", "rect": [0, 0, 5, 5]}] if card[0] else []
                return {"O": {"boxes": boxes}}
            if method == "scene.query":
                return [{"id": "notify-card-body", "text": "x"}] if card[0] else []
            return {}

        def clients():
            card[0] = True
            return {}

        s.core = type("C", (), {"call": staticmethod(call)})()
        s.label, s.looks, s.wallpaper = "xp", ["xp"], set()
        s.outputs = lambda: []
        s.shots = lambda outputs: {"O": (1, [b"card" if card[0] else b"wallpaper"])}
        s.clients = clients
        s.blocks = lambda clients: {}
        s.cube = lambda: {}
        s.new_log_text = lambda: {}
        s.exits = lambda: {}
        s.states = lambda: {}
        shot = s.snapshot(states={})
        self.assertEqual([n["id"] for n in shot.nodes], ["notify-card-body"])
        self.assertEqual(shot.shots["O"][1], [b"card"])
        self.assertEqual(shot.boxes["O"]["boxes"][0]["node"], "notify-card-body")

    def test_a_scene_that_settles_logs_nothing(self):
        import contextlib
        import io

        s = object.__new__(harness.Session)
        s.core = type("C", (), {"call": staticmethod(lambda method, _p=None: [{"id": "n", "text": "x"}] if method == "scene.query" else {})})()
        s.label, s.looks = "xp", ["xp"]
        s.shots = lambda outputs: {}
        err = io.StringIO()
        with contextlib.redirect_stderr(err):
            s.settled_scene([], sleep=lambda _t: None)
        self.assertEqual(err.getvalue(), "")

    def test_a_snapshot_reads_the_boxes_after_the_clients(self):
        s = object.__new__(harness.Session)
        order = []
        s.core = type("C", (), {"call": staticmethod(lambda m, _p=None: order.append(m) or {})})()
        s.label, s.looks, s.wallpaper = "construct", ["construct"], set()
        s.outputs = lambda: []
        s.settled_scene = lambda outputs: order.append("settled") or ([], {}, None)
        s.clients = lambda: order.append("clients") or {}
        s.blocks = lambda clients: order.append("blocks") or {}
        s.cube = lambda: order.append("cube") or {}
        s.new_log_text = lambda: {}
        s.snapshot(states={})
        self.assertEqual(order[-1], "scene.boxes")
        self.assertGreater(order.index("scene.boxes"), order.index("blocks"))

    def test_a_construct_session_takes_no_capture_once_construct_is_gone(self):
        s = object.__new__(harness.Session)
        s.label, s.looks = "construct", ["xp"]
        s.core = type("C", (), {"call": staticmethod(lambda m, _p=None: [] if m == "scene.query" else {})})()
        s.shots = lambda outputs: self.fail("a capture of the flat xp reopens no cube again")
        self.assertIsNone(s.settled_scene([{"name": "O"}], sleep=lambda _t: None)[2])
        s.looks = ["construct", "xp"]
        s.shots = lambda outputs: {"O": (1, [])}
        self.assertEqual(s.settled_scene([{"name": "O"}], sleep=lambda _t: None)[2], {"O": (1, [])})

    def test_a_snapshot_without_captures_measures_no_contrast(self):
        after = snap(nodes=snap().nodes + [text_node("l1", "Hello", (0, 0, 70, 14))])
        after.shots = None
        self.assertEqual([b for b in invariants.guide_breaches(after) if "capture" in b or "contrast" in b], [])

    def test_a_cell_keeps_the_breaches_of_every_act(self):
        s = object.__new__(harness.Session)
        s.results = {}
        s.runs = pathlib.Path(tempfile.mkdtemp())
        s.outputs = lambda: [{"name": "O"}]
        s.capture = lambda *a: ({}, b"png")
        s.record("xp", "I6", ["one"], "a")
        s.record("xp", "I6", ["two"], "b")
        self.assertEqual(len(s.results[("xp", "I6")]), 2)

    def test_the_rows_of_the_guide_checks_name_act_and_capture(self):
        for row in harness.known_red():
            if row["finding"].startswith("guide-checks"):
                with self.subTest(finding=row["finding"]):
                    self.assertTrue(
                        row.get("flaky")
                        or row.get("capture", "").endswith("*") and row.get("match")
                        or row.get("act") and (row.get("capture") or row.get("match")),
                        row,
                    )

    def test_a_new_acts_tray_breach_is_not_covered_by_a_flaky_row(self):
        row = dict(
            self.rows[0],
            act="a",
            capture="xp-tray-O",
            match="the contrast",
            flaky=True,
            evidence="run 1 with, run 2 without, in act 'a'",
        )
        seen = "in act 'new': on O '12:00' (xp-tray-O) has the contrast 1.00:1 (a on a), the guide wants 4.5:1"
        self.assertTrue(harness.judge({("xp", "I1"): [seen]}, [row]).blocking)

    TRAY = {"look": "xp", "node": "xp-tray-", "floor": 3.0, "reason": "authentic Luna tray, vision card"}

    @staticmethod
    def contrast(node, ratio, act="a"):
        return f"in act {act!r}: on O '12:00' ({node}) has the contrast {ratio:.2f}:1 (#fff on #3a6ea5), the guide wants 4.5:1"

    def test_a_tray_text_breach_passes_by_one_exemption_in_any_act(self):
        for look, act in (("xp", "a"), ("construct", "b")):
            seen = self.contrast("xp-tray-VIRTUAL-1", 3.27, act)
            verdict = harness.judge({(look, "I6"): [seen]}, [], exempt=[self.TRAY])
            self.assertEqual(verdict.blocking, [])
            self.assertIn((look, "I6"), verdict.passed)

    def test_the_same_breach_on_another_xp_node_still_fails(self):
        seen = self.contrast("xp-clock-O", 3.27)
        self.assertTrue(harness.judge({("xp", "I6"): [seen]}, [], exempt=[self.TRAY]).blocking)

    def test_a_tray_breach_that_is_not_the_luna_contrast_still_fails(self):
        self.assertTrue(harness.judge({("xp", "I6"): [self.contrast("xp-tray-O", 1.0)]}, [], exempt=[self.TRAY]).blocking)
        small = "in act 'a': on O '12:00' (xp-tray-O) is 12 px, the guide wants 14 px"
        self.assertTrue(harness.judge({("xp", "I6"): [small]}, [], exempt=[self.TRAY]).blocking)

    def test_an_exemption_row_needs_look_node_and_reason(self):
        for key in ("look", "node", "reason"):
            with self.assertRaises(ValueError):
                harness.judge({}, [], exempt=[{k: v for k, v in self.TRAY.items() if k != key}])

    def test_the_list_exempts_the_tray_once_and_has_no_flaky_tray_row(self):
        self.assertEqual([e["node"] for e in harness.exemptions()], ["xp-tray-"])
        self.assertFalse([r for r in harness.known_red() if r.get("flaky") and "xp-tray" in r.get("capture", "")])

    def test_a_flaky_row_needs_evidence_and_an_act(self):
        row = dict(self.rows[0], capture="l1", flaky=True)
        with self.assertRaises(ValueError):
            harness.judge({("xp", "I1"): []}, [row])
        with self.assertRaises(ValueError):
            harness.judge({("xp", "I1"): []}, [dict(row, evidence="two runs")])

    def test_the_list_has_no_flaky_row_without_evidence(self):
        for row in harness.known_red():
            if row.get("flaky"):
                with self.subTest(finding=row["finding"]):
                    self.assertTrue(row.get("evidence") and row.get("act"), row)

    def test_the_list_parses(self):
        rows = harness.known_red()
        self.assertIsInstance(rows, list)


class GuideSize(unittest.TestCase):
    """I6: text a person must read is 14 px or more."""

    def sized(self, px, label="mc-line", owner="mc"):
        node = text_node(label, "Next step", (100, 100, 80, 20), owner=owner) | {"font_size": px}
        return snap(nodes=snap().nodes + [node], shots=shot(at=(100, 104, 70, 10)))

    def test_sixteen_passes(self):
        self.assertEqual(breaches("I6", snap(), self.sized(16)), [])

    def test_ten_fails(self):
        self.assertTrue(any("14 px" in b for b in breaches("I6", snap(), self.sized(10))))

    def test_xp_text_is_eleven_px_the_replica(self):
        """design/product-briefs.md: 8 pt Tahoma at 96 DPI is 11 px; it replaces the 14 px floor for xp."""
        self.assertEqual([b for b in breaches("I6", snap(), self.sized(11, "xp-icon-0", "xp")) if " px" in b], [])

    def test_xp_text_under_eleven_px_fails(self):
        self.assertTrue(any("is 10 px, the guide wants 11 px" in b for b in breaches("I6", snap(), self.sized(10, "xp-icon-0", "xp"))))

    def test_a_card_of_the_notice_in_xp_is_xp_size(self):
        self.assertEqual([b for b in breaches("I6", snap(), self.sized(11, "notify-card-body", "notify")) if " px" in b], [])

    def test_a_card_in_a_construct_session_keeps_fourteen(self):
        """The xp that holds the focus on a face of the cube is no xp session: the card there is the plain one."""
        states = snap().states | {"construct": {"windows": [], "focus": {"holder": None}, "open": [], "busy": []}}
        node = text_node("notify-card-body", "Hello", (100, 100, 80, 20), owner="notify") | {"font_size": 11}
        after = snap(states=states, nodes=snap().nodes + [node], shots=shot(at=(100, 104, 70, 10)), clients={"xp", "construct", "acts"})
        self.assertTrue(any("is 11 px, the guide wants 14 px" in b for b in breaches("I6", snap(), after)))

    def test_the_other_looks_keep_fourteen(self):
        for owner in ("mc", "tiling", "construct"):
            self.assertTrue(any("is 11 px, the guide wants 14 px" in b for b in breaches("I6", snap(), self.sized(11, f"{owner}-line", owner))), owner)

    def test_a_state_word_may_be_small(self):
        self.assertEqual(breaches("I6", snap(), self.sized(11, "mc-helper-state")), [])

    def test_an_unknown_size_fails(self):
        node = text_node("mc-line", "Next step", (100, 100, 80, 20))
        after = snap(nodes=snap().nodes + [node], shots=shot(at=(100, 104, 70, 10)))
        self.assertTrue(any("no font size" in b for b in breaches("I6", snap(), after)))

    def test_an_unknown_size_is_not_matched_by_a_size_row(self):
        node = text_node("mc-line", "Next step", (100, 100, 80, 20))
        after = snap(nodes=snap().nodes + [node], shots=shot(at=(100, 104, 70, 10)))
        row = {"look": "x", "id": "I6", "match": "guide wants 14 px"}
        found = [b for b in breaches("I6", snap(), after) if "no font size" in b]
        self.assertTrue(found)
        for b in found:
            self.assertFalse(harness.row_covers(row, b), b)

    def test_only_a_state_label_is_exempt(self):
        self.assertTrue(any("14 px" in b for b in breaches("I6", snap(), self.sized(11, "mc-estate-line"))))

    def test_a_small_run_fails(self):
        nodes = snap().nodes + [panel("xp-tip", (100, 100, 200, 30), size=(200, 30))]
        states = with_texts(run("xp-tip", "Tooltip", (5, 3, 80, 14), px=10))
        self.assertTrue(any("is 10 px, the guide wants 11 px" in b for b in breaches("I6", snap(), snap(states=states, nodes=nodes))))



class GuideHeld(unittest.TestCase):
    """I2 held: a screen-fixed surface stays on its pixel in a held act."""

    def at(self, x):
        return snap(nodes=snap().nodes + [surf("hud-island", (x, 32, 300, 60))])

    def test_still_passes(self):
        self.assertEqual(breaches("I2", self.at(900), self.at(900), HELD), [])

    def test_moved_fails(self):
        self.assertTrue(any("held" in b for b in breaches("I2", self.at(900), self.at(903), HELD)))

    def test_not_held_not_checked(self):
        self.assertEqual(breaches("I2", self.at(900), self.at(903)), [])


class GuideWords(unittest.TestCase):
    """I10: the words of the table; no "agent", no "HUD"."""

    def shown(self, words):
        return snap(nodes=snap().nodes + [text_node("a", words, (0, 0, 120, 14))])

    def test_table_words_pass(self):
        self.assertEqual(breaches("I10", snap(), self.shown("Chat with a helper")), [])

    def test_agent_fails(self):
        self.assertTrue(breaches("I10", snap(), self.shown("Ask the agent")))

    def test_hud_fails(self):
        self.assertTrue(breaches("I10", snap(), self.shown("HUD on")))

    def test_a_word_that_contains_it_passes(self):
        self.assertEqual(breaches("I10", snap(), self.shown("Magenta, shudder")), [])

    def test_a_long_key_name_fails(self):
        self.assertTrue(breaches("I10", snap(), self.cap("Escape")))

    def cap(self, words, label="mc-key-line"):
        return snap(nodes=snap().nodes + [text_node(label, words, (0, 0, 120, 14))])

    def test_a_key_name_inside_a_text_fails(self):
        self.assertTrue(breaches("I10", snap(), self.cap("Press Escape to close")))

    def test_a_plain_label_may_say_left(self):
        self.assertEqual(breaches("I10", snap(), self.shown("Left")), [])

    def test_plural_and_snake_case_fail(self):
        self.assertTrue(breaches("I10", snap(), self.shown("Two HUDs")))
        self.assertTrue(breaches("I10", snap(), self.shown("agent_id")))


if __name__ == "__main__":
    unittest.main()
