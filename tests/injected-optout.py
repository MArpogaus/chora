#!/usr/bin/env python3
"""pre-commit hook: no pattern on an event with `injected` opts out of it.

Usage: injected-optout.py [FILE...]   (pre-commit passes the files; none = all tracked Rust files)

The events with an `injected` field are read from crates/chora-scene/src/event.rs
(NodeKey, PointerPressed, PointerReleased, and every other one with the field).
A pattern on such an event that binds the field to a name that starts with an
underscore (`injected: _`, `injected: _flag`), or that ends in the rest pattern `..`
without naming the field, drops the flag without a decision, so an injected key,
press or move (from `input.inject`) acts as a person's. The flag bound inside
another field (`text: Some(injected)`) does not name the field. Every such line needs a
`// injected: <reason>` comment on that line (or alone on the next line, where rustfmt puts a
comment that follows a brace) that says why the event is safe to take as it is.
A pattern that names the field (`injected,`, `injected: true` or `injected: name`) may end in `..`.
"""
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
EVENT_FILE = ROOT / "crates/chora-scene/src/event.rs"
REASON = re.compile(r"//\s*injected:\s*\S")
MESSAGE = """a pattern on an event with `injected` must name the field, not drop it.
Match `injected` (and decide what an injected event does), or end the line
that has `injected: _` or `..` with `// injected: <reason>`."""


def injected_events(text):
    names = []
    for match in re.finditer(r"^    (\w+) \{\n(.*?)^    \},?$", text, re.S | re.M):
        if re.search(r"^\s+injected: bool,", match.group(2), re.M):
            names.append(match.group(1))
    return names


def mask_comments(text):
    out = []
    for line in text.split("\n"):
        stripped = line.lstrip()
        if stripped.startswith("//"):
            out.append(" " * len(line))
            continue
        index = line.find("//")
        in_string = line[:index].count('"') % 2 == 1 if index >= 0 else False
        out.append(line if index < 0 or in_string else line[:index] + " " * (len(line) - index))
    return "\n".join(out)


def bodies(code, names):
    pattern = re.compile(r"\b(?:%s)\s*\{" % "|".join(names))
    for match in pattern.finditer(code):
        start = match.end()
        depth = 0
        index = start
        while index < len(code):
            char = code[index]
            if char in "([{":
                depth += 1
            elif char in ")]}":
                if depth == 0:
                    break
                depth -= 1
            index += 1
        yield match.group(0), start, index


def top_level_fields(code, start, end):
    """The comma-separated items of a pattern or literal at its own depth: (offset, text)."""
    depth = 0
    item = start
    for index in range(start, end + 1):
        char = code[index] if index < end else ","
        if char in "([{":
            depth += 1
        elif char in ")]}":
            depth -= 1
        elif char == "," and depth == 0:
            text = code[item:index]
            if text.strip():
                yield item + len(text) - len(text.lstrip()), text.strip()
            item = index + 1


def injected_binding(field):
    """The target of the field `injected` in an item, or None for any other item."""
    if field == "injected":
        return "injected"
    match = re.match(r"injected\s*:\s*(.*)\Z", field, re.S)
    return match.group(1) if match else None


def opt_outs(code, start, end):
    fields = list(top_level_fields(code, start, end))
    bindings = [(offset, injected_binding(field)) for offset, field in fields]
    decided = False
    for offset, target in bindings:
        if target is None:
            continue
        if re.match(r"_\w*\Z", target):
            yield offset
        else:
            decided = True
    for offset, field in fields:
        if field == ".." and not decided:
            yield offset


def violations(path, names):
    try:
        text = pathlib.Path(path).read_text()
    except (OSError, UnicodeDecodeError):
        return
    code = mask_comments(text)
    lines = text.split("\n")
    seen = set()
    for _, start, end in bodies(code, names):
        for index in opt_outs(code, start, end):
            number = code.count("\n", 0, index) + 1
            following = lines[number] if number < len(lines) else ""
            if number in seen or REASON.search(lines[number - 1]) or REASON.match(following.strip()):
                continue
            seen.add(number)
            yield number, lines[number - 1].strip()


def tracked_rust_files():
    result = subprocess.run(
        ["git", "-C", str(ROOT), "ls-files", "*.rs"], capture_output=True, text=True, check=True
    )
    return [str(ROOT / name) for name in result.stdout.split("\n") if name]


def main(paths):
    names = injected_events(EVENT_FILE.read_text())
    if not names:
        print(f"no event with `injected` found in {EVENT_FILE}")
        return 1
    failed = False
    for path in paths or tracked_rust_files():
        if not path.endswith(".rs"):
            continue
        for number, text in violations(path, names):
            print(f"{path}:{number}: {text}")
            failed = True
    if failed:
        print(MESSAGE)
    return 1 if failed else 0


sys.exit(main(sys.argv[1:]))
