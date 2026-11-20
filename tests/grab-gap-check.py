#!/usr/bin/env python3
"""Fails on a start_grabs that reaches end_grabs with no grab call between.

Such a pair is ungrab_all in disguise: the plugin lets its keys go and
grabs none (GR1, GR2). It also flags a grab callee of the same file that
returns early, or grabs only inside an if or a match, before its first grab.
A grab in a loop passes, and a callee in another file is not read.

Usage: tests/grab-gap-check.py [FILE.rs ...]  (default: every tracked .rs)
"""
import re
import subprocess
import sys

GRAB = re.compile(r"(?<![A-Za-z_])grab\w*\(|\.grab\w*\(")
START = re.compile(r"\bstart_grabs\(\)")
END = re.compile(r"\bend_grabs\(\)")


def body_of(text, name):
    """The text of the body of `fn name` in `text`, or None."""
    head = re.search(r"fn " + re.escape(name) + r"\b[^{;]*\{", text)
    if not head:
        return None
    depth, i = 1, head.end()
    while i < len(text) and depth:
        depth += {"{": 1, "}": -1}.get(text[i], 0)
        i += 1
    return text[head.end() : i - 1]


def skips_grab(body, text="", seen=()):
    """True if a path of `body` ends with no grab: a return before the
    first grab, or a first grab inside an if, else or match block. When
    the first grab is a call to a callee of the same file, that callee is
    read the same way (`seen` stops a recursion)."""
    first = GRAB.search(body)
    if not first:
        return False
    before = body[: first.start()]
    if re.search(r"\breturn\b", before):
        return True
    open_blocks = []
    for line in before.splitlines():
        stripped = line.strip()
        for ch in stripped:
            if ch == "{":
                open_blocks.append(stripped)
            elif ch == "}" and open_blocks:
                open_blocks.pop()
    if any(re.match(r"(\}\s*)?(if|else|match)\b", b) for b in open_blocks):
        return True
    name = re.search(r"grab\w*", first.group())
    if name and name.group() not in seen:
        callee = body_of(text, name.group())
        return callee is not None and skips_grab(callee, text, (*seen, name.group()))
    return False


def skipping_callees(text, between):
    for call in re.finditer(r"(?<![A-Za-z_])(grab\w*)\(", between):
        body = body_of(text, call.group(1))
        if body is not None and skips_grab(body, text, (call.group(1),)):
            yield call.group(1)


def gaps(path):
    text = open(path, encoding="utf-8").read()
    for start in START.finditer(text):
        end = END.search(text, start.end())
        in_ungrab_all = re.findall(r"fn (\w+)", text[: start.start()])[-1:] == ["ungrab_all"]
        if not end or in_ungrab_all:
            continue
        between = text[start.end() : end.start()]
        line = text.count("\n", 0, start.start()) + 1
        if not GRAB.search(between):
            yield line, "start_grabs reaches end_grabs with no grab between: use grab, or ungrab_all"
        else:
            for name in skipping_callees(text, between):
                yield line, f"{name} skips its grab on a branch: a path reaches end_grabs with no grab"


def main(files):
    if not files:
        files = subprocess.run(
            ["git", "ls-files", "*.rs"], capture_output=True, text=True, check=True
        ).stdout.split()
    bad = [(f"{f}:{line}", why) for f in files for line, why in gaps(f)]
    for where, why in bad:
        print(f"{where}: {why}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
