"""Selftest of the wait of tests/persona/act.py on the checker, without a
session: a slow check.py (a look that is busy until the hint chip is gone,
9 s) must not hold the persona (r1: act.py returned from an unbound key 9.7 s
later, so a look after it missed the 8 s chip), every check still runs, and
an input waits for the checks of the actions before it.

Usage: python3 -I tests/persona/test_act_wait.py
"""

import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import time
import unittest

HERE = pathlib.Path(__file__).resolve().parent
SLOW = 4
FAST = 3

FAKE_CHECK = f"""
import pathlib, sys, time
d = pathlib.Path(sys.argv[0]).parent
time.sleep({SLOW})
with open(d / "checked", "a") as f:
    f.write(sys.argv[2] + "\\n")
"""


class ActWait(unittest.TestCase):
    def setUp(self):
        self.dir = pathlib.Path(tempfile.mkdtemp(prefix="act-wait-"))
        self.addCleanup(shutil.rmtree, self.dir, True)
        shutil.copy(HERE / "act.py", self.dir / "act.py")
        (self.dir / "check.py").write_text(FAKE_CHECK)
        bin_dir = self.dir / "bin"
        bin_dir.mkdir()
        choractl = bin_dir / "choractl"
        choractl.write_text('#!/bin/sh\necho \'{"scale":1,"origin":[0,0],"width":1,"height":1}\'\n')
        choractl.chmod(0o755)
        (self.dir / "shots").mkdir()
        env = self.dir / "session.env"
        env.write_text(
            f"PATH={bin_dir}:/usr/bin:/bin\nCHORA_SOCKET=x\nXDG_RUNTIME_DIR={self.dir}\n"
            f"PERSONA_DIR={self.dir}\nPERSONA_SHOTS={self.dir}/shots\n"
            f"PERSONA_LOG={self.dir}/log\nPERSONA_CHECK=1\n"
        )
        self.env = env

    def act(self, *args):
        start = time.monotonic()
        subprocess.run(
            [sys.executable, "-I", str(self.dir / "act.py"), *args],
            env={"PERSONA_ENV": str(self.env), "PATH": "/usr/bin:/bin"},
            check=True,
            capture_output=True,
        )
        return time.monotonic() - start

    def checked(self):
        path = self.dir / "checked"
        return path.read_text().split() if path.exists() else []

    def test_key_returns_before_the_check_ends(self):
        took = self.act("key", "Super+q")
        self.assertLess(took, FAST, f"act.py held the persona {took:.1f} s for the checker")

    def test_a_look_returns_at_once_after_a_key(self):
        self.act("key", "Super+q")
        took = self.act("look")
        self.assertLess(took, FAST, f"a look after a key waited {took:.1f} s")

    def test_every_check_still_runs(self):
        self.act("key", "Super+q")
        self.act("look")
        deadline = time.monotonic() + 3 * SLOW + 5
        while len(self.checked()) < 2 and time.monotonic() < deadline:
            time.sleep(0.2)
        self.assertEqual(self.checked(), ["key", "look"])

    def test_an_input_waits_for_the_checks_before_it(self):
        self.act("key", "Super+q")
        self.act("key", "Super+w")
        self.assertEqual(self.checked(), ["key"], "the second input ran before the first check ended")


if __name__ == "__main__":
    os.environ.pop("PERSONA_CHECK", None)
    unittest.main()
