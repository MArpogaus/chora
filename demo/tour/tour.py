#!/usr/bin/env python3
"""The recorder of the tour: acts with a pace, stills, timeline rows.

An act is a list of cues for `choractl record --script`. The cues of a
segment run with one `choractl record`; between segments the script reads
the state of the session (queries), so that the next act aims at what is
on the screen. Frames become an H.264 chunk and are deleted at once.
"""

import json
import math
import os
import shutil
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO_ROOT, "tests", "acts"))
FPS = 30
MOVE = float(os.environ.get('TOUR_MOVE', '1.0'))
CPS = float(os.environ.get('TOUR_CPS', '6'))
LEAD = float(os.environ.get('TOUR_LEAD', '1.0'))
SMOOTH = lambda k: k * k * (3 - 2 * k)


def pair_filter():
    return os.environ.get("TOUR_PAIR_FILTER", "[0][1]hstack=inputs=2,scale=1280:360,pad=1280:720:0:180:black")


def slide_size():
    return os.environ.get("TOUR_SLIDE_SIZE", "")


def checking():
    """`run.py --check`: the rows run and are judged, nothing is encoded."""
    return bool(os.environ.get("TOUR_CHECK"))


def mmss(frame):
    seconds = int(frame / FPS)
    return f"{seconds // 60:02d}:{seconds % 60:02d}"


class Session:
    """A running chora session of this tour: its socket and its pids."""

    def __init__(self, env, pids, bin_dir):
        self.env = env
        self.pids = pids
        self.bin = bin_dir

    def ctl(self, *args, stdin=None, timeout=30, check=True):
        p = subprocess.run(
            [f"{self.bin}/choractl", *args],
            input=stdin,
            capture_output=True,
            text=True,
            timeout=timeout,
            env=self.env,
        )
        if check and p.returncode != 0:
            raise RuntimeError(f"choractl {' '.join(args)}: {p.stdout} {p.stderr}")
        return p.stdout

    def call(self, client, method, params=None):
        out = self.ctl("call", client, method, json.dumps(params or {}))
        try:
            return json.loads(out)
        except ValueError:
            return out

    def query(self, selector):
        out = self.ctl("query", selector, check=False)
        try:
            return json.loads(out)
        except ValueError:
            return []

    def rect(self, selector, output=None):
        """The rect [x, y, w, h] of the first node that matches."""
        nodes = self.query(selector)
        for node in nodes:
            rects = node.get("rects") or {}
            if output and output in rects:
                return rects[output]
            if rects:
                return next(iter(rects.values()))
        return None

    def centre(self, selector, output=None):
        r = self.rect(selector, output)
        return None if r is None else (r[0] + r[2] / 2, r[1] + r[3] / 2)


class Checker:
    """Judges an act: `Expect.check` of the act harness on the snapshots before and after, and every
    invariant of invariants.ALL. `source` answers a `harness.Snapshot`; `Checker.attach` reads the
    live session of the tour as the persona check does."""

    def __init__(self, source):
        self.source = source

    @classmethod
    def attach(cls, env, looks, runs, wallpaper=None):
        import pathlib

        import harness

        h = harness.Session(*looks, runs=runs, wallpaper=wallpaper)
        h.attached = True
        h.work = pathlib.Path(runs)
        h.env = dict(env)
        h.runtime = pathlib.Path(env["XDG_RUNTIME_DIR"])
        h.connect(env["CHORA_SOCKET"], "tour-check")
        return cls(h.snapshot)

    def snapshot(self):
        return self.source()

    def judge(self, row, expect, before, after):
        import harness
        import invariants

        act = harness.Act(row["description"], list(row.get("opens") or ()), list(row.get("closes") or ()), list(row.get("disconnects") or ()), False)
        red = expect.check(before, after) if expect else []
        for key, check in invariants.ALL.items():
            red += [f"{key}: {b}" for b in check(before, after, act)]
        return red


def checker_for(session, looks, work, wallpaper=None):
    """The checker of a live session, or None (said once) when the harness cannot attach."""
    try:
        for name in ("tour-check", "tour-check-events"):
            session.ctl("grant", name, "screen.capture", "input.inject", "methods.call", "methods.privileged")
        return Checker.attach(session.env, looks, os.path.join(work, "evidence"), wallpaper)
    except Exception as err:  # noqa: BLE001
        print(f"checker: cannot attach, acts are not judged: {type(err).__name__}: {err}", flush=True)
        return None


class Tour:
    def __init__(self, session, out_dir, work_dir, checker=None):
        self.s = session
        self.out = out_dir
        self.work = work_dir
        os.makedirs(self.out, exist_ok=True)
        os.makedirs(os.path.join(self.out, "stills"), exist_ok=True)
        os.makedirs(self.work, exist_ok=True)
        self.rows = []
        self.chapter = None
        self.output = None
        self.px, self.py = 640.0, 360.0
        self.stills = 0
        self.act_no = 0
        self.chunks = []
        self.cues = []
        self.frame = 0
        self.stills_wanted = []
        self.chapter_frames = 0
        self.chunk_no = 0
        self.pair = None
        self.pointer_output = None
        self.cuts = []
        self.checker = checker
        self.before = None
        self.expects = {}
        self.armed = []

    def begin(self, chapter, slug):
        self.chapter = chapter
        self.slug = slug
        self.chunks = []
        self.cues = []
        self.frame = 0
        self.chapter_frames = 0
        self.act_no = 0
        self.stills_wanted = []
        self.cuts = []

    def cut(self, seconds):
        """Let `seconds` pass on the clock but keep them out of the video."""
        start = self.frame + 1
        self.wait(seconds)
        self.cuts.append((start, self.frame))

    def cut_before(self, frame):
        return sum(min(b, frame - 1) - a + 1 for a, b in self.cuts if a < frame)

    def apply_cuts(self, frames_dir, count):
        if not self.cuts:
            return count
        drop = set()
        for a, b in self.cuts:
            drop.update(range(a, b + 1))
        keep = [n for n in range(1, count + 1) if n not in drop]
        for new, old in enumerate(keep, 1):
            src = os.path.join(frames_dir, f"frame-{old:05d}.png")
            if new != old:
                os.replace(src, os.path.join(frames_dir, f"frame-{new:05d}.png"))
        for n in range(len(keep) + 1, count + 1):
            path = os.path.join(frames_dir, f"frame-{n:05d}.png")
            if os.path.exists(path):
                os.remove(path)
        return len(keep)

    def cue(self, method, params, frame=None):
        self.cues.append((self.frame if frame is None else frame, method, params))

    def inject(self, events):
        if self.pointer_output or self.output:
            for e in events:
                if "pointer_move" in e and "position" in e["pointer_move"]:
                    e["pointer_move"].setdefault("output", self.pointer_output or self.output)
        self.cue("input.inject", {"events": events})

    def wait(self, seconds):
        self.frame += max(1, round(seconds * FPS))

    def jump(self, x, y):
        self.px, self.py = float(x), float(y)
        self.inject([{"pointer_move": {"position": [round(x, 1), round(y, 1)]}}])
        self.frame += 1

    def move(self, x, y, seconds=1.0):
        n = max(1, round(seconds * FPS * MOVE))
        x0, y0 = self.px, self.py
        for i in range(1, n + 1):
            k = SMOOTH(i / n)
            self.cue_at(i, "input.inject", {"events": [self._pm(x0 + (x - x0) * k, y0 + (y - y0) * k)]})
        self.frame += n
        self.px, self.py = float(x), float(y)

    def _pm(self, x, y):
        e = {"pointer_move": {"position": [round(x, 1), round(y, 1)]}}
        if self.pointer_output or self.output:
            e["pointer_move"]["output"] = self.pointer_output or self.output
        return e

    def cue_at(self, offset, method, params):
        self.cues.append((self.frame + offset, method, params))

    def click(self, button=272):
        self.inject([{"click": {"button": button}}])
        self.frame += 1

    def key(self, combo):
        self.inject([{"shortcut": {"keys": combo}}])
        self.frame += 1

    def keys(self, names, gap=0.5):
        for name in names:
            self.key(name)
            self.wait(gap)

    def text(self, text, cps=None):
        cps = cps or CPS
        per = max(1, round(FPS / cps))
        for ch in text:
            if ch == "\n":
                self.inject([{"key": {"key": "Return", "state": "pressed"}}, {"key": {"key": "Return", "state": "released"}}])
            else:
                self.inject([{"text": {"text": ch}}])
            self.frame += per

    def hold(self, key, seconds):
        self.inject([{"key": {"key": key, "state": "pressed"}}])
        self.wait(seconds)
        self.inject([{"key": {"key": key, "state": "released"}}])
        self.frame += 1

    def call(self, client, method, params=None):
        self.cue("methods.call", {"client": client, "method": method, "params": params or {}})
        self.frame += 1

    def drag(self, x1, y1, x2, y2, seconds=1.5, mods=(), button=272, hold=0.3, mid_still=None):
        """Press at the start, move over `seconds`, release at the end."""
        self.move(x1, y1, 1.0)
        presses = [{"key": {"key": m, "state": "pressed"}} for m in mods]
        presses.append({"button": {"button": button, "state": "pressed", "hold": True}})
        self.inject(presses)
        self.wait(hold)
        if mid_still:
            xm, ym = x1 + (x2 - x1) * 0.6, y1 + (y2 - y1) * 0.6
            self.move(xm, ym, seconds * 0.6)
            self.still(mid_still)
            if self.rows and not self.rows[-1].get('still'):
                self.rows[-1]['still'] = mid_still
            self.move(x2, y2, seconds * 0.4)
        else:
            self.move(x2, y2, seconds)
        self.wait(hold)
        self.inject([{"button": {"button": button, "state": "released"}}] + [{"key": {"key": m, "state": "released"}} for m in reversed(mods)])
        self.frame += 1

    def still(self, name, after=0.0):
        """A still at the current frame (plus `after` seconds)."""
        at = self.frame + round(after * FPS)
        self.stills_wanted.append((max(1, at), name))
        return at

    def act(self, kind, description, expected, still=None, settle=0.8, lead=None, tail=None, *, expect=None, contract=None, cells=(), capture=None, todo=False, known=None, opens=(), closes=(), disconnects=()):
        """Open an act: pause, mark its start. Use with a following `done`.

        An act is a row of design/tour-v9.md: `expect` (an `Expect` of tests/acts/harness.py,
        judged with every invariant at the next read of the session after `done`), `contract`
        (a persona contract, T1..M2), `cells` (the gate cells the act repeats on screen),
        `capture` (Q, Q+t, H50), `todo` (a stub for an act that waits for xp-give: no
        check, a `todo` row) and `known` (why a red act is red today, the way a row of
        tests/acts/known-red.toml does: shown as `known`, not counted as a failure; a `known`
        act that turns green says so, so the row can go)."""
        lead = LEAD if lead is None else lead
        tail = LEAD if tail is None else tail
        if not todo:
            self.wait(lead)
        self.act_no += 1
        row = {
            "chapter": self.chapter,
            "act": self.act_no,
            "frame": self.frame + 1,
            "kind": kind,
            "description": description,
            "expected": expected,
            "still": None,
            "happened": "",
            "contract": contract,
            "cells": list(cells),
            "capture": capture,
            "expect": repr(expect) if expect is not None else None,
            "status": "todo" if todo else "checked",
            "red": [],
            "known": known,
            "opens": list(opens),
            "closes": list(closes),
            "disconnects": list(disconnects),
        }
        if not todo:
            import harness

            self.expects[id(row)] = expect or harness.Expect()
        self.rows.append(row)
        self._open = (row, still, settle, tail)
        return row

    def done(self):
        row, still, settle, tail = self._open
        if row["status"] == "todo":
            return
        self.wait(settle)
        if still:
            self.still(still)
            row["still"] = still
        self.wait(tail)
        if id(row) in self.expects:
            self.armed.append(row)
            if self.checker:
                self.flush("act")

    def check_armed(self):
        """Judge the acts that are done: the state of the session now against the state at the last
        read. A red act is a red row of the timeline (`row["red"]`, status `RED`)."""
        if not self.checker or not self.armed:
            return
        try:
            after = self.checker.snapshot()
        except Exception as err:  # noqa: BLE001
            after = None
            note = f"no snapshot: {type(err).__name__}: {err}"
        for row in self.armed:
            if after is None:
                row["red"] = [note]
            else:
                row["red"] = self.checker.judge(row, self.expects.pop(id(row)), self.before or after, after)
            row["status"] = ("known" if row.get("known") else "RED") if row["red"] else "green"
            if row["red"] and row.get("known"):
                print(f"known act {row['act']} {row['description']}: {row['known']}", flush=True)
            elif row["red"]:
                print(f"RED act {row['act']} {row['description']}: {row['red']}", flush=True)
        self.armed = []
        if after is not None:
            self.before = after

    def run_segment(self, label=""):
        """Record the pending cues as one chunk, keep the stills."""
        if self.frame <= 0:
            return
        count = self.frame + 1
        frames_dir = os.path.join(self.work, "frames")
        shutil.rmtree(frames_dir, ignore_errors=True)
        os.makedirs(frames_dir)
        cue_file = os.path.join(self.work, "cues.jsonl")
        with open(cue_file, "w", encoding="utf-8") as f:
            for frame, method, params in sorted(self.cues, key=lambda c: c[0]):
                f.write(json.dumps({"frame": max(1, frame), "method": method, "params": params}) + "\n")
        started = time.time()
        if self.pair:
            import dual
            dirs = []
            for k in range(2):
                d = os.path.join(self.work, f"frames{k}")
                shutil.rmtree(d, ignore_errors=True)
                os.makedirs(d)
                dirs.append(d)
            cue_list = [(max(1, f), m, p) for f, m, p in self.cues]
            dual.record(self.s.env["CHORA_SOCKET"], FPS, count, self.pair, dirs, cue_list)
            took = time.time() - started
            if checking():
                for d in dirs:
                    shutil.rmtree(d, ignore_errors=True)
                self.stills_wanted, self.cues, self.frame, self.cuts = [], [], 0, []
                return
            for at, name in self.stills_wanted:
                for k, tag in enumerate(("L", "R")):
                    src = os.path.join(dirs[k], f"frame-{min(at, count):05d}.png")
                    if os.path.exists(src):
                        shutil.copy(src, os.path.join(self.out, "stills", f"{name}-{tag}.png"))
            self.stills_wanted = []
            self.chunk_no += 1
            chunk = os.path.join(self.work, f"{self.slug}-{self.chunk_no:03d}.mkv")
            subprocess.run(
                ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-framerate", str(FPS), "-i", os.path.join(dirs[0], "frame-%05d.png"),
                 "-framerate", str(FPS), "-i", os.path.join(dirs[1], "frame-%05d.png"),
                 "-filter_complex", pair_filter(),
                 "-c:v", "libx264", "-preset", "veryfast", "-crf", "16", "-pix_fmt", "yuv420p", chunk],
                check=True,
            )
            for d in dirs:
                shutil.rmtree(d, ignore_errors=True)
            self.chunks.append(chunk)
            self.chapter_frames += count
            print(f"segment {label}: {count} frames in {took:.0f}s (two screens)", flush=True)
            self.cues = []
            self.frame = 0
            return
        args = ["record", "--fps", str(FPS), "--frames", str(count), "--out", frames_dir, "--script", cue_file, "--pointer", "arrow"]
        if self.output:
            args += ["--output", self.output]
        self.s.ctl(*args, timeout=3600)
        took = time.time() - started
        if checking():
            shutil.rmtree(frames_dir, ignore_errors=True)
            self.stills_wanted, self.cues, self.frame, self.cuts = [], [], 0, []
            return
        for at, name in self.stills_wanted:
            src = os.path.join(frames_dir, f"frame-{min(at, count):05d}.png")
            if os.path.exists(src):
                shutil.copy(src, os.path.join(self.out, "stills", f"{name}.png"))
        self.stills_wanted = []
        count = self.apply_cuts(frames_dir, count)
        self.chunk_no += 1
        chunk = os.path.join(self.work, f"{self.slug}-{self.chunk_no:03d}.mkv")
        subprocess.run(
            ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-framerate", str(FPS), "-i",
             os.path.join(frames_dir, "frame-%05d.png"), "-c:v", "libx264", "-preset", "veryfast", "-crf", "16",
             "-pix_fmt", "yuv420p", chunk],
            check=True,
        )
        shutil.rmtree(frames_dir, ignore_errors=True)
        self.chunks.append(chunk)
        self.chapter_frames += count
        print(f"segment {label}: {count} frames in {took:.0f}s ({count / max(took, 0.1):.1f} fps)", flush=True)
        self.cues = []
        self.frame = 0
        self.cuts = []

    def flush(self, label=""):
        """Record what is pending; rows keep their time in the chapter."""
        if self.checker and self.before is None:
            try:
                self.before = self.checker.snapshot()
            except Exception:  # noqa: BLE001
                self.before = None
        offset = self.chapter_frames
        for row in self.rows:
            if "abs" not in row and row["chapter"] == self.chapter:
                row["abs"] = offset + row["frame"] - self.cut_before(row["frame"])
        self.run_segment(label)
        self.check_armed()

    def finish(self, mp4_name, slide=None):
        self.flush("last")
        if checking():
            return None
        if slide:
            seconds = 2.5
            chunk = os.path.join(self.work, f"{self.slug}-slide.mkv")
            subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-loop", "1", "-framerate", str(FPS), "-t", str(seconds), "-i", slide,
                            *(["-vf", f"scale={slide_size()}"] if slide_size() else []), "-c:v", "libx264", "-preset", "veryfast", "-crf", "16", "-pix_fmt", "yuv420p", chunk], check=True)
            self.chunks.insert(0, chunk)
            for row in self.rows:
                if row["chapter"] == self.chapter and "abs" in row:
                    row["abs"] += round(seconds * FPS)
        listing = os.path.join(self.work, f"{self.slug}.list")
        with open(listing, "w", encoding="utf-8") as f:
            for chunk in self.chunks:
                f.write(f"file '{chunk}'\n")
        out = os.path.join(self.out, mp4_name)
        subprocess.run(
            ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-f", "concat", "-safe", "0", "-i", listing,
             "-c:v", "libx264", "-preset", "slow", "-crf", "28", "-pix_fmt", "yuv420p", "-movflags", "+faststart", out],
            check=True,
        )
        return out
