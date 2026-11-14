"""Red and green cases for tools/check_log.py. Run: python3 -m unittest discover tests"""
import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent.parent / "tools" / "check_log.py"
LANES = Path(__file__).resolve().parent.parent / "tools" / "lanes.py"
NOW = "2026-11-12T10:00"
GOOD_STATE = (
    "## Now (2026-11-12 09:30 UTC)\nbudget: 87 % at 01:26 UTC\n\n## RC\n- a\n\n## Lanes\n- a\n\n## Queue\n- a\n\n"
    "## Needs owner (defaults hold after 2026-11-13 12:00 UTC)\n- none\n"
)
GOOD_DECISIONS = "intro\nscreened-through: 2026-11-12T09:00:00Z\n- x\n"
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

    def test_state_section_missing(self):
        self.red({"state.md": GOOD_STATE.replace("## Lanes", "## Queue")}, "no '## Lanes' section")

    def test_state_bytes_and_line(self):
        self.red({"state.md": GOOD_STATE + "- x\n" * 800}, "state.md: 3361 bytes, limit 3000")
        self.red({"state.md": GOOD_STATE + "x" * 301 + "\n"}, "301 characters, limit 300")
        self.assertEqual(run({"state.md": GOOD_STATE + "x" * 300 + "\n"})[0], 0)

    def test_sizes_are_bytes(self):
        self.red({"vision-card.md": "\u00e4" * 1251}, "vision-card.md: 2502 bytes, limit 2500")
        self.red({"rules.md": "x" * 3001}, "rules.md: 3001 bytes, limit 3000")
        self.red({"decisions-now.md": GOOD_DECISIONS + "x" * 4000}, "limit 4000")
        self.assertEqual(run({"vision-card.md": "x" * 2500, "rules.md": "x" * 3000})[0], 0)

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
        reworded = {"state.md": GOOD_STATE + "session branch: aaa (merged)\n"}
        self.assertEqual(run(reworded, True, head=head)[0], 0)

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

    def test_screened_missing(self):
        self.red({"decisions-now.md": "intro\n- x\n"}, "no 'screened-through:' line")

    def test_rules(self):
        self.red({"rules.md": GOOD_RULES + "3. No guard here.\n"}, "without a Guard")
        many = "".join(f"{i}. r\n   Guard: g\n" for i in range(1, 12))
        self.red({"rules.md": many}, "11 numbered rules")
        self.assertEqual(run({"rules.md": "1. One. Guard: manual.\n"})[0], 0)

    def test_rules_and_vision_stamps_checked(self):
        for name in ("vision-card.md", "rules.md"):
            self.red({name: "- 11-12 10:30 UTC: x\n"}, "later than now", True)
            self.red({name: "- 09:00 x\n"}, "without zone", True)
            self.assertEqual(run({name: "- 11-12 09:00 UTC: x\n"}, True)[0], 0)

    def test_tilde_stamp_in_every_checked_file(self):
        for name in ("state.md", "decisions-now.md", "answers.md", "vision-card.md", "rules.md"):
            self.red({name: "- 2026-11-12 ~10:30 UTC: x\n"}, "later than now", True)
            self.red({name: "~09:00 x\n"}, "without zone", True)


def load_lanes():
    spec = importlib.util.spec_from_file_location("lanes", LANES)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Lanes(unittest.TestCase):
    def test_reviewers_and_judges_are_not_lanes(self):
        lanes = load_lanes()
        for meta in (
            {"description": "Review of the tooling branch"},
            {"description": "daily Judge"},
            {"description": "x", "prompt": "write the reflection of today"},
            {"description": "x", "prompt": "You are a reviewer"},
        ):
            self.assertTrue(lanes.is_reviewer(meta), meta)
        for meta in ({"description": "build the hud"}, {"description": "x", "prompt": "fix the gate"}, {}):
            self.assertFalse(lanes.is_reviewer(meta), meta)

    def test_compact_now_only_when_the_context_grew(self):
        lanes = load_lanes()
        with tempfile.TemporaryDirectory() as d:
            last = Path(d) / "last"
            limit = lanes.PM_CONTEXT_LIMIT
            self.assertEqual(lanes.compact_flag(limit - 1, last), "")
            self.assertEqual(lanes.compact_flag(limit + 1000, last), "compact now")
            self.assertEqual(lanes.compact_flag(limit + 1000, last), "")
            self.assertEqual(lanes.compact_flag(limit + 500, last), "")
            self.assertEqual(lanes.compact_flag(limit + 2000, last), "compact now")
            self.assertEqual(last.read_text().strip(), str(limit + 2000))


if __name__ == "__main__":
    unittest.main()
