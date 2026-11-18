#!/usr/bin/python3
"""The gate step `acts`: the live selftest of the harness, then the acts
of each look, each in its own headless session, judged against
tests/acts/known-red.toml.

The acts of a look are the function `run(s)` of tests/acts/looks/LOOK.py.
Prints one line per cell: `cell LOOK ID pass|red|FAIL`, then the
blocking lines, and exits 1 when one blocks.

Usage: tests/acts/run.py [--no-selftest] [LOOK...]   BIN: the binaries
  LOOK  xp, tiling, mc, construct (default all)
"""

import importlib
import pathlib
import shutil
import sys
import tempfile
import traceback

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "looks"))

import harness  # noqa: E402
import planted  # noqa: E402

LOOKS = ["xp", "tiling", "mc", "construct"]


def main():
    """Run the selftest and the acts, print the cells, judge."""
    args = sys.argv[1:]
    selftest = "--no-selftest" not in args
    looks = [a for a in args if not a.startswith("--")] or LOOKS
    runs = pathlib.Path(tempfile.mkdtemp(prefix="acts-run-"))
    print(f"acts: evidence in {runs}")
    blocking = []
    if selftest:
        checks = {}
        try:
            planted.run(checks)
        except Exception:  # noqa: BLE001
            blocking.append(f"FAIL: the live selftest broke: {traceback.format_exc()}")
        for (_, name), failures in sorted(checks.items()):
            print(f"cell selftest {name!r} {'FAIL' if failures else 'pass'}")
            blocking += [f"FAIL: selftest {name}: {f}" for f in failures]
        if not checks:
            blocking.append("FAIL: the live selftest ran no cell")
    results = {}
    for look in looks:
        module = importlib.import_module(look)
        try:
            with harness.Session(look, results=results, runs=runs) as s:
                module.run(s)
        except Exception:  # noqa: BLE001
            results[(look, "act:session")] = [traceback.format_exc()]
    rows = [r for r in harness.known_red() if r["look"] in looks]
    verdict = harness.judge(results, rows, full=True)
    for key in sorted(results):
        word = "red" if key in verdict.red else "pass" if key in verdict.passed else "FAIL"
        print(f"cell {key[0]} {key[1]} {word}")
        if word == "red":
            print(f"  red: {results[key][0][:300]}")
    blocking += verdict.blocking
    if blocking:
        print(f"acts: evidence kept in {runs}")
    else:
        shutil.rmtree(runs, ignore_errors=True)
    print(
        f"acts: {len(verdict.passed)} pass, {len(verdict.red)} known red, {len(blocking)} blocking"
    )
    for line in blocking:
        print(line)
    return 1 if blocking else 0


if __name__ == "__main__":
    sys.exit(main())
