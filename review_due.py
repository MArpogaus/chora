#!/usr/bin/env python3
"""Say whether a review round is due on the session branch.

Usage: review_due.py [REPO] [LAST_REVIEWED]
  REPO defaults to /home/user/chora, LAST_REVIEWED to the value in
  state.md. Prints the weighted size of the unreviewed range, its
  critical paths and DUE or NOT DUE with the reason (charter section 3).
  Then the size of each plugin: SPLIT REVIEW DUE when it has
  SPLIT_REVIEW_AT_LINES or more and state.md records no split review at
  a size within SPLIT_REVIEW_REGROWTH of today (charter section 1, item
  18). A recorded review is a line "split review: NAME LINES DATE".
"""

import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

CRITICAL = (
    "crates/chora/src/",
    "crates/chora-scene/",
    "crates/choractl/src/mcp",
    "services/voice/src/config",
    "services/voice/src/chat",
    "services/voice/src/tools",
    "services/voice/src/agents",
    "dist/",
    ".github/",
    "Cargo.lock",
)
LIGHT = ("tests/", "docs/", "examples/", "flavors/")
WEIGHT_CRITICAL, WEIGHT_CODE, WEIGHT_LIGHT = 3.0, 1.0, 0.2
DUE_AT_WEIGHTED_LINES = 1500
DUE_AFTER_DAYS_WITH_CRITICAL_CHANGES = 3
SPLIT_REVIEW_AT_LINES = 5000
SPLIT_REVIEW_REGROWTH = 1.5
UNITS = ("plugins/*", "services/*")


def git(repo, *args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True, check=True).stdout


def weight_of(path):
    if path.startswith(CRITICAL):
        return WEIGHT_CRITICAL
    if path.startswith(LIGHT) or path.endswith(".md"):
        return WEIGHT_LIGHT
    return WEIGHT_CODE


def lines_of_rust(directory):
    return sum(len(path.read_text(encoding="utf-8", errors="replace").splitlines()) for path in directory.rglob("*.rs"))


def split_reviews(state_text):
    reviewed = {}
    for name, lines in re.findall(r"^split review: (\S+) (\d+)", state_text, re.M):
        reviewed[name] = max(reviewed.get(name, 0), int(lines))
    return reviewed


def report_split_reviews(repo, state_text):
    reviewed = split_reviews(state_text)
    for pattern in UNITS:
        for unit in sorted(Path(repo).glob(pattern)):
            if not (unit / "src").is_dir():
                continue
            size = lines_of_rust(unit / "src")
            last = reviewed.get(unit.name)
            if size < SPLIT_REVIEW_AT_LINES:
                verdict = "below threshold"
            elif last and size < last * SPLIT_REVIEW_REGROWTH:
                verdict = f"reviewed at {last} lines"
            else:
                verdict = "SPLIT REVIEW DUE"
            print(f"  {unit.name}: {size} lines, {verdict}")


def main():
    repo = sys.argv[1] if len(sys.argv) > 1 else "/home/user/chora"
    state = (Path(__file__).parent / "state.md").read_text(encoding="utf-8")
    if len(sys.argv) > 2:
        last_reviewed = sys.argv[2]
    else:
        last_reviewed = re.search(r"^last_reviewed: (\w+)", state, re.M).group(1)
    weighted, critical_paths = 0.0, set()
    for line in git(repo, "diff", "--numstat", f"{last_reviewed}..HEAD").splitlines():
        added, deleted, path = line.split("\t", 2)
        changed = 0 if added == "-" else int(added) + int(deleted)
        weighted += changed * weight_of(path)
        if path.startswith(CRITICAL):
            critical_paths.add(path)
    oldest = git(repo, "log", "--reverse", "--format=%ct", f"{last_reviewed}..HEAD").split()
    age_days = (datetime.now(timezone.utc).timestamp() - int(oldest[0])) / 86400 if oldest else 0
    print(f"range {last_reviewed}..HEAD: weighted lines {weighted:.0f}, critical files {len(critical_paths)}, oldest unreviewed commit {age_days:.1f} days")
    for path in sorted(critical_paths):
        print(f"  critical: {path}")
    if weighted >= DUE_AT_WEIGHTED_LINES:
        print(f"DUE: weighted lines >= {DUE_AT_WEIGHTED_LINES}")
    elif critical_paths and age_days >= DUE_AFTER_DAYS_WITH_CRITICAL_CHANGES:
        print(f"DUE: critical changes unreviewed for {DUE_AFTER_DAYS_WITH_CRITICAL_CHANGES} days")
    else:
        print("NOT DUE (a merge, a tag or the end of the week still needs a round)")
    print("plugin sizes:")
    report_split_reviews(repo, state)


if __name__ == "__main__":
    main()
