#!/usr/bin/python3
"""Check a chapter video of the tour (design/tour-v9.md section 6, "Video checker").

Usage: tests/tour-video-check.py VIDEO.mp4 [--cut START,END]... [--samples samples.json]

Flags, as timestamps in seconds (exit 1 when there is one):
- black: a black stretch over 2 frames (ffmpeg blackdetect);
- freeze: a still over 1.5 s that is not inside a cut (ffmpeg freezedetect, as the v4 check);
- flicker: a frame that differs from both neighbours while they are equal to each other;
- fixed-moved: with --samples (a list of {t, camera, nodes: {id: {fixed, rect}}} read from the
  session while the video ran), a screen-fixed node whose rect moved while the camera moved.
A cut is a stretch where the film is meant to stand still (a slide, a title).
"""

import json
import re
import subprocess
import sys

FREEZE_SECONDS = 1.5
FLICKER_STEP = 8.0
RECT_SLACK = 2
SIZE = (32, 18)


def ffmpeg(path, vf, *extra):
    return subprocess.run(["ffmpeg", "-hide_banner", "-nostats", "-i", path, *vf, *extra, "-f", "null", "-"], capture_output=True, text=True, check=True).stderr


def fps_of(path):
    out = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=r_frame_rate:format=duration", "-of", "json", path], capture_output=True, text=True, check=True).stdout
    data = json.loads(out)
    num, den = data["streams"][0]["r_frame_rate"].split("/")
    return int(num) / int(den), float(data["format"]["duration"])


def black(path, fps):
    log = ffmpeg(path, ["-vf", f"blackdetect=d={1.9 / fps:.4f}:pix_th=0.1"])
    return [
        {"kind": "black", "start": float(s), "end": float(e)}
        for s, e in re.findall(r"black_start:([\d.]+) black_end:([\d.]+)", log)
    ]


def freeze(path, length, cuts):
    log = ffmpeg(path, ["-vf", f"freezedetect=n=-60dB:d={FREEZE_SECONDS}"])
    starts = [float(s) for s in re.findall(r"freeze_start: ([\d.]+)", log)]
    ends = [float(e) for e in re.findall(r"freeze_end: ([\d.]+)", log)]
    out = []
    for i, start in enumerate(starts):
        end = ends[i] if i < len(ends) else length
        if any(start >= cs - 0.1 and end <= ce + 0.1 for cs, ce in cuts):
            continue
        out.append({"kind": "freeze", "start": start, "end": end})
    return out


def frames(path):
    """The video as small grey frames (bytes each)."""
    w, h = SIZE
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path, "-vf", f"scale={w}:{h},format=gray", "-f", "rawvideo", "-"], capture_output=True, check=True).stdout
    return [raw[i : i + w * h] for i in range(0, len(raw), w * h)]


def step(a, b):
    return sum(abs(x - y) for x, y in zip(a, b)) / len(a)


def flicker(path, fps):
    grey, out = frames(path), []
    for i in range(1, len(grey) - 1):
        if step(grey[i - 1], grey[i]) > FLICKER_STEP and step(grey[i], grey[i + 1]) > FLICKER_STEP and step(grey[i - 1], grey[i + 1]) < FLICKER_STEP / 4:
            out.append({"kind": "flicker", "start": i / fps, "end": (i + 1) / fps})
    return out


def check(path, cuts=()):
    """The flagged stretches of a video: dicts with kind, start and end, by start time."""
    fps, length = fps_of(path)
    found = [*black(path, fps), *freeze(path, length, cuts), *flicker(path, fps)]
    return sorted(found, key=lambda f: f["start"])


def fixed_in_move(samples):
    """A screen-fixed node that moved between two samples in which the camera moved."""
    out = []
    for a, b in zip(samples, samples[1:]):
        if a["camera"] == b["camera"]:
            continue
        for nid, na in a["nodes"].items():
            nb = b["nodes"].get(nid)
            if nb and na.get("fixed") and nb.get("fixed") and any(abs(x - y) > RECT_SLACK for x, y in zip(na["rect"], nb["rect"])):
                out.append({"kind": "fixed-moved", "id": nid, "start": a["t"], "end": b["t"], "detail": f"{na['rect']} -> {nb['rect']}"})
    return out


def main():
    args, cuts, samples = sys.argv[1:], [], None
    while "--cut" in args:
        i = args.index("--cut")
        cuts.append(tuple(float(v) for v in args[i + 1].split(",")))
        del args[i : i + 2]
    if "--samples" in args:
        i = args.index("--samples")
        samples = json.load(open(args[i + 1]))
        del args[i : i + 2]
    if len(args) != 1:
        sys.exit(__doc__)
    found = check(args[0], cuts) + (fixed_in_move(samples) if samples else [])
    for f in found:
        print(f"{f['kind']:<12} {f['start']:7.2f} {f['end']:7.2f} {f.get('detail', f.get('id', ''))}")
    print(f"tour-video-check: {len(found)} flagged")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
