#!/usr/bin/env python3
"""pre-commit hook: reject comments on lines added in the staged diff.

Usage: no-inline-comments.py FILE...   (pre-commit passes the files)

Checks Rust, shell and Python files, by suffix or by shebang. Allowed:
Rust doc comments, SAFETY comments, the shebang, the usage header at the
top of a script, and tool directives (shellcheck, noqa, type:, pylint,
fmt:, pragma:, rustfmt::skip).

The scan runs over the whole staged file, so an added line inside a
multi-line string is not a comment: Rust strings and raw strings, Python
triple-quoted strings, shell here-documents. Limits: block comments are
not checked, shell quotes do not span lines, and a here-document
starts only when a line equal to its delimiter follows.
"""
import re
import subprocess
import sys

DIRECTIVES = (
    "shellcheck", "noqa", "type:", "pylint", "fmt:", "pragma:", "rustfmt::skip",
)
HUNK = re.compile(r"^@@ -\S+ \+(\d+)")
RUST_RAW = re.compile(r"b?r(#*)\"")
HEREDOC = re.compile(r"(?<!<)<<(-?)\s*(['\"]?)([A-Za-z_]\w*)\2")
MESSAGE = """inline comments are not allowed: names explain the code.
Allowed: doc comments (///, //!, docstrings), // SAFETY:, tool directives
(shellcheck, noqa, type:, pylint, fmt:, pragma:, rustfmt::skip) and the
usage header at the top of a script. Remove the comment or move the
reason into the doc comment of the item."""


def git(*args):
    result = subprocess.run(
        ["git", "-c", "core.quotepath=off", *args],
        capture_output=True, text=True, check=True,
    )
    return result.stdout


def staged(path):
    try:
        return git("show", f":{path}").splitlines()
    except subprocess.CalledProcessError:
        return []


def kind_of(path, lines):
    if path.endswith(".rs"):
        return "rust"
    if path.endswith(".py"):
        return "python"
    if path.endswith(".sh"):
        return "shell"
    first = lines[0] if lines else ""
    if first.startswith("#!"):
        if "python" in first:
            return "python"
        if re.search(r"\b(ba|z|da)?sh\b", first):
            return "shell"
    return None


def string_opener(line, index, kind):
    """The closer, escape rule and persistence of a string starting here."""
    char = line[index]
    if kind == "rust":
        raw = RUST_RAW.match(line, index) if not line[index - 1 : index].isalnum() else None
        if raw:
            return raw.end() - index, '"' + raw.group(1), False, True
        if char == '"':
            return 1, '"', True, True
        return None
    if char in "\"'":
        triple = line[index : index + 3]
        if kind == "python" and triple == char * 3:
            return 3, triple, True, True
        return 1, char, char == '"' or kind == "python", False
    return None


def skip_char_literal(line, index):
    if line[index + 1 : index + 2] == "\\":
        end = line.find("'", index + 3)
        return end + 1 if end > 0 else index + 1
    if line[index + 2 : index + 3] == "'":
        return index + 3
    return index + 1


def comment_start(line, kind, state):
    index = 0
    while index < len(line):
        if state:
            closer, escapes, _ = state
            if escapes and line[index] == "\\":
                index += 2
            elif line.startswith(closer, index):
                index += len(closer)
                state = None
            else:
                index += 1
            continue
        opener = string_opener(line, index, kind)
        if opener:
            length, closer, escapes, persists = opener
            state = (closer, escapes, persists)
            index += length
            continue
        before = line[index - 1] if index else " "
        if kind == "rust":
            if line[index] == "'":
                index = skip_char_literal(line, index)
                continue
            if line.startswith("//", index) and before != ":":
                return index, state
        elif line[index] == "#" and before.isspace():
            return index, state
        index += 1
    if state and not state[2]:
        state = None
    return -1, state


def closes_later(lines, start, word, dash):
    return any(
        (line.lstrip("\t") if dash else line) == word for line in lines[start:]
    )


def comments(lines, kind):
    state = None
    heredoc = None
    for number, line in enumerate(lines, 1):
        if heredoc:
            word, dash = heredoc
            if (line.lstrip("\t") if dash else line) == word:
                heredoc = None
            continue
        column, state = comment_start(line, kind, state)
        if column >= 0:
            yield number, column
        if kind == "shell":
            found = HEREDOC.search(line[:column] if column >= 0 else line)
            dash = bool(found and found.group(1))
            if found and closes_later(lines, number, found.group(3), dash):
                heredoc = (found.group(3), dash)


def allowed(text, kind):
    if kind == "rust":
        if text.startswith(("///", "//!")):
            return True
        body = text[2:].strip()
        return body.startswith("SAFETY:") or body.startswith(DIRECTIVES)
    return text[1:].strip().startswith(DIRECTIVES)


def header_lines(lines):
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


def renamed_from(path):
    fields = git(
        "diff", "--cached", "-M", "--name-status", "-z", "--diff-filter=R",
    ).split("\0")
    for index in range(0, len(fields) - 2, 3):
        if fields[index + 2] == path:
            return fields[index + 1]
    return None


def added_numbers(path):
    old = renamed_from(path)
    diff = git(
        "diff", "--cached", "-M", "-U0", "--no-color", "--no-ext-diff",
        "--diff-filter=ACMR", "--", path, *([old] if old else []),
    )
    numbers = set()
    number = 0
    in_hunk = False
    for line in diff.splitlines():
        if line.startswith("@@"):
            number = int(HUNK.match(line).group(1))
            in_hunk = True
        elif in_hunk and line.startswith("+"):
            numbers.add(number)
            number += 1
    return numbers


def violations(path):
    added = added_numbers(path)
    if not added:
        return
    lines = staged(path)
    kind = kind_of(path, lines)
    if kind is None:
        return
    header = header_lines(lines) if kind != "rust" else set()
    for number, column in comments(lines, kind):
        text = lines[number - 1][column:]
        if number not in added or number in header:
            continue
        if not allowed(text.strip(), kind):
            yield number, lines[number - 1].strip()


def main(paths):
    failed = False
    for path in paths:
        for number, text in violations(path):
            print(f"{path}:{number}: {text}")
            failed = True
    if failed:
        print(MESSAGE)
    return 1 if failed else 0


sys.exit(main(sys.argv[1:]))
