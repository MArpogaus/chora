#!/usr/bin/python3
"""Selftest of the act harness (tests/acts): every invariant passes on a
sound snapshot and fails on a planted breach, the known-red list blocks a
listed pass and an unlisted failure, quiet waits for every `busy` to
empty and fails at its timeout, and an `Expect` fails on a mismatch.

The cells need no session; tests/acts/planted.py plants breaches in a real
one. Usage: python3 -I tests/test_acts.py [-v]
"""

import json
import os
import pathlib
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent / "acts"))

import harness  # noqa: E402
import invariants  # noqa: E402
from harness import Act, Expect, Snapshot  # noqa: E402


def surface(sid, owner, output="HEADLESS-1", focused=False, rect=(10, 10, 300, 200)):
    node = {
        "id": sid,
        "kind": "surface",
        "owner": owner,
        "parent": "root",
        "meta": {},
        "rects": {output: list(rect)},
    }
    if focused:
        node["focused"] = True
    return node


def probe_surface(sid="p1"):
    node = surface(sid, "construct")
    node["surface"] = {"app_id": "chora-probe"}
    return node


def window(sid, shown=True):
    return {"surface": sid, "shown": shown, "output": "HEADLESS-1"}


def snap(states=None, nodes=None, **kw):
    """A sound snapshot of one look xp with one shown, focused window."""
    if states is None:
        states = {
            "xp": {
                "windows": [window("s1")],
                "focus": {"holder": "xp", "surface": "s1"},
                "open": [],
                "busy": [],
            }
        }
    if nodes is None:
        nodes = [surface("s1", "xp", focused=True)]
    fields = {
        "states": states,
        "nodes": nodes,
        "toplevels": {"s1"},
        "colours": {"HEADLESS-1": 812},
        "wallpaper": {"HEADLESS-1"},
        "clients": {"xp", "acts"},
    }
    fields.update(kw)
    if "boxes" not in kw:
        fields["boxes"] = boxes_of(fields["nodes"])
    return Snapshot(**fields)


def boxes_of(nodes, size=(1280, 800)):
    """The scene.boxes answer of nodes with rects, bottom first in list
    order; a surface is opaque."""
    out = {}
    for node in nodes:
        for output, rect in (node.get("rects") or {}).items():
            entry = out.setdefault(output, {"size": list(size), "boxes": []})
            entry["boxes"].append(
                {
                    "node": node["id"],
                    "kind": node.get("kind"),
                    "owner": node.get("owner"),
                    "layer": "scene",
                    "rect": list(rect),
                    "opaque": node.get("kind") == "surface",
                }
                | {k: node[k] for k in ("text", "corners", "order", "font_size") if k in node}
            )
    return out


ACT = Act("plain")
HELD = Act("held", held=True)


def breaches(name, before, after, act=ACT):
    return getattr(invariants, name.lower())(before, after, act)


class I1(unittest.TestCase):
    def test_sound(self):
        self.assertEqual(breaches("I1", snap(), snap()), [])

    def test_toplevel_of_no_look(self):
        after = snap(
            nodes=[surface("s1", "xp", focused=True), surface("s2", "acts")], toplevels={"s1", "s2"}
        )
        self.assertTrue(breaches("I1", snap(), after))

    def test_toplevel_missing_from_its_look(self):
        state = snap().states["xp"] | {"windows": []}
        self.assertTrue(breaches("I1", snap(), snap(states={"xp": state})))

    def test_toplevel_in_two_looks(self):
        states = snap().states | {
            "mc": {"windows": [window("s1")], "focus": {"holder": None}, "open": [], "busy": []}
        }
        self.assertTrue(breaches("I1", snap(), snap(states=states, clients={"xp", "mc"})))

    def test_a_view_is_no_owner(self):
        state = snap().states["xp"] | {"views": [{"surface": "s1", "owner": "xp", "pane": "p"}]}
        self.assertTrue(breaches("I1", snap(), snap(states={"xp": state})))

    def test_detached_toplevel(self):
        node = surface("s1", "xp", focused=True) | {"parent": None}
        self.assertTrue(breaches("I1", snap(), snap(nodes=[node])))

    def test_look_without_state(self):
        self.assertTrue(breaches("I1", snap(), snap(states={"xp": None})))


class I3(unittest.TestCase):
    def test_sound(self):
        self.assertEqual(breaches("I3", snap(), snap()), [])

    def test_nothing_focused_and_nobody_claims(self):
        state = snap().states["xp"] | {"focus": {"holder": None}}
        after = snap(states={"xp": state}, nodes=[surface("s1", "xp")])
        self.assertEqual(breaches("I3", snap(), after), [])

    def test_focus_with_another_look(self):
        states = snap().states | {
            "mc": {"windows": [], "focus": {"holder": "mc"}, "open": [], "busy": []}
        }
        states["xp"] = states["xp"] | {"focus": {"holder": None}}
        self.assertTrue(breaches("I3", snap(), snap(states=states)))

    def test_two_holders(self):
        states = snap().states | {
            "mc": {"windows": [], "focus": {"holder": "mc"}, "open": [], "busy": []}
        }
        self.assertTrue(breaches("I3", snap(), snap(states=states)))

    def test_focus_on_a_hidden_window(self):
        state = snap().states["xp"] | {"windows": [window("s1", shown=False)]}
        self.assertTrue(breaches("I3", snap(), snap(states={"xp": state})))

    def test_focus_on_a_node_of_another_client(self):
        nodes = [
            surface("s1", "xp"),
            {
                "id": "x",
                "kind": "rect",
                "owner": "acts",
                "parent": "root",
                "focused": True,
                "rects": {},
            },
        ]
        self.assertTrue(breaches("I3", snap(), snap(nodes=nodes)))

    def test_holder_names_another_surface(self):
        state = snap().states["xp"] | {"focus": {"holder": "xp", "surface": "s9"}}
        self.assertTrue(breaches("I3", snap(), snap(states={"xp": state})))


def with_open(*ids):
    state = snap().states["xp"] | {
        "open": [{"id": i, "kind": "menu", "modal": False, "by": "key"} for i in ids]
    }
    return snap(states={"xp": state})


class I4(unittest.TestCase):
    def test_sound(self):
        self.assertEqual(breaches("I4", with_open("a"), with_open("a")), [])

    def test_undeclared_open(self):
        self.assertTrue(breaches("I4", with_open(), with_open("start")))

    def test_declared_open(self):
        act = Act("menu", opens=["start"])
        self.assertEqual(breaches("I4", with_open(), with_open("start"), act), [])

    def test_declared_but_not_open(self):
        self.assertTrue(breaches("I4", with_open(), with_open(), Act("menu", opens=["start"])))

    def test_undeclared_close(self):
        self.assertTrue(breaches("I4", with_open("start"), with_open()))

    def test_declared_close(self):
        act = Act("close", closes=["start"])
        self.assertEqual(breaches("I4", with_open("start"), with_open(), act), [])


class I7(unittest.TestCase):
    def test_sound(self):
        self.assertEqual(breaches("I7", snap(), snap()), [])

    def test_one_colour(self):
        self.assertTrue(breaches("I7", snap(), snap(colours={"HEADLESS-1": 1})))

    def test_no_wallpaper_of_the_look(self):
        after = snap(colours={"HEADLESS-1": 1}, wallpaper=set())
        self.assertEqual(breaches("I7", snap(), after), [])

    def test_no_capture(self):
        self.assertTrue(breaches("I7", snap(), snap(colours={})))


def helper_state(helper, shown=()):
    return {
        "windows": [],
        "focus": {"holder": None},
        "open": [],
        "busy": [],
        "helper": helper,
        "shown": list(shown),
    }


class WorkRect(unittest.TestCase):
    def test_a_screen_at_the_origin(self):
        output = {
            "position": [0, 0],
            "size": [1280, 800],
            "work_area": [0, 0, 1280, 770],
            "default_work_area": [0, 0, 1280, 770],
        }
        self.assertEqual(harness.work_rect(output), [0, 0, 1280, 770])

    def test_a_virtual_output_far_off_the_screens(self):
        output = {
            "position": [-262144, -262144],
            "size": [1280, 800],
            "work_area": [-262144, -262144, 1280, 770],
            "default_work_area": [-262144, -262144, 1280, 770],
        }
        self.assertEqual(harness.work_rect(output), [0, 0, 1280, 770])

    def test_a_moved_camera_leaves_the_zone_where_it_is(self):
        output = {
            "position": [-262144, -262144],
            "size": [1280, 800],
            "work_area": [300, 200, 640, 385],
            "default_work_area": [-262144, -262144, 1280, 770],
        }
        self.assertEqual(harness.work_rect(output), [0, 0, 1280, 770])


class I8(unittest.TestCase):
    def states(self, helper):
        return snap().states | {"assistant": helper_state(helper)}

    def test_sound(self):
        after = snap(
            states=self.states("idle"),
            events=[
                {"event": "voice.state", "data": {"state": "working"}},
                {"event": "voice.state", "data": {"state": "idle"}},
            ],
        )
        self.assertEqual(breaches("I8", snap(), after), [])

    def test_working_after_the_answer(self):
        after = snap(
            states=self.states("working"),
            events=[
                {"event": "voice.state", "data": {"state": "working"}},
                {"event": "voice.state", "data": {"state": "idle"}},
            ],
        )
        self.assertTrue(breaches("I8", snap(), after))

    def test_working_while_the_helper_works(self):
        after = snap(
            states=self.states("working"),
            events=[{"event": "voice.state", "data": {"state": "working"}}],
        )
        self.assertEqual(breaches("I8", snap(), after), [])

    def test_two_helper_surfaces_on_one_output(self):
        nodes = snap().nodes + [
            {
                "id": "hud-chat",
                "kind": "group",
                "owner": "assistant",
                "parent": "root",
                "visible": True,
                "rects": {"HEADLESS-1": [0, 0, 400, 300]},
            },
            {
                "id": "xp-helper-chat",
                "kind": "group",
                "owner": "xp",
                "parent": "root",
                "meta": {"helper": "surface"},
                "rects": {"HEADLESS-1": [0, 0, 400, 300]},
            },
        ]
        self.assertTrue(breaches("I8", snap(), snap(nodes=nodes)))


class I14(unittest.TestCase):
    def test_sound(self):
        self.assertEqual(breaches("I14", snap(), snap(logs={"core": "fine\n"})), [])

    def test_effect_failed(self):
        after = snap(events=[{"event": "effect.failed", "error": "x.frag: 0:1 error"}])
        self.assertTrue(breaches("I14", snap(), after))

    def test_panic_line(self):
        after = snap(logs={"xp": "thread 'main' panicked at src/wm.rs:1:1:\nboom\n"})
        self.assertTrue(breaches("I14", snap(), after))

    def test_undeclared_disconnect(self):
        self.assertTrue(breaches("I14", snap(), snap(clients={"acts"})))

    def test_declared_disconnect(self):
        act = Act("restart", disconnects=["xp"])
        self.assertEqual(breaches("I14", snap(), snap(clients={"acts"}), act), [])


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

    def test_the_list_names_the_authentic_tahoma_volume_label_of_the_xp_popup(self):
        rows = [r for r in harness.known_red() if r.get("capture") == "xp-ctx-0-7"]
        self.assertEqual(len(rows), 1)
        row = rows[0]
        self.assertEqual((row["look"], row["id"], row["act"]), ("xp", "I6", "a click on the speaker opens the volume popup"))
        self.assertIn("authentic Tahoma 8 pt", row["slice"])
        seen = "in act 'a click on the speaker opens the volume popup': on O 'Volume' (xp-ctx-0-7) is 11 px, the guide wants 14 px"
        other = seen.replace("xp-ctx-0-7", "xp-ctx-0-6")
        elsewhere = seen.replace("'a click on the speaker opens the volume popup'", "'another act'")
        self.assertEqual(harness.judge({("xp", "I6"): [seen]}, [row]).blocking, [])
        self.assertTrue(harness.judge({("xp", "I6"): [other]}, [row]).blocking)
        self.assertTrue(harness.judge({("xp", "I6"): [elsewhere]}, [row]).blocking)

    def test_the_list_names_the_authentic_tahoma_text_of_the_xp_taskbar_buttons(self):
        rows = [r for r in harness.known_red() if r.get("capture") == "xp-task-*"]
        self.assertEqual(len(rows), 1)
        row = rows[0]
        self.assertEqual((row["look"], row["id"]), ("xp", "I6"))
        self.assertIn("authentic Tahoma 8 pt", row["slice"])
        size = "on O 'chora-probe' (xp-task-s4-HEADLESS-1) is 11 px, the guide wants 14 px"
        for act in ("a window opens to hear the keys", "another act"):
            self.assertEqual(harness.judge({("xp", "I6"): [f"in act {act!r}: {size}"]}, [row]).blocking, [])
        for other in (
            size.replace("xp-task-s4", "xp-tasks-s4"),
            size.replace("xp-task-s4-HEADLESS-1", "xp-ctx-0-7"),
            size.replace("xp-task-s4-HEADLESS-1", "xp-tray-O"),
            "on O 'chora-probe' (xp-task-s4-HEADLESS-1) leaves its panel xp-bar [0, 0, 1, 1]",
            "on O 'chora-probe' (xp-task-s4-HEADLESS-1) has the contrast 2.1:1 (grey on blue), the guide wants 4.5:1",
        ):
            with self.subTest(other=other):
                self.assertTrue(harness.judge({("xp", "I6"): ["in act 'a': " + other]}, [row]).blocking)
        self.assertTrue(harness.judge({("xp", "I3"): ["in act 'a': " + size]}, [row]).blocking)

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


class Quiet(unittest.TestCase):
    def test_waits_until_every_busy_is_empty(self):
        reads = iter([{"xp": {"busy": ["anim"]}}, {"xp": {"busy": []}}, {"xp": {"busy": []}}])
        clock = [0.0]
        states = harness.wait_quiet(
            lambda: next(reads),
            timeout=2,
            sleep=lambda s: clock.__setitem__(0, clock[0] + s),
            now=lambda: clock[0],
        )
        self.assertEqual(states, {"xp": {"busy": []}})

    def test_fails_at_the_timeout(self):
        clock = [0.0]
        with self.assertRaises(harness.NotQuiet) as caught:
            harness.wait_quiet(
                lambda: {"xp": {"busy": ["anim"]}},
                timeout=2,
                sleep=lambda s: clock.__setitem__(0, clock[0] + s),
                now=lambda: clock[0],
            )
        self.assertIn("anim", str(caught.exception))

    def test_one_empty_read_is_not_quiet(self):
        reads = iter(
            [{"xp": {"busy": []}}, {"xp": {"busy": ["anim"]}}] + [{"xp": {"busy": []}}] * 2
        )
        clock = [0.0]
        harness.wait_quiet(
            lambda: next(reads),
            timeout=5,
            sleep=lambda s: clock.__setitem__(0, clock[0] + s),
            now=lambda: clock[0],
        )
        self.assertEqual(list(reads), [])

    def test_a_state_without_busy_is_not_quiet(self):
        with self.assertRaises(harness.NotQuiet) as caught:
            harness.wait_quiet(lambda: {"xp": {"windows": []}}, timeout=1)
        self.assertIn("missing key busy", str(caught.exception))

    def test_a_look_without_state_is_quiet(self):
        states = harness.wait_quiet(lambda: {"xp": None}, timeout=1)
        self.assertEqual(states, {"xp": None})


class Expectations(unittest.TestCase):
    def before(self):
        return snap(
            states={"xp": {"windows": [], "focus": {"holder": None}, "open": [], "busy": []}},
            nodes=[],
            toplevels=set(),
        )

    def after(self, app_id="notepad"):
        node = surface("s1", "xp", focused=True)
        node["surface"] = {"app_id": app_id, "title": None}
        return snap(nodes=[node])

    def test_new_window_matches(self):
        expect = Expect(new_window={"app_id": "notepad", "owner": "xp", "focused": True})
        self.assertEqual(expect.check(self.before(), self.after()), [])

    def test_new_window_mismatch(self):
        expect = Expect(new_window={"app_id": "notepad", "owner": "xp", "focused": True})
        self.assertTrue(expect.check(self.before(), self.after("foot")))

    def test_no_new_window(self):
        expect = Expect(new_window={"app_id": "notepad"})
        self.assertTrue(expect.check(self.before(), self.before()))

    def test_gone(self):
        self.assertEqual(Expect(gone=1).check(self.after(), self.before()), [])
        self.assertTrue(Expect(gone=1).check(self.after(), self.after()))

    def test_cube(self):
        listing = {"faces": [{}, {}, {}, {}], "front": 1, "turning": False, "mode": "use"}
        after = snap(cube=listing)
        self.assertEqual(Expect(cube={"faces": 4, "front": 1}).check(snap(), after), [])
        found = Expect(cube={"front": 2}).check(snap(), after)
        self.assertEqual(found, ["the cube: front is 1, expected 2"])
        missing = snap(cube={"error": "no client exports cube.list"})
        self.assertTrue(Expect(cube={"front": 0}).check(snap(), missing))

    def test_verify(self):
        expect = Expect(verify=lambda after: ["shown"] if after.nodes else [])
        self.assertEqual(expect.check(snap(), snap()), ["shown"])


OUT = "HEADLESS-1"


def text_node(nid, text, rect, parent="root", owner="mc"):
    return {
        "id": nid,
        "kind": "text",
        "owner": owner,
        "parent": parent,
        "text": text,
        "rects": {OUT: list(rect)},
    }


def panel(nid, rect, owner="xp", kind="image", size=None):
    node = {"id": nid, "kind": kind, "owner": owner, "parent": "root", "rects": {OUT: list(rect)}}
    if size:
        node["size"] = list(size)
    return node


def with_texts(*runs, look="xp"):
    return snap().states | {look: snap().states[look] | {"texts": list(runs)}}


def translate(x, y):
    return [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, x, y, 0, 1]


def run(node, text, rect, px=14):
    return {"node": node, "text": text, "rect": list(rect), "px": px, "fg": [0, 0, 0, 255]}


class I2(unittest.TestCase):
    def test_sound(self):
        self.assertEqual(breaches("I2", snap(), snap()), [])

    def test_a_window_off_screen(self):
        nodes = [surface("s1", "xp", focused=True, rect=(1270, 10, 300, 200))]
        self.assertTrue(breaches("I2", snap(), snap(nodes=nodes)))

    def test_a_shown_window_without_a_box(self):
        self.assertTrue(
            breaches("I2", snap(), snap(nodes=[surface("s1", "xp", rect=(0, 0, 0, 0))]))
        )

    def test_a_declared_float_may_leave(self):
        state = snap().states["xp"] | {"windows": [window("s1") | {"floating": True}]}
        nodes = [surface("s1", "xp", focused=True, rect=(1270, 10, 300, 200))]
        self.assertEqual(breaches("I2", snap(), snap(states={"xp": state}, nodes=nodes)), [])


class I5(unittest.TestCase):
    def test_sound(self):
        nodes = snap().nodes + [
            text_node("a", "Start", (0, 0, 40, 14)),
            text_node("b", "12:00", (60, 0, 40, 14)),
        ]
        self.assertEqual(breaches("I5", snap(), snap(nodes=nodes)), [])

    def test_two_text_nodes_overprint(self):
        nodes = snap().nodes + [
            text_node("a", "No helper", (0, 0, 80, 14)),
            text_node("b", "Volume 40%", (40, 2, 80, 14)),
        ]
        self.assertTrue(breaches("I5", snap(), snap(nodes=nodes)))

    def test_the_leading_of_two_lines_may_touch(self):
        nodes = snap().nodes + [
            text_node("a", "NOTIFICATIONS", (888, 92, 128, 20)),
            text_node("b", "Nothing new.", (888, 110, 128, 20)),
        ]
        self.assertEqual(breaches("I5", snap(), snap(nodes=nodes)), [])

    def test_a_look_under_the_wallpaper_of_another_is_not_on_the_screen(self):
        nodes = [
            text_node("mc-r1-right-doors-volume", "Volume 50%", (988, 756, 87, 28)),
            panel("xp-wall-HEADLESS-1", (0, 0, 1280, 800)),
            text_node("xp-pager-0-HEADLESS-1", "1", (1060, 778, 12, 14), owner="xp"),
        ]
        self.assertEqual(breaches("I5", snap(), snap(nodes=nodes)), [])

    def test_a_picture_of_mc_over_xp_text_may_be_translucent(self):
        nodes = [
            text_node("xp-icons-label", "Documents", (988, 756, 87, 28), owner="xp"),
            panel("mc-bg", (0, 0, 1280, 800), owner="mc"),
            text_node("xp-pager-0-HEADLESS-1", "1", (1060, 778, 12, 14), owner="xp"),
        ]
        self.assertTrue(breaches("I5", snap(), snap(nodes=nodes)))

    def test_a_picture_of_its_own_look_between_its_texts_hides_nothing(self):
        nodes = [
            text_node("mc-a", "Volume 50%", (988, 756, 87, 28)),
            panel("mc-bg", (0, 0, 1280, 800), owner="mc"),
            text_node("mc-b", "Mute", (1060, 778, 12, 14)),
        ]
        self.assertTrue(breaches("I5", snap(), snap(nodes=nodes)))

    def test_mc_drawn_over_xp_while_xp_has_the_focus_is_no_cover(self):
        """rv-r4-1: xp text, the picture of mc over the whole output, mc
        text. xp is the look shown (it holds the focus), so a picture of mc
        is no wallpaper of the look shown and the overlap is a breach."""
        nodes = snap().nodes + [
            text_node("xp-icons-label", "Documents", (988, 756, 87, 28), owner="xp"),
            panel("mc-bg", (0, 0, 1280, 800), owner="mc"),
            text_node("mc-volume", "Volume 50%", (1000, 760, 87, 28)),
        ]
        self.assertTrue(breaches("I5", snap(), snap(nodes=nodes)))

    def test_the_wallpaper_hides_nothing_above_it(self):
        nodes = [
            panel("xp-wall-HEADLESS-1", (0, 0, 1280, 800)),
            text_node("mc-r1-right-doors-volume", "Volume 50%", (988, 756, 87, 28)),
            text_node("xp-pager-0-HEADLESS-1", "1", (1060, 778, 12, 14), owner="xp"),
        ]
        self.assertTrue(breaches("I5", snap(), snap(nodes=nodes)))

    def test_a_picture_of_part_of_the_output_hides_nothing(self):
        nodes = [
            text_node("mc-r1-right-doors-volume", "Volume 50%", (988, 756, 87, 28)),
            panel("xp-panel-HEADLESS-1", (0, 770, 1280, 30)),
            text_node("xp-pager-0-HEADLESS-1", "1", (1060, 778, 12, 14), owner="xp"),
        ]
        self.assertTrue(breaches("I5", snap(), snap(nodes=nodes)))

    def test_an_opaque_window_between_hides_the_lower_text(self):
        nodes = [
            text_node("a", "label", (20, 20, 80, 14)),
            surface("s1", "xp", focused=True),
            text_node("b", "title", (30, 22, 80, 14)),
        ]
        self.assertEqual(breaches("I5", snap(), snap(nodes=nodes)), [])

    def test_a_run_over_a_text_node(self):
        nodes = snap().nodes + [
            panel("xp-tip", (100, 100, 200, 30), size=(200, 30)),
            text_node("a", "clock", (110, 105, 60, 14)),
        ]
        states = with_texts(run("xp-tip", "Tooltip", (5, 3, 80, 14)))
        self.assertTrue(breaches("I5", snap(), snap(states=states, nodes=nodes)))

    def scrolled(self, text_at):
        """A menu image 200 px wide, scrolled 100 px to the left under a
        clip 100 px wide: its box is the visible part, the run 'Lines'
        sits at 20 to 70 of the screen."""
        menu = panel("menu", (0, 0, 100, 40), size=(200, 40)) | {
            "transform": {"matrix": translate(-100, 0)}
        }
        nodes = [menu, text_node("a", "other", text_at)]
        states = with_texts(run("menu", "Lines", (120, 5, 50, 14)))
        return snap(states=states, nodes=nodes)

    def test_a_clipped_image_keeps_its_runs_unsqueezed(self):
        self.assertEqual(breaches("I5", snap(), self.scrolled((70, 5, 30, 14))), [])

    def test_a_clipped_image_still_overprints_where_its_run_lies(self):
        self.assertTrue(breaches("I5", snap(), self.scrolled((20, 5, 30, 14))))


class I6(unittest.TestCase):
    def test_sound(self):
        nodes = snap().nodes + [panel("xp-tip", (100, 100, 200, 30), size=(200, 30))]
        states = with_texts(run("xp-tip", "Tooltip", (5, 3, 80, 14), px=16))
        after = snap(states=states, nodes=nodes, shots=shot(at=(100, 104, 70, 10)))
        self.assertEqual(breaches("I6", snap(), after), [])

    def test_a_run_leaves_its_picture(self):
        nodes = snap().nodes + [panel("xp-tip", (100, 100, 200, 30), size=(200, 30))]
        states = with_texts(run("xp-tip", "a long key name", (150, 3, 80, 14)))
        self.assertTrue(breaches("I6", snap(), snap(states=states, nodes=nodes)))

    def test_a_text_node_leaves_its_panel(self):
        nodes = snap().nodes + [
            panel("mc-row", (0, 0, 100, 20), owner="mc", kind="rect"),
            text_node("mc-row-key", "Super+Shift+Return", (60, 2, 120, 14), parent="mc-row"),
        ]
        self.assertTrue(breaches("I6", snap(), snap(nodes=nodes)))


def notify(*shown, accepted=None, waiting=(), closed=()):
    return {
        "notify": {
            "shown": [{"id": i, "summary": f"note {i}", "node": "notify-card"} for i in shown],
            "accepted": list(shown if accepted is None else accepted),
            "waiting": list(waiting),
            "closed": list(closed),
            "texts": [],
            "busy": [],
        }
    }


class I9(unittest.TestCase):
    def test_sound(self):
        nodes = snap().nodes + [panel("notify-card", (900, 600, 300, 80), owner="notify")]
        self.assertEqual(breaches("I9", snap(), snap(nodes=nodes, blocks=notify(1))), [])

    def test_a_notice_without_a_box(self):
        self.assertTrue(breaches("I9", snap(), snap(blocks=notify(1))))

    def test_a_notice_under_the_chat(self):
        nodes = snap().nodes + [
            panel("notify-card", (900, 600, 300, 80), owner="notify"),
            panel("hud-chat", (800, 400, 480, 400), owner="assistant", kind="group"),
        ]
        self.assertTrue(breaches("I9", snap(), snap(nodes=nodes, blocks=notify(1))))

    def test_a_notice_off_the_output(self):
        nodes = snap().nodes + [panel("notify-card", (1100, 600, 300, 80), owner="notify")]
        self.assertTrue(breaches("I9", snap(), snap(nodes=nodes, blocks=notify(1))))

    def test_a_notice_from_before_the_act_is_not_checked(self):
        self.assertEqual(breaches("I9", snap(blocks=notify(1)), snap(blocks=notify(1))), [])

    def cards(self):
        return snap().nodes + [panel("notify-card", (900, 600, 300, 80), owner="notify")]

    def test_an_accepted_notice_that_never_showed(self):
        after = snap(nodes=self.cards(), blocks=notify(1, accepted=[1, 2]))
        out = breaches("I9", snap(blocks=notify(1)), after)
        self.assertTrue(any("2" in b and "never" in b for b in out), out)

    def test_an_accepted_notice_that_waits_or_closed_is_no_breach(self):
        after = snap(
            nodes=self.cards(), blocks=notify(1, accepted=[1, 2, 3], waiting=[2], closed=[3])
        )
        self.assertEqual(breaches("I9", snap(blocks=notify(1)), after), [])

    def test_an_accepted_notice_of_before_the_act_is_not_checked(self):
        before = snap(blocks=notify(accepted=[5]))
        after = snap(blocks=notify(accepted=[5]))
        self.assertEqual(breaches("I9", before, after), [])


class I10(unittest.TestCase):
    def test_sound(self):
        nodes = snap().nodes + [text_node("a", "Text Editor", (0, 0, 80, 14))]
        self.assertEqual(breaches("I10", snap(), snap(nodes=nodes)), [])

    def test_raw_entries(self):
        for raw in (
            "org.gnome.TextEditor.desktop",
            "firefox %U",
            "org.gnome.Nautilus",
            "/usr/bin/foot",
        ):
            nodes = snap().nodes + [text_node("a", raw, (0, 0, 80, 14))]
            self.assertTrue(breaches("I10", snap(), snap(nodes=nodes)), raw)

    def test_a_raw_run(self):
        nodes = snap().nodes + [panel("xp-menu", (0, 300, 300, 480), size=(300, 480))]
        states = with_texts(run("xp-menu", "org.kde.dolphin.desktop", (40, 10, 200, 14)))
        self.assertTrue(breaches("I10", snap(), snap(states=states, nodes=nodes)))


class I12(unittest.TestCase):
    def fullscreen(self, *above):
        state = snap().states["xp"] | {"windows": [window("s1") | {"fullscreen": True}]}
        nodes = [surface("s1", "xp", focused=True, rect=(0, 0, 1280, 800)), *above]
        return snap(states={"xp": state}, nodes=nodes)

    def test_sound(self):
        self.assertEqual(breaches("I12", snap(), self.fullscreen()), [])

    def test_a_taskbar_over_a_fullscreen_window(self):
        bar = panel("xp-taskbar", (0, 770, 1280, 30))
        self.assertTrue(breaches("I12", snap(), self.fullscreen(bar)))

    def test_a_layer_bar_over_a_fullscreen_window(self):
        after = self.fullscreen()
        after.boxes[OUT]["boxes"].append(
            {"namespace": "waybar", "layer": "top", "rect": [0, 0, 1280, 30], "opaque": False}
        )
        self.assertTrue(breaches("I12", snap(), after))

    def test_a_dialog_surface_above_its_fullscreen_parent_is_no_bar(self):
        dialog = surface("s2", "xp", rect=(400, 300, 300, 200))
        self.assertEqual(breaches("I12", snap(), self.fullscreen(dialog)), [])

    def test_a_modal_node_of_the_look_above_its_fullscreen_parent_is_no_bar(self):
        state = snap().states["xp"] | {
            "windows": [window("s1") | {"fullscreen": True}],
            "open": [{"id": "xp-dlg-run", "modal": True, "by": "s1"}],
        }
        nodes = [
            surface("s1", "xp", focused=True, rect=(0, 0, 1280, 800)),
            panel("xp-dlg-run", (400, 300, 300, 200)),
        ]
        self.assertEqual(breaches("I12", snap(), snap(states={"xp": state}, nodes=nodes)), [])

    def test_a_modal_below_its_parent(self):
        state = snap().states["xp"] | {"open": [{"id": "xp-dlg-run", "modal": True, "by": "s1"}]}
        nodes = [panel("xp-dlg-run", (100, 100, 300, 200)), surface("s1", "xp", focused=True)]
        self.assertTrue(breaches("I12", snap(), snap(states={"xp": state}, nodes=nodes)))

    def held(self, *above):
        state = snap().states["xp"] | {"windows": [window("s1") | {"held": True}]}
        nodes = [surface("s1", "xp", focused=True, rect=(100, 100, 400, 300)), *above]
        return snap(states={"xp": state}, nodes=nodes)

    def test_sound_with_a_held_window_on_top(self):
        below = surface("s2", "tiling", rect=(0, 0, 1280, 800))
        after = self.held()
        after.boxes[OUT]["boxes"].insert(0, boxes_of([below])[OUT]["boxes"][0])
        self.assertEqual(breaches("I12", snap(), after), [])

    def test_a_window_of_another_look_over_the_held_window(self):
        other = surface("s2", "tiling", rect=(300, 200, 400, 300))
        self.assertTrue(breaches("I12", snap(), self.held(other)))

    def test_a_root_of_another_look_over_the_held_window(self):
        page = panel("mc-bg", (0, 0, 1280, 800), owner="mc", kind="rect") | {"order": "normal"}
        self.assertTrue(breaches("I12", snap(), self.held(page)))

    def test_a_bar_in_the_overlay_order_over_the_held_window_is_no_breach(self):
        bar = panel("bar", (0, 0, 1280, 120), owner="tiling") | {"order": "overlay"}
        self.assertEqual(breaches("I12", snap(), self.held(bar)), [])

    def real_state_of_a_drag(self):
        """The state.get of tiling during the drag of a floating window
        under another, from the cell of the plugin that dumps it."""
        with tempfile.TemporaryDirectory() as tmp:
            dump = pathlib.Path(tmp) / "state.json"
            subprocess.run(
                ["cargo", "test", "-p", "chora-tiling", "--locked", "--", "state_get_names"],
                cwd=ROOT,
                env={**os.environ, "CHORA_STATE_DUMP": str(dump)},
                capture_output=True,
                check=False,
            )
            return json.loads(dump.read_text())

    def test_tiling_reports_the_dragged_window_as_held(self):
        state = self.real_state_of_a_drag()
        held = [w["surface"] for w in state["windows"] if w.get("held")]
        self.assertEqual(held, ["s1"], state)
        under = surface("s1", "tiling", rect=(100, 100, 400, 300))
        over = surface("s2", "other", rect=(300, 200, 400, 300))
        after = snap(states={"tiling": state}, nodes=[under, over])
        self.assertTrue(breaches("I12", snap(), after))

    def test_the_frame_of_the_held_window_is_no_breach(self):
        title = panel("xp-title-s1", (100, 70, 400, 30))
        self.assertEqual(breaches("I12", snap(), self.held(title)), [])


def face(k, corners, owner="construct"):
    xs = [c[0] for c in corners]
    ys = [c[1] for c in corners]
    rect = (min(xs), min(ys), max(xs) - min(xs), max(ys) - min(ys))
    return {
        "id": f"construct-cube-face-{k}",
        "kind": "shader",
        "owner": owner,
        "parent": "construct-cube",
        "corners": [list(c) for c in corners],
        "rects": {OUT: list(rect)},
    }


class I13(unittest.TestCase):
    FRONT = [[300, 200], [700, 220], [700, 580], [300, 600]]

    def test_sound(self):
        self.assertEqual(breaches("I13", snap(), self.ring(0, 1, 2, 3)), [])

    def ring(self, *ks, bare=()):
        """The four faces of a cube turned a little: each meets the next."""
        quads = {
            0: self.FRONT,
            1: [[700, 220], [820, 240], [820, 560], [700, 580]],
            2: [[820, 240], [250, 228], [250, 572], [820, 560]],
            3: [[250, 228], [300, 200], [300, 600], [250, 572]],
        }
        nodes = [face(k, quads[k]) for k in ks]
        for node in nodes:
            if int(node["id"].rsplit("-", 1)[1]) in bare:
                del node["corners"]
        return snap(nodes=snap().nodes + nodes)

    def test_all_four_faces_meeting(self):
        self.assertEqual(breaches("I13", snap(), self.ring(0, 1, 2, 3)), [])

    def test_a_neighbour_of_a_shown_face_is_missing(self):
        out = breaches("I13", snap(), self.ring(1), HELD)
        self.assertTrue(any("construct-cube-face-1" in b and "alone" in b for b in out), out)

    def test_held_in_a_turn_a_closed_cube_hides_its_back_faces(self):
        """co-21: halfway through a turn faces 1 and 2 show and meet; 0
        and 3 face away and have no box, as a closed cube hides them."""
        self.assertEqual(breaches("I13", snap(), self.ring(1, 2), HELD), [])

    def test_a_neighbour_of_a_shown_face_has_no_corners(self):
        out = breaches("I13", snap(), self.ring(0, 1, 2, 3, bare=(1,)), HELD)
        self.assertTrue(any("construct-cube-face-1" in b and "corners" in b for b in out), out)

    def test_at_rest_a_hidden_neighbour_is_no_breach(self):
        self.assertEqual(breaches("I13", snap(), self.ring(0, 1)), [])

    def test_no_face_shown_is_no_breach(self):
        self.assertEqual(breaches("I13", snap(), snap()), [])

    def test_a_seam_open_by_a_few_pixels(self):
        side = [[705, 214], [830, 240], [830, 560], [705, 586]]
        nodes = snap().nodes + [face(0, self.FRONT), face(1, side)]
        self.assertTrue(breaches("I13", snap(), snap(nodes=nodes)))

    def test_the_last_face_meets_the_first(self):
        self.assertEqual(breaches("I13", snap(), self.ring(0, 1, 2, 3)), [])
        after = self.ring(0, 1, 2, 3)
        last = next(n for n in after.nodes if n["id"].endswith("face-3"))
        last["corners"] = [[250, 228], [290, 200], [290, 600], [250, 572]]
        after.boxes = boxes_of(after.nodes)
        self.assertTrue(breaches("I13", snap(), after))


class MissingKeys(unittest.TestCase):
    """A look that answers state.get without a key an invariant needs is a
    breach, not a pass."""

    def without(self, key, **kw):
        state = {k: v for k, v in snap().states["xp"].items() if k != key}
        return snap(states={"xp": state}, **kw)

    def test_i1_without_windows(self):
        found = breaches("I1", snap(), self.without("windows"))
        self.assertTrue(any("missing key windows" in b for b in found), found)

    def test_i3_without_focus(self):
        found = breaches("I3", snap(), self.without("focus"))
        self.assertTrue(any("missing key focus" in b for b in found), found)

    def test_i4_without_open(self):
        found = breaches("I4", snap(), self.without("open"))
        self.assertTrue(any("missing key open" in b for b in found), found)

    def test_i8_without_helper_after_a_voice_event(self):
        events = [{"event": "voice.state", "data": {"state": "idle"}}]
        found = breaches("I8", snap(), self.without("helper", events=events))
        self.assertTrue(any("missing key helper" in b for b in found), found)

    def test_i8_helper_surface_without_boxes(self):
        node = {"id": "hud-chat", "kind": "group", "owner": "xp", "parent": "root"}
        found = breaches("I8", snap(), snap(nodes=snap().nodes + [node], boxes=None))
        self.assertTrue(any("missing boxes" in b for b in found), found)

    def test_i9_without_shown(self):
        found = breaches("I9", snap(), snap(blocks={"notify": {"texts": [], "busy": []}}))
        self.assertTrue(any("missing key shown" in b for b in found), found)

    def test_a_look_without_state_stays_no_state(self):
        found = breaches("I1", snap(), snap(states={"xp": None}))
        self.assertEqual(found, ["xp answers no state.get"])
        self.assertEqual(breaches("I4", snap(), snap(states={"xp": None})), [])


class Teardown(unittest.TestCase):
    """Session setup that fails leaves no process and no temp dir."""

    def test_failed_setup_cleans_up(self):
        started = []
        real = subprocess.Popen

        def fake(argv, **kw):
            proc = real(["sleep", "60"], start_new_session=True)
            started.append(proc)
            return proc

        session = harness.Session("xp")
        with (
            mock.patch.object(harness.subprocess, "Popen", fake),
            mock.patch.object(session, "wait_env", side_effect=RuntimeError("no core")),
            self.assertRaises(RuntimeError),
        ):
            session.__enter__()
        self.assertEqual(len(started), 1)
        self.assertIsNotNone(started[0].poll(), "the core process group still runs")
        self.assertFalse(session.work.exists(), "the temp dir is left")

    def test_exit_waits_for_a_slow_term(self):
        script = "trap '' TERM; sleep 60"
        proc = subprocess.Popen(["sh", "-c", script], start_new_session=True)
        time.sleep(0.2)
        session = harness.Session("xp")
        session.work = pathlib.Path("/nonexistent-acts-work")
        session.procs = [proc]
        start = time.monotonic()
        with mock.patch.object(harness, "TERM_WAIT", 1):
            session.__exit__(None, None, None)
        self.assertGreaterEqual(time.monotonic() - start, 0.9, "SIGKILL came before the wait")
        self.assertIsNotNone(proc.poll())


sys.path.insert(0, str(ROOT / "tests" / "persona"))

import check  # noqa: E402


def ctx(look="xp", hist=(), calls=(), probe=None, keys=(), entries=(), left=()):
    more = {"left": list(left)} if left else {}
    return check.Ctx(look, list(hist), list(calls), probe, list(keys), list(entries), **more)


def seen(open_ids=(), helper="idle", keysheet=None, look="xp", page=None):
    return {
        "t": 1.0,
        "action": "a",
        "open": list(open_ids),
        "helper": {look: helper},
        "page": page,
        "keysheet": keysheet,
    }


def page_node(name, visible=True, opacity=1.0):
    return {
        "id": f"mc-s-{name}",
        "kind": "group",
        "owner": "mc",
        "parent": "mc",
        "visible": visible,
        "opacity": opacity,
    }


def verdict(task, context, before, after):
    expect = check.contracts(task, context, before)
    return expect.check(before, after)


class PersonaContracts(unittest.TestCase):
    """The end-state contracts of tests/persona/check.py, each green on a
    sound end state and red on the breach it exists for."""

    def test_snapshot_round_trip(self):
        before = snap()
        self.assertEqual(check.restore(json.loads(check.dump(before))), before)

    def test_t1_new_focused_window_of_the_look(self):
        before = snap()
        two = [surface("s1", "xp"), surface("s2", "xp", focused=True)]
        after = snap(nodes=two, toplevels={"s1", "s2"})
        self.assertEqual(verdict("T1", ctx(), before, after), [])
        self.assertIn("no new focused window", verdict("T1", ctx(), before, snap())[0])

    def test_t1_launcher_still_open(self):
        before = snap()
        states = {"xp": {"windows": [], "focus": {}, "open": [{"id": "xp-menu"}], "busy": []}}
        two = [surface("s1", "xp"), surface("s2", "xp", focused=True)]
        after = snap(states=states, nodes=two, toplevels={"s1", "s2"})
        self.assertIn("expected open []", verdict("T1", ctx(), before, after)[0])

    def test_t1_the_key_list_of_the_persona_may_stay_open(self):
        before = snap()
        states = {"xp": {"windows": [], "focus": {}, "open": [{"id": "xp-dlg-help"}], "busy": []}}
        two = [surface("s1", "xp"), surface("s2", "xp", focused=True)]
        after = snap(states=states, nodes=two, toplevels={"s1", "s2"})
        self.assertEqual(verdict("T1", ctx(left=["xp-dlg-help"], keys=[59]), before, after), [])

    def test_t1_a_key_list_that_opened_unasked_stays_red(self):
        before = snap()
        states = {"xp": {"windows": [], "focus": {}, "open": [{"id": "xp-dlg-help"}], "busy": []}}
        two = [surface("s1", "xp"), surface("s2", "xp", focused=True)]
        after = snap(states=states, nodes=two, toplevels={"s1", "s2"})
        unasked = ctx(left=["xp-dlg-help"], hist=[seen(["xp-dlg-help"])])
        self.assertIn("expected open []", verdict("T1", unasked, before, after)[0])

    def test_t4_a_key_list_that_opened_unasked_stays_red(self):
        states = {"xp": {"windows": [], "focus": {}, "open": [{"id": "xp-dlg-help"}], "busy": []}}
        calls = ["wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+"]
        call = ctx(calls=calls, left=["xp-dlg-help"], hist=[seen(["xp-dlg-help"])])
        self.assertIn("expected open", verdict("T4:volume", call, snap(), snap(states=states))[0])

    def test_t4_the_key_list_of_the_persona_may_stay_open(self):
        states = {"xp": {"windows": [], "focus": {}, "open": [{"id": "xp-dlg-help"}], "busy": []}}
        calls = ["wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+"]
        call = ctx(calls=calls, left=["xp-dlg-help"], keys=[59])
        self.assertEqual(verdict("T4:volume", call, snap(), snap(states=states)), [])
        call = ctx(calls=calls)
        menu = {"xp": {"windows": [], "focus": {}, "open": [{"id": "xp-menu"}], "busy": []}}
        self.assertIn("expected open", verdict("T4:volume", call, snap(), snap(states=menu))[0])

    def test_t2_key_list_opened_spoke_and_shut(self):
        hist = [seen(["xp-dlg-help"], keysheet=[]), seen()]
        self.assertEqual(verdict("T2", ctx(hist=hist), snap(), snap()), [])
        self.assertIn("never open", verdict("T2", ctx(hist=[seen()]), snap(), snap())[0])
        wrong = [seen(["xp-dlg-help"], keysheet=["the key list does not say 'Mute'"]), seen()]
        self.assertIn("does not say", verdict("T2", ctx(hist=wrong), snap(), snap())[0])

    def test_t3_helper_worked_answered_and_focus_back(self):
        hist = [seen(helper="idle"), seen(helper="working"), seen(helper="idle")]
        entries = [{"role": "user"}, {"role": "agent"}]
        self.assertEqual(verdict("T3", ctx(hist=hist, entries=entries), snap(), snap()), [])
        never = verdict("T3", ctx(hist=[seen(), seen()], entries=entries), snap(), snap())
        self.assertIn("never worked", never[0])
        silent = verdict("T3", ctx(hist=hist, entries=[{"role": "user"}]), snap(), snap())
        self.assertIn("no agent entry", silent[0])

    def test_t4_each_part_reads_its_stand_in(self):
        for part, line in (
            ("volume", "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+"),
            ("screenshot", "grim /tmp/x.png"),
            ("lock", "loginctl lock-session"),
        ):
            with self.subTest(part):
                self.assertEqual(verdict(f"T4:{part}", ctx(calls=[line]), snap(), snap()), [])
                self.assertIn("PERSONA_CALLS", verdict(f"T4:{part}", ctx(), snap(), snap())[0])

    def test_t4_a_poll_is_no_action(self):
        polls = ["wpctl get-volume @DEFAULT_AUDIO_SINK@", "systemctl --user is-system-running"]
        self.assertIn("no call of", verdict("T4:volume", ctx(calls=polls), snap(), snap())[0])
        self.assertEqual(verdict("T4:power", ctx(calls=polls), snap(), snap()), [])

    def test_t4_power_cancelled_calls_no_systemctl(self):
        self.assertEqual(verdict("T4:power", ctx(), snap(), snap()), [])
        out = verdict("T4:power", ctx(calls=["systemctl poweroff"]), snap(), snap())
        self.assertIn("systemctl was called", out[0])

    def test_t4_power_a_log_out_is_a_power_call(self):
        out = verdict("T4:power", ctx(calls=["loginctl terminate-session 3"]), snap(), snap())
        self.assertIn("was called", out[0])
        self.assertEqual(
            verdict("T4:power", ctx(calls=["loginctl lock-session"]), snap(), snap()), []
        )

    def test_t4_clock_needs_a_clock_node(self):
        clock = snap(nodes=[surface("s1", "xp"), {"id": "xp-clock", "kind": "text"}])
        self.assertEqual(verdict("T4:clock", ctx(), snap(), clock), [])
        self.assertIn("no clock", verdict("T4:clock", ctx(), snap(), snap())[0])

    def test_t5_probe_got_no_key_of_a_super_chord(self):
        with tempfile.NamedTemporaryFile("w", suffix=".report") as report:
            report.write("key 30\nkey 57\n")
            report.flush()
            alive = snap(nodes=[probe_surface()], toplevels={"p1"})
            quiet = ctx(probe=report.name, keys=[28])
            self.assertEqual(verdict("T5", quiet, snap(), alive), [])
            stray = ctx(probe=report.name, keys=[57])
            self.assertIn("key 57", verdict("T5", stray, snap(), alive)[0])
        self.assertIn("--probe", verdict("T5", ctx(), snap(), snap())[0])

    def test_t5_chords_become_codes(self):
        actions = [
            {"action": "key", "args": {"combo": "Super+Space"}},
            {"action": "key", "args": {"combo": "Return"}},
            {"action": "type", "args": {"text": "Super+a"}},
        ]
        self.assertEqual(check.parse_chords(actions), [57])

    def test_t5_the_chords_of_the_persona_briefs_have_codes(self):
        combos = ["Super+d", "Super+q", "Super+Tab", "Super+Shift+Escape", "Super+t", "Super+c"]
        actions = [{"action": "key", "args": {"combo": c}} for c in combos]
        self.assertEqual(check.parse_chords(actions), [32, 16, 15, 1, 20, 46])

    def test_t6_a_notice_or_no_helper(self):
        say = snap(nodes=[surface("s1", "xp"), {"id": "t", "kind": "text", "text": "No helper"}])
        self.assertEqual(verdict("T6", ctx(), snap(), say), [])
        self.assertIn("neither", verdict("T6", ctx(), snap(), snap())[0])

    def test_t6_the_launcher_row_nothing_found_counts(self):
        row = ctx(calls=['fuzzel-first-row Nothing found for "nosuchprogram"'])
        self.assertEqual(verdict("T6", row, snap(), snap()), [])
        other = ctx(calls=["fuzzel-first-row Foot"])
        self.assertIn("neither", verdict("T6", other, snap(), snap())[0])

    def test_t6_an_old_notice_alone_is_red(self):
        old = snap(blocks=notify(1))
        self.assertIn("neither", verdict("T6", ctx(), old, old)[0])
        self.assertEqual(verdict("T6", ctx(), old, snap(blocks=notify(1, 2))), [])
        self.assertEqual(verdict("T6", ctx(), snap(), snap(blocks=notify(1))), [])

    def test_t7_no_window_lost(self):
        self.assertEqual(verdict("T7", ctx(), snap(), snap()), [])
        gone = snap(nodes=[], toplevels=set())
        self.assertIn("windows lost ['s1']", verdict("T7", ctx(), snap(), gone)[0])

    def test_t5_a_dead_probe_is_red(self):
        with tempfile.TemporaryDirectory() as tmp:
            report = pathlib.Path(tmp) / "probe.report"
            report.write_text("")
            alive = snap(nodes=[probe_surface()], toplevels={"p1"})
            self.assertEqual(verdict("T5", ctx(probe=str(report)), snap(), alive), [])
            out = verdict("T5", ctx(probe=str(report)), snap(), snap())
            self.assertIn("no chora-probe window", out[0])
            missing = verdict("T5", ctx(probe=str(report) + ".none"), snap(), alive)
            self.assertIn("no report", missing[0])

    def test_t5_a_key_that_reached_the_probe_is_red(self):
        with tempfile.TemporaryDirectory() as tmp:
            report = pathlib.Path(tmp) / "probe.report"
            report.write_text("key 32\n")
            alive = snap(nodes=[probe_surface()], toplevels={"p1"})
            out = verdict("T5", ctx(probe=str(report), keys=[32]), snap(), alive)
            self.assertIn("got the key 32", out[0])

    def test_x1_window_shown_again_with_the_focus(self):
        hidden = {"xp": {"windows": [window("s1", shown=False)], "focus": {"holder": "xp"}}}
        self.assertEqual(verdict("X1", ctx(), snap(), snap()), [])
        self.assertIn("hidden windows ['s1']", verdict("X1", ctx(), snap(), snap(states=hidden))[0])

    def test_m1_the_page_changed(self):
        def page(scene):
            return {"mc": {"windows": [{"surface": "s1", "scene": scene, "shown": True}]}}

        before = snap(states=page("agenda"))
        self.assertEqual(verdict("M1", ctx("mc"), before, snap(states=page("photos"))), [])
        self.assertIn("nothing changed", verdict("M1", ctx("mc"), before, before)[0])

    def test_m1_a_page_that_turned_and_came_back_counts(self):
        agenda = snap(nodes=[page_node("agenda"), page_node("morning", False, 0.0)])
        hist = [
            seen(page=["mc-s-agenda"]),
            seen(page=["mc-s-morning"]),
            seen(page=["mc-s-agenda"]),
        ]
        self.assertEqual(verdict("M1", ctx("mc", hist=hist), agenda, agenda), [])
        still = [seen(page=["mc-s-agenda"]), seen(page=["mc-s-agenda"])]
        self.assertIn("nothing changed", verdict("M1", ctx("mc", hist=still), agenda, agenda)[0])

    def test_m1_a_page_that_fades_is_not_on(self):
        agenda = snap(nodes=[page_node("agenda"), page_node("morning", True, 0.2)])
        self.assertEqual(check.page_of(agenda, "mc"), ["mc-s-agenda"])
        turned = snap(nodes=[page_node("agenda", True, 0.1), page_node("morning")])
        self.assertEqual(verdict("M1", ctx("mc"), agenda, turned), [])

    def test_c1_the_cube_answers(self):
        self.assertEqual(verdict("C1", ctx("construct"), snap(), snap(cube={"faces": []})), [])
        broken = snap(cube={"error": "no client exports cube.list"})
        self.assertIn("no client exports", verdict("C1", ctx("construct"), snap(), broken)[0])

    def test_x2_a_window_moved_and_nothing_open(self):
        moved = snap(nodes=[surface("s1", "xp", focused=True)])
        before = snap(
            states={
                "xp": {
                    "windows": [dict(window("s1"), rect=[0, 0, 5, 5])],
                    "focus": {"holder": "xp"},
                    "open": [],
                    "busy": [],
                }
            }
        )
        after = snap(
            states={
                "xp": {
                    "windows": [dict(window("s1"), rect=[9, 9, 5, 5])],
                    "focus": {"holder": "xp"},
                    "open": [],
                    "busy": [],
                }
            },
            nodes=moved.nodes,
        )
        self.assertEqual(verdict("X2", ctx(), before, after), [])
        self.assertIn("nothing changed", verdict("X2", ctx(), before, before)[0])
        menu = {
            "xp": {
                "windows": after.states["xp"]["windows"],
                "focus": {"holder": "xp"},
                "open": [{"id": "xp-menu"}],
                "busy": [],
            }
        }
        self.assertIn("expected open", verdict("X2", ctx(), before, snap(states=menu))[0])

    def test_l1_one_window_closed_focus_with_the_look(self):
        before = snap(nodes=[surface("s1", "xp"), surface("s2", "xp")], toplevels={"s1", "s2"})
        self.assertEqual(verdict("L1", ctx(), before, snap()), [])
        self.assertIn("1 windows gone", verdict("L1", ctx(), before, before)[0])
        elsewhere = snap(states={"xp": {"windows": [window("s1")], "focus": {"holder": None}}})
        self.assertIn("the focus", verdict("L1", ctx(), before, elsewhere)[0])

    def test_l2_the_look_holds_the_focus(self):
        self.assertEqual(verdict("L2", ctx(), snap(), snap()), [])
        lost = snap(states={"xp": {"windows": [window("s1")], "focus": {"holder": "mc"}}})
        self.assertIn("the focus", verdict("L2", ctx(), snap(), lost)[0])

    def test_l3_windows_shown_again_with_the_focus(self):
        self.assertEqual(verdict("L3", ctx(), snap(), snap()), [])
        hidden = {"xp": {"windows": [window("s1", shown=False)], "focus": {"holder": "xp"}}}
        self.assertIn("hidden windows ['s1']", verdict("L3", ctx(), snap(), snap(states=hidden))[0])
        lost = {"xp": {"windows": [window("s1")], "focus": {"holder": None}}}
        self.assertTrue(verdict("L3", ctx(), snap(), snap(states=lost)))

    def test_m2_nothing_open_and_mc_holds_the_focus(self):
        def mc(open_ids=(), holder="mc"):
            state = {"windows": [], "focus": {"holder": holder}, "busy": []}
            return snap(states={"mc": dict(state, open=[{"id": i} for i in open_ids])})

        self.assertEqual(verdict("M2", ctx("mc"), mc(), mc()), [])
        self.assertIn("expected open", verdict("M2", ctx("mc"), mc(), mc(["mc-menu"]))[0])
        self.assertIn("the focus", verdict("M2", ctx("mc"), mc(), mc(holder=None))[0])

    def test_c2_the_same_windows(self):
        self.assertEqual(verdict("C2", ctx("construct"), snap(), snap()), [])
        gone = snap(nodes=[], toplevels=set())
        self.assertIn("windows lost", verdict("C2", ctx("construct"), snap(), gone)[0])

    def test_c2_a_window_opened_during_the_task_is_no_breach(self):
        empty = snap(nodes=[], toplevels=set())
        opened = snap()
        self.assertEqual(verdict("C2", ctx("construct"), empty, opened), [])

    def test_c2_a_window_seen_during_the_task_and_gone_is_lost(self):
        empty = snap(nodes=[], toplevels=set())
        hist = [dict(seen(look="construct"), toplevels=["s1"])]
        out = verdict("C2", ctx("construct", hist=hist), empty, empty)
        self.assertIn("windows lost ['s1']", out[0])

    def test_end_reads_the_newest_begin_else_start(self):
        with tempfile.TemporaryDirectory() as tmp:
            run = object.__new__(check.Run)
            run.state_dir = pathlib.Path(tmp)
            run.calls = pathlib.Path(tmp) / "calls.log"
            run.calls.write_text("")
            run.save("start", snap(), {})
            self.assertEqual(check.baseline(run), "start")
            run.save("begin", snap(), {})
            self.assertEqual(check.baseline(run), "begin")
            run.state_dir.joinpath("begin.json").unlink()
            self.assertEqual(check.baseline(run), "start")

    def test_c3_the_look_holds_the_focus(self):
        self.assertEqual(verdict("C3", ctx(), snap(), snap()), [])
        lost = snap(states={"xp": {"windows": [window("s1")], "focus": {"holder": None}}})
        self.assertIn("the focus", verdict("C3", ctx(), snap(), lost)[0])

    def test_a_key_list_left_open_is_no_red_but_a_menu_is(self):
        help_open = snap(
            states={
                "xp": {
                    "windows": [],
                    "focus": {"holder": "xp"},
                    "open": [{"id": "xp-dlg-help"}],
                    "busy": [],
                }
            }
        )
        menu_open = snap(
            states={
                "xp": {
                    "windows": [],
                    "focus": {"holder": "xp"},
                    "open": [{"id": "xp-menu"}],
                    "busy": [],
                }
            }
        )
        left = ["xp-dlg-help"]
        lock = ["swaylock -f"]
        self.assertIn("expected open", verdict("T4:lock", ctx(calls=lock), snap(), help_open)[0])
        self.assertEqual(
            verdict("T4:lock", ctx(calls=lock, left=left, keys=[59]), snap(), help_open), []
        )
        self.assertIn(
            "expected open",
            verdict("T4:lock", ctx(calls=lock, left=left, keys=[59]), snap(), menu_open)[0],
        )
        hist = [seen(["xp-dlg-help"], keysheet=[])]
        self.assertEqual(verdict("T2", ctx(hist=hist, left=left), snap(), help_open), [])

    def test_the_key_list_is_left_open_only_when_it_is_open(self):
        help_open = snap(
            states={"xp": {"windows": [], "open": [{"id": "xp-dlg-help"}], "busy": []}}
        )
        menu_open = snap(states={"xp": {"windows": [], "open": [{"id": "xp-menu"}], "busy": []}})
        self.assertEqual(check.left_open("xp", help_open), ["xp-dlg-help"])
        self.assertEqual(check.left_open("xp", menu_open), [])
        self.assertEqual(check.left_open("xp", snap()), [])

    def test_no_contract_for_an_unknown_task(self):
        self.assertIsNone(check.contracts("T99", ctx(), snap()))


def shot(width=1280, height=800, back=(255, 255, 255), ink=(0, 0, 0), at=None):
    """A capture {output: (scale, rows)} of one colour, with a bar of ink
    in the rect `at` (the stand-in for a line of text)."""
    rows = [bytearray(bytes((*back, 255)) * width) for _ in range(height)]
    if at:
        x, y, w, h = at
        for r in range(y, y + h):
            for c in range(x, x + w):
                rows[r][c * 4 : c * 4 + 4] = bytes((*ink, 255))
    return {OUT: (width / 1280, [bytes(r) for r in rows])}


class GuideContrast(unittest.TestCase):
    """I6: contrast 4.5:1 for text, 7:1 for a key line and buttons."""

    def after(self, ink, label="mc-line", back=(255, 255, 255)):
        node = text_node(label, "Next step", (100, 100, 80, 20)) | {"font_size": 16}
        nodes = snap().nodes + [node]
        return snap(nodes=nodes, shots=shot(back=back, ink=ink, at=(100, 104, 70, 10)))

    def test_black_on_white_passes(self):
        self.assertEqual(breaches("I6", snap(), self.after((0, 0, 0))), [])

    def test_grey_on_white_fails(self):
        self.assertTrue(any("contrast" in b for b in breaches("I6", snap(), self.after((200, 200, 200)))))

    def test_a_key_line_wants_seven(self):
        """A grey of about 4.5:1 on white passes as text, fails as a key line."""
        mid = (118, 118, 118)
        self.assertEqual(breaches("I6", snap(), self.after(mid)), [])
        self.assertTrue(any("7" in b for b in breaches("I6", snap(), self.after(mid, "mc-key-line"))))

    def test_no_capture_is_a_finding(self):
        nodes = snap().nodes + [text_node("mc-line", "Next step", (100, 100, 80, 20))]
        self.assertTrue(any("no capture" in b for b in breaches("I6", snap(), snap(nodes=nodes))))

    def test_a_box_that_cannot_be_measured_is_a_finding(self):
        nodes = snap().nodes + [text_node("mc-line", "Next step", (100, 100, 0.5, 20))]
        after = snap(nodes=nodes, shots=shot())
        self.assertTrue(any("cannot be measured" in b for b in breaches("I6", snap(), after)))

    def test_ink_of_the_colour_of_the_back_fails(self):
        found = breaches("I6", snap(), self.after((255, 255, 255)))
        self.assertTrue(any("1.00:1" in b for b in found), found)

    def test_every_key_line_and_button_wants_seven(self):
        mid = (118, 118, 118)
        for label in (
            "tiling-keys-body-1-k",
            "construct-hints-open-keys-0",
            "construct-mode-step-in",
            "xp-session-button-3",
        ):
            with self.subTest(label=label):
                self.assertTrue(any("7" in b for b in breaches("I6", snap(), self.after(mid, label))))

    def test_a_mode_label_that_is_not_a_key_line_wants_four_and_a_half(self):
        mid = (118, 118, 118)
        self.assertEqual(breaches("I6", snap(), self.after(mid, "xp-mode-label")), [])

    def test_a_plain_label_keeps_four_and_a_half(self):
        mid = (118, 118, 118)
        self.assertEqual(breaches("I6", snap(), self.after(mid, "mc-monkey-line")), [])


class GuideSize(unittest.TestCase):
    """I6: text a person must read is 14 px or more."""

    def sized(self, px, label="mc-line"):
        node = text_node(label, "Next step", (100, 100, 80, 20)) | {"font_size": px}
        return snap(nodes=snap().nodes + [node], shots=shot(at=(100, 104, 70, 10)))

    def test_sixteen_passes(self):
        self.assertEqual(breaches("I6", snap(), self.sized(16)), [])

    def test_ten_fails(self):
        self.assertTrue(any("14 px" in b for b in breaches("I6", snap(), self.sized(10))))

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
        self.assertTrue(any("14 px" in b for b in breaches("I6", snap(), snap(states=states, nodes=nodes))))


def surf(nid, rect, owner="assistant"):
    return {
        "id": nid,
        "kind": "rect",
        "owner": owner,
        "parent": "root",
        "meta": {"surface": nid},
        "rects": {OUT: list(rect)},
    }


class GuideGaps(unittest.TestCase):
    """I2: gaps between helper surfaces and tiles are whole units."""

    def pair(self, gap, unit_owner="assistant"):
        a = surf("hud-a", (400, 100, 200, 100), unit_owner)
        b = surf("hud-b", (600 + gap, 100, 200, 100), unit_owner)
        return snap(nodes=snap().nodes + [a, b])

    def test_eight_passes(self):
        self.assertEqual(breaches("I2", snap(), self.pair(8)), [])

    def test_touching_passes(self):
        self.assertEqual(breaches("I2", snap(), self.pair(0)), [])

    def test_thirteen_fails(self):
        self.assertTrue(any("gap" in b for b in breaches("I2", snap(), self.pair(13))))

    def tiles(self, gap):
        def tile(nid, x):
            return {"id": nid, "kind": "rect", "owner": "tiling", "parent": "root", "rects": {OUT: [x, 40, 400, 300]}}

        return snap(nodes=snap().nodes + [tile("tb-1", 100), tile("tb-2", 500 + gap)])

    def test_tiles_on_the_grid_pass(self):
        self.assertEqual(breaches("I2", snap(), self.tiles(8)), [])

    def test_tiles_off_the_grid_fail(self):
        self.assertTrue(any("gap" in b for b in breaches("I2", snap(), self.tiles(7))))


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


class WithChildren(unittest.TestCase):
    """check.with_children reads the state of xp, tiling or mc only as the
    manager of a face of the cube (`cube.list`), and a window of it shows
    only while its face is at the front (co-pane r2, rv-7)."""

    FACES = [{"output": f"VIRTUAL-{n}", "manager": "xp", "health": "up"} for n in (1, 2)]

    class Session:
        label = "construct"

        def state(self, look):
            return {
                "windows": [
                    {"surface": "s1", "output": "VIRTUAL-1", "shown": True},
                    {"surface": "s2", "output": "VIRTUAL-2", "shown": True},
                ],
                "focus": {"holder": look, "surface": "s1"},
                "open": [],
                "busy": [],
            }

    def child(self, cube):
        states = {"construct": {"windows": [], "focus": {"holder": None}, "open": [], "busy": []}}
        nodes = [surface("s1", "xp", focused=True), surface("s2", "xp")]
        after = snap(states=states, nodes=nodes, toplevels={"s1", "s2"}, cube=cube)
        return check.with_children(self.Session(), after)

    def test_the_manager_of_a_face_is_read_and_only_its_front_face_shows(self):
        after = self.child({"faces": self.FACES, "front": 0, "turning": False, "mode": "use"})
        shown = {w["surface"]: w["shown"] for w in after.states["xp"]["windows"]}
        self.assertEqual(shown, {"s1": True, "s2": False})
        self.assertEqual(invariants.i3(snap(), after, Act("a")), [])

    def test_a_focus_on_a_face_that_is_not_at_the_front_is_a_breach(self):
        after = self.child({"faces": self.FACES, "front": 1, "turning": False, "mode": "use"})
        found = invariants.i3(snap(), after, Act("a"))
        self.assertEqual(found, ["the focused s1 is no shown window of xp"])

    def test_a_client_that_manages_no_face_is_no_look(self):
        for cube in (
            {"error": "no client exports cube.list"},
            {"faces": [{"output": "VIRTUAL-1", "manager": "tiling", "health": "up"}], "front": 0},
        ):
            after = self.child(cube)
            self.assertNotIn("xp", after.states)
            found = invariants.i1(snap(), after, Act("a"))
            self.assertIn("the toplevel s1 is owned by xp, which is no look", found)


if __name__ == "__main__":
    unittest.main()
