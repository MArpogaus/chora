#!/usr/bin/env python3
"""Runs the tour: one session per chapter, the acts judged as they run.

    run.py [--check] [CHAPTER...]     CHAPTER: xp tiling mc construct handover (default all)

Without `--check` each chapter is recorded (MP4, stills, rows) in TOUR_OUT and the timeline table
is assembled. With `--check` the same rows run without video and pace work: no chunks, no stills,
nothing encoded; the rows go to a scratch folder. Either way every act with an `expect` is judged
by the act harness (design/tour-v9.md section 0): `green` when its `expect` held, `inv` when it has
none and only the invariants held. Prints one line per chapter and a `RED` line per red act; `todo` rows (stubs that wait for xp-give) are counted, not judged. Exit 1 when a row is red.
"""

import importlib
import json
import os
import shutil
import sys
import tempfile
import traceback

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import tour  # noqa: E402

CHAPTERS = {
    "xp": ["rows-xp.json"],
    "tiling": ["rows-tiling.json"],
    "mc": ["rows-mc-light.json", "rows-mc-dark.json"],
    "construct": ["rows-construct.json"],
    "handover": ["rows-handover.json"],
}


class Ctx:
    def __init__(self, out, check):
        self.out = out
        self.check = check


def tally(rows):
    """Counts by status: green (an `expect` held), inv (no `expect`, only the invariants held),
    known, red, todo and unchecked."""
    count = {"green": 0, "inv": 0, "known": 0, "red": 0, "todo": 0, "unchecked": 0}
    for r in rows:
        key = {"RED": "red"}.get(r.get("status"), r.get("status"))
        count[key if key in count else "unchecked"] += 1
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
    """With TOUR_BASELINE set (the replay on 733967aa) a chapter that stops at an act the old build
    cannot do, a RuntimeError that says the method is missing (`tour.NO_METHOD`), is `baseline: none`
    from there on, with its reason (design/tour-v9.md section 6), and no BROKE; any other error is
    BROKE as on the candidate. `baseline-none.json` in the folder of the take lists, per chapter, its
    reason and the `<act>.<capture>` names that the chapter planned and did not take: the
    `baseline_none` of intended.toml."""
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
        shutil.rmtree(os.path.join(out, tour.CAPTURES), ignore_errors=True)
    os.environ["TOUR_OUT"] = out
    baseline = bool(os.environ.get("TOUR_BASELINE"))
    tour.TAKEN.clear()
    red = 0
    broke = 0
    none = []
    for name in names:
        try:
            importlib.import_module(f"chapters.{name}").run(Ctx(out, check))
        except Exception as err:  # noqa: BLE001
            if baseline and isinstance(err, RuntimeError) and tour.NO_METHOD.search(str(err)):
                reason = " ".join(str(err).split())
                missing = [n for n in tour.planned(name) if n not in tour.TAKEN]
                none.append({"chapter": name, "reason": reason, "names": missing, "trace": traceback.format_exc()})
                print(f"chapter {name}: baseline: none from the act that stopped it: {reason} ({len(missing)} captures)")
                continue
            broke += 1
            print(f"chapter {name}: BROKE\n{traceback.format_exc()}")
            continue
        rows = rows_of(out, CHAPTERS[name])
        c = tally(rows)
        red += c["red"]
        print(f"chapter {name}: {len(rows)} acts, {c['green']} green, {c['inv']} inv, {c['known']} known, {c['red']} red, {c['todo']} todo, {c['unchecked']} unchecked")
        for r in rows:
            if r.get("status") == "RED":
                print(f"  RED act {r['act']}: {r['description']}: {'; '.join(r['red'])}")
    if none and not check:
        with open(os.path.join(out, "baseline-none.json"), "w", encoding="utf-8") as f:
            json.dump(none, f, indent=1)
    if check:
        shutil.rmtree(out, ignore_errors=True)
    elif not baseline:
        import assemble

        assemble.main()
    print(f"tour: {'check' if check else 'record'}: {red} red, {broke} broken" + (f", {len(none)} baseline: none" if none else ""))
    return 1 if red or broke else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
