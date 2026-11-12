"""Red and green cases for tools/check_log.py. Run: python3 -m unittest discover tests"""
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent.parent / "tools" / "check_log.py"
NOW = "2026-11-12T10:00"
GOOD_STATE = "## Now (2026-11-12 09:30 UTC)\nbudget: 87 % at 01:26 UTC, rate 1.8/h\n\n## Queue\n- a\n\n## Blocked\n- none\n"
GOOD_DECISIONS = "intro\nscreened-through: 2026-11-12T09:00:00Z\n## PM process\n- x\n"
GOOD_RULES = "1. One.\n   Guard: a\n2. Two.\n   Guard: b\n"


def run(files, added=None, head=None, now=NOW):
    with tempfile.TemporaryDirectory() as d:
        root = Path(d)
        subprocess.run(["git", "init", "-q"], cwd=root, check=True)
        if head:
            for name, text in head.items():
                (root / name).parent.mkdir(exist_ok=True)
                (root / name).write_text(text)
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            subprocess.run(
                ["git", "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "base"], cwd=root, check=True
            )
        for name, text in files.items():
            (root / name).parent.mkdir(exist_ok=True)
            (root / name).write_text(text)
        if added:
            subprocess.run(["git", "add", "-N", "."], cwd=root, check=True)
            subprocess.run(["git", "add", "."], cwd=root, check=True)
        p = subprocess.run(
            [sys.executable, str(SCRIPT), "--root", d, "--now", now], capture_output=True, text=True
        )
        return p.returncode, p.stdout


class CheckLog(unittest.TestCase):
    def red(self, files, needle, added=None, **kw):
        code, out = run(files, added, **kw)
        self.assertEqual(code, 1, out)
        self.assertIn(needle, out)

    def test_green(self):
        files = {"state.md": GOOD_STATE, "decisions-now.md": GOOD_DECISIONS, "rules.md": GOOD_RULES, "answers.md": "x 09:00 UTC\n"}
        self.assertEqual(run(files, True), (0, "\n"))

    def test_state_section(self):
        self.red({"state.md": GOOD_STATE + "\n## Done\n"}, "section not allowed")

    def test_state_long(self):
        self.red({"state.md": GOOD_STATE + "- x\n" * 80}, "limit 80")

    def test_now_future(self):
        self.red({"state.md": GOOD_STATE.replace("09:30", "10:30")}, "later than now")

    def test_now_bkk_is_converted(self):
        ok = GOOD_STATE.replace("09:30 UTC", "16:30 Bkk")
        self.assertEqual(run({"state.md": ok})[0], 0)

    def test_no_budget(self):
        self.red({"state.md": GOOD_STATE.replace("budget:", "money:")}, "budget:")

    def test_added_time_future(self):
        self.red({"state.md": GOOD_STATE + "- 11:00 UTC seen\n"}, "later than now", True)

    def test_plan_time_mid_line_ok(self):
        self.assertEqual(run({"state.md": GOOD_STATE + "- stop about 23:00 UTC, Sat 04:05\n"}, True)[0], 0)

    def test_stamp_future_dated(self):
        self.red({"answers.md": "- 11-12 10:30 UTC: x\n"}, "later than now", True)

    def test_tilde_stamp_checked(self):
        self.red({"answers.md": "- 11-12 ~10:55 UTC: x\n"}, "later than now", True)
        self.red({"answers.md": "- ~09:00 x\n"}, "without zone", True)

    def test_merge_needs_status(self):
        head = {"state.md": GOOD_STATE + "session branch: aaa\n"}
        new = {"state.md": GOOD_STATE + "session branch: bbb\n"}
        self.red(new, "nothing under status/", True, head=head)
        self.assertEqual(run({**new, "status/index.html": "x"}, True, head=head)[0], 0)

    def test_reflection_needs_answers(self):
        body = "## Ranked\n\n### 1. A point\n\ntext\n\n### 2. Another\n\nAnswer: built\n"
        self.red({"reflections/2026-11-12-x.md": body}, "point 1 has no", now="2026-11-14T00:00")
        self.assertEqual(run({"reflections/2026-11-12-x.md": body}, now="2026-11-12T20:00")[0], 0)
        listed = body + "\n## Answers\n\n1. Answer: built\n"
        self.assertEqual(run({"reflections/2026-11-12-x.md": listed}, now="2026-11-14T00:00")[0], 0)

    def test_rules_may_not_grow(self):
        head = {"rules.md": GOOD_RULES}
        self.red({"rules.md": GOOD_RULES + "3. New.\n   Guard: c\n"}, "remove a rule", head=head)
        swapped = "1. One.\n   Guard: a\n2. Swapped.\n   Guard: z\n"
        self.assertEqual(run({"rules.md": swapped}, head=head)[0], 0)

    def test_added_time_no_zone(self):
        self.red({"answers.md": "- 09:00 user said\n"}, "without zone", True)

    def test_added_time_dated_past_ok(self):
        self.assertEqual(run({"answers.md": "11-11 17:48 UTC ok\n"}, True)[0], 0)

    def test_screened_future(self):
        self.red({"decisions-now.md": GOOD_DECISIONS.replace("09:00", "10:56")}, "screened-through")

    def test_decisions_part_long(self):
        self.red({"decisions-now.md": "a" * 9001 + "\n## PM process\n"}, "before ## PM process")
        self.red({"decisions-now.md": "a\n## PM process\n" + "b" * 9001}, "from ## PM process")

    def test_rules(self):
        self.red({"rules.md": GOOD_RULES + "3. No guard here.\n"}, "without a Guard")
        many = "".join(f"{i}. r\n   Guard: g\n" for i in range(1, 22))
        self.red({"rules.md": many}, "21 numbered rules")


if __name__ == "__main__":
    unittest.main()
