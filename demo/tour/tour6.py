"""The pace of tour v4: short pauses between acts, typing at about 12
characters a second, and cuts for waits that a viewer should not sit
through. Built on `Human`."""

import os

from human import Human
from tour import FPS

OUT = os.environ.get("TOUR_OUT", "/tmp/claude-0/tour-v6")


class Quick(Human):
    def act(self, kind, description, expected, still=None, settle=0.3, lead=0.1, tail=0.1):
        return super().act(kind, description, expected, still=still, settle=settle, lead=lead, tail=tail)

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
