#!/usr/bin/env python3
"""commit-msg hook: subject at most 68 characters, body lines at most 72.

Lines that are or start with a URL and git's # comment lines are skipped.
Usage: commit-message-length.py MESSAGE_FILE
"""
import re
import sys

SUBJECT_LIMIT = 68
BODY_LIMIT = 72
URL_START = re.compile(r"\s*https?://")


def main(path):
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().splitlines()
    failed = False
    subject_seen = False
    for line in lines:
        if line.startswith("#"):
            continue
        limit = BODY_LIMIT if subject_seen else SUBJECT_LIMIT
        subject_seen = True
        if len(line) > limit and not URL_START.match(line):
            print(f"{len(line)} characters (limit {limit}): {line}")
            failed = True
    return 1 if failed else 0


sys.exit(main(sys.argv[1]))
