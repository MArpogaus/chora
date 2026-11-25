#!/usr/bin/python3
"""Compare two tour captures per act and capture (design/tour-v9.md section 6).

Usage: tests/tour-diff.py BASE CAND [--intended demo/tour/intended.toml] [--json]

BASE and CAND are folders (the replay on 733967aa and the v9 take) with, per act and capture,
`<act>.<capture>.png` (a still, 8-bit RGB or RGBA, no pointer) and `<act>.<capture>.json`:
`{"nodes": [scene.query nodes], "masks": [[x, y, w, h], ...]}`; `checks` optionally maps a check
name to "green" or "red". A capture with no file in BASE has `baseline: none` only when
intended.toml lists it in `baseline_none = ["<act>.<capture>"]` or the `baseline-none.json` of BASE (or of the folder
above it) names it (rv-3); any other absence is a regression, and
so is a capture in BASE that CAND lacks. No capture at all (an empty or missing folder) exits 1.

Two diffs per capture:
- scene diff (every capture, also H50): nodes matched by id, windows by app_id and title; visible,
  rect (more than 2 px), meta text, owner, the z-order of the top layers, a node that is gone or new.
- pixel diff (not H50): clock and text nodes and the listed masks are ignored; a pixel differs when
  a channel differs by more than 24; same = at most 0.1 % of the unmasked pixels differ and no
  8-connected blob of them has a box over 12x12.

Verdict: same, change (every diff is named in intended.toml: act, kind, cause), regression (an
unnamed diff, a check green on BASE that is not green on CAND, a check red on CAND that is not red on
BASE, a capture missing in either folder), none (a declared `baseline: none`, with no red check), open
(every diff has a row, and a row carries `open = true`: the cause is not found; the F7 count fails on it,
exit 1). A red check that is meant has a row of kind "check" and `check = "I2"` (the name of the check it
absorbs; without it the row absorbs every flip of the act). A check that either side marks "not measured"
is left out of the flips and listed in the row as `unmeasured` (`cand_red` where the candidate reads it red). Exit 1 on a regression, an open act or no capture.
"""

import collections
import importlib.util
import json
import pathlib
import re
import sys
import tomllib

NOT_MEASURED = "not measured"
"""The colour of a check that a build could not read (I6 on a build without `scene.boxes`: 733967aa):
neither green nor red, so a candidate that measures it red is no flip (TOUR-BASE)."""

RECT_SLACK = 2
CHANNEL = 24
FRACTION = 0.001
BLOB = 12
TOP_LAYERS = 8

_spec = importlib.util.spec_from_file_location("png_pixel", pathlib.Path(__file__).with_name("png-pixel.py"))
png = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(png)


CLOCK_NODE = re.compile(r"^xp-tray-[A-Z]+-\d+$")
"""The only nodes whose meta text is the wall clock: the xp tray of one output ("1:54 AM"); the other
clocks (mc-clock, construct-door-clock) carry no text in their meta. The product reads the system
time (plugins/chora-xp/src/wm/dispatch.rs `now`), so two runs never agree on it (TOUR-CLOCK)."""


BAR_CLOCK_NODE = "tiling"
BAR_CLOCK = (154, 8, 32)
"""The clock island of the tiling bar (waybar, a foreign surface: no scene node, no text): width,
top and height, centred on the output (flavors/tiling/waybar/config.jsonc `modules-center`). Masked
only where the node `tiling` is in the scene (RV9-1)."""


def key_of(node):
    """A window by app_id and title, any other node by id."""
    if node.get("kind") == "surface":
        app = (node.get("surface") or {}).get("app_id")
        title = (node.get("meta") or {}).get("title")
        if app or title:
            return ("window", app, title)
    return ("node", node.get("id"))


def keyed(nodes):
    """The nodes by key in list order; a repeated key gets a counter."""
    seen, out = collections.Counter(), {}
    for node in nodes:
        k = key_of(node)
        seen[k] += 1
        out[(*k, seen[k])] = node
    return out


def label(k):
    return k[1] if k[0] == "node" else f"{k[1]}:{k[2]}"


def scene_diff(a, b):
    """The differences of two scene.query answers, as dicts with kind, id and detail."""
    ka, kb = keyed(a), keyed(b)
    found = []
    for k, na in ka.items():
        nb = kb.get(k)
        name = label(k)
        if nb is None:
            found.append({"kind": "missing", "id": name, "detail": "in the baseline only"})
            continue
        if na.get("visible", True) != nb.get("visible", True):
            found.append({"kind": "visible", "id": name, "detail": f"{na.get('visible', True)} -> {nb.get('visible', True)}"})
        for output in sorted(set(na.get("rects") or {}) | set(nb.get("rects") or {})):
            ra, rb = (na.get("rects") or {}).get(output), (nb.get("rects") or {}).get(output)
            if ra is None or rb is None or any(abs(x - y) > RECT_SLACK for x, y in zip(ra, rb)):
                found.append({"kind": "rect", "id": name, "detail": f"{output} {ra} -> {rb}"})
        ta, tb = (na.get("meta") or {}).get("text"), (nb.get("meta") or {}).get("text")
        if ta != tb and not (k[0] == "node" and CLOCK_NODE.match(str(k[1]))):
            found.append({"kind": "meta", "id": name, "detail": f"{ta!r} -> {tb!r}"})
        if na.get("owner") != nb.get("owner"):
            found.append({"kind": "owner", "id": name, "detail": f"{na.get('owner')} -> {nb.get('owner')}"})
    for k in kb:
        if k not in ka:
            found.append({"kind": "added", "id": label(k), "detail": "in the candidate only"})
    common = [k for k in ka if k in kb]
    top_a = [k for k in common][-TOP_LAYERS:]
    top_b = [k for k in kb if k in ka][-TOP_LAYERS:]
    if top_a != top_b:
        found.append({"kind": "z-order", "id": "top layers", "detail": f"{[label(k) for k in top_a]} -> {[label(k) for k in top_b]}"})
    return found


def blobs_of(points):
    """The 8-connected blobs of a set of (x, y) points, as (x, y, w, h) boxes."""
    left, out = set(points), []
    while left:
        stack = [left.pop()]
        x0 = x1 = stack[0][0]
        y0 = y1 = stack[0][1]
        while stack:
            x, y = stack.pop()
            x0, x1, y0, y1 = min(x0, x), max(x1, x), min(y0, y), max(y1, y)
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    p = (x + dx, y + dy)
                    if p in left:
                        left.remove(p)
                        stack.append(p)
        out.append((x0, y0, x1 - x0 + 1, y1 - y0 + 1))
    return sorted(out)


def pixel_diff(a, b, masks=()):
    """Compare two PNG files (bytes): same, the fraction of differing unmasked pixels, the big blobs."""
    ra, rb = list(png.rows(a)), list(png.rows(b))
    if len(ra) != len(rb) or len(ra[0]) != len(rb[0]):
        return {"same": False, "size": [len(ra[0]) // 4, len(ra), len(rb[0]) // 4, len(rb)], "blobs": [], "fraction": 1.0}
    height, width = len(ra), len(ra[0]) // 4

    def masked(x, y):
        return any(mx <= x < mx + mw and my <= y < my + mh for mx, my, mw, mh in masks)

    points, unmasked = [], width * height
    for y in range(height):
        for mx, my, mw, mh in masks:
            if my <= y < my + mh:
                unmasked -= max(0, min(width, mx + mw) - max(0, mx))
        if ra[y] == rb[y]:
            continue
        for x in range(width):
            i = x * 4
            if ra[y][i : i + 3] != rb[y][i : i + 3] and max(abs(ra[y][i + c] - rb[y][i + c]) for c in range(3)) > CHANNEL and not masked(x, y):
                points.append((x, y))
    big = [box for box in blobs_of(points) if box[2] > BLOB or box[3] > BLOB]
    fraction = len(points) / max(unmasked, 1)
    return {"same": fraction <= FRACTION and not big, "fraction": fraction, "blobs": big}


def named(rows, kinds):
    """The row of the first kind in `kinds` that a row names, an `open` row before one with a cause found:
    the open diff is not hidden by a found one of the same kind."""
    hits = [r for r in rows if r["kind"] in kinds]
    return next((r for r in hits if r.get("open")), hits[0] if hits else None)


def verdict(act, scene, pixel_same, intended, checks=None):
    """same, change, open or regression for one capture; `checks` is (baseline, candidate) name -> colour.
    A row with `open = true` names a diff whose cause is not found: the verdict is `open`, not `change`."""
    kinds = [d["kind"] for d in scene] + ([] if pixel_same else ["pixel"])
    rows = [r for r in intended if r["act"] == act]
    causes, unnamed, opened = [], [], False
    for kind in dict.fromkeys(kinds):
        row = named(rows, (kind, "*"))
        if row:
            causes.append(row["cause"])
            opened = opened or bool(row.get("open"))
        else:
            unnamed.append(kind)
    base, cand = checks or ({}, {})
    unmeasured = sorted(n for n in set(base) | set(cand) if NOT_MEASURED in (base.get(n), cand.get(n)))
    cand_red = [n for n in unmeasured if cand.get(n) == "red"]
    cand_lost = [n for n in unmeasured if cand.get(n) == NOT_MEASURED and base.get(n) != NOT_MEASURED]
    base, cand = ({n: c for n, c in side.items() if n not in unmeasured} for side in (base, cand))
    flipped = sorted({n for n, c in base.items() if c == "green" and cand.get(n) != "green"} | {n for n, c in cand.items() if c == "red" and base.get(n) != "red"} | set(cand_red) | set(cand_lost))
    absorbed = [n for n in flipped if any(r["kind"] in ("check", "*") and r.get("check", n) == n for r in rows)]
    for n in absorbed:
        row = next(r for r in rows if r["kind"] in ("check", "*") and r.get("check", n) == n)
        causes.append(row["cause"])
        opened = opened or bool(row.get("open"))
    flipped = [n for n in flipped if n not in absorbed]
    extra = ({"unmeasured": unmeasured} if unmeasured else {}) | ({"cand_red": cand_red} if cand_red else {})
    if unnamed or flipped:
        return {"verdict": "regression", "unnamed": unnamed, "flipped": flipped, **extra}
    if causes:
        return {"verdict": "open" if opened else "change", "cause": "; ".join(dict.fromkeys(causes)), **extra}
    return {"verdict": "same", **extra}


def masks_of(*sides, width=None):
    """Rects to ignore: clock and text nodes of both builds, the listed masks, and with `width` (of the
    output) the clock island of the tiling bar where a build has the node `tiling`."""
    out = []
    for side in sides:
        if width and any(n.get("id") == BAR_CLOCK_NODE for n in side.get("nodes", [])):
            w, top, h = BAR_CLOCK
            out.append(((width - w) // 2, top, w, h))
        for node in side.get("nodes", []):
            meta = node.get("meta") or {}
            if "clock" in str(node.get("id")) or node.get("kind") == "text" or "text" in meta:
                out += [tuple(r) for r in (node.get("rects") or {}).values()]
        out += [tuple(m) for m in side.get("masks", [])]
    return out


def compare(base, cand, intended, none=()):
    """The rows for every capture in BASE or CAND: act, capture, verdict and what differs; `none`
    lists the captures (`act.capture`) that have no baseline by design."""
    base, cand, rows = pathlib.Path(base), pathlib.Path(cand), []
    names = sorted({p.name for p in base.glob("*.png")} | {p.name for p in cand.glob("*.png")})
    for name in names:
        stem = name[: -len(".png")]
        act, _, capture = stem.partition(".")
        row = {"act": act, "capture": capture}
        if not (cand / name).exists():
            rows.append({**row, "verdict": "regression", "unnamed": ["missing capture"], "flipped": [], "detail": "in the baseline only"})
            continue
        side = {}
        for key, folder in (("base", base), ("cand", cand)):
            path = folder / f"{stem}.json"
            side[key] = json.loads(path.read_text()) if path.exists() else {}
        if not (base / name).exists():
            if stem in none:
                result = verdict(act, [], True, intended, ({}, side["cand"].get("checks", {})))
                rows.append({**row, **(result if result["verdict"] == "regression" else {"verdict": "none", "detail": "baseline: none"})})
            else:
                rows.append({**row, "verdict": "regression", "unnamed": ["baseline missing"], "flipped": [], "detail": "no baseline and not listed in baseline_none"})
            continue
        scene = scene_diff(side["base"].get("nodes", []), side["cand"].get("nodes", []))
        if capture == "H50":
            pixel = {"same": True, "blobs": [], "fraction": 0.0}
        else:
            pixel = pixel_diff((base / name).read_bytes(), (cand / name).read_bytes(), masks_of(side["base"], side["cand"], width=int.from_bytes((base / name).read_bytes()[16:20], "big")))
        checks = (side["base"].get("checks", {}), side["cand"].get("checks", {}))
        rows.append({**row, **verdict(act, scene, pixel["same"], intended, checks), "scene": scene, "blobs": pixel["blobs"], "fraction": pixel["fraction"]})
    return rows


def none_of(base):
    """The `<act>.<capture>` names of `baseline-none.json` (run.py writes it in the folder of the take
    on the old build) in the base folder or the folder above it, where `captures/` of the take lies."""
    names = []
    for folder in (pathlib.Path(base), pathlib.Path(base).parent):
        path = folder / "baseline-none.json"
        if path.exists():
            names += [n for entry in json.loads(path.read_text()) for n in entry.get("names", [])]
    return names


def main():
    args = sys.argv[1:]
    intended_path, as_json = None, False
    if "--json" in args:
        args.remove("--json")
        as_json = True
    if "--intended" in args:
        i = args.index("--intended")
        intended_path = args[i + 1]
        del args[i : i + 2]
    if len(args) != 2:
        sys.exit(__doc__)
    table = tomllib.loads(pathlib.Path(intended_path).read_text()) if intended_path else {}
    rows = compare(args[0], args[1], table.get("change", []), table.get("baseline_none", []) + none_of(args[0]))
    if as_json:
        print(json.dumps(rows, indent=1))
    else:
        for r in rows:
            extra = r.get("cause") or ", ".join(r.get("unnamed", []) + r.get("flipped", [])) or r.get("detail", "")
            if r.get("unmeasured"):
                red = r.get("cand_red", [])
                extra = "; ".join(filter(None, [extra, ", ".join(f"{n} not measured" + (" (cand red)" if n in red else "") for n in r["unmeasured"])]))
            print(f"{r['act']:<8} {r['capture']:<5} {r['verdict']:<11} {extra}")
        count = collections.Counter(r["verdict"] for r in rows)
        print("tour-diff:", ", ".join(f"{count[v]} {v}" for v in ("same", "change", "open", "regression", "none")))
    if not rows:
        print("tour-diff: no capture in BASE or CAND", file=sys.stderr)
        return 1
    return 1 if any(r["verdict"] in ("regression", "open") for r in rows) else 0


if __name__ == "__main__":
    sys.exit(main())
