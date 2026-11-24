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
        row = {"act": "x1", "kind": "check", "cause": "I9 is known red"}
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

    def test_a_check_row_without_a_name_still_absorbs_the_flips(self):
        rows = [{"act": "c2", "kind": "check", "cause": "co-44"}]
        v = diff.verdict("c2", [], True, rows, checks=({"I2": "green"}, {"I2": "red"}))
        self.assertEqual(v["verdict"], "change", v)

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

    def test_a_row_without_nodes_keeps_the_match_by_kind(self):
        row = {"act": "x5", "kind": "meta", "cause": "29281054 labels"}
        v = diff.verdict("x5", self.SCENE, True, [row])
        self.assertEqual((v["verdict"], v["cause"]), ("change", "29281054 labels"), v)

    def test_a_pixel_row_has_no_node(self):
        row = {"act": "x5", "kind": "pixel", "cause": "d78fc4bb pager", "nodes": ["xp-pager-1"]}
        v = diff.verdict("x5", [], False, [row])
        self.assertEqual((v["verdict"], v["cause"]), ("change", "d78fc4bb pager"), v)


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
        wide = [(r["act"], r["kind"], len(set(self.TOKEN.findall(r["cause"])))) for r in self.rows() if len(set(self.TOKEN.findall(r["cause"]))) > 3 and not r.get("open")]
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
                self.assertIn(r["kind"], ("meta", "rect", "missing", "added", "visible", "owner"), r)

    SESSION_CAPTIONS = ("fc9675ac", "11e34cf8", "94579f91")

    def test_no_row_of_the_xp_or_handover_acts_cites_the_session_captions(self):
        """rv9j-5: those commits change the captions of the session buttons, which no act of the xp chapter or of the
        handover shows; the blobs there are the pager (d78fc4bb) and the bin (62708d35)."""
        acts = {r["act"] for r in self.rows() if re.fullmatch(r"x\d+[a-z]?|h1[a-d]?", r["act"])}
        cited = [(r["act"], r["kind"]) for r in self.rows() if r["act"] in acts and any(c in r["cause"] for c in self.SESSION_CAPTIONS)]
        self.assertEqual(cited, [])

    def test_the_greeting_in_the_chat_is_no_label_change(self):
        """rv9j-5: the text of `hud-chat` is the greeting (4fec75c3), not the desktop labels (29281054)."""
        wrong = [(r["act"], r["kind"]) for r in self.rows() if any(n.startswith("hud-chat") for n in r.get("nodes", [])) and "29281054" in r["cause"]]
        self.assertEqual(wrong, [])

    UNPROVEN = ("cause not found", "(open)", "(proposed)")
    UNPROVEN_FINDINGS = ("mc-59", "co-63", "xp-43")
    """The findings whose ledger state is open with the cause not found (findings/mc.md:50, construct.md:65, xp.md:44)."""

    def test_an_open_row_says_what_is_open(self):
        self.assertEqual([(r["act"], r["kind"]) for r in self.rows() if r.get("open") and "open" not in r["cause"]], [])

    def test_a_row_whose_cause_is_unproven_is_open(self):
        """rtb2-1, two-way: a cause that says it is not found, open or proposed, or cites a finding that is open with
        no cause found, is `open = true`; else tour-diff prints `change` and a later unrelated diff passes."""
        marks = self.UNPROVEN + self.UNPROVEN_FINDINGS
        unproven = [(r["act"], r["kind"]) for r in self.rows() if any(m in r["cause"] for m in marks) and not r.get("open")]
        self.assertEqual(unproven, [])


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
