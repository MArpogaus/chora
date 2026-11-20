#!/usr/bin/env python3
"""The timeline table and the joined MP4 from the rows of the chapters.

A row that has an `expect` shows its verdict in the last column: `green`, `RED: ...` (what the
act harness found: a mismatch of `expect` or a broken invariant), `todo` (a stub that waits for
xp-give) or `-` (no check). `table(...)` is pure; `main()` reads TOUR_OUT.
"""

import json
import os
import subprocess

CHAPTERS = [
    ("1 xp", "01-xp.mp4", "rows-ch1.json"),
    ("2 tiling", "02-tiling.mp4", "rows-ch2.json"),
    ("3 mc (light)", "03-mc-light.mp4", "rows-ch3-light.json"),
    ("3b mc (dark)", "03b-mc-dark.mp4", "rows-ch3b-dark.json"),
    ("4 construct", "04-construct.mp4", "rows-ch4.json"),
    ("5 handover", "05-handover.mp4", "rows-ch5.json"),
]
HEAD = ["chapter", "mm:ss", "joined", "what the person does", "what should be seen", "still", "check"]


def mmss(seconds):
    seconds = int(seconds)
    return f"{seconds // 60:02d}:{seconds % 60:02d}"


def verdict(row):
    """The check column of a row."""
    if row.get("red") and row.get("known"):
        return f"known: {row['known']}"
    if row.get("red"):
        return "RED: " + "; ".join(row["red"])
    return {"todo": "todo", "green": "green"}.get(row.get("status"), "-")


def row_line(chapter, row, offset=0.0):
    sec = row.get("abs", row["frame"]) / 30
    cells = [chapter, mmss(sec), mmss(offset + sec), row["description"], row["expected"], row.get("still") or "", verdict(row)]
    return "| " + " | ".join(str(c).replace("|", "/").replace("\n", " ") for c in cells) + " |"


def table(chapters, title, build="unknown"):
    """Lines of the table; `chapters` is a list of (name, rows, offset seconds)."""
    lines = [
        f"# Timeline of {title}",
        "",
        f"One row per act. `mm:ss` is the start of the act in the MP4 of its chapter; `joined` the same moment in `tour-all.mp4`. Build: {build}. `check` is the verdict of the act harness (`expect` and the invariants I1 to I14): a red act is a red row.",
        "",
        "| " + " | ".join(HEAD) + " |",
        "|" + "---|" * len(HEAD),
    ]
    for name, rows, offset in chapters:
        lines += [row_line(name, r, offset) for r in rows]
    return lines


def red_rows(chapters):
    return [(name, r) for name, rows, _ in chapters for r in rows if r.get("red")]


def duration(path):
    out = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", path], capture_output=True, text=True).stdout
    return float(out.strip())


def load_rows(path):
    if not os.path.exists(path):
        return []
    data = json.load(open(path))
    return data["rows"] if isinstance(data, dict) else data


def main():
    out = os.environ.get("TOUR_OUT", "/tmp/claude-0/tour-v9")
    build = os.environ.get("TOUR_BUILD", "unknown")
    chapters, joined, off = [], [], 0.0
    for name, mp4, rows_file in CHAPTERS:
        path = os.path.join(out, mp4)
        if not os.path.exists(path):
            continue
        chapters.append((name, load_rows(os.path.join(out, rows_file)), off))
        off += duration(path)
        joined.append(path)
    lines = table(chapters, os.path.basename(out), build)
    open(os.path.join(out, "timeline-table.md"), "w").write("\n".join(lines) + "\n")
    norm = []
    for p in joined:
        q = p.replace(".mp4", "-1080.mp4")
        subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", p, "-vf", "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2:0x20242c,setsar=1", "-r", "30", "-c:v", "libx264", "-preset", "veryfast", "-crf", "20", "-pix_fmt", "yuv420p", q], check=True)
        norm.append(q)
    lst = os.path.join(out, "joined.list")
    open(lst, "w").write("".join(f"file '{p}'\n" for p in norm))
    subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-f", "concat", "-safe", "0", "-i", lst, "-c:v", "libx264", "-preset", "slow", "-crf", "28", "-pix_fmt", "yuv420p", "-movflags", "+faststart", os.path.join(out, "tour-all.mp4")], check=True)
    print("joined", mmss(off), f"red rows: {len(red_rows(chapters))}")


if __name__ == "__main__":
    main()
