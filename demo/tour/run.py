#!/usr/bin/env python3
"""Runs the tour: one session per chapter, the acts judged as they run.

    run.py [--check] [CHAPTER...]     CHAPTER: xp tiling mc construct handover (default all)

Without `--check` each chapter is recorded (MP4, stills, rows) in TOUR_OUT and the timeline table
is assembled. With `--check` the same rows run without video and pace work: no chunks, no stills,
nothing encoded; the rows go to a scratch folder. Either way every act with an `expect` is judged
by the act harness (design/tour-v9.md section 0). Prints one line per chapter and a `RED` line per
red act; `todo` rows (stubs that wait for xp-give) are counted, not judged. Exit 1 when a row is red.
"""

import importlib
import json
import os
import shutil
import sys
import tempfile
import traceback

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

CHAPTERS = {
    "xp": ["rows-ch1.json"],
    "tiling": ["rows-ch2.json"],
    "mc": ["rows-ch3-light.json", "rows-ch3b-dark.json"],
    "construct": ["rows-ch4.json"],
    "handover": ["rows-ch5.json"],
}


class Ctx:
    def __init__(self, out, check):
        self.out = out
        self.check = check


def tally(rows):
    """Counts by status: green, red, todo and unchecked."""
    count = {"green": 0, "red": 0, "known": 0, "todo": 0, "unchecked": 0}
    for r in rows:
        if r.get("red") and r.get("known"):
            count["known"] += 1
        elif r.get("red"):
            count["red"] += 1
        elif r.get("status") == "todo":
            count["todo"] += 1
        elif r.get("status") == "green":
            count["green"] += 1
        else:
            count["unchecked"] += 1
    return count


def rows_of(out, files):
    rows = []
    for name in files:
        path = os.path.join(out, name)
        if os.path.exists(path):
            data = json.load(open(path))
            rows += data["rows"] if isinstance(data, dict) else data
    return rows


def main(argv):
    check = "--check" in argv
    names = [a for a in argv if not a.startswith("--")] or list(CHAPTERS)
    unknown = [n for n in names if n not in CHAPTERS]
    if unknown:
        print(f"run.py: no chapter {unknown}; chapters: {' '.join(CHAPTERS)}")
        return 2
    if check:
        os.environ["TOUR_CHECK"] = "1"
        out = tempfile.mkdtemp(prefix="tour-check-")
    else:
        out = os.environ.get("TOUR_OUT", "/tmp/claude-0/tour-v9")
        os.makedirs(out, exist_ok=True)
    os.environ["TOUR_OUT"] = out
    red = 0
    broke = 0
    for name in names:
        try:
            importlib.import_module(f"chapters.{name}").run(Ctx(out, check))
        except Exception:  # noqa: BLE001
            broke += 1
            print(f"chapter {name}: BROKE\n{traceback.format_exc()}")
            continue
        rows = rows_of(out, CHAPTERS[name])
        c = tally(rows)
        red += c["red"]
        print(f"chapter {name}: {len(rows)} acts, {c['green']} green, {c['red']} red, {c['known']} known, {c['todo']} todo, {c['unchecked']} unchecked")
        for r in rows:
            if r.get("red") and not r.get("known"):
                print(f"  RED act {r['act']}: {r['description']}: {'; '.join(r['red'])}")
    if check:
        shutil.rmtree(out, ignore_errors=True)
    else:
        import assemble

        assemble.main()
    print(f"tour: {'check' if check else 'record'}: {red} red, {broke} broken")
    return 1 if red or broke else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
