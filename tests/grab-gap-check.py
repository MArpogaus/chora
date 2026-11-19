#!/usr/bin/env python3
"""Fails on a start_grabs that reaches end_grabs with no grab call between.

Such a pair is ungrab_all in disguise: the plugin lets its keys go and
grabs none (GR1, GR2).

Usage: tests/grab-gap-check.py [FILE.rs ...]  (default: every tracked .rs)
"""
import re
import subprocess
import sys

GRAB = re.compile(r"(?<![A-Za-z_])grab\w*\(|\.grab\w*\(")
START = re.compile(r"\bstart_grabs\(\)")
END = re.compile(r"\bend_grabs\(\)")


def gaps(path):
    text = open(path, encoding="utf-8").read()
    for start in START.finditer(text):
        end = END.search(text, start.end())
        in_ungrab_all = re.findall(r"fn (\w+)", text[: start.start()])[-1:] == ["ungrab_all"]
        if end and not in_ungrab_all and not GRAB.search(text[start.end() : end.start()]):
            yield text.count("\n", 0, start.start()) + 1


def main(files):
    if not files:
        files = subprocess.run(
            ["git", "ls-files", "*.rs"], capture_output=True, text=True, check=True
        ).stdout.split()
    bad = [f"{f}:{line}" for f in files for line in gaps(f)]
    for where in bad:
        print(f"{where}: start_grabs reaches end_grabs with no grab between: use grab, or ungrab_all")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
