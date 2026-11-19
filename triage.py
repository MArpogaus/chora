#!/usr/bin/env python3
"""Count the feedback findings that wait for triage.

Usage: triage.py [--format]
Groups by (flavor, check); `--format` exits 1 when a run file dated
2026-11-20 or later lacks `task:` or `check:`. Prints each finding of feedback/*.md whose status is still `new`, with
its age in days, and OVERDUE when one is older than a day (archive/charter.md
section 8). A run file with no finding of kind `ux` and no line
`ux: none seen, because ...` is reported as MISSING UX, from 2026-11-10 on.
"""

import re
import sys
from datetime import date, datetime, timezone
from pathlib import Path

INBOX = Path(__file__).parent / "feedback"


FORMAT_START = date(2026, 11, 20)


def findings(path):
    """Yield dicts (title, status, flavor, severity, task, check) per finding."""
    cur = None
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("## "):
            cur = {"title": line[3:].strip(), "has_task": False, "has_check": False}
            cur.update(status=None, flavor="?", severity="", task="none", check="none")
            continue
        key, sep, val = line.strip().partition(":")
        if not cur or not sep:
            continue
        val = val.strip()
        if key == "status":
            cur["status"] = val
            yield cur
        elif key in ("flavor", "severity", "task", "check"):
            cur[key] = val or "none"
            cur["has_" + key] = True


UX_RULE_START = date(2026, 11, 10)


def reports_ux(path):
    text = path.read_text(encoding="utf-8")
    return bool(
        re.search(r"^kind:\s*ux\b|^ux: none seen, because \S", text, re.MULTILINE)
    )


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    check_format = "--format" in argv
    overdue = waiting = missing_ux = bad_format = 0
    groups = {}
    for path in sorted(INBOX.glob("20??-??-??-*.md")):
        found = re.match(r"(\d{4}-\d{2}-\d{2})", path.name)
        run_date = date.fromisoformat(found.group(1))
        age = (datetime.now(tz=timezone.utc).date() - run_date).days
        if run_date >= UX_RULE_START and not reports_ux(path):
            missing_ux += 1
            print(f"{path.name}: MISSING UX")
        for f in findings(path):
            if run_date >= FORMAT_START and not (f["has_task"] and f["has_check"]):
                bad_format += 1
                print(f"{path.name}: {f['title']}: FORMAT (task: or check: missing)")
            if f["status"] == "new":
                waiting += 1
                overdue += age >= 1
                groups.setdefault((f["flavor"], f["check"]), []).append(
                    (path.name, f, age)
                )
    if check_format:
        print("FORMAT FAIL" if bad_format else "FORMAT OK")
        return 1 if bad_format else 0
    for (flavor, check), items in sorted(groups.items()):
        print(f"== {flavor} / {check} ({len(items)})")
        for name, f, age in items:
            high = f["severity"].startswith("high") and check == "none"
            mark = " HIGH, NO CHECK" if high else ""
            print(f"  {name}: {f['title']} ({age} days){mark}")
    print(f"waiting {waiting}, overdue {overdue}, missing ux {missing_ux}")
    print("OVERDUE" if overdue else "MISSING UX" if missing_ux else "OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
