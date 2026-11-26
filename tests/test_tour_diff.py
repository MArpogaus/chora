#!/usr/bin/python3
"""Selftest of the tour comparison tools (design/tour-v9.md section 6): each planted case is
flagged by the tool (a moved node, a changed blob, a flicker frame, a freeze, a black stretch,
a screen-fixed node that moves in a camera move) and a clean case is not.

Usage: python3 -I tests/test_tour_diff.py [-v]
"""

import importlib.util
import json
import os
import pathlib
import re
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

    def test_the_tray_clock_of_two_takes_twenty_minutes_apart_is_no_diff(self):
        """TOUR-CLOCK: the xp tray (xp-tray-<output>) carries the wall clock as its meta text; the two
        runs of a comparison never share a minute (r1: 91 meta diffs of "2:22 PM" and "2:40 PM")."""
        a = [node("xp-tray-HEADLESS-1", (0, 0, 1, 1), meta={"text": "2:22 PM"}), node("xp-tray-VIRTUAL-2", (0, 0, 1, 1), meta={"text": "2:22 PM"})]
        b = [node("xp-tray-HEADLESS-1", (0, 0, 1, 1), meta={"text": "2:42 PM"}), node("xp-tray-VIRTUAL-2", (0, 0, 1, 1), meta={"text": "2:42 PM"})]
        self.assertEqual(diff.scene_diff(a, b), [])

    def test_the_clock_exemption_is_by_node_id_only(self):
        a = [node("xp-tray-fake", (0, 0, 1, 1), meta={"text": "2:22 PM"}), node("xp-item-3", (0, 0, 1, 1), meta={"text": "UXTerm"})]
        b = [node("xp-tray-fake", (0, 0, 1, 1), meta={"text": "2:42 PM"}), node("xp-item-3", (0, 0, 1, 1), meta={"text": "Foot"})]
        self.assertEqual(sorted(d["id"] for d in diff.scene_diff(a, b)), ["xp-item-3", "xp-tray-fake"])

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


class CutText(unittest.TestCase):
    """rv9ka-3: a text node whose rect is shorter than its lines times its line height is cut."""

    @staticmethod
    def text(nid, h, text="For two\n200 g spaghetti\n4 cloves of garlic\n6 tbsp olive oil"):
        return node(nid, (160, 530, 164, h), kind="text", text=text, font_size=19.0, line_height=1.5)

    def test_a_text_that_loses_lines_is_a_cut(self):
        a, b = [self.text("list-lines", 114)], [self.text("list-lines", 29)]
        found = [(d["kind"], d["id"]) for d in diff.scene_diff(a, b)]
        self.assertIn(("cut", "list-lines"), found)

    def test_a_whole_text_is_no_cut(self):
        a, b = [self.text("list-lines", 114)], [self.text("list-lines", 112)]
        self.assertEqual([d for d in diff.scene_diff(a, b) if d["kind"] == "cut"], [])

    def test_a_text_without_line_height_has_the_core_default(self):
        """rtr1-4: the field is skipped when None; the core default is 1.2."""
        def t(h):
            n = self.text("list-lines", h)
            del n["line_height"]
            return n
        found = [(d["kind"], d["id"]) for d in diff.scene_diff([t(91)], [t(29)])]
        self.assertIn(("cut", "list-lines"), found)

    def test_a_text_cut_in_both_builds_but_cut_more_is_a_cut(self):
        """rtr1-4: v8 shows 2 of 4 lines, v9 shows 1."""
        a, b = [self.text("list-lines", 57)], [self.text("list-lines", 29)]
        found = [(d["kind"], d["id"]) for d in diff.scene_diff(a, b)]
        self.assertIn(("cut", "list-lines"), found)

    def test_no_row_absorbs_a_cut_unless_it_names_the_node(self):
        scene = [{"kind": "cut", "id": "list-lines", "detail": "29 px for 4 lines"}]
        self.assertEqual(diff.verdict("m1", scene, True, [{"act": "m1", "kind": "rect", "cause": "x", "open": True}])["verdict"], "regression")
        for rows in ([{"act": "m1", "kind": "*", "cause": "x"}], [{"act": "m1", "kind": "cut", "cause": "x"}], [{"act": "m1", "kind": "cut", "cause": "x", "nodes": ["other"]}]):
            self.assertNotEqual(diff.verdict("m1", scene, True, rows)["verdict"], "change", rows)
        named = [{"act": "m1", "kind": "cut", "cause": "x", "nodes": ["list-lines"]}]
        self.assertEqual(diff.verdict("m1", scene, True, named)["verdict"], "change")


class PixelDiff(unittest.TestCase):
    def test_changed_blob_is_flagged(self):
        a, b = png(200, 100), png(200, 100, box(50, 40, 20, 20))
        result = diff.pixel_diff(a, b)
        self.assertFalse(result["same"])
        self.assertEqual(result["blobs"], [(50, 40, 20, 20)])

    def test_small_change_is_same(self):
        a, b = png(200, 100), png(200, 100, box(50, 40, 3, 3))
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
    intended = [{"act": "x7", "kind": "rect", "cause": "xp-give Send To row", "nodes": ["n"]}]

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
        v = diff.verdict("x7", [], False, [{"act": "x7", "kind": "pixel", "cause": "N-6 fixed", "boxes": [[0, 0, 100, 100]]}], blobs=[(10, 10, 20, 20)])
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


class PaperclipBlink(unittest.TestCase):
    """rv9kb-1: the blink of the paperclip runs on the wall clock, so a still catches shut eyes at random."""

    figure = {"id": "hud-figure", "kind": "image", "owner": "xp", "meta": {}, "rects": {"HEADLESS-1": [1192, 578, 80, 104]}}

    def test_the_eyes_of_the_figure_are_masked(self):
        masks = diff.masks_of({"nodes": [self.figure]})
        self.assertTrue(any(m[0] <= 1213 and m[1] <= 593 and m[0] + m[2] >= 1252 and m[1] + m[3] >= 613 for m in masks), masks)

    def test_a_blink_is_no_pixel_diff_and_the_body_still_is(self):
        side = {"nodes": [self.figure]}
        masks = diff.masks_of(side, side)
        shut = diff.pixel_diff(png(1280, 720), png(1280, 720, box(1213, 593, 19, 20)), masks)
        self.assertTrue(shut["same"], shut)
        moved = diff.pixel_diff(png(1280, 720), png(1280, 720, box(1200, 640, 40, 30)), masks)
        self.assertFalse(moved["same"])


class BarClock(unittest.TestCase):
    """RV9-1: the bar of the tiling look is a foreign surface (waybar): `scene.query` has no node and no
    text for its clock, so the pixel diff of two takes minutes apart saw the clock. The one node every
    tiling capture has, `tiling`, names the look; the clock island is masked there and nowhere else."""

    def compare(self, shape, nodes):
        with tempfile.TemporaryDirectory() as tmp:
            base, cand = pathlib.Path(tmp, "b"), pathlib.Path(tmp, "c")
            for d, paint in ((base, None), (cand, shape)):
                write_capture(d, "t1.Q", paint, size=(1280, 100), data=json.dumps({"nodes": nodes}))
            return diff.compare(base, cand, [])[0]["verdict"]

    TILING = [{"id": "tiling", "kind": "group", "owner": "tiling", "meta": {}, "rects": {}}]

    def test_the_clock_island_of_the_tiling_bar_is_masked(self):
        self.assertEqual(self.compare(box(580, 14, 100, 20), self.TILING), "same")

    def test_the_mask_is_by_the_tiling_node_only(self):
        self.assertEqual(self.compare(box(580, 14, 100, 20), []), "regression")

    def test_a_diff_beside_the_clock_island_is_still_a_diff(self):
        self.assertEqual(self.compare(box(100, 14, 100, 20), self.TILING), "regression")


def write_capture(folder, name, shape=None, size=(200, 100), data='{"nodes": []}'):
    """<name>.png (the shape painted on grey) and <name>.json in a folder."""
    folder = pathlib.Path(folder)
    folder.mkdir(parents=True, exist_ok=True)
    (folder / f"{name}.png").write_bytes(png(*size, shape))
    (folder / f"{name}.json").write_text(data)


def run_tool(tool, *args, path=None):
    """The tool as a command: (exit code, stdout, stderr); `path` replaces PATH when given."""
    env = {**os.environ, **({"PATH": path} if path is not None else {})}
    done = subprocess.run([sys.executable, "-I", str(HERE / tool), *map(str, args)], capture_output=True, text=True, env=env)
    return done.returncode, done.stdout, done.stderr


class CompareDirs(unittest.TestCase):
    """rd-1, rd-5, rd-6 and the cases r1 left uncovered: size, H50, baseline none, the exit code."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.base, self.cand = pathlib.Path(self.tmp.name, "b"), pathlib.Path(self.tmp.name, "c")
        self.addCleanup(self.tmp.cleanup)

    def verdicts(self, intended=(), none=()):
        return [(r["act"], r["capture"], r["verdict"]) for r in diff.compare(self.base, self.cand, list(intended), none)]

    def test_a_capture_missing_in_cand_is_a_regression(self):
        write_capture(self.base, "x1.Q")
        write_capture(self.base, "x2.Q")
        write_capture(self.cand, "x1.Q")
        self.assertEqual(self.verdicts(), [("x1", "Q", "same"), ("x2", "Q", "regression")])

    def test_empty_or_missing_cand_exits_1(self):
        write_capture(self.base, "x1.Q")
        self.cand.mkdir()
        self.assertEqual(run_tool("tour-diff.py", self.base, self.cand)[0], 1)
        self.assertEqual(run_tool("tour-diff.py", self.base, self.cand / "nope")[0], 1)

    def test_the_exit_code_follows_the_verdict(self):
        write_capture(self.base, "x1.Q")
        write_capture(self.cand, "x1.Q")
        self.assertEqual(run_tool("tour-diff.py", self.base, self.cand)[0], 0)
        write_capture(self.cand, "x1.Q", box(50, 40, 20, 20))
        code, out, _ = run_tool("tour-diff.py", self.base, self.cand)
        self.assertEqual(code, 1)
        self.assertIn("1 regression", out)

    def test_a_size_mismatch_is_a_regression(self):
        write_capture(self.base, "x1.Q", size=(200, 100))
        write_capture(self.cand, "x1.Q", size=(160, 100))
        self.assertEqual(self.verdicts(), [("x1", "Q", "regression")])

    def test_h50_is_scene_only(self):
        write_capture(self.base, "x1.H50")
        write_capture(self.cand, "x1.H50", box(50, 40, 20, 20))
        self.assertEqual(self.verdicts(), [("x1", "H50", "same")])

    def test_a_declared_baseline_none_is_none(self):
        write_capture(self.cand, "x1.Q")
        self.assertEqual(self.verdicts(none=["x1.Q"]), [("x1", "Q", "none")])

    def test_an_undeclared_absence_in_base_is_a_regression(self):
        write_capture(self.cand, "x1.Q")
        self.assertEqual(self.verdicts(), [("x1", "Q", "regression")])

    def test_the_cli_reads_the_none_list_of_intended_toml(self):
        write_capture(self.cand, "x1.Q")
        self.base.mkdir()
        toml = pathlib.Path(self.tmp.name, "intended.toml")
        toml.write_text('baseline_none = ["x1.Q"]\n')
        self.assertEqual(run_tool("tour-diff.py", self.base, self.cand, "--intended", toml)[0], 0)
        toml.write_text("")
        self.assertEqual(run_tool("tour-diff.py", self.base, self.cand, "--intended", toml)[0], 1)

    def test_the_cli_reads_baseline_none_json_of_the_base_folder(self):
        """rv-3: the names that run.py wrote (`baseline-none.json` of the take on the old build) are not
        regressions; a name only in the capture and not in the json still is."""
        write_capture(self.cand, "x1.Q")
        write_capture(self.cand, "x2.Q")
        self.base.mkdir()
        listing = [{"chapter": "xp", "reason": "no method", "names": ["x1.Q"], "trace": ""}]
        (self.base / "baseline-none.json").write_text(json.dumps(listing))
        code, out = run_tool("tour-diff.py", self.base, self.cand)[:2]
        self.assertEqual(code, 1, out)
        self.assertIn("x2", out)
        self.assertNotIn("x1       Q     regression", out)
        (self.base / "baseline-none.json").write_text(json.dumps([{"names": ["x1.Q", "x2.Q"]}]))
        self.assertEqual(run_tool("tour-diff.py", self.base, self.cand)[0], 0)

    def test_a_check_red_on_cand_and_new_is_a_regression(self):
        write_capture(self.base, "x1.Q", data='{"nodes": [], "checks": {"I4": "green"}}')
        write_capture(self.cand, "x1.Q", data='{"nodes": [], "checks": {"I4": "green", "I9": "red"}}')
        self.assertEqual(self.verdicts(), [("x1", "Q", "regression")])

    def test_a_red_check_on_a_none_row_is_a_regression(self):
        write_capture(self.cand, "x1.Q", data='{"nodes": [], "checks": {"I9": "red"}}')
        self.assertEqual(self.verdicts(none=["x1.Q"]), [("x1", "Q", "regression")])

    def test_a_new_red_check_with_a_check_row_is_a_change(self):
        write_capture(self.base, "x1.Q")
        write_capture(self.cand, "x1.Q", data='{"nodes": [], "checks": {"I9": "red"}}')
        row = {"act": "x1", "kind": "check", "check": "I9", "cause": "I9 is known red"}
        self.assertEqual(self.verdicts(intended=[row]), [("x1", "Q", "change")])

    def test_a_check_green_on_base_and_gone_on_cand_is_a_regression(self):
        write_capture(self.base, "x1.Q", data='{"nodes": [], "checks": {"I4": "green"}}')
        write_capture(self.cand, "x1.Q")
        self.assertEqual(self.verdicts(), [("x1", "Q", "regression")])

    def test_a_check_red_on_both_is_not_new(self):
        data = '{"nodes": [], "checks": {"I4": "red"}}'
        write_capture(self.base, "x1.Q", data=data)
        write_capture(self.cand, "x1.Q", data=data)
        self.assertEqual(self.verdicts(), [("x1", "Q", "same")])


class OpenAndNamedChecks(unittest.TestCase):
    """TOUR-BASE r2: an act whose cause is not found is `open`, never `change`; a check row names its check;
    a check that the baseline could not measure and the candidate reads red is said so."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.base, self.cand = pathlib.Path(self.tmp.name, "b"), pathlib.Path(self.tmp.name, "c")
        self.addCleanup(self.tmp.cleanup)

    def test_an_open_row_gives_the_verdict_open_not_change(self):
        rows = [{"act": "m1", "kind": "rect", "cause": "mc-59", "open": True}]
        v = diff.verdict("m1", [{"kind": "rect", "id": "n"}], True, rows)
        self.assertEqual(v["verdict"], "open", v)
        self.assertEqual(v["cause"], "mc-59")

    def test_an_open_row_does_not_hide_an_unnamed_kind(self):
        rows = [{"act": "m1", "kind": "rect", "cause": "mc-59", "open": True}]
        v = diff.verdict("m1", [{"kind": "rect", "id": "n"}, {"kind": "missing", "id": "k"}], True, rows)
        self.assertEqual((v["verdict"], v["unnamed"]), ("regression", ["missing"]), v)

    def test_the_f7_count_fails_on_an_open_act(self):
        write_capture(self.base, "m1.Q", data='{"nodes": [{"id": "n", "kind": "node", "rects": {"O": [0, 0, 9, 9]}}]}')
        write_capture(self.cand, "m1.Q", data='{"nodes": [{"id": "n", "kind": "node", "rects": {"O": [50, 0, 9, 9]}}]}')
        toml = pathlib.Path(self.tmp.name, "i.toml")
        toml.write_text('[[change]]\nact = "m1"\nkind = "rect"\ncause = "mc-59"\nopen = true\n')
        code, out, _ = run_tool("tour-diff.py", self.base, self.cand, "--intended", toml)
        self.assertIn("1 open", out)
        self.assertNotIn("1 change", out)
        self.assertEqual(code, 1, out)

    def test_a_check_row_with_a_name_does_not_absorb_another_check(self):
        rows = [{"act": "c2", "kind": "check", "check": "I2", "cause": "co-44"}]
        both = ({"I2": "green", "I6": "green"}, {"I2": "red", "I6": "red"})
        v = diff.verdict("c2", [], True, rows, checks=both)
        self.assertEqual((v["verdict"], v["flipped"]), ("regression", ["I6"]), v)
        v = diff.verdict("c2", [], True, rows, checks=({"I2": "green"}, {"I2": "red"}))
        self.assertEqual(v["verdict"], "change", v)

    def test_a_check_row_without_a_name_proves_nothing_and_leaves_the_act_open(self):
        """rtb7-2: a check or `*` row without `check =` named no check; it absorbed every flip as `change`. Like a row
        without `nodes`, it makes the act `open` (`unproven` `check`)."""
        for kind in ("check", "*"):
            with self.subTest(kind=kind):
                rows = [{"act": "c2", "kind": kind, "cause": "co-44"}]
                v = diff.verdict("c2", [], True, rows, checks=({"I2": "green", "I6": "green"}, {"I2": "red", "I6": "red"}))
                self.assertEqual(v["verdict"], "open", v)
                self.assertIn("check", v.get("unproven", []), v)

    def test_base_not_measured_and_cand_red_is_a_regression_unless_a_check_row_names_it(self):
        """rtb2-2: a check the base marks "not measured" and the candidate reads red is a flip."""
        write_capture(self.base, "x1.Q", data='{"nodes": [], "checks": {"I6": "not measured", "I2": "green"}}')
        write_capture(self.cand, "x1.Q", data='{"nodes": [], "checks": {"I6": "red", "I2": "green"}}')
        code, out, _ = run_tool("tour-diff.py", self.base, self.cand)
        self.assertIn("I6 not measured (cand red)", out)
        self.assertEqual(code, 1, out)
        [row] = diff.compare(self.base, self.cand, [])
        self.assertEqual((row["verdict"], row["flipped"], row["unmeasured"]), ("regression", ["I6"], ["I6"]), row)
        other = [{"act": "x1", "kind": "check", "check": "I2", "cause": "co-44"}]
        [row] = diff.compare(self.base, self.cand, other)
        self.assertEqual((row["verdict"], row["flipped"]), ("regression", ["I6"]), row)
        named = [{"act": "x1", "kind": "check", "check": "I6", "cause": "xp-39"}]
        [row] = diff.compare(self.base, self.cand, named)
        self.assertEqual((row["verdict"], row["cause"]), ("change", "xp-39"), row)

    def test_a_kind_with_a_found_row_and_an_open_row_gives_open_in_either_order(self):
        """rtb2-4: the open row is not dropped for the found one."""
        found = {"act": "m1", "kind": "rect", "cause": "675ebd55 the mc column"}
        opened = {"act": "m1", "kind": "rect", "cause": "mc-59 open", "open": True}
        scene = [{"kind": "rect", "id": "n"}]
        for rows in ([found, opened], [opened, found]):
            v = diff.verdict("m1", scene, True, rows)
            self.assertEqual(v["verdict"], "open", v)

    def test_a_check_the_candidate_stops_measuring_is_not_same(self):
        """rtb3-1: base green or red, candidate "not measured": a flip, absorbed only by a check row naming it."""
        for was in ("green", "red"):
            v = diff.verdict("a", [], True, [], ({"I6": was}, {"I6": "not measured"}))
            self.assertEqual((v["verdict"], v.get("flipped")), ("regression", ["I6"]), v)
        row = [{"act": "a", "kind": "check", "check": "I6", "cause": "xp-39"}]
        v = diff.verdict("a", [], True, row, ({"I6": "green"}, {"I6": "not measured"}))
        self.assertEqual(v["verdict"], "change", v)
        v = diff.verdict("a", [], True, [], ({"I6": "not measured"}, {"I6": "not measured"}))
        self.assertEqual(v["verdict"], "same", v)

    def test_a_check_absent_on_the_base_and_not_measured_on_the_candidate_is_not_a_flip(self):
        """rtb4-1: a check new in the candidate that it cannot measure is no loss; a build that adds I9 would flag every act."""
        v = diff.verdict("a", [], True, [], ({}, {"I9": "not measured"}))
        self.assertEqual(v["verdict"], "same", v)
        self.assertEqual(v["unmeasured"], ["I9"], v)
        v = diff.verdict("a", [], True, [], ({"I6": "green"}, {"I6": "not measured", "I9": "not measured"}))
        self.assertEqual((v["verdict"], v["flipped"]), ("regression", ["I6"]), v)

    def test_base_not_measured_and_cand_green_has_no_red_note(self):
        write_capture(self.base, "x1.Q", data='{"nodes": [], "checks": {"I6": "not measured"}}')
        write_capture(self.cand, "x1.Q", data='{"nodes": [], "checks": {"I6": "green"}}')
        code, out, _ = run_tool("tour-diff.py", self.base, self.cand)
        self.assertIn("I6 not measured", out)
        self.assertNotIn("cand red", out)


class RowsPerNode(unittest.TestCase):
    """rv9j-5: a row of intended.toml may name the nodes it explains (`nodes = [...]`); it absorbs those
    only, so "proven" can be checked per node. A row without `nodes` matches by act and kind as before."""

    SCENE = [{"kind": "meta", "id": "desk-label"}, {"kind": "meta", "id": "hud-chat"}]

    def test_a_row_with_nodes_absorbs_only_those_nodes(self):
        found = {"act": "x5", "kind": "meta", "cause": "29281054 labels", "nodes": ["desk-label"]}
        opened = {"act": "x5", "kind": "meta", "cause": "xp-47 greeting (open)", "open": True}
        for rows in ([found, opened], [opened, found]):
            v = diff.verdict("x5", self.SCENE, True, rows)
            self.assertEqual(v["verdict"], "open", v)
            self.assertEqual(v["cause"], "29281054 labels; xp-47 greeting (open)", v)
            v = diff.verdict("x5", self.SCENE[:1], True, rows)
            self.assertEqual((v["verdict"], v["cause"]), ("change", "29281054 labels"), v)

    def test_a_node_not_in_the_row_stays_open_where_an_open_row_is_the_rest(self):
        found = {"act": "x5", "kind": "meta", "cause": "29281054 labels", "nodes": ["desk-label"]}
        opened = {"act": "x5", "kind": "meta", "cause": "xp-47 greeting (open)", "open": True}
        v = diff.verdict("x5", self.SCENE[1:], True, [found, opened])
        self.assertEqual((v["verdict"], v["cause"]), ("open", "xp-47 greeting (open)"), v)

    def test_a_node_no_row_names_is_unnamed(self):
        found = {"act": "x5", "kind": "meta", "cause": "29281054 labels", "nodes": ["desk-label"]}
        v = diff.verdict("x5", self.SCENE, True, [found])
        self.assertEqual((v["verdict"], v["unnamed"]), ("regression", ["meta"]), v)

    def test_a_row_without_nodes_names_the_kind_but_proves_nothing(self):
        """rtb6-1: was `change` (the match by kind); the act of a row that lists no nodes stays `open`."""
        row = {"act": "x5", "kind": "meta", "cause": "29281054 labels"}
        v = diff.verdict("x5", self.SCENE, True, [row])
        self.assertEqual((v["verdict"], v["cause"], v["unproven"]), ("open", "29281054 labels", ["meta"]), v)

    def test_a_pixel_row_has_boxes_not_nodes(self):
        row = {"act": "x5", "kind": "pixel", "cause": "d78fc4bb pager", "boxes": [[100, 680, 200, 40]]}
        v = diff.verdict("x5", [], False, [row], blobs=[(120, 690, 30, 20)])
        self.assertEqual((v["verdict"], v["cause"]), ("change", "d78fc4bb pager"), v)


class RowsCannotAbsorbWhatTheyDoNotName(unittest.TestCase):
    """rtb5-1 (MAJOR): an `added` or `visible` row without `nodes` and a `pixel` row without `boxes` absorbed any later
    diff of their kind, so an act that is `change` stayed `change` when something unrelated moved. Such a row matches
    nothing: the act stays `open`."""

    UNRELATED = {"kind": "added", "id": "unrelated-popup"}
    METAS = {"kind": "meta", "id": "xp-item-3"}

    def rows(self, **extra):
        return [
            {"act": "x1", "kind": "meta", "cause": "29281054 labels", "nodes": ["xp-item-3"]},
            {"act": "x1", "kind": "added", "cause": "371f0fa1 xp desktop", **extra.get("added", {})},
            {"act": "x1", "kind": "pixel", "cause": "d78fc4bb pager", **extra.get("pixel", {})},
        ]

    def test_an_unrelated_added_node_and_a_pixel_diff_are_not_a_change(self):
        """The finding's run: rows without nodes or boxes gave `change`."""
        v = diff.verdict("x1", [self.UNRELATED, self.METAS], False, self.rows(), blobs=[(500, 300, 40, 40)])
        self.assertNotEqual(v["verdict"], "change", v)
        self.assertEqual(v["verdict"], "open", v)

    def test_an_added_row_without_nodes_absorbs_nothing(self):
        v = diff.verdict("x1", [self.UNRELATED], True, self.rows())
        self.assertEqual(v["verdict"], "open", v)

    def test_a_visible_row_without_nodes_absorbs_nothing(self):
        rows = [{"act": "x12c", "kind": "visible", "cause": "20e4ad6e menus shut"}]
        v = diff.verdict("x12c", [{"kind": "visible", "id": "xp-menu"}], True, rows)
        self.assertEqual(v["verdict"], "open", v)

    def test_a_pixel_row_without_boxes_absorbs_nothing(self):
        v = diff.verdict("x1", [], False, [self.rows()[2]], blobs=[(500, 300, 40, 40)])
        self.assertEqual(v["verdict"], "open", v)

    def test_a_row_with_nodes_is_closed_for_those_nodes_only(self):
        rows = self.rows(added={"nodes": ["xp-desktop"]}, pixel={"boxes": [[0, 680, 400, 40]]})
        v = diff.verdict("x1", [self.METAS, {"kind": "added", "id": "xp-desktop"}], False, rows, blobs=[(10, 690, 30, 20)])
        self.assertEqual((v["verdict"], v["cause"]), ("change", "29281054 labels; 371f0fa1 xp desktop; d78fc4bb pager"), v)
        v = diff.verdict("x1", [self.UNRELATED, self.METAS], False, rows, blobs=[(10, 690, 30, 20)])
        self.assertEqual((v["verdict"], v["unnamed"]), ("regression", ["added"]), v)

    def test_a_blob_outside_the_boxes_is_unnamed(self):
        rows = self.rows(added={"nodes": ["xp-desktop"]}, pixel={"boxes": [[0, 680, 400, 40]]})
        scene = [{"kind": "added", "id": "xp-desktop"}, self.METAS]
        v = diff.verdict("x1", scene, False, rows, blobs=[(10, 690, 30, 20), (500, 300, 40, 40)])
        self.assertEqual((v["verdict"], v["unnamed"]), ("regression", ["pixel"]), v)

    def test_a_pixel_diff_with_no_blob_to_place_is_not_proven_by_boxes(self):
        rows = self.rows(pixel={"boxes": [[0, 680, 400, 40]]})
        for blobs in ([], None):
            v = diff.verdict("x1", [], False, rows, blobs=blobs)
            self.assertNotEqual(v["verdict"], "change", (blobs, v))

    def test_compare_dirs_passes_the_blobs_of_the_diff(self):
        """compare() hands the blobs to verdict: a blob outside the boxes is a regression on real files."""
        with tempfile.TemporaryDirectory() as tmp:
            base, cand = pathlib.Path(tmp, "b"), pathlib.Path(tmp, "c")
            for d in (base, cand):
                d.mkdir()
            write_capture(base, "x1.Q", size=(100, 100))
            write_capture(cand, "x1.Q", lambda x, y: (250, 0, 0) if 60 <= x < 90 and 60 <= y < 90 else None, size=(100, 100))
            row = {"act": "x1", "kind": "pixel", "cause": "d78fc4bb pager"}
            inside = diff.compare(base, cand, [{**row, "boxes": [[50, 50, 50, 50]]}])
            outside = diff.compare(base, cand, [{**row, "boxes": [[0, 0, 40, 40]]}])
            listless = diff.compare(base, cand, [row])
        self.assertEqual([r["verdict"] for r in inside + outside + listless], ["change", "regression", "open"])


class NoRowKindAbsorbsWhatItDoesNotName(unittest.TestCase):
    """rtb6-1 (MAJOR, third round of this class): r5 made `added`, `visible` and `pixel` strict and left `missing`, `rect`,
    `meta` and `owner` to absorb any unrelated node of their kind. One rule for every kind tour-diff knows: a row absorbs
    the nodes (or, for pixel, the boxes) it names; a row that names none matches nothing and its act stays `open`."""

    @staticmethod
    def kinds():
        """Every kind of scene diff tour-diff.py can emit (read from its source, so a kind added later is in the table),
        and the wildcard."""
        source = (HERE / "tour-diff.py").read_text()
        return sorted(set(re.findall(r'"kind": "([a-z-]+)"', source)) | {"*"})

    def test_the_table_has_the_kinds_of_the_scene_diff(self):
        self.assertTrue({"missing", "rect", "meta", "owner", "added", "visible", "z-order"} <= set(self.kinds()), self.kinds())

    def test_a_row_without_nodes_leaves_an_unrelated_node_open_for_every_kind(self):
        for kind in self.kinds():
            with self.subTest(kind=kind):
                rows = [{"act": "m1", "kind": kind, "cause": "675ebd55 the column"}]
                v = diff.verdict("m1", [{"kind": kind if kind != "*" else "missing", "id": "unrelated-node"}], True, rows)
                self.assertNotEqual(v["verdict"], "change", v)
                self.assertEqual(v["verdict"], "open", v)

    def test_a_row_with_nodes_absorbs_its_node_and_not_an_unrelated_one_for_every_kind(self):
        for kind in self.kinds():
            if kind == "*":
                continue
            with self.subTest(kind=kind):
                rows = [{"act": "m1", "kind": kind, "cause": "675ebd55 the column", "nodes": ["named-node"]}]
                v = diff.verdict("m1", [{"kind": kind, "id": "named-node"}], True, rows)
                self.assertEqual((v["verdict"], v["cause"]), ("change", "675ebd55 the column"), v)
                v = diff.verdict("m1", [{"kind": kind, "id": "unrelated-node"}], True, rows)
                self.assertEqual(v["verdict"], "regression", v)

    def test_the_run_of_the_finding(self):
        """rtb6-1: `verdict('m4', [missing unrelated, rect unrelated, visible mc-regions-0], True, rows)` was `change`."""
        rows = [
            {"act": "m4", "kind": "missing", "cause": "675ebd55 column"},
            {"act": "m4", "kind": "rect", "cause": "675ebd55 column"},
            {"act": "m4", "kind": "visible", "cause": "675ebd55 column", "nodes": ["mc-regions-0"]},
        ]
        scene = [{"kind": "missing", "id": "unrelated"}, {"kind": "rect", "id": "unrelated"}, {"kind": "visible", "id": "mc-regions-0"}]
        self.assertEqual(diff.verdict("m4", scene, True, rows)["verdict"], "open")

    def test_every_closed_row_of_intended_toml_names_its_nodes_or_boxes(self):
        """The rule on the data: a closed row of a scene kind lists `nodes`, a closed pixel row `boxes` (the `check` rows
        name a check, not a node); a row that cannot say which is `open = true`."""
        import tomllib

        rows = tomllib.loads((HERE.parent / "demo/tour/intended.toml").read_text())["change"]
        loose = [(r["act"], r["kind"]) for r in rows if r["kind"] != "check" and not r.get("open") and not (r.get("boxes") if r["kind"] == "pixel" else r.get("nodes"))]
        self.assertEqual(loose, [])

    FRAME = (1280, 720)
    """The size of the outputs the tour captures (HEADLESS-1)."""

    def test_no_pixel_row_box_covers_the_whole_frame(self):
        """rtr4r-1: a pixel row of c5d had the box [0, 0, 1280, 720]: it names every blob of its act, so no pixel
        regression of that act is ever seen again. A box names what one commit changes, never the whole frame."""
        import tomllib

        rows = tomllib.loads((HERE.parent / "demo/tour/intended.toml").read_text())["change"]
        w, h = self.FRAME
        whole = [(r["act"], b) for r in rows if r["kind"] == "pixel" for b in r.get("boxes", []) if b[0] <= 0 and b[1] <= 0 and b[0] + b[2] >= w and b[1] + b[3] >= h]
        self.assertEqual(whole, [])


class IntendedRows(unittest.TestCase):
    def test_h2_lists_kinds_and_no_wildcard(self):
        import tomllib

        rows = tomllib.loads((HERE.parent / "demo/tour/intended.toml").read_text())["change"]
        h2 = sorted(r["kind"] for r in rows if r["act"] == "h2")
        self.assertEqual(h2, ["added", "meta", "rect"])


class IntendedCauses(unittest.TestCase):
    """TOUR-BASE r2 (rtb-2, rtb-5, rtb-6, rtb-7): what a row of intended.toml may claim."""

    @classmethod
    def rows(cls):
        import tomllib

        return tomllib.loads((HERE.parent / "demo/tour/intended.toml").read_text())["change"]

    TOKEN = re.compile(r"\b(?=[0-9a-f]*\d)[0-9a-f]{8}\b|\b[a-z]{2,3}\d?-\d+\b")
    """A cause is a commit (8 hex digits with a digit) or a finding id (`co-44`, `rv9-2`)."""

    def test_a_row_names_at_most_three_causes_or_is_open(self):
        """rtb2-3: the shas and finding ids of the cause text are counted, not its `; ` units; no row is grandfathered."""
        limit = lambda r: 8 if r["kind"] == "pixel" else 3
        wide = [(r["act"], r["kind"], len(set(self.TOKEN.findall(r["cause"])))) for r in self.rows() if len(set(self.TOKEN.findall(r["cause"]))) > limit(r) and not r.get("open")]
        self.assertEqual(wide, [])

    def test_no_row_cites_co_61_the_ledger_id_of_another_finding(self):
        self.assertEqual([(r["act"], r["kind"]) for r in self.rows() if "co-61" in r["cause"]], [])

    def test_a_check_row_names_its_check(self):
        self.assertEqual([r["act"] for r in self.rows() if r["kind"] == "check" and not r.get("check")], [])

    def test_xp_43_is_cited_only_on_the_acts_that_finding_names(self):
        named = {"x10b", "x5e", "x5g", "x7", "x8", "x9b"}
        self.assertEqual(sorted({r["act"] for r in self.rows() if "xp-43" in r["cause"]} - named), [])

    def test_a_row_with_nodes_names_node_ids_of_a_scene_kind(self):
        """rv9j-5: `nodes` is a list of ids (a window `app_id:title`); a pixel or check row has no node."""
        for r in self.rows():
            if "nodes" in r:
                self.assertIsInstance(r["nodes"], list, r)
                self.assertTrue(r["nodes"] and all(isinstance(n, str) and n for n in r["nodes"]), r)
                self.assertIn(r["kind"], ("meta", "rect", "missing", "added", "visible", "owner", "cut"), r)

    def test_the_boil_page_side_panel_of_m2_m2b_m3_has_its_rows(self):
        """rtr4-1: c3925fc55 (a panel takes whole lines at the page edge) (c3925fc5 is its 8-digit form) makes `mc-s-boil-panels-side-lines` one whole line
        of 28 px; the cut check still flags it and the panels change. Each act names the rect of lead and side and the cut
        of side-lines with that commit, by node and no wider than the judged act."""
        rows = self.rows()
        for act in ("m2", "m2b", "m3"):
            mine = [r for r in rows if r["act"] == act and "c3925fc5" in r["cause"]]
            kinds = {r["kind"]: r for r in mine}
            self.assertEqual(sorted(kinds), ["cut", "rect"], (act, mine))
            self.assertEqual(kinds["cut"]["nodes"], ["mc-s-boil-panels-side-lines"], act)
            self.assertTrue({"mc-s-boil-panels-lead", "mc-s-boil-panels-side"} <= set(kinds["rect"]["nodes"]), act)
            for r in mine:
                self.assertFalse(r.get("open"), r)
                self.assertNotIn("*", r["nodes"], r)

    def test_a_row_with_boxes_is_a_pixel_row_of_x_y_w_h(self):
        """rtb5-1: `boxes` is a list of [x, y, w, h] in the coordinates of tour-diff (pixels of the capture)."""
        for r in self.rows():
            if "boxes" in r:
                self.assertEqual(r["kind"], "pixel", r)
                self.assertTrue(r["boxes"] and all(isinstance(b, list) and len(b) == 4 and all(isinstance(v, int) for v in b) and b[2] > 0 and b[3] > 0 for b in r["boxes"]), r)

    def test_a_closed_added_visible_or_pixel_row_names_its_nodes_or_boxes(self):
        """rtb5-1: without them the row absorbs any later diff of its kind (tour-diff keeps its act `open`); a row that
        cannot say which nodes or where is `open = true`, with the cause it has."""
        loose = [(r["act"], r["kind"]) for r in self.rows() if r["kind"] in ("added", "visible", "pixel") and not r.get("open") and not r.get("nodes") and not r.get("boxes")]
        self.assertEqual(loose, [])

    def cites_a_capture(self, row):
        return "captures/" in row["cause"] or "judge r1" in row["cause"]

    def test_a_row_of_xp_desktop_or_a_visible_row_cites_a_capture(self):
        """rtb6-5: the `added xp-desktop` rows (35), `visible xp-menu` of x12c and `visible mc-regions-*` of m4, m4b gave a
        sha only; each cites its act's still of the take (`r7 take, captures/<act>.<capture>.png`) or the judge's."""
        bare = [(r["act"], r["kind"]) for r in self.rows() if not r.get("open") and ("xp-desktop" in r.get("nodes", ()) or r["kind"] == "visible") and not self.cites_a_capture(r)]
        self.assertEqual(bare, [])

    def test_a_closed_missing_or_rect_row_cites_a_capture(self):
        """rtb6-1: a row that lists the nodes it explains cites the capture that shows them (the act and the still)."""
        bare = [(r["act"], r["kind"]) for r in self.rows() if r["kind"] in ("missing", "rect") and not r.get("open") and not self.cites_a_capture(r)]
        self.assertEqual(bare, [])

    CHAPTERS = HERE.parent / "demo/tour/chapters"

    @classmethod
    def captures_of_acts(cls):
        """act id -> capture name, read from the `t.act(..., scene=, capture=)` lines of the chapters (ids as `act_id`
        of tour.py: a later act of a scene gets a letter)."""
        seen, out = {}, {}
        for path in sorted(cls.CHAPTERS.glob("*.py")):
            for line in path.read_text().splitlines():
                m = re.search(r't\.act\(.*scene="(\w+)"', line)
                if m:
                    scene = m.group(1)
                    seen[scene] = seen.get(scene, 0) + 1
                    act = scene if seen[scene] == 1 else f"{scene}{chr(ord('a') + seen[scene] - 1)}"
                    c = re.search(r'capture="([^"]+)"', line)
                    out[act] = c.group(1) if c else "Q"
        return out

    def test_a_cited_capture_is_a_capture_the_act_has(self):
        """rtb7-5: a cause cites `captures/<act>.<capture>.png`; the act has that capture (x5d, x6d: `Q+1`, x8: `Q+0.5`)
        and the row is of that act or cites one it names."""
        known = self.captures_of_acts()
        wrong = []
        for r in self.rows():
            for act, capture in re.findall(r"captures/(\w+)\.([^\s)]+?)\.png", r["cause"]):
                if act in known and known[act] != capture:
                    wrong.append((r["act"], r["kind"], f"{act}.{capture}", f"has {known[act]}"))
        self.assertEqual(wrong, [])

    def test_the_hairline_rows_of_m1_name_the_open_design_question(self):
        """rtb6-4: the KEYS hairline moves with 675ebd55 (judge r1 m1c), which the judge called intended by the commit,
        not by the design (rv9j-2: mc-v2 section 3 asks for "Nothing new." in the column)."""
        rows = [r for r in self.rows() if r["act"] in ("m1", "m1c") and r["kind"] == "pixel"]
        self.assertTrue(rows)
        self.assertEqual([(r["act"]) for r in rows if "rv9j-2" not in r["cause"]], [])

    def test_the_rows_of_m1_and_m1c_name_the_open_design_question(self):
        """rtb7-1: rv9j-2 on the `missing` rows of m1 and m1c too: the notify block that goes is the design decision."""
        rows = [r for r in self.rows() if r["act"] in ("m1", "m1c") and r["kind"] == "missing"]
        self.assertTrue(rows)
        self.assertEqual([r["act"] for r in rows if "rv9j-2: the design decision is open" not in r["cause"]], [])

    def test_a_rect_row_of_a_card_the_take_shows_as_added_is_open(self):
        """rtb7-3: `xp-ctx-1-0` of h2 names a node the cited capture does not show as moved (h2 has no baseline): open.
        The `rect notify-card-*` rows of x2, x2b, x8 closed in r1 of tour-rows: the take shows them as `rect` (judge r2b)."""
        closed = [(r["act"], r["kind"]) for r in self.rows() if not r.get("open") and r["kind"] in ("rect", "meta") and "xp-ctx-1-0" in r.get("nodes", ())]
        self.assertEqual(closed, [])

    def test_the_notify_card_of_x2_x2b_x8_moves_by_62708d35_and_the_blink_has_no_box(self):
        """tour-rows r1 (judge r2b section 8 row 4, rv9kb-1): `rect notify-card-body` and `-close` are 62708d35 (x -8) and
        29281054 (the second line), not 675ebd55 of mc; the blink of the paperclip is masked by tour-diff, not boxed."""
        for act in ("x2", "x2b", "x8"):
            rows = [r for r in self.rows() if r["act"] == act and r["kind"] == "rect" and "notify-card-body" in r.get("nodes", ())]
            self.assertEqual(len(rows), 1, act)
            self.assertFalse(rows[0].get("open"), act)
            self.assertIn("62708d35", rows[0]["cause"])
            self.assertNotIn("675ebd55", rows[0]["cause"])
        boxes = [b for r in self.rows() if r["act"] == "x2b" and r["kind"] == "pixel" for b in r.get("boxes", [])]
        self.assertNotIn([1213, 593, 39, 20], boxes)

    SESSION_CAPTIONS = ("fc9675ac", "11e34cf8", "94579f91")

    def test_no_row_of_the_xp_or_handover_acts_cites_the_session_captions(self):
        """rv9j-5: those commits change the captions of the session buttons, which no act of the xp chapter or of the
        handover shows; the blobs there are the pager (d78fc4bb) and the bin (62708d35)."""
        acts = {r["act"] for r in self.rows() if re.fullmatch(r"x\d+[a-z]?|h1[a-d]?", r["act"])}
        cited = [(r["act"], r["kind"]) for r in self.rows() if r["act"] in acts and any(c in r["cause"] for c in self.SESSION_CAPTIONS)]
        self.assertEqual(cited, [])

    def test_a_row_cites_a_judge_capture_only_for_the_act_the_judge_looked_at(self):
        """rtb5-2: `judge r1 x7, x7.jpg` on the pixel row of x2..x12b and h1..h1d: the judge looked at x2 and x7 (and x1 by
        x7-tray.jpg of x7, h1, h1c, x5d, x6b, x10b); the blob classification of the other acts is the builder's own, read in
        the still of its take. A row cites `judge r1 <its own act>` or no judge capture."""
        wrong = [(r["act"], r["kind"], m) for r in self.rows() if r["kind"] == "pixel" and re.fullmatch(r"x\d+[a-z]?|h1[a-d]?", r["act"]) for m in re.findall(r"judge r1 (\w+),", r["cause"]) if m != r["act"]]
        self.assertEqual(wrong, [])

    def test_the_greeting_in_the_chat_is_no_label_change(self):
        """rv9j-5: the text of `hud-chat` is the greeting (4fec75c3), not the desktop labels (29281054)."""
        wrong = [(r["act"], r["kind"]) for r in self.rows() if any(n.startswith("hud-chat") for n in r.get("nodes", [])) and "29281054" in r["cause"]]
        self.assertEqual(wrong, [])

    UNPROVEN = ("cause not found", "(open)", "(proposed)")
    UNPROVEN_FINDINGS = ("xp-43",)
    """The findings whose ledger state is open with the cause not found (xp.md:44); mc-59 and co-63 are answered by judges
    r2a and r2b (the double buffer of mc-48; co-63 intended for c4, c5e)."""

    def test_an_open_row_says_what_is_open(self):
        self.assertEqual([(r["act"], r["kind"]) for r in self.rows() if r.get("open") and "open" not in r["cause"]], [])

    def test_a_row_whose_cause_is_unproven_is_open(self):
        """rtb2-1, two-way: a cause that says it is not found, open or proposed, or cites a finding that is open with
        no cause found, is `open = true`; else tour-diff prints `change` and a later unrelated diff passes."""
        marks = self.UNPROVEN + self.UNPROVEN_FINDINGS
        unproven = [(r["act"], r["kind"]) for r in self.rows() if any(m in r["cause"] for m in marks) and not r.get("open")]
        self.assertEqual(unproven, [])


V8 = "733967aa"
"""The build of the baseline replay (TOUR_BASELINE): a cause of a v9 diff is a commit after it."""


def git(*args):
    return subprocess.run(["git", "-C", str(HERE.parent), *args], capture_output=True, text=True)


@unittest.skipUnless(git("cat-file", "-e", f"{V8}^{{commit}}").returncode == 0, "no history back to 733967aa (git fetch --unshallow)")
class IntendedCommits(unittest.TestCase):
    """tour-rows r4 (rtr3-2): a row says which commit made its change, and that commit is in the history the two
    takes span: an ancestor of HEAD, not of the v8 build, and it changes what the binaries are built from. Both takes
    run the head's harness, so a commit of tests/, demo/ or docs/ alone cannot change a capture."""

    SHA = re.compile(r"\b(?=[0-9a-f]*\d)[0-9a-f]{7,40}\b")
    PRODUCT = ("crates/", "plugins/", "flavors/", "services/", "dist/", "Cargo.toml", "Cargo.lock")

    @classmethod
    def setUpClass(cls):
        import tomllib

        cls.rows = tomllib.loads((HERE.parent / "demo/tour/intended.toml").read_text())["change"]
        cls.shas = sorted({s for r in cls.rows for s in cls.SHA.findall(r["cause"])})

    def test_every_row_names_a_commit_sha(self):
        self.assertEqual([(r["act"], r["kind"]) for r in self.rows if not self.SHA.search(r["cause"])], [])

    def test_every_named_sha_is_a_commit_between_v8_and_head(self):
        wrong = []
        for sha in self.shas:
            if git("cat-file", "-e", f"{sha}^{{commit}}").returncode:
                wrong.append((sha, "no such commit"))
            elif git("merge-base", "--is-ancestor", sha, "HEAD").returncode:
                wrong.append((sha, "not in the history of HEAD"))
            elif not git("merge-base", "--is-ancestor", sha, V8).returncode:
                wrong.append((sha, f"already in {V8} (v8)"))
        self.assertEqual(wrong, [])

    @classmethod
    def files_of(cls, sha):
        """The files a commit brings to its first parent (a merge too: `git show` lists nothing for a clean merge)."""
        return git("diff", "--name-only", f"{sha}^1", sha).stdout.split()

    @classmethod
    def no_product_code(cls, sha):
        """The files of a commit when none of them is product code (a commit that cannot change a capture), else []."""
        files = cls.files_of(sha)
        return [] if files and any(f.startswith(cls.PRODUCT) for f in files) else files or ["(no files)"]

    def test_every_named_sha_changes_product_code(self):
        wrong = [(sha, " ".join(self.no_product_code(sha)[:3])) for sha in self.shas if self.no_product_code(sha)]
        self.assertEqual(wrong, [])

    @unittest.skipUnless(git("cat-file", "-e", "19348b95^{commit}").returncode == 0, "no commit 19348b95")
    def test_a_clean_merge_is_judged_by_what_it_brings_to_its_first_parent(self):
        """rtr4r-3: `git show --name-only` lists nothing for a clean merge, so a row citing 19348b95 (chore: merge
        session head a93bc75b; it brings tests/persona-selftest.sh only) passed. A merge counts what it brings to its
        first parent: `git diff --name-only <sha>^1 <sha>`."""
        self.assertEqual(self.no_product_code("19348b95"), ["tests/persona-selftest.sh"])


class MinorRegressionsOfTakeR5(unittest.TestCase):
    """tour-rows r5 (F7, rc 11): take r2 left 24 MINOR regressions on b16c888f. Each was judged on the v8 and v9 stills:
    the change is the intended result of a named commit and the capture meets design/looks/ (whole text, nothing
    under 12 px, aligned chips), or it stays a regression with its finding. `tour-diff-take-r5.json` holds, per act,
    the diffs of the r5 take (v9 head, v8 733967aa, TOUR_BASELINE=1) that no row of b16c888f named."""

    TAKE = json.loads((HERE / "tour-diff-take-r5.json").read_text())
    ROWS = "c2b c3b c3c c3d c4 c4b m1 m1b m1c m4 m4b t3f t5 t6 t6b t6c t6d t7 t10c x1 x5b".split()
    """The 21 acts that get rows; c5d and c5e (rtf1-1: v9 steps into the face, every pixel differs) and m8 (rtr4-2: the
    launcher has no icons) stay regressions."""

    @classmethod
    def rows(cls):
        import tomllib

        return tomllib.loads((HERE.parent / "demo/tour/intended.toml").read_text())["change"]

    def verdict(self, act):
        u = self.TAKE[act]
        scene = [{"kind": k, "id": i, "detail": ""} for k, i in u["scene"]]
        return diff.verdict(act, scene, not u["blobs"], self.rows(), blobs=[tuple(b) for b in u["blobs"]])

    def test_every_diff_the_take_shows_in_the_judged_acts_is_named(self):
        wrong = {act: self.verdict(act) for act in self.ROWS if self.verdict(act)["verdict"] != "change"}
        self.assertEqual(wrong, {})

    def test_m8_keeps_its_launcher_diff_unnamed_and_the_c5_acts_have_no_box_of_the_frame(self):
        """rtr4-2: no row may name the launcher of m8 (463, 198, 354, 324); rtf1-1: no box may name the whole frame of c5d, c5e."""
        self.assertEqual(self.verdict("m8")["verdict"], "regression")
        self.assertEqual(self.verdict("m8")["unnamed"], ["pixel"])
        boxes = [b for r in self.rows() if r["act"] in ("c5d", "c5e", "m8") and r["kind"] == "pixel" for b in r.get("boxes", [])]
        self.assertEqual([b for b in boxes if b[2] * b[3] > 1280 * 720 // 2], [])

    def test_the_x1_row_boxes_the_tail_node_of_the_balloon_and_cites_its_commits(self):
        """rtf1-6: the blob (1182, 560, 50, 18) lies in the rect of `hud-line-tail` (1067, 559, 166, 19) in the r5 take of x1: the
        greeting (4fec75c3) has a tail node of its own (e4157dbd); the v8 build has neither."""
        rows = [r for r in self.rows() if r["act"] == "x1" and r["kind"] == "pixel" and "e4157dbd" in r["cause"]]
        self.assertEqual([r["boxes"] for r in rows], [[[1067, 559, 166, 19]]])
        self.assertIn("4fec75c3", rows[0]["cause"])

    def test_the_tiling_acts_box_the_left_island_of_the_bar_with_the_chip_commit(self):
        """rtf1-3: the blob is the left island of the tiling bar, [12, 8, 438, 32] and the layout word in its chip; e6afbf97
        gives the word its own chip. One box of the island, no wider than the island (x 12 to 538)."""
        for act in "t3f t5 t6 t6b t6c t6d t7 t10c".split():
            mine = [r for r in self.rows() if r["act"] == act and r["kind"] == "pixel" and "e6afbf97" in r["cause"]]
            self.assertEqual([r["boxes"] for r in mine], [[[12, 8, 526, 32]]], act)

    def test_the_mc_lead_panels_name_their_rects_with_the_whole_lines_commit(self):
        """rtf1-4: c3925fc5 (a panel takes whole lines at the page edge): the lead panel is 287 px for 10 lines of 28 (was 280,
        the last line cut at 273); a rect row per node, no pixel row."""
        want = {"m1": ["mc-s-ingredients-panels-lead"], "m1b": ["mc-s-ingredients-panels-lead"], "m1c": ["mc-s-boil-panels-lead", "mc-s-boil-panels-lead-lines"],
                "m4": ["mc-s-sauce-panels-lead", "mc-s-sauce-panels-lead-lines"], "m4b": ["mc-s-boil-panels-lead", "mc-s-boil-panels-lead-lines"]}
        for act, nodes in want.items():
            mine = [r for r in self.rows() if r["act"] == act and r["kind"] == "rect" and "c3925fc5" in r["cause"] and set(nodes) <= set(r["nodes"])]
            self.assertEqual(len(mine), 1, act)
            self.assertFalse(mine[0].get("open"), act)

    def test_x5b_names_the_working_header_with_the_send_commit(self):
        """rtf1-5: the header of the chat reads Working at once after a send (88951ae4), where v8 still read Idle."""
        mine = [r for r in self.rows() if r["act"] == "x5b" and r["kind"] == "meta" and "hud-chat" in r.get("nodes", ())]
        self.assertEqual([("88951ae4" in r["cause"]) for r in mine], [True])


def load_launch(repo):
    """demo/tour/launch.py as a take on the build in REPO loads it (TOUR_REPO; the v8 tree of the replay)."""
    from unittest import mock

    sys.path.insert(0, str(HERE.parent / "demo/tour"))
    try:
        with mock.patch.dict(os.environ, {"TOUR_REPO": str(repo)}):
            spec = importlib.util.spec_from_file_location("launch_of_take", HERE.parent / "demo/tour/launch.py")
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
    finally:
        sys.path.remove(str(HERE.parent / "demo/tour"))
    return module


class LaunchOnTheBaseline(unittest.TestCase):
    """The harness of the head runs the v8 replay (TOUR_REPO the 733967aa tree, TOUR_BASELINE=1): what it needs of
    its own lives next to it, and what the old build cannot answer is not waited for."""

    def tool(self, d, text):
        path = pathlib.Path(d, "fuzzel")
        path.write_text(f"#!/bin/sh\necho '{text}'\n")
        path.chmod(0o755)
        return str(path)

    def test_the_tool_version_script_is_the_harness_own(self):
        """rtr4-5: `fuzzel_version` ran `{TOUR_REPO}/tests/tool-version.sh`; the v8 tree has none, so every v8
        chapter broke with FileNotFoundError."""
        with tempfile.TemporaryDirectory() as old, tempfile.TemporaryDirectory() as bins:
            launch = load_launch(old)
            self.assertEqual(launch.fuzzel_version(self.tool(bins, "fuzzel version: 1.11.1 +cairo")), (1, 11, 1))

    def test_the_baseline_neither_starts_nor_waits_for_a_notify_it_cannot_ask(self):
        """rtr4-4: the 733967aa chora-notify has no method (no `state.get`), so `ensure_notify` stopped the v8
        mc dark and handover chapters after 15 s with no capture. On the replay a look that does not list
        chora-notify runs without it, as v8 did (the r3 replay)."""
        from unittest import mock

        with tempfile.TemporaryDirectory() as old:
            launch = load_launch(old)
            calls = []
            fake = lambda argv, **kw: calls.append(argv) or subprocess.CompletedProcess(argv, 1, "", "")
            with mock.patch.dict(os.environ, {"TOUR_BASELINE": "1"}), mock.patch.object(launch.subprocess, "run", fake), mock.patch.object(launch, "NOTIFY_WAIT", 0.3):
                launch.Launcher.ensure_notify(object(), {})
            self.assertEqual(calls, [])


class ConstructAfterTheDrop(unittest.TestCase):
    """rtr4-3: since 75d4623e the drop of c5c steps into the front face; the chapter steps out where an act needs the
    cube, and the replay on 733967aa (which stays stood back) spends the same frame."""

    @classmethod
    def setUpClass(cls):
        import ast

        cls.source = (HERE.parent / "demo/tour/chapters/construct.py").read_text()
        tree = ast.parse(cls.source)
        cls.functions = {f.name: f for f in tree.body if isinstance(f, ast.FunctionDef)}

    def step_out(self):
        import ast

        if "step_out" not in self.functions:
            self.fail("construct.py has no step_out")
        scope = {}
        exec(compile(ast.Module([self.functions["step_out"]], []), "construct.py", "exec"), scope)
        return scope["step_out"]

    class Take:
        def __init__(self):
            self.frame, self.keys = 0, []

        def key(self, combo):
            self.keys.append(combo)
            self.frame += 1

    def test_the_step_out_presses_alt_escape_only_inside_a_face_and_spends_one_frame_either_way(self):
        for inside, keys in ((True, ["Alt+Escape"]), (False, [])):
            t = self.Take()
            self.step_out()(t, inside)
            self.assertEqual((t.keys, t.frame), (keys, 1), inside)

    def stepped_in(self):
        import ast
        import re as regex

        scope = {"NO_METHOD": regex.compile(r"no (?:exported )?method", regex.IGNORECASE)}
        exec(compile(ast.Module([self.functions["stepped_in"]], []), "construct.py", "exec"), scope)
        return scope["stepped_in"]

    class Old:
        """A session on 733967aa: construct exports no `cube.list`."""

        def call(self, look, method):
            raise RuntimeError(f"choractl call {look} {method} {{}}: choractl: {look}.{method} is no exported method (code 1)")

    class New:
        def __init__(self, mode):
            self.mode = mode

        def call(self, look, method):
            return {"mode": self.mode, "front": 0}

    def test_the_replay_without_cube_list_is_stood_back_and_does_not_stop(self):
        """The v8 take of construct stopped at `stepped_in` with `construct.cube.list is no exported method`, so c3..c2b
        were `baseline: none` (tour-fix r1 take)."""
        self.assertFalse(self.stepped_in()(self.Old()))
        self.assertTrue(self.stepped_in()(self.New("use")))
        self.assertFalse(self.stepped_in()(self.New("navigate")))

    def test_c5d_looks_at_the_stepped_in_face_and_c3_steps_out_before_it_turns(self):
        lines = self.source.splitlines()
        c5d = next(i for i, line in enumerate(lines) if '"04-13-back-on-face"' in line)
        self.assertIn('"mode": "use"', lines[c5d])
        self.assertNotIn("The cube with the window on the face", lines[c5d])
        turn = next(i for i in range(c5d, len(lines)) if 't.key("Ctrl+Alt+Right")' in lines[i])
        self.assertTrue(any("step_out(t, " in lines[i] for i in range(c5d, turn)), "no step out between c5d and the first turn")


@unittest.skipUnless(shutil.which("ffmpeg"), "no ffmpeg")
class VideoCheck(unittest.TestCase):
    def clip(self, *parts):
        """An mp4 of lavfi sources joined in order, 10 fps 160x90."""
        tmp = tempfile.mkdtemp(prefix="tour-video-")
        files = []
        for i, (src, secs) in enumerate(parts):
            path = f"{tmp}/p{i}.mp4"
            subprocess.run(["ffmpeg", "-v", "error", "-f", "lavfi", "-i", f"{src}{':' if '=' in src else '='}s=160x90:r=10:d={secs}", "-pix_fmt", "yuv420p", "-y", path], check=True)
            files.append(path)
        listing = pathlib.Path(tmp, "list.txt")
        listing.write_text("".join(f"file '{f}'\n" for f in files))
        out = f"{tmp}/clip.mp4"
        subprocess.run(["ffmpeg", "-v", "error", "-f", "concat", "-safe", "0", "-i", str(listing), "-c", "copy", "-y", out], check=True)
        return out

    def flash_clip(self, frames):
        """testsrc2 for 4 s at 10 fps with the given frame numbers painted over in one colour."""
        tmp = tempfile.mkdtemp(prefix="tour-flash-")
        enable = "+".join(f"eq(n,{n})" for n in frames)
        out = f"{tmp}/flash.mp4"
        vf = f"drawbox=x=0:y=0:w=iw:h=ih:color=0xcc6633:t=fill:enable='{enable}'"
        subprocess.run(["ffmpeg", "-v", "error", "-f", "lavfi", "-i", "testsrc2=s=160x90:r=10:d=4", "-vf", vf, "-pix_fmt", "yuv420p", "-y", out], check=True)
        return out

    def test_a_two_frame_flash_on_testsrc2_is_flagged(self):
        found = [f for f in video.check(self.flash_clip([20, 21])) if f["kind"] == "flicker"]
        self.assertEqual(len(found), 1)
        self.assertAlmostEqual(found[0]["start"], 2.0, delta=0.15)

    def test_a_one_frame_flash_on_testsrc2_is_flagged(self):
        found = [f for f in video.check(self.flash_clip([20])) if f["kind"] == "flicker"]
        self.assertEqual(len(found), 1)

    def test_testsrc2_alone_has_no_flicker(self):
        self.assertEqual([f for f in video.check(self.flash_clip([999])) if f["kind"] == "flicker"], [])

    def test_a_missing_ffmpeg_is_a_clear_error(self):
        clip = self.clip(("testsrc2", 1))
        code, out, err = run_tool("tour-video-check.py", clip, path="")
        self.assertEqual(code, 2)
        self.assertIn("ffmpeg missing: cannot check", out + err)
        self.assertNotIn("Traceback", err)

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


class GateHook(unittest.TestCase):
    def test_the_acts_hook_fails_when_ffmpeg_is_absent(self):
        """rd-2: a runner with no ffmpeg must not pass the gate with the video tests skipped."""
        with tempfile.TemporaryDirectory() as tmp:
            for tool in ("bash", "dirname", "basename", "python3", "env", "mktemp", "flock", "cat", "rm", "mkdir", "sleep", "date", "sed", "tr", "true"):
                found = shutil.which(tool)
                if found:
                    os.symlink(found, f"{tmp}/{tool}")
            self.assertFalse(shutil.which("ffmpeg", path=tmp))
            done = subprocess.run([f"{tmp}/bash", str(HERE / "gate-acts.sh")], capture_output=True, text=True, env={"PATH": tmp, "HOME": tmp, "CHORA_GATE_LOCK_HELD": "1"})
            self.assertEqual(done.returncode, 1, done.stdout + done.stderr)
            self.assertIn("FAIL", done.stderr)
            self.assertIn("ffmpeg", done.stderr)


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
