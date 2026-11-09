#!/usr/bin/env python3
"""pre-commit hook: reject comments on lines added in the staged diff.

Checks *.rs, *.sh and *.py. Allowed: Rust doc comments and SAFETY, the
shebang, the usage header at the top of a script, and tool directives.
"""
import re
import subprocess
import sys

DIRECTIVES = ("shellcheck", "noqa", "type:", "pylint", "fmt:")
HUNK = re.compile(r"^@@ -\S+ \+(\d+)")


def git(*args):
    result = subprocess.run(
        ["git", "-c", "core.quotepath=off", *args],
        capture_output=True, text=True, check=True,
    )
    return result.stdout


def allowed_rust(text):
    return text.startswith(("///", "//!")) or text.startswith("// SAFETY:")


def allowed_hash(text):
    body = text[1:].strip()
    return body.startswith(DIRECTIVES)


def find_trailing(line, marker):
    quote = None
    index = 0
    while index < len(line):
        char = line[index]
        if quote:
            if char == "\\":
                index += 1
            elif char == quote:
                quote = None
        elif char in ('"', "'") and (marker == "#" or char == '"'):
            quote = char
        elif line.startswith(marker, index):
            before = line[index - 1] if index else " "
            if marker == "//" and before != ":":
                return index
            if marker == "#" and before.isspace():
                return index
        index += 1
    return -1


def violates(line, path):
    stripped = line.strip()
    if path.endswith(".rs"):
        marker, allowed = "//", allowed_rust
    else:
        marker, allowed = "#", allowed_hash
    if stripped.startswith(marker):
        return not allowed(stripped)
    start = find_trailing(line, marker)
    return start > 0 and not allowed(line[start:].strip())


def header_lines(path):
    try:
        lines = git("show", f":{path}").splitlines()
    except subprocess.CalledProcessError:
        return set()
    found = set()
    for number, text in enumerate(lines, 1):
        stripped = text.strip()
        if stripped.startswith("#"):
            found.add(number)
        elif stripped == "" and not found:
            continue
        else:
            break
    return found


def added_lines(diff):
    path = None
    number = 0
    for line in diff.splitlines():
        if line.startswith("+++ "):
            path = line[6:]
        elif line.startswith("@@"):
            number = int(HUNK.match(line).group(1))
        elif line.startswith("+") and path:
            yield path, number, line[1:]
            number += 1


def main():
    diff = git(
        "diff", "--cached", "-U0", "--no-color", "--no-ext-diff",
        "--diff-filter=ACMR", "--", "*.rs", "*.sh", "*.py",
    )
    headers = {}
    failed = False
    for path, number, line in added_lines(diff):
        if not path.endswith(".rs"):
            if number == 1 and line.startswith("#!"):
                continue
            if path not in headers:
                headers[path] = header_lines(path)
            if number in headers[path]:
                continue
        if violates(line, path):
            print(f"{path}:{number}: {line.strip()}")
            failed = True
    if failed:
        print("inline comments are not allowed: names explain the code")
    return 1 if failed else 0


sys.exit(main())
