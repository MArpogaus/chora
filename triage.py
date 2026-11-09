#!/usr/bin/env python3
"""Count the feedback findings that wait for triage.

Usage: triage.py
Prints each finding of feedback/*.md whose status is still `new`, with
its age in days, and OVERDUE when one is older than a day (charter
section 8). A run file with no finding of kind `ux` and no line
`ux: none seen, because ...` is reported as MISSING UX, from 2026-11-10 on.
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


UX_RULE_START = date(2026, 11, 10)


def reports_ux(path):
    text = path.read_text(encoding="utf-8")
    return bool(re.search(r"^kind:\s*ux\b|^ux: none seen, because \S", text, re.M))


def main():
    overdue = waiting = missing_ux = 0
    for path in sorted(INBOX.glob("20??-??-??-*.md")):
        found = re.match(r"(\d{4}-\d{2}-\d{2})", path.name)
        run_date = date.fromisoformat(found.group(1))
        age = (date.today() - run_date).days
        if run_date >= UX_RULE_START and not reports_ux(path):
            missing_ux += 1
            print(f"{path.name}: MISSING UX")
        for title, status in findings(path):
            if status == "new":
                waiting += 1
                overdue += age >= 1
                print(f"{path.name}: {title} ({age} days)")
    print(f"waiting {waiting}, overdue {overdue}, missing ux {missing_ux}")
    print("OVERDUE" if overdue else "MISSING UX" if missing_ux else "OK")


if __name__ == "__main__":
    main()
