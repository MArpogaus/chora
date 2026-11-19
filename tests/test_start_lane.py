"""Red and green cases for tools/start_lane.sh. Run: python3 -m unittest discover tests"""
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent.parent / "tools" / "start_lane.sh"
STATE = "## Lanes\n| Lane | Where | Brief | State |\n|---|---|---|---|\n| tiling | wt | briefs/tiling-r1.md | running |\n\n## Queue\n1. x. (process)\n"


def git(d, *a):
    subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=d, check=True, capture_output=True)


class StartLane(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = Path(self.tmp.name)
        git(self.d, "init", "-q")
        (self.d / "briefs").mkdir()
        (self.d / "state.md").write_text(STATE)
        (self.d / "briefs" / "tiling-r1.md").write_text("brief\nModel: sonnet, effort medium\npoll it in the same turn\nStep 0: cargo check -p chora\n")
        (self.d / "briefs" / "xp-r1.md").write_text("brief\nModel: sonnet, effort medium\npoll it in the same turn\nStep 0: cargo check -p chora\n")
        git(self.d, "add", "state.md", "briefs/tiling-r1.md")
        git(self.d, "commit", "-q", "-m", "x")

    def tearDown(self):
        self.tmp.cleanup()

    def run_it(self, arg):
        p = subprocess.run(
            ["bash", str(SCRIPT), arg], capture_output=True, text=True, env={**os.environ, "LOG_ROOT": str(self.d)}
        )
        return p.returncode, p.stdout, p.stderr

    def test_green_prints_send_later(self):
        code, out, _ = self.run_it("briefs/tiling-r1.md")
        self.assertEqual(code, 0)
        self.assertIn("send_later delay_minutes=6", out)
        self.assertIn("tiling-r1.md", out)

    def test_missing_file(self):
        self.assertIn("does not exist", self.run_it("briefs/none-r1.md")[2])

    def test_outside_briefs(self):
        code, _, err = self.run_it("state.md")
        self.assertEqual(code, 1)
        self.assertIn("not a briefs/", err)

    def test_not_committed(self):
        code, _, err = self.run_it("briefs/xp-r1.md")
        self.assertEqual(code, 1)
        self.assertIn("not committed", err)

    def test_modified_after_commit(self):
        (self.d / "briefs" / "tiling-r1.md").write_text("changed\n")
        self.assertIn("uncommitted changes", self.run_it("briefs/tiling-r1.md")[2])

    def test_no_lane_row(self):
        git(self.d, "add", "briefs/xp-r1.md")
        git(self.d, "commit", "-q", "-m", "y")
        code, _, err = self.run_it("briefs/xp-r1.md")
        self.assertEqual(code, 1)
        self.assertIn("no row", err)

    def commit_brief(self, text):
        (self.d / "briefs" / "tiling-r1.md").write_text(text)
        git(self.d, "commit", "-q", "-am", "z")
        return self.run_it("briefs/tiling-r1.md")

    def test_no_same_turn_line(self):
        code, _, err = self.commit_brief("brief\nModel: sonnet, effort medium\n")
        self.assertEqual(code, 1)
        self.assertIn("same turn", err)

    def test_opus_refused_when_budget_says_no(self):
        (self.d / "state.md").write_text("budget: 70 %, 9 % over. No new opus today.\n" + STATE)
        code, _, err = self.commit_brief("b\nModel: opus, effort high because a hard root cause\nsame turn\nStep 0: cargo check -p chora\n")
        self.assertEqual(code, 1)
        self.assertIn("no new opus", err)

    def test_opus_allowed_on_pace(self):
        (self.d / "state.md").write_text("budget: 60 %, on pace.\n" + STATE)
        self.assertEqual(self.commit_brief("b\nModel: opus, effort high because a hard root cause\nsame turn\nStep 0: cargo check -p chora\n")[0], 0)

    def test_no_setup_step(self):
        code, _, err = self.commit_brief("brief\nModel: sonnet, effort medium\nsame turn\n")
        self.assertNotEqual(code, 0)
        self.assertIn("Step 0", err)

    def test_only_without_ci_check(self):
        code, _, err = self.commit_brief("brief\nModel: sonnet, effort medium\nsame turn\nStep 0: cargo check -p chora\n"
                                         "`tests/merge-gate.sh --only build,acts` green\n")
        self.assertNotEqual(code, 0)
        self.assertIn("ci-check", err)

    def test_only_with_ci_check(self):
        self.assertEqual(self.commit_brief("brief\nModel: sonnet, effort medium\nsame turn\nStep 0: cargo check -p chora\n"
                                           "`tests/merge-gate.sh --only build,ci-check,acts` green\n")[0], 0)

    def test_no_argument(self):
        self.assertEqual(self.run_it("")[0], 1)


if __name__ == "__main__":
    unittest.main()
