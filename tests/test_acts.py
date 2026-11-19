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
    return Snapshot(**fields)


OUTPUT = {"name": "HEADLESS-1", "size": [1280, 800]}
ACT = Act("plain")


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


class I9(unittest.TestCase):
    sent = Act("notice", notices=["Hello"])

    def notify(self, shown=("Hello",), rects=None):
        node = {
            "id": "notify-card",
            "kind": "group",
            "owner": "notify",
            "parent": "root",
            "rects": {"HEADLESS-1": [900, 700, 360, 80]} if rects is None else rects,
        }
        state = {"shown": [{"id": 1, "summary": t} for t in shown], "card": "notify-card"}
        return snap(nodes=snap().nodes + [node], notify=state, outputs=[OUTPUT])

    def test_nothing_sent(self):
        self.assertEqual(breaches("I9", snap(), snap()), [])

    def test_sound(self):
        self.assertEqual(breaches("I9", snap(), self.notify(), self.sent), [])

    def test_no_server(self):
        self.assertTrue(breaches("I9", snap(), snap(), self.sent))

    def test_not_shown(self):
        self.assertTrue(breaches("I9", snap(), self.notify(shown=()), self.sent))

    def test_no_card(self):
        after = self.notify()
        after.nodes = snap().nodes
        self.assertTrue(breaches("I9", snap(), after, self.sent))

    def test_card_outside_the_output(self):
        after = self.notify(rects={"HEADLESS-1": [1200, 700, 360, 80]})
        self.assertTrue(breaches("I9", snap(), after, self.sent))

    def test_card_over_the_chat(self):
        after = self.notify()
        after.nodes.append(
            {
                "id": "hud-chat",
                "kind": "group",
                "owner": "assistant",
                "parent": "root",
                "rects": {"HEADLESS-1": [800, 600, 400, 300]},
            }
        )
        self.assertTrue(breaches("I9", snap(), after, self.sent))

    def test_dismissed_and_gone(self):
        gone = Act("dismiss", dismissed=["Hello"])
        self.assertEqual(breaches("I9", snap(), self.notify(shown=()), gone), [])

    def test_dismissed_and_still_shown(self):
        gone = Act("dismiss", dismissed=["Hello"])
        self.assertTrue(breaches("I9", snap(), self.notify(), gone))


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

    def test_i8_helper_surface_without_rects(self):
        node = {"id": "hud-chat", "kind": "group", "owner": "xp", "parent": "root"}
        found = breaches("I8", snap(), snap(nodes=snap().nodes + [node]))
        self.assertTrue(any("missing key rects" in b for b in found), found)

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
