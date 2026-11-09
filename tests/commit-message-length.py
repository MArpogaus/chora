#!/usr/bin/env python3
"""commit-msg hook: subject at most 68 characters, body lines at most 72.

Skipped: git's # comment lines, everything from git's scissors line on
(`git commit -v` appends the diff there), lines that are or start with a
URL, leading blank lines, and the length of a subject that git
generates (`Merge `, `Revert "`, `fixup! `, `squash! `, `amend! `).
Usage: commit-message-length.py MESSAGE_FILE
"""
import re
import sys

SUBJECT_LIMIT = 68
BODY_LIMIT = 72
SCISSORS = "# ------------------------ >8 ------------------------"
GENERATED = ("Merge ", 'Revert "', "fixup! ", "squash! ", "amend! ")
URL_START = re.compile(r"\s*https?://")


def main(path):
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().splitlines()
    failed = False
    subject_seen = False
    for number, line in enumerate(lines, 1):
        if line == SCISSORS:
            break
        if line.startswith("#") or (not subject_seen and not line.strip()):
            continue
        is_subject = not subject_seen
        subject_seen = True
        if is_subject and line.startswith(GENERATED):
            continue
        limit = SUBJECT_LIMIT if is_subject else BODY_LIMIT
        label = "subject" if is_subject else f"body line {number}"
        if len(line) > limit and not URL_START.match(line):
            print(f"{label}: {len(line)} characters (limit {limit}): {line}")
            failed = True
    return 1 if failed else 0


sys.exit(main(sys.argv[1]))
