"""Fixture cases for the Queue budget check of tools/lanes.py. Run: python3 tools/test_lanes_budget.py"""
import importlib.util
import unittest
from datetime import datetime, timezone
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent / "lanes.py"
spec = importlib.util.spec_from_file_location("lanes", SCRIPT)
lanes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lanes)
NOW = datetime(2026, 11, 21, 12, 0, tzinfo=timezone.utc)


def state(used, *percents):
    queue = "".join(f"{i}. item {p} % (process)\n" for i, p in enumerate(percents, 1))
    return f"## Now (x)\nbudget: new week 11-21 to 11-28; tokens.py 16:01 week {used} % (0.47 %/h)\n\n## Lanes\n| a |\n|---|\n\n## Queue (% week)\n{queue}\n## Needs owner\n- none\n"


def budget_flags(text):
    return [f for f in lanes.flags(text, NOW, [], lambda c: True, lambda r, c: None) if "budget" in f]


class Budget(unittest.TestCase):
    def test_under(self):
        self.assertEqual(budget_flags(state(5.6, 1.6, 0.82)), [])

    def test_exactly_at_limit(self):
        self.assertEqual(budget_flags(state(50, 45)), [])

    def test_over(self):
        found = budget_flags(state(60, 20, 20))
        self.assertEqual(len(found), 1)
        self.assertIn("40", found[0])
        self.assertIn("35", found[0])

    def test_no_week_figure(self):
        self.assertEqual(budget_flags("## Now (x)\nbudget: none\n\n## Queue\n1. a 99 % (process)\n"), [])


if __name__ == "__main__":
    unittest.main()
