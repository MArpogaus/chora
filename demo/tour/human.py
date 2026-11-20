#!/usr/bin/env python3
"""A person's pace for the tour (`Human`, and `Quick`, the pace of v4 to v9: short pauses, about 12
characters a second, `cut` for waits a viewer should not sit through): pointer paths with overshoot and a short
hesitation, typing with pauses and one corrected typo, a look at what is
new. Built on `Tour`; the random source is seeded, so a run repeats."""

import math
import random

import os

from tour import FPS, MOVE, Tour

OUT = os.environ.get("TOUR_OUT", "/tmp/claude-0/tour-v9")

SEED = 1116


def ease(k):
    return k * k * (3 - 2 * k)


class Human(Tour):
    def __init__(self, *args, **kw):
        super().__init__(*args, **kw)
        self.rng = random.Random(SEED)

    def glance(self, low=0.5, high=1.2):
        self.wait(self.rng.uniform(low, high))

    def reach(self, x, y, overshoot=True, hesitate=True):
        """Move the pointer as a hand does: fast at first, past the target,
        back onto it, and a short stop before the click."""
        dx, dy = x - self.px, y - self.py
        dist = math.hypot(dx, dy)
        if dist < 3:
            return
        seconds = min(1.6, 0.35 + 0.35 * math.log2(1 + dist / 40)) * self.rng.uniform(0.9, 1.15)
        if overshoot and dist > 120:
            push = min(18, dist * 0.04) + self.rng.uniform(2, 6)
            ox, oy = x + dx / dist * push, y + dy / dist * push
            self._curve(ox, oy, seconds * 0.82)
            self._curve(x, y, 0.28)
        else:
            self._curve(x, y, seconds)
        if hesitate:
            self.wait(self.rng.uniform(0.12, 0.4))

    def _curve(self, x, y, seconds):
        n = max(2, round(seconds * FPS * MOVE))
        x0, y0 = self.px, self.py
        dx, dy = x - x0, y - y0
        dist = math.hypot(dx, dy) or 1.0
        nx, ny = -dy / dist, dx / dist
        bend = self.rng.uniform(-0.05, 0.05) * dist
        for i in range(1, n + 1):
            k = ease(i / n)
            side = math.sin(math.pi * i / n) * bend
            self.cue_at(i, "input.inject", {"events": [self._pm(x0 + dx * k + nx * side, y0 + dy * k + ny * side)]})
        self.frame += n
        self.px, self.py = float(x), float(y)

    def press(self, x, y):
        """Reach and click."""
        self.reach(x, y)
        self.click()

    def wrong_then_right(self, wrong, right):
        """Click where a person would first look, notice, and go on."""
        self.reach(*wrong)
        self.click()
        self.wait(self.rng.uniform(0.7, 1.2))
        self.reach(*right)
        self.click()

    def type_human(self, text, cps=None, typo_at=None, typo_char="x", think=()):
        """Type with an uneven rate. `typo_at` is the index of a character
        that is typed wrong first, then deleted and typed right; `think` are
        indexes before which the person pauses."""
        base = cps or 7.0
        for i, ch in enumerate(text):
            if i in think:
                self.wait(self.rng.uniform(0.6, 1.3))
            if typo_at is not None and i == typo_at:
                self.inject([{"text": {"text": typo_char}}])
                self.wait(self.rng.uniform(0.25, 0.5))
                self.wait(self.rng.uniform(0.5, 0.9))
                self.inject([{"key": {"key": "BackSpace", "state": "pressed"}}, {"key": {"key": "BackSpace", "state": "released"}}])
                self.wait(self.rng.uniform(0.2, 0.4))
            if ch == "\n":
                self.wait(self.rng.uniform(0.15, 0.4))
                self.inject([{"key": {"key": "Return", "state": "pressed"}}, {"key": {"key": "Return", "state": "released"}}])
            else:
                self.inject([{"text": {"text": ch}}])
            gap = 1.0 / base * self.rng.uniform(0.6, 1.5)
            if ch in " ./":
                gap += self.rng.uniform(0.05, 0.2)
            self.wait(gap)

    def tap(self, name, after=0.4):
        self.key(name)
        self.wait(after)


class Quick(Human):
    def act(self, kind, description, expected, still=None, settle=0.3, lead=0.1, tail=0.1, **row):
        """The row fields of `Tour.act` (expect, contract, cells, capture, todo) go through."""
        return super().act(kind, description, expected, still=still, settle=settle, lead=lead, tail=tail, **row)

    def type_quick(self, text, typo_at=None, typo_char="x"):
        """About 12 characters a second; one typo is typed, noticed within
        0.5 s and corrected."""
        for i, ch in enumerate(text):
            if typo_at is not None and i == typo_at:
                self.inject([{"text": {"text": typo_char}}])
                self.wait(0.35)
                self.inject([{"key": {"key": "BackSpace", "state": "pressed"}}, {"key": {"key": "BackSpace", "state": "released"}}])
                self.wait(0.15)
            if ch == "\n":
                self.inject([{"key": {"key": "Return", "state": "pressed"}}, {"key": {"key": "Return", "state": "released"}}])
            else:
                self.inject([{"text": {"text": ch}}])
            self.wait(self.rng.uniform(0.05, 0.12) + (0.08 if ch in " ,.\n" else 0))

    def tap(self, name, after=0.15):
        self.key(name)
        self.wait(after)

    def reach(self, x, y, overshoot=True, hesitate=True):
        super().reach(x, y, overshoot=overshoot, hesitate=False)
        if hesitate:
            self.wait(0.1)
