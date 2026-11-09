#!/usr/bin/env python3
"""Count the feedback findings that wait for triage.

Usage: triage.py
Prints each finding of feedback/*.md whose status is still `new`, with
its age in days, and OVERDUE when one is older than a day (charter
section 8).
"""

import re
from datetime import date
from pathlib import Path

INBOX = Path(__file__).parent / "feedback"


def findings(path):
    title = None
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("## "):
            title = line[3:].strip()
        elif line.strip().startswith("status:") and title:
            yield title, line.split(":", 1)[1].strip()


def main():
    overdue = waiting = 0
    for path in sorted(INBOX.glob("20??-??-??-*.md")):
        found = re.match(r"(\d{4}-\d{2}-\d{2})", path.name)
        age = (date.today() - date.fromisoformat(found.group(1))).days
        for title, status in findings(path):
            if status == "new":
                waiting += 1
                overdue += age >= 1
                print(f"{path.name}: {title} ({age} days)")
    print(f"waiting {waiting}, overdue {overdue}")
    print("OVERDUE" if overdue else "OK")


if __name__ == "__main__":
    main()
