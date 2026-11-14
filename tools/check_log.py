#!/usr/bin/env python3
"""Pre-commit check of the session log: the four injected notes (sizes in bytes) and entry times.

Sizes: vision-card.md 2,500; rules.md 3,000; state.md 3,000 and no line over 300 characters;
decisions-now.md 4,000. state.md has only the sections Now, RC, Lanes, Queue, Needs owner.

Run from the log worktree (or pass --root). Added lines come from the staged
diff; only the entry stamp that starts a line is checked (later times in a line may be plans). The clock is the real UTC time; --now YYYY-MM-DDTHH:MM overrides it.
Exit 1 and one line per problem when a rule is broken.
"""
import argparse
import re
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

SECTIONS = ("Now", "RC", "Lanes", "Queue", "Needs owner")
MAX_BYTES = {"vision-card.md": 2500, "rules.md": 3000, "state.md": 3000, "decisions-now.md": 4000}
STATE_MAX_LINE = 300
RULES_MAX = 10
STAMPED = ("state.md", "decisions-now.md", "answers.md", "rules.md", "vision-card.md")
TIME = re.compile(
    r"(?<![\d:])(?:(\d{4})-)?(?:(\d{2})-(\d{2})[ T]+~?)?(\d{2}):(\d{2})(?::\d{2})?(?![\d:])\s*(Z\b|UTC\b|Bkk\b)?"
)
NOW_HEADER = re.compile(r"^## Now \((\d{4}-\d{2}-\d{2}) (\d{2}):(\d{2}) (UTC|Bkk)\)")


def parse_time(year, month, day, hour, minute, zone, now):
    base = now.date()
    date = datetime(int(year or base.year), int(month or base.month), int(day or base.day), tzinfo=timezone.utc)
    moment = date + timedelta(hours=int(hour), minutes=int(minute))
    return moment - timedelta(hours=7) if zone == "Bkk" else moment


def line_times(line, now):
    for m in TIME.finditer(line):
        year, month, day, hour, minute, zone = m.groups()
        if int(hour) > 23 or int(minute) > 59:
            continue
        if month and not 1 <= int(month) <= 12:
            continue
        yield m.group(0).strip(), zone, parse_time(year, month, day, hour, minute, zone, now)


STAMP = re.compile(r"^\s*(?:[-*]\s*)?(?:\*\*)?(?:\d{4}-)?(?:\d{2}-\d{2}[ T]+)?~?\d{2}:\d{2}")


def check_added_line(name, line, now):
    if not STAMP.match(line):
        return
    for text, zone, moment in list(line_times(line, now))[:1]:
        if zone is None:
            yield f"{name}: time without zone (UTC or Bkk): {text!r} in: {line.strip()[:80]}"
        elif moment > now:
            yield f"{name}: time later than now ({now:%Y-%m-%d %H:%M} UTC): {text!r} in: {line.strip()[:80]}"


def section_lines(text, title):
    out, on = [], False
    for line in text.splitlines():
        if line.startswith("## "):
            on = line[3:].split(" (")[0].strip() == title
            continue
        if on:
            out.append(line)
    return out


def check_state(text, now):
    lines = text.splitlines()
    for number, l in enumerate(lines, 1):
        if len(l) > STATE_MAX_LINE:
            yield f"state.md: line {number} has {len(l)} characters, limit {STATE_MAX_LINE}"
    tops = [l for l in lines if l.startswith("## ")]
    titles = [l[3:].split(" (")[0].strip() for l in tops]
    for l, title in zip(tops, titles):
        if title not in SECTIONS:
            yield f"state.md: section not allowed: {l.strip()}"
    for title in SECTIONS:
        if title not in titles:
            yield f"state.md: no '## {title}' section"
    header = next((l for l in tops if l.startswith("## Now")), None)
    if header is None:
        yield "state.md: no '## Now' section"
        return
    m = NOW_HEADER.match(header)
    if not m:
        yield "state.md: Now header must read '## Now (YYYY-MM-DD HH:MM UTC)'"
    else:
        date, hour, minute, zone = m.groups()
        y, mo, d = date.split("-")
        if parse_time(y, mo, d, hour, minute, zone, now) > now:
            yield f"state.md: Now time {date} {hour}:{minute} {zone} is later than now"
    if not any(l.startswith("budget:") for l in section_lines(text, "Now")):
        yield "state.md: '## Now' has no line starting 'budget:'"


def check_decisions(text, now):
    m = re.search(r"^screened-through:\s*(\S+)", text, re.M)
    if not m:
        yield "decisions-now.md: no 'screened-through:' line"
    else:
        try:
            moment = datetime.fromisoformat(m.group(1).replace("Z", "+00:00"))
        except ValueError:
            yield f"decisions-now.md: screened-through not an ISO time: {m.group(1)}"
        else:
            if moment.tzinfo is None:
                moment = moment.replace(tzinfo=timezone.utc)
            if moment > now:
                yield f"decisions-now.md: screened-through {m.group(1)} is later than now"


def check_rules(text):
    rules, current = [], None
    for line in text.splitlines():
        if re.match(r"^\d+\.\s", line):
            current = [line, "Guard:" in line]
            rules.append(current)
        elif line.startswith("#"):
            current = None
        elif current and "Guard:" in line:
            current[1] = True
    if len(rules) > RULES_MAX:
        yield f"rules.md: {len(rules)} numbered rules, limit {RULES_MAX}"
    for first, guarded in rules:
        if not guarded:
            yield f"rules.md: rule without a Guard: line: {first.strip()[:60]}"


def count_rules(text):
    return sum(1 for l in text.splitlines() if re.match(r"^\d+\.\s", l))


def check_rules_growth(root, text):
    old = subprocess.run(["git", "show", "HEAD:rules.md"], cwd=root, capture_output=True, text=True)
    if old.returncode == 0 and count_rules(text) > count_rules(old.stdout):
        yield f"rules.md: {count_rules(text)} rules, HEAD has {count_rules(old.stdout)}: remove a rule to add one"


def session_commit(line):
    return (line.split(":", 1)[1].split() or [""])[0]


def check_merge_has_status(root):
    staged = subprocess.run(
        ["git", "diff", "--cached", "--name-only"], cwd=root, capture_output=True, text=True
    ).stdout.split()
    old = subprocess.run(["git", "show", "HEAD:state.md"], cwd=root, capture_output=True, text=True).stdout
    new = [session_commit(l) for l in added_lines(root, "state.md") if l.startswith("session branch:")]
    if new and new[0] != next((session_commit(l) for l in old.splitlines() if l.startswith("session branch:")), new[0]):
        if not any(n.startswith("status/") for n in staged):
            yield "state.md: the session branch line changed (a merge) but nothing under status/ changed"


def added_lines(root, name):
    out = subprocess.run(
        ["git", "diff", "--cached", "-U0", "--", name], cwd=root, capture_output=True, text=True
    ).stdout
    return [l[1:] for l in out.splitlines() if l.startswith("+") and not l.startswith("+++")]


def run(root, now):
    root = Path(root)
    problems = []
    state, decisions, rules = (root / n for n in ("state.md", "decisions-now.md", "rules.md"))
    for name, limit in MAX_BYTES.items():
        if (root / name).exists() and (root / name).stat().st_size > limit:
            problems.append(f"{name}: {(root / name).stat().st_size} bytes, limit {limit}")
    if state.exists():
        problems += check_state(state.read_text(), now)
    if decisions.exists():
        problems += check_decisions(decisions.read_text(), now)
    if rules.exists():
        problems += check_rules(rules.read_text())
        problems += check_rules_growth(root, rules.read_text())
    problems += check_merge_has_status(root)
    for name in STAMPED:
        for line in added_lines(root, name):
            problems += check_added_line(name, line, now)
    return problems


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--root", default=".")
    ap.add_argument("--now")
    args = ap.parse_args()
    now = datetime.fromisoformat(args.now).replace(tzinfo=timezone.utc) if args.now else datetime.now(timezone.utc)
    problems = run(args.root, now)
    print("\n".join(problems))
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
