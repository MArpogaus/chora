#!/usr/bin/env python3
"""Check that both packages install every file of the flavors and the HUDs.

Usage: tests/packaging-files.py   (from any directory; exits 1 on a gap)

Reads the install lines of dist/arch/PKGBUILD and the assets of
crates/chora/Cargo.toml (the RPM). Every regular file under flavors/ and
plugins/*/hud must match a source of each; an executable file must match a
755 source and no 644 one.
"""
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TOKEN = re.compile(r"(?:flavors|plugins)/[^\s\"')]+")


def glob_regex(pattern):
    out = ""
    i = 0
    while i < len(pattern):
        if pattern.startswith("**/", i):
            out += "(?:.*/)?"
            i += 3
        elif pattern[i] == "*":
            out += "[^/]*"
            i += 1
        else:
            out += re.escape(pattern[i])
            i += 1
    return re.compile(out + r"\Z")


def arch_sources(text):
    lines = [ln for ln in text.replace("\\\n", " ").splitlines() if not ln.lstrip().startswith("#")]
    loops = {}
    out = []
    for ln in lines:
        loop = re.search(r"for (\w+) in ([^;]+); do", ln)
        if loop:
            loops[loop.group(1)] = loop.group(2).split()
        mode = re.search(r"-Dm(\d+)", ln)
        mode = mode.group(1) if mode else "644"
        recursive = " cp -r" in ln or ln.lstrip().startswith("cp -r")
        in_find = "find " in ln
        for tok in TOKEN.findall(ln):
            names = [tok]
            for var, vals in loops.items():
                if "$" + var in tok:
                    names = [tok.replace("$" + var, v) for v in vals]
            for name in names:
                name = os.path.normpath(name)
                if in_find or recursive:
                    name = name + ("/**/*" if recursive else "/*")
                out.append((name, mode))
    return out


def rpm_sources(text):
    rx = re.compile(r'source = "\.\./\.\./((?:flavors|plugins)/[^"]+)".*?mode = "(\d+)"')
    return [(m.group(1), m.group(2)) for m in rx.finditer(text)]


def shipped():
    files = []
    roots = [ROOT / "flavors"] + sorted((ROOT / "plugins").glob("*/hud"))
    for base in roots:
        for dirpath, _, names in os.walk(base):
            for name in names:
                path = Path(dirpath) / name
                if path.is_symlink():
                    continue
                files.append((path.relative_to(ROOT).as_posix(), os.access(path, os.X_OK)))
    return sorted(files)


def check(label, sources, files):
    rules = [(glob_regex(pat), mode) for pat, mode in sources]
    bad = []
    for path, executable in files:
        modes = {mode for rx, mode in rules if rx.match(path)}
        if not modes:
            bad.append(f"{label}: not installed: {path}")
        elif executable and ("755" not in modes or "644" in modes):
            bad.append(f"{label}: wrong mode for the executable {path}: {sorted(modes)}")
    return bad


def main():
    files = shipped()
    arch = arch_sources((ROOT / "dist/arch/PKGBUILD").read_text())
    rpm = rpm_sources((ROOT / "crates/chora/Cargo.toml").read_text())
    bad = check("PKGBUILD", arch, files) + check("rpm", rpm, files)
    print("\n".join(bad))
    print(f"{len(files)} files, {len(bad)} gaps")
    return 1 if bad else 0


sys.exit(main())
