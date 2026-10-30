#!/usr/bin/env python3
"""Fill the scene `morning` of chora-mc, the front page of the day.

Panels: the lead (the one thing due today, else the next entry), the
weather of wttr.in (cached for an hour, nothing when offline), the day
from agenda.sh, the mail of notmuch (a hint without it), what the agents
of agent.set finished (nothing without them) and the reading list. All
sources are only read.

Usage: morning.py [--print] [ORG_FILE...]   (agenda.sh takes the files)

Environment:
  MC_WEATHER  a place for wttr.in, for example Konstanz (default: by IP)
  MC_READING  the reading list, one entry per line; markdown or org
              items, done ones ([x], DONE) left out
              (default: ~/Notes/reading.md)
"""

import json
import os
import re
import subprocess
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE_SECONDS = 3600
RAIN_CHANCE = 50  # percent
CACHE = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache")) / "chora-mc"


def run(*args, timeout=30):
    """Stdout of a command, or None when it is missing or fails."""
    try:
        out = subprocess.run(args, capture_output=True, text=True, timeout=timeout, check=True)
    except (OSError, subprocess.SubprocessError):
        return None
    return out.stdout


def day(org_files):
    """Lines TIME<tab>TEXT of today from agenda.sh."""
    out = run(str(HERE / "agenda.sh"), "--print", *org_files, timeout=60)
    if not out:
        return None
    return json.loads(out.splitlines()[0])["lines"]


def split(lines):
    """Split the entries with a time, as (HH:MM, text), from the tasks."""
    timed = [line.split("\t", 1) for line in lines if not line.startswith("\t")]
    timed = [(t.split("-")[0].zfill(5), x) for t, x in timed]
    due = [line.split("\t", 1)[1] for line in lines if line.startswith("\t")]
    return timed, due


def day_lines(lines):
    """Set the day as the paper does: each start before its entry, then the tasks.

    A page without a margin has no room for a tab.
    """
    if lines is None:
        return []
    timed, due = split(lines)
    out = [f"{t}  {x}" for t, x in timed]
    if due:
        out += ["", "Due today: " + "; ".join(due) + "."]
    return out


def lead(lines):
    """Pick the one thing that matters: a task due today, else the next entry."""
    if lines is None:
        return "The day", ["No agenda: agenda.sh did not run."]
    timed, due = split(lines)
    now = time.strftime("%H:%M")
    coming = [(t, x) for t, x in timed if t >= now]
    text = []
    if timed:
        many = "entry" if len(timed) == 1 else "entries"
        rest = f"the next at {coming[0][0]}: {coming[0][1]}" if coming else "none left"
        text.append(f"{len(timed)} {many} today, {rest}.")
    if len(due) > 1:
        text += ["", "Also due: " + "; ".join(due[1:]) + "."]
    if due:
        return due[0], text
    if coming:
        return coming[0][1], text
    return "A free day", text or ["Nothing in the agenda today."]


def weather():
    """One or two sentences, from the cache when it is young enough."""
    place = os.environ.get("MC_WEATHER", "")
    cache = CACHE / "weather.json"
    data = None
    if cache.exists() and time.time() - cache.stat().st_mtime < CACHE_SECONDS:
        data = json.loads(cache.read_text())
    if data is None:
        url = f"https://wttr.in/{urllib.parse.quote(place)}?format=j1"
        try:
            with urllib.request.urlopen(url, timeout=8) as answer:
                data = json.load(answer)
            CACHE.mkdir(parents=True, exist_ok=True)
            cache.write_text(json.dumps(data))
        except (OSError, ValueError):
            # Offline: an old cache is better than nothing.
            data = json.loads(cache.read_text()) if cache.exists() else None
    if data is None:
        return None
    now = data["current_condition"][0]
    today = data["weather"][0]
    lines = [
        f"{now['weatherDesc'][0]['value'].strip()}, {now['temp_C']} °C, "
        f"from {today['mintempC']} to {today['maxtempC']} °C today."
    ]
    hour = time.localtime().tm_hour * 100
    rain = [
        int(h["time"]) // 100
        for h in today["hourly"]
        if int(h["time"]) >= hour - 200 and int(h["chanceofrain"]) >= RAIN_CHANCE
    ]
    if rain:
        lines.append(f"Rain likely from {rain[0]:02d}:00.")
    return lines


def mail():
    """Answer the title and the lines of the unread mail in the inbox."""
    query = "tag:inbox and tag:unread"
    count = run("notmuch", "count", query)
    if count is None:
        return "Mail", ["notmuch is not installed: no digest."]
    found = json.loads(run("notmuch", "search", "--format=json", "--limit=6", query) or "[]")
    n = int(count)
    lines = [f"{m['authors'].split(',')[0].split('|')[0]}: {m['subject']}" for m in found]
    if n > len(found):
        lines.append(f"… and {n - len(found)} more.")
    return f"Mail, {n} new" if n else "Mail", lines or ["Nothing new."]


def overnight():
    """Answer what the agents finished, by their last words; None without any."""
    out = run("choractl", "call", "mc", "agent.list", "{}", timeout=5)
    agents = json.loads(out)["agents"] if out else []
    lines = [
        f"{a['name'].capitalize()}: {a['said'] or a['task']}"
        for a in agents
        if a["state"] == "done" and (a["said"] or a["task"])
    ]
    return lines or None


def reading():
    """Up to five open entries of the reading list."""
    path = Path(os.environ.get("MC_READING", Path.home() / "Notes/reading.md"))
    if not path.exists():
        return [f"No reading list: {path} (set MC_READING)."]
    items = []
    for line in path.read_text().splitlines():
        if re.search(r"\[[xX]\]|^\*+ (DONE|KILL) ", line):
            continue
        text = re.sub(r"^(\s*[-+*]\s+(\[ \]\s+)?|\*+\s+(TODO\s+)?)", "", line).strip()
        # An org or markdown link shows its description.
        text = re.sub(r"\[\[[^]]*\]\[([^]]*)\]\]|\[([^]]*)\]\([^)]*\)", r"\1\2", text)
        if text and not text.startswith(("#", ":")):
            items.append(text)
    return items[:5] or ["The list is empty."]


def main():
    """Fill the panels, or print their parameters."""
    args = sys.argv[1:]
    show = args[:1] == ["--print"]
    if show:
        args = args[1:]
    lines = day(args)
    title, text = lead(lines)
    w = weather()
    mail_title, mail_lines = mail()
    done = overnight()
    panels = [
        ("lead", title, text),
        ("weather", "Weather" if w else "", w or []),
        ("day", "The day", day_lines(lines)),
        ("mail", mail_title, mail_lines),
        ("overnight", "Overnight" if done else "", done or []),
        ("reading", "To read", reading()),
    ]
    for panel, title, lines in panels:
        params = json.dumps(
            {"scene": "morning", "panel": panel, "title": title, "lines": lines},
            ensure_ascii=False,
        )
        if show:
            print(params)
        elif run("choractl", "call", "mc", "panel.set", params, timeout=5) is None:
            sys.exit(f"morning.py: panel.set {panel} failed")


if __name__ == "__main__":
    main()
