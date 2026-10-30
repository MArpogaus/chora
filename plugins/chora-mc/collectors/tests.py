#!/usr/bin/env python3
"""Fill the content `overlay` of chora-mc, the test sheet.

Lines: the last runs of CI (`gh run list`), the results of the gates
(files of tests/gate-*.sh output), the nodes of the scene by owner, and
the memory of the chora processes. A source that is missing here is left
out. Super+F12 or `region.solo` shows the sheet in the drawer.

Usage: tests.py [--print] [--every SECONDS]   (one round without it)

Environment:
  MC_REPO   the repository for gh (default: the one of this script)
  MC_GATES  a directory with one file per gate, NAME.log or NAME.txt,
            whose last "ok:", "SKIP:" or "FAIL:" line counts
            (default: MC_REPO/target/gates)
"""

import argparse
import json
import os
import re
import subprocess
import sys
import time
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
SIGN = {"success": "✓", "failure": "✕", "cancelled": "–", "skipped": "–"}


def run(*args, cwd=None):
    """Stdout of a command, or None when it is missing or fails."""
    try:
        out = subprocess.run(args, capture_output=True, text=True, timeout=20, cwd=cwd)
    except (OSError, subprocess.SubprocessError):
        return None
    return out.stdout if out.returncode == 0 else None


def row(key, value):
    """Format a line of the sheet."""
    return f"{key:<9} {value}"


def ci(repo):
    """Answer the last runs of CI."""
    out = run(
        "gh",
        "run",
        "list",
        "--limit",
        "4",
        "--json",
        "status,conclusion,workflowName,headBranch",
        cwd=repo,
    )
    if out is None:
        return []
    runs = []
    for r in json.loads(out):
        sign = SIGN.get(r["conclusion"], "…") if r["status"] == "completed" else "…"
        runs.append(f"{r['headBranch']} {sign} {r['workflowName']}")
    return [row("ci" if i == 0 else "", text) for i, text in enumerate(runs)]


def gates(folder):
    """Answer the gate results, six to a line."""
    if not folder.is_dir():
        return []
    results = []
    for path in sorted(folder.glob("*.log")) + sorted(folder.glob("*.txt")):
        status = re.findall(r"^(ok|SKIP|FAIL):", path.read_text(errors="replace"), re.M)
        if status:
            sign = {"ok": "✓", "SKIP": "–", "FAIL": "✕"}[status[-1]]
            results.append(f"{path.stem.removeprefix('gate-')} {sign}")
    # Six gates to a line.
    lines = ["  ".join(results[i : i + 6]) for i in range(0, len(results), 6)]
    return [row("gates" if i == 0 else "", text) for i, text in enumerate(lines)]


def nodes():
    """Answer the nodes of the scene by owner."""
    out = run("choractl", "query", "*")
    if out is None:
        return []
    found = json.loads(out)
    owners = Counter(n.get("owner") or "none" for n in found)
    by = "   ".join(f"{o} {c}" for o, c in owners.most_common(4))
    surfaces = sum(1 for n in found if n.get("kind") == "surface")
    return [row("nodes", f"{len(found)} total   {by}   surfaces {surfaces}")]


def memory():
    """Answer the memory of the chora processes."""
    out = run("ps", "-C", "chora,chora-mc,chora-voice", "-o", "stat=,comm=,rss=")
    if not out:
        return []
    rows = [line.split() for line in out.splitlines()]
    sizes = [f"{r[1]} {int(r[-1]) // 1024} MB" for r in rows if not r[0].startswith("Z")]
    return [row("memory", "   ".join(sizes))] if sizes else []


def main():
    """Fill the overlay once or each round."""
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--print", action="store_true", help="print the JSON, do not send it")
    parser.add_argument("--every", type=float, help="seconds between rounds; one round without")
    args = parser.parse_args()
    top = run("git", "rev-parse", "--show-toplevel", cwd=HERE)
    repo = os.environ.get("MC_REPO") or (top or str(HERE)).strip()
    folder = Path(os.environ.get("MC_GATES", Path(repo) / "target/gates"))
    while True:
        lines = ci(repo) + gates(folder) + nodes() + memory()
        params = json.dumps(
            {
                "content": "overlay",
                "title": f"Tests  ·  {time.strftime('%H:%M')}  ·  Super+F12",
                "lines": lines or ["nothing to show: no gh, gates or chora here"],
            },
            ensure_ascii=False,
        )
        if args.print:
            print(params)
        elif run("choractl", "call", "mc", "content.set", params) is None:
            sys.exit("tests.py: content.set failed")
        if not args.every:
            return
        time.sleep(args.every)


if __name__ == "__main__":
    main()
