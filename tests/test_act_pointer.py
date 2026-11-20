#!/usr/bin/python3 -I
"""Selftest of the pointer actions of tests/persona/act.py, without a session.

A stand-in choractl in the PATH of a fake session file writes the events of
each `inject` to a file. A real hand moves the pointer many times before it
presses: a program that reads the first motion as the start of the pointer
(fuzzel 1.11.1, wayland.c wl_pointer_motion) selects the row under the pointer
only from the second motion on, and a pointer_move that enters its surface is
an enter, not a motion. So a click or the press of a drag has to come after
at least three moves that all end on the target and stay within a few pixels
of it, so that the target is under the pointer at the second motion and no
neighbour is crossed.

  click, drag   at least MIN_MOVES moves before the first press, the last one
                on the target, all within NEAR pixels of it
  move          one move on the target
  scroll        no move: it acts where the pointer is, and takes no position

Usage: python3 -I tests/test_act_pointer.py
"""

import json
import os
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
ACT = os.path.join(HERE, "persona", "act.py")
MIN_MOVES = 3
NEAR = 8


class ActPointer(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.mkdtemp(prefix="act-pointer.")
        stub = os.path.join(self.dir, "bin")
        os.mkdir(stub)
        self.sent = os.path.join(self.dir, "sent.jsonl")
        script = os.path.join(stub, "choractl")
        with open(script, "w", encoding="utf-8") as f:
            f.write(f'#!/bin/sh\ncat >>"{self.sent}"\necho >>"{self.sent}"\n')
        os.chmod(script, 0o755)
        self.env_file = os.path.join(self.dir, "session.env")
        with open(self.env_file, "w", encoding="utf-8") as f:
            for key, value in {
                "PATH": f"{stub}:/usr/bin:/bin",
                "CHORA_SOCKET": os.path.join(self.dir, "sock"),
                "XDG_RUNTIME_DIR": self.dir,
                "PERSONA_DIR": self.dir,
                "PERSONA_LOG": os.path.join(self.dir, "actions.jsonl"),
                "PERSONA_SHOTS": self.dir,
            }.items():
                f.write(f"{key}={value}\n")

    def act(self, *args):
        env = {k: v for k, v in os.environ.items() if k != "PERSONA_CHECK"}
        env["PERSONA_ENV"] = self.env_file
        p = subprocess.run(
            [sys.executable, "-I", ACT, *args],
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(p.returncode, 0, p.stderr)
        with open(self.sent, encoding="utf-8") as f:
            events = [e for line in f.read().splitlines() if line for e in json.loads(line)]
        os.remove(self.sent)
        return events

    @staticmethod
    def kind(event):
        return next(iter(event))

    def moves_before_press(self, events, press):
        first = next(i for i, e in enumerate(events) if self.kind(e) == press)
        before = events[:first]
        self.assertTrue(
            all(self.kind(e) == "pointer_move" for e in before),
            f"only moves before the first {press}: {events}",
        )
        return [e["pointer_move"]["position"] for e in before]

    def assert_hand(self, positions, target):
        self.assertGreaterEqual(
            len(positions),
            MIN_MOVES,
            f"{len(positions)} move(s) before the press, a hand makes at least {MIN_MOVES}: {positions}",
        )
        self.assertEqual(positions[-1], list(target), "the last move is on the target")
        for x, y in positions:
            self.assertLessEqual(abs(x - target[0]), NEAR, f"{positions}: a move leaves the target")
            self.assertLessEqual(abs(y - target[1]), NEAR, f"{positions}: a move leaves the target")
        self.assertGreater(len({tuple(p) for p in positions}), 1, "the moves differ, else they are no motion")

    def test_click_moves_as_a_hand_then_presses(self):
        events = self.act("click", "400", "300")
        self.assert_hand(self.moves_before_press(events, "click"), (400, 300))
        self.assertEqual([self.kind(e) for e in events].count("click"), 1)

    def test_double_click_moves_once_then_clicks_twice(self):
        events = self.act("click", "400", "300", "--double")
        self.assert_hand(self.moves_before_press(events, "click"), (400, 300))
        self.assertEqual([self.kind(e) for e in events].count("click"), 2)

    def test_right_click_keeps_its_button(self):
        events = self.act("click", "400", "300", "--button", "right")
        self.assertEqual([e["click"]["button"] for e in events if self.kind(e) == "click"], [273])

    def test_drag_moves_as_a_hand_before_the_press(self):
        events = self.act("drag", "100", "120", "500", "420")
        self.assert_hand(self.moves_before_press(events, "button"), (100, 120))
        kinds = [self.kind(e) for e in events]
        self.assertEqual(events[kinds.index("button")]["button"]["state"], "pressed")
        self.assertEqual(events[-1]["button"]["state"], "released")
        self.assertEqual(events[-2]["pointer_move"]["position"], [500, 420])

    def test_move_is_one_move_on_the_target(self):
        events = self.act("move", "250", "260")
        self.assertEqual(events, [{"pointer_move": {"position": [250.0, 260.0]}}])

    def test_scroll_takes_no_position_and_sends_no_move(self):
        events = self.act("scroll", "0", "-3")
        self.assertEqual([self.kind(e) for e in events], ["scroll"])


if __name__ == "__main__":
    unittest.main()
