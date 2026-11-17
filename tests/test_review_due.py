"""Red and green cases for review_due.py. Run: python3 -m unittest discover tests"""
import importlib.util
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent.parent / "review_due.py"
spec = importlib.util.spec_from_file_location("review_due", SCRIPT)
review_due = importlib.util.module_from_spec(spec)
spec.loader.exec_module(review_due)


class ReviewDue(unittest.TestCase):
    def test_record_is_read_from_review_state(self):
        with tempfile.TemporaryDirectory() as directory:
            record = Path(directory) / "review-state.txt"
            record.write_text("# x\nlast_reviewed: abc123\nsplit review: chora-mc 11141 2026-11-20\n", encoding="utf-8")
            last, splits = review_due.read_record(record)
        self.assertEqual(last, "abc123")
        self.assertEqual(splits, {"chora-mc": 11141})

    def test_missing_record_fails_loudly(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(SystemExit) as stop:
                review_due.read_record(Path(directory) / "review-state.txt")
        self.assertIn("review-state.txt", str(stop.exception.code))

    def test_record_without_base_fails_loudly(self):
        with tempfile.TemporaryDirectory() as directory:
            record = Path(directory) / "review-state.txt"
            record.write_text("# nothing\n", encoding="utf-8")
            with self.assertRaises(SystemExit) as stop:
                review_due.read_record(record)
        self.assertIn("last_reviewed", str(stop.exception.code))


if __name__ == "__main__":
    unittest.main()
