#!/usr/bin/python3
"""Selftest of the act harness (tests/acts): every invariant passes on a
sound snapshot and fails on a planted breach, the known-red list blocks a
listed pass and an unlisted failure, quiet waits for every `busy` to
empty and fails at its timeout, and an `Expect` fails on a mismatch.

The cells need no session; tests/acts/planted.py plants breaches in a real
one. Usage: python3 -I tests/test_acts.py [-v]
"""

import pathlib
import signal
import subprocess
import sys
import time
import unittest
from unittest import mock

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
                | {k: node[k] for k in ("text", "corners") if k in node}
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

    def test_the_list_parses(self):
        rows = harness.known_red()
        self.assertIsInstance(rows, list)

    def listed(self, look):
        """The one row of the real list for the I5 of a look."""
        rows = [r for r in harness.known_red() if (r["look"], r["id"]) == (look, "I5")]
        self.assertEqual(len(rows), 1, rows)
        return rows

    def test_a_second_overprint_of_the_tiling_key_list_blocks(self):
        rows = self.listed("tiling")
        named = "in act 'a': on HEADLESS-1 'Run a program or a typed command' (tiling-keys-body) overlaps 'System' (tiling-keys-heading) at []"
        other = "in act 'a': on HEADLESS-1 'Quit' (tiling-keys-body) overlaps 'Other' (tiling-keys-body) at []"
        self.assertEqual(harness.judge({("tiling", "I5"): [named]}, rows).blocking, [])
        self.assertTrue(harness.judge({("tiling", "I5"): [named, other]}, rows).blocking)
        self.assertTrue(harness.judge({("tiling", "I5"): [other]}, rows).blocking)

    def test_a_second_overprint_of_the_mc_key_column_blocks(self):
        rows = self.listed("mc")
        named = "in act 'a': on HEADLESS-1 'Start a program or run a command' (mc-r1-right-keys) overlaps 'Chat with a helper' (mc-r2-label) at []"
        other = "in act 'a': on HEADLESS-1 'Quit' (mc-r1-right-keys) overlaps 'Other' (mc-r3-right-keys) at []"
        self.assertEqual(harness.judge({("mc", "I5"): [named]}, rows).blocking, [])
        self.assertTrue(harness.judge({("mc", "I5"): [named, other]}, rows).blocking)
        self.assertTrue(harness.judge({("mc", "I5"): [other]}, rows).blocking)


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
        states = with_texts(run("xp-tip", "Tooltip", (5, 3, 80, 14)))
        self.assertEqual(breaches("I6", snap(), snap(states=states, nodes=nodes)), [])

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
        out = breaches("I13", snap(), self.ring(0, 1, 2), HELD)
        self.assertTrue(any("construct-cube-face-3" in b and "missing" in b for b in out), out)

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


if __name__ == "__main__":
    unittest.main()
