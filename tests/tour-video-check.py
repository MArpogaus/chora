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
Exit 2 with "ffmpeg missing: cannot check" when ffmpeg or ffprobe is not on PATH.

The thresholds, from the six v8 chapter videos (media/2026-11-17/tour-v8, 30 fps, 1280x720):
- FLICKER_STEP 8.0: the mean grey step between two frames at 32x18 is 0.00 at the median (the
  chapters hold stills), 0.85 to 1.6 at p99 in 03-mc-light and 03b-mc-dark, 4.8 to 11 at p99 in
  01-xp, 02-tiling and 04-construct (windows and the cube moving), 22.9 at p99 in 05-handover; a
  whole-frame flash steps 80 to 210 (03-mc-light max 211, 01-xp max 140). 8 sits above the p99 of
  the calm chapters and far below a flash; the second condition (the frames around the flash are
  alike, FLICKER_RATIO) keeps real motion, which does not come back, from being flagged.
- FLICKER_RATIO 2: the step from the frame before the flash to the frame after it must be under
  half of the smaller step into and out of the flash. testsrc2 moves 5 to 7 per frame and 14 over
  three frames, a flash on it steps 54 to 55, so it is found; a step that stays or a move gives a
  back step as large as the steps in.
- FLICKER_RUN 2: a flash of one or two frames (a longer one reads as a cut or a slide).
- SIZE 32x18: 16:9 at 1/40; 576 pixels average a window opening (a 200x150 window is 3 % of the
  frame, 4 grey levels of the mean step) and drop noise of the encoder (the median step is 0).
- BLACK_PIX 0.1: blackdetect counts a pixel black at luma 25 or below; v8 frames of the dark theme
  have a darkest pixel of 10 to 28 (03b-mc-dark) but never 98 % of the frame under 25, so the six
  videos give 0 black stretches, and a planted black stretch is found.
"""

import json
import re
import shutil
import subprocess
import sys

FREEZE_SECONDS = 1.5
FLICKER_STEP = 8.0
FLICKER_RATIO = 2
FLICKER_RUN = 2
RECT_SLACK = 2
SIZE = (32, 18)
BLACK_PIX = 0.1


def ffmpeg(path, vf, *extra):
    return subprocess.run(["ffmpeg", "-hide_banner", "-nostats", "-i", path, *vf, *extra, "-f", "null", "-"], capture_output=True, text=True, check=True).stderr


def fps_of(path):
    out = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=r_frame_rate:format=duration", "-of", "json", path], capture_output=True, text=True, check=True).stdout
    data = json.loads(out)
    num, den = data["streams"][0]["r_frame_rate"].split("/")
    return int(num) / int(den), float(data["format"]["duration"])


def black(path, fps):
    log = ffmpeg(path, ["-vf", f"blackdetect=d={1.9 / fps:.4f}:pix_th={BLACK_PIX}"])
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
    """A run of 1 to FLICKER_RUN frames that differs from the frame before and the frame after, which are alike."""
    grey, out = frames(path), []
    for run in range(1, FLICKER_RUN + 1):
        for i in range(1, len(grey) - run):
            before, after = grey[i - 1], grey[i + run]
            into, out_of = step(before, grey[i]), step(grey[i + run - 1], after)
            if min(into, out_of) > FLICKER_STEP and step(before, after) < min(into, out_of) / FLICKER_RATIO:
                if not any(f["start"] <= i / fps < f["end"] for f in out):
                    out.append({"kind": "flicker", "start": i / fps, "end": (i + run) / fps})
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
    if not (shutil.which("ffmpeg") and shutil.which("ffprobe")):
        print("tour-video-check: ffmpeg missing: cannot check", file=sys.stderr)
        return 2
    found = check(args[0], cuts) + (fixed_in_move(samples) if samples else [])
    for f in found:
        print(f"{f['kind']:<12} {f['start']:7.2f} {f['end']:7.2f} {f.get('detail', f.get('id', ''))}")
    print(f"tour-video-check: {len(found)} flagged")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
