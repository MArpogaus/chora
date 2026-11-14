#!/usr/bin/env python3
"""PM guard report: one line per flag, or "lanes: ok". Exit 0 (a report) unless --strict and a flag.

Reads state.md (## Lanes table, ## Queue) and git. Flags:
  rule 1  a lane row names no Queue item (queue N, its lane name in a Queue line) or criterion (rc N)
  rule 1  a Queue line without an "(rc ...)" or "(process)" tag
  rule 8  a Lanes row marked reviewed or MERGE-OK whose commit is not in the base branch and whose
          review report (reports/pixel-*.md naming the commit or lane, or a time in the row) is over 12 h old
  rule 9  more than 3 commits on claude/session-log in the last hour
  rule 5  more than four lanes
The fetch of the base branch is tolerant of no network (the local ref is used).
Usage: python3 tools/lanes.py [--strict] [--root DIR] [--code DIR] [--now YYYY-MM-DDTHH:MM]
"""
import argparse
import re
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CODE = Path("/home/user/chora")
BASE_BRANCH = "claude/autonomous-dev-session-3w-i33pg7"
LOG_BRANCH = "claude/session-log"
MAX_LANES = 4
MAX_COMMITS_HOUR = 3
REVIEW_MAX_AGE = timedelta(hours=12)
TAG = re.compile(r"\((?:rc [^)]*|process)\)")
REVIEWED = re.compile(r"(?<!un)reviewed|merge-ok", re.I)
COMMIT = re.compile(r"\b(?=[0-9a-f]*\d)[0-9a-f]{7,40}\b")
TIME = re.compile(r"(?:(\d{4})-)?(\d{2})-(\d{2})[ T]+(\d{2}):(\d{2})")


def section(text, title):
    out, on = [], False
    for line in text.splitlines():
        if line.startswith("## "):
            on = line[3:].split(" (")[0].strip() == title
        elif on:
            out.append(line)
    return out


def lane_rows(text):
    rows = [[c.strip() for c in l.strip().strip("|").split("|")] for l in section(text, "Lanes") if l.lstrip().startswith("|")]
    return [r for r in rows[1:] if not all(re.fullmatch(r":?-+:?", c) for c in r)]


def queue_lines(text):
    return [l for l in section(text, "Queue") if re.match(r"\s*\d+\.\s", l)]


def names_queue_or_criterion(row, queue):
    joined = " ".join(row).lower()
    if re.search(r"\b(?:queue|q)\s*#?\d+\b|\brc\s*\d+", joined):
        return True
    name = row[0].lower()
    return bool(name) and any(name in q.lower() for q in queue)


def row_time(row, now):
    times = [
        datetime(int(y or now.year), int(mo), int(d), int(h), int(mi), tzinfo=timezone.utc)
        for y, mo, d, h, mi in TIME.findall(" ".join(row))
    ]
    return max(times) if times else None


def flags(state, now, recent_commits, in_base, report_time):
    """in_base(commit) -> bool; report_time(row, commits) -> datetime or None (pixel report time)."""
    out = []
    rows, queue = lane_rows(state), queue_lines(state)
    for row in rows:
        if not names_queue_or_criterion(row, queue):
            out.append(f"rule 1: lane '{row[0]}' names no Queue item or criterion")
    for line in queue:
        if not TAG.search(line):
            out.append(f"rule 1: Queue line without (rc ...) or (process): {line.strip()[:70]}")
    for row in rows:
        if not REVIEWED.search(" ".join(row)):
            continue
        commits = COMMIT.findall(" ".join(row))
        open_commits = [c for c in commits if not in_base(c)]
        if not open_commits:
            continue
        times = [t for t in (row_time(row, now), report_time(row, open_commits)) if t]
        if not times:
            out.append(f"rule 8: lane '{row[0]}' reviewed, {open_commits[0]} not in base, review time unknown")
        elif now - max(times) > REVIEW_MAX_AGE:
            hours = (now - max(times)).total_seconds() / 3600
            out.append(f"rule 8: lane '{row[0]}' reviewed {hours:.0f} h ago, {open_commits[0]} not in base: merge")
    last_hour = [t for t in recent_commits if now - t <= timedelta(hours=1)]
    if len(last_hour) > MAX_COMMITS_HOUR:
        out.append(f"rule 9: {len(last_hour)} commits on {LOG_BRANCH} in the last hour, limit {MAX_COMMITS_HOUR}")
    if len(rows) > MAX_LANES:
        out.append(f"rule 5: {len(rows)} lanes, limit {MAX_LANES}")
    return out


def git(root, *args, timeout=30):
    return subprocess.run(["git", "-C", str(root), *args], capture_output=True, text=True, timeout=timeout)


def recent_commit_times(root):
    ref = LOG_BRANCH if git(root, "rev-parse", "--verify", "-q", LOG_BRANCH).returncode == 0 else "HEAD"
    out = git(root, "log", "-n", "50", "--format=%ct", ref).stdout.split()
    return [datetime.fromtimestamp(int(t), timezone.utc) for t in out]


def make_in_base(code):
    try:
        git(code, "fetch", "-q", "origin", BASE_BRANCH, timeout=20)
    except (subprocess.TimeoutExpired, OSError):
        pass  # no network: the local ref is used (rule 8)
    base = f"origin/{BASE_BRANCH}"
    return lambda commit: git(code, "merge-base", "--is-ancestor", commit, base).returncode == 0


def make_report_time(root):
    def report_time(row, commits):
        keys = [c[:7] for c in commits] + [w for w in re.split(r"\W+", row[0].lower()) if len(w) > 2]
        best = None
        for path in (root / "reports").glob("pixel-*.md"):
            try:
                text = path.read_text().lower()
            except OSError:
                continue
            if any(c[:7] in text or c[:7] in path.name for c in commits) or (
                not commits and any(k in path.name.lower() for k in keys)
            ):
                moment = datetime.fromtimestamp(path.stat().st_mtime, timezone.utc)
                best = max(best, moment) if best else moment
        return best

    return report_time


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--strict", action="store_true")
    ap.add_argument("--root", default=str(ROOT))
    ap.add_argument("--code", default=str(CODE))
    ap.add_argument("--now")
    args = ap.parse_args()
    root = Path(args.root)
    now = datetime.fromisoformat(args.now).replace(tzinfo=timezone.utc) if args.now else datetime.now(timezone.utc)
    try:
        state = (root / "state.md").read_text()
    except OSError as e:
        print(f"lanes: cannot read state.md: {e}")
        return 1 if args.strict else 0
    try:
        recent = recent_commit_times(root)
    except (subprocess.TimeoutExpired, OSError, ValueError):
        recent = []
    found = flags(state, now, recent, make_in_base(Path(args.code)), make_report_time(root))
    print("\n".join(found) if found else "lanes: ok")
    return 1 if found and args.strict else 0


if __name__ == "__main__":
    sys.exit(main())
