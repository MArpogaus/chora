#!/usr/bin/python3
"""Selftest of the tour comparison tools (design/tour-v9.md section 6): each planted case is
flagged by the tool (a moved node, a changed blob, a flicker frame, a freeze, a black stretch,
a screen-fixed node that moves in a camera move) and a clean case is not.

Usage: python3 -I tests/test_tour_diff.py [-v]
"""

import importlib.util
import pathlib
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
import zlib

HERE = pathlib.Path(__file__).resolve().parent


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), HERE / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


diff = load("tour-diff")
video = load("tour-video-check")


def png(width, height, paint=None):
    """An 8-bit RGBA PNG, grey 100, with paint(x, y) -> (r, g, b) where it returns one."""
    raw = bytearray()
    for y in range(height):
        raw.append(0)
        for x in range(width):
            raw += bytes(paint(x, y) if paint and paint(x, y) else (100, 100, 100)) + b"\xff"

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    head = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", head) + chunk(b"IDAT", zlib.compress(bytes(raw))) + chunk(b"IEND", b"")


def box(x, y, w, h):
    return lambda px, py: (250, 0, 0) if x <= px < x + w and y <= py < y + h else None


def node(nid, rect, **kw):
    return {"id": nid, "kind": "node", "owner": "xp", "meta": {}, "rects": {"HEADLESS-1": list(rect)}, **kw}


class SceneDiff(unittest.TestCase):
    def test_moved_node_is_flagged(self):
        a = [node("xp-bar", (0, 0, 100, 20)), node("xp-start", (0, 780, 80, 20))]
        b = [node("xp-bar", (0, 0, 100, 20)), node("xp-start", (0, 760, 80, 20))]
        found = diff.scene_diff(a, b)
        self.assertEqual([(d["kind"], d["id"]) for d in found], [("rect", "xp-start")])

    def test_a_move_of_two_pixels_is_not(self):
        a = [node("xp-bar", (0, 0, 100, 20))]
        b = [node("xp-bar", (2, -2, 100, 20))]
        self.assertEqual(diff.scene_diff(a, b), [])

    def test_hidden_text_owner_and_missing_are_flagged(self):
        a = [node("n1", (0, 0, 9, 9)), node("n2", (0, 0, 9, 9)), node("n3", (0, 0, 9, 9), meta={"text": "Start"}), node("n4", (0, 0, 9, 9))]
        b = [node("n1", (0, 0, 9, 9), visible=False), node("n2", (0, 0, 9, 9), owner="mc"), node("n3", (0, 0, 9, 9), meta={"text": "Apps"})]
        kinds = sorted((d["kind"], d["id"]) for d in diff.scene_diff(a, b))
        self.assertEqual(kinds, [("meta", "n3"), ("missing", "n4"), ("owner", "n2"), ("visible", "n1")])

    def test_windows_match_by_app_id_and_title(self):
        def win(sid, x):
            return {"id": sid, "kind": "surface", "owner": "xp", "meta": {"title": "Notepad"}, "surface": {"app_id": "mousepad"}, "rects": {"HEADLESS-1": [x, 50, 400, 300]}}

        self.assertEqual(diff.scene_diff([win("s1", 10)], [win("s77", 10)]), [])
        self.assertEqual([d["kind"] for d in diff.scene_diff([win("s1", 10)], [win("s77", 90)])], ["rect"])

    def test_z_order_of_top_layers(self):
        a = [node("a", (0, 0, 9, 9)), node("b", (0, 0, 9, 9))]
        b = [node("b", (0, 0, 9, 9)), node("a", (0, 0, 9, 9))]
        self.assertEqual([d["kind"] for d in diff.scene_diff(a, b)], ["z-order"])


class PixelDiff(unittest.TestCase):
    def test_changed_blob_is_flagged(self):
        a, b = png(200, 100), png(200, 100, box(50, 40, 20, 20))
        result = diff.pixel_diff(a, b)
        self.assertFalse(result["same"])
        self.assertEqual(result["blobs"], [(50, 40, 20, 20)])

    def test_small_change_is_same(self):
        a, b = png(200, 100), png(200, 100, box(50, 40, 5, 5))
        self.assertTrue(diff.pixel_diff(a, b)["same"])

    def test_masked_change_is_same(self):
        a, b = png(200, 100), png(200, 100, box(50, 40, 20, 20))
        self.assertTrue(diff.pixel_diff(a, b, masks=[(40, 30, 50, 50)])["same"])

    def test_many_scattered_pixels_are_a_change(self):
        a, b = png(100, 100), png(100, 100, lambda x, y: (250, 0, 0) if x % 5 == 0 and y % 5 == 0 else None)
        result = diff.pixel_diff(a, b)
        self.assertFalse(result["same"])
        self.assertEqual(result["blobs"], [])

    def test_below_the_channel_threshold_is_same(self):
        a, b = png(100, 100), png(100, 100, lambda x, y: (120, 100, 100))
        self.assertTrue(diff.pixel_diff(a, b)["same"])


class Verdict(unittest.TestCase):
    intended = [{"act": "x7", "kind": "rect", "cause": "xp-give Send To row"}]

    def test_same(self):
        self.assertEqual(diff.verdict("x1", [], True, self.intended)["verdict"], "same")

    def test_unnamed_diff_is_a_regression(self):
        v = diff.verdict("x1", [{"kind": "rect", "id": "n"}], True, self.intended)
        self.assertEqual(v["verdict"], "regression")

    def test_named_diff_is_a_change(self):
        v = diff.verdict("x7", [{"kind": "rect", "id": "n"}], True, self.intended)
        self.assertEqual((v["verdict"], v["cause"]), ("change", "xp-give Send To row"))

    def test_a_pixel_diff_needs_a_pixel_row(self):
        v = diff.verdict("x7", [], False, self.intended)
        self.assertEqual(v["verdict"], "regression")
        v = diff.verdict("x7", [], False, [{"act": "x7", "kind": "pixel", "cause": "N-6 fixed"}])
        self.assertEqual(v["verdict"], "change")

    def test_a_check_green_on_replay_and_red_on_v9_is_a_regression(self):
        v = diff.verdict("x1", [], True, [], checks=({"I4": "green"}, {"I4": "red"}))
        self.assertEqual(v["verdict"], "regression")

    def test_compare_dirs(self):
        with tempfile.TemporaryDirectory() as tmp:
            base, cand = pathlib.Path(tmp, "b"), pathlib.Path(tmp, "c")
            for d, shape in ((base, None), (cand, box(50, 40, 20, 20))):
                d.mkdir()
                (d / "x1.Q.png").write_bytes(png(200, 100, shape))
                (d / "x1.Q.json").write_text('{"nodes": []}')
            rows = diff.compare(base, cand, [])
            self.assertEqual([(r["act"], r["capture"], r["verdict"]) for r in rows], [("x1", "Q", "regression")])


@unittest.skipUnless(shutil.which("ffmpeg"), "no ffmpeg")
class VideoCheck(unittest.TestCase):
    def clip(self, *parts):
        """An mp4 of lavfi sources joined in order, 10 fps 160x90."""
        tmp = tempfile.mkdtemp(prefix="tour-video-")
        files = []
        for i, (src, secs) in enumerate(parts):
            path = f"{tmp}/p{i}.mp4"
            subprocess.run(["ffmpeg", "-v", "error", "-f", "lavfi", "-i", f"{src}:s=160x90:r=10:d={secs}", "-pix_fmt", "yuv420p", "-y", path], check=True)
            files.append(path)
        listing = pathlib.Path(tmp, "list.txt")
        listing.write_text("".join(f"file '{f}'\n" for f in files))
        out = f"{tmp}/clip.mp4"
        subprocess.run(["ffmpeg", "-v", "error", "-f", "concat", "-safe", "0", "-i", str(listing), "-c", "copy", "-y", out], check=True)
        return out

    def kinds(self, path, **kw):
        return sorted({f["kind"] for f in video.check(path, **kw)})

    def test_moving_clip_is_clean(self):
        self.assertEqual(self.kinds(self.clip(("testsrc2", 4))), [])

    def test_freeze_is_flagged(self):
        clip = self.clip(("testsrc2", 2), ("color=c=0x336699", 2.5), ("testsrc2", 2))
        found = [f for f in video.check(clip) if f["kind"] == "freeze"]
        self.assertTrue(found)
        self.assertAlmostEqual(found[0]["start"], 2.0, delta=0.3)

    def test_freeze_inside_a_cut_is_not(self):
        clip = self.clip(("testsrc2", 2), ("color=c=0x336699", 2.5), ("testsrc2", 2))
        self.assertEqual([f for f in video.check(clip, cuts=[(2.0, 4.5)]) if f["kind"] == "freeze"], [])

    def test_black_stretch_is_flagged(self):
        clip = self.clip(("testsrc2", 2), ("color=c=black", 0.5), ("testsrc2", 2))
        self.assertIn("black", self.kinds(clip))

    def test_one_black_frame_is_not(self):
        clip = self.clip(("testsrc2", 2), ("color=c=black", 0.1), ("testsrc2", 2))
        self.assertNotIn("black", self.kinds(clip))

    def test_flicker_frame_is_flagged(self):
        clip = self.clip(("color=c=0x336699", 1), ("color=c=0xcc6633", 0.1), ("color=c=0x336699", 1))
        found = [f for f in video.check(clip, cuts=[(0, 3)]) if f["kind"] == "flicker"]
        self.assertEqual(len(found), 1)
        self.assertAlmostEqual(found[0]["start"], 1.0, delta=0.15)

    def test_a_step_that_stays_is_not_a_flicker(self):
        clip = self.clip(("color=c=0x336699", 1), ("color=c=0xcc6633", 1))
        self.assertEqual([f for f in video.check(clip, cuts=[(0, 3)]) if f["kind"] == "flicker"], [])


class FixedInCameraMove(unittest.TestCase):
    def samples(self, hud_x):
        return [
            {"t": 0.0, "camera": [0, 0, 0], "nodes": {"hud-clock": {"fixed": True, "rect": [10, 10, 50, 20]}, "cube": {"fixed": False, "rect": [100, 0, 50, 50]}}},
            {"t": 0.5, "camera": [20, 0, 0], "nodes": {"hud-clock": {"fixed": True, "rect": [hud_x, 10, 50, 20]}, "cube": {"fixed": False, "rect": [140, 0, 50, 50]}}},
        ]

    def test_fixed_node_that_moves_is_flagged(self):
        found = video.fixed_in_move(self.samples(60))
        self.assertEqual([(f["kind"], f["id"]) for f in found], [("fixed-moved", "hud-clock")])

    def test_fixed_node_that_stays_is_clean(self):
        self.assertEqual(video.fixed_in_move(self.samples(11)), [])


if __name__ == "__main__":
    unittest.main()
