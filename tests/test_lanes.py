"""Red and green cases for tools/lanes.py. Run: python3 -m unittest discover tests"""
import importlib.util
import subprocess
import sys
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent.parent / "tools" / "lanes.py"
spec = importlib.util.spec_from_file_location("lanes", SCRIPT)
lanes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lanes)

NOW = datetime(2026, 11, 14, 12, 0, tzinfo=timezone.utc)
HEAD = "## Lanes\n| Lane | Where | Branch and commit | State |\n|---|---|---|---|\n"
QUEUE = "\n## Queue\n1. Train 4 lands. (rc 1, 3)\n2. Notes prune. (process)\n"


def state(rows, queue=QUEUE):
    return "## Now (x)\nbudget: x\n\n" + HEAD + rows + queue + "\n## Needs owner\n- none\n"


def run(text, recent=(), merged=lambda c: True, report=lambda r, c: None):
    return lanes.flags(text, NOW, list(recent), merged, report)


class Lanes(unittest.TestCase):
    def test_ok(self):
        self.assertEqual(run(state("| train 4 | wt | mc 10eb3a7f | running |\n")), [])

    def test_lane_names_criterion_or_queue_number(self):
        self.assertEqual(run(state("| xyz | wt | b | running, rc 3 |\n")), [])
        self.assertEqual(run(state("| xyz | wt | b | queue 2 |\n")), [])

    def test_lane_without_queue_item(self):
        out = run(state("| mystery | wt | b | running |\n"))
        self.assertEqual(len(out), 1)
        self.assertIn("rule 1", out[0])
        self.assertIn("mystery", out[0])

    def test_untagged_queue_line(self):
        out = run(state("| train 4 | wt | b | x |\n", QUEUE + "3. No tag here.\n4. Wrong tag (rcs). \n"))
        self.assertEqual(len(out), 2)
        self.assertIn("No tag here", out[0])

    def test_reviewed_unmerged_old_is_flagged(self):
        row = "| train 4 | wt | mc 10eb3a7f | reviewed MERGE-OK |\n"
        old = lambda r, c: NOW - timedelta(hours=13)
        out = run(state(row), merged=lambda c: False, report=old)
        self.assertEqual(len(out), 1)
        self.assertIn("rule 8", out[0])
        self.assertIn("10eb3a7f", out[0])

    def test_reviewed_young_merged_or_unreviewed_not_flagged(self):
        row = "| train 4 | wt | mc 10eb3a7f | reviewed MERGE-OK |\n"
        young = lambda r, c: NOW - timedelta(hours=11)
        self.assertEqual(run(state(row), merged=lambda c: False, report=young), [])
        self.assertEqual(run(state(row), merged=lambda c: True, report=lambda r, c: NOW - timedelta(hours=40)), [])
        plain = "| train 4 | wt | mc 10eb3a7f | running |\n"
        self.assertEqual(run(state(plain), merged=lambda c: False, report=lambda r, c: NOW - timedelta(hours=40)), [])

    def test_row_time_counts_as_review_time(self):
        row = "| train 4 | wt | mc 10eb3a7f | reviewed 11-13 10:00 UTC |\n"
        out = run(state(row), merged=lambda c: False)
        self.assertIn("26 h ago", out[0])
        row = "| train 4 | wt | mc 10eb3a7f | reviewed 11-14 08:00 UTC |\n"
        self.assertEqual(run(state(row), merged=lambda c: False), [])

    def test_unreviewed_is_not_reviewed(self):
        row = "| train 4 | wt | r5 d102940f unreviewed, not taken | running |\n"
        self.assertEqual(run(state(row), merged=lambda c: False), [])

    def test_unknown_review_time(self):
        row = "| train 4 | wt | mc 10eb3a7f | MERGE-OK |\n"
        self.assertIn("unknown", run(state(row), merged=lambda c: False)[0])

    def test_commits_per_hour(self):
        four = [NOW - timedelta(minutes=m) for m in (1, 10, 20, 59)]
        out = run(state("| train 4 | w | b | x |\n"), recent=four)
        self.assertEqual(len(out), 1)
        self.assertIn("rule 9", out[0])
        three = four[:3] + [NOW - timedelta(minutes=61)]
        self.assertEqual(run(state("| train 4 | w | b | x |\n"), recent=three), [])

    def test_more_than_four_lanes(self):
        rows = "".join(f"| train 4 | w{i} | b | x |\n" for i in range(5))
        out = run(state(rows))
        self.assertEqual(out, ["rule 5: 5 lanes, limit 4"])
        self.assertEqual(run(state(rows[: rows.rindex("| train 4 | w4")])), [])

    def test_report_time_finds_pixel_report(self):
        import os
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "reports").mkdir()
            report = Path(d) / "reports" / "pixel-mc.md"
            report.write_text("verdict MERGE-OK for 10eb3a7f\n")
            self.assertIsNotNone(lanes.make_report_time(Path(d))(["train 4"], ["10eb3a7f"]))
            self.assertIsNone(lanes.make_report_time(Path(d))(["train 4"], ["deadbeef"]))

    def test_cli_is_a_report(self):
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            subprocess.run(["git", "init", "-q"], cwd=d, check=True)
            (Path(d) / "state.md").write_text(state("| mystery | w | b | x |\n"))
            args = [sys.executable, str(SCRIPT), "--root", d, "--code", d, "--now", "2026-11-14T12:00"]
            p = subprocess.run(args, capture_output=True, text=True)
            self.assertEqual(p.returncode, 0)
            self.assertIn("mystery", p.stdout)
            self.assertEqual(subprocess.run(args + ["--strict"], capture_output=True, text=True).returncode, 1)
            (Path(d) / "state.md").write_text(state("| train 4 | w | b | x |\n"))
            p = subprocess.run(args + ["--strict"], capture_output=True, text=True)
            self.assertEqual((p.returncode, p.stdout), (0, "lanes: ok\n"))


if __name__ == "__main__":
    unittest.main()
