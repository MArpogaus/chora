#!/usr/bin/env python3
"""Check that every session starts chora-notify, the one server of notices.

Usage: tests/flavor-notify.py   (from any directory; exits 1 on a gap)

Each target of flavors/*/ wants a service of its own flavor whose
ExecStart runs /usr/bin/chora-notify; without it every notice of the
look (kit::notice, a refusal, a dead key) is lost. The start of mc in
README.md, which has no target, runs chora-notify with the mc slot.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def wants(unit):
    return [w for line in unit.read_text().splitlines() if line.startswith("Wants=") for w in line[6:].split()]


def runs_notify(unit):
    return any(
        line.startswith("ExecStart=/usr/bin/chora-notify")
        for line in unit.read_text().splitlines()
    )


def main():
    bad = []
    targets = sorted((ROOT / "flavors").glob("*/*.target"))
    for target in targets:
        units = [target.parent / name for name in wants(target)]
        if not any(unit.is_file() and runs_notify(unit) for unit in units):
            bad.append(f"{target.relative_to(ROOT)} starts no chora-notify")
    readme = (ROOT / "README.md").read_text()
    row = re.search(r"^\| mc \|.*$", readme, re.M)
    if not row or "chora-notify --assets /usr/share/chora/mc/notify" not in row.group(0):
        bad.append("README.md: the start of mc runs no chora-notify with the mc slot")
    print("\n".join(bad))
    print(f"{len(targets)} targets, {len(bad)} gaps")
    return 1 if bad else 0


sys.exit(main())
