#!/usr/bin/python3
"""Audit the layout of the helper surfaces of the construct on the screen.

Reads the outputs (`choractl outputs`) and the nodes (`choractl query '*'`)
of a running session and prints one `FAIL:` line per finding:

  overlap   two helper surfaces share pixels and neither holds the other,
            or a surface lies on another that it does not name in its meta
            `in`
  outside   a surface or a text leaves the work area of its output
  image     an image is drawn at another size than its file has at the
            scale of the output, so it stretches or resamples, or it starts
            between two pixels of the output
  text      a text leaves the surface that it stands on, or comes closer
            than 2 pixels to its edge, or stands on no surface at all

A surface is a rect or an image of the owners with the meta `surface`, and
any other rect or image of them that no such surface holds. The meta
`surface=backdrop` marks a scrim, which is no helper surface; a rect or
image without the meta that covers the whole output is the space behind,
and a node named `*-glow` or `*-shadow` is the light around a surface. Nodes that
do not show (no rect, or an opacity of the node and its parents below
0.05) and the probes that measure text are left out. Shader nodes are
light around a surface, never a surface.

Usage: tests/hud-audit.py OUTPUTS.json NODES.json [OWNER...]
  OWNER  the clients whose nodes count (default: construct assistant)
  A sliced image is made to stretch; the image check leaves it out.
  AUDIT_KINDS  environment: the kinds to report, comma separated
         (overlap, outside, image, text; default all). The gates of the
         tiling and mc looks use outside,image and xp image: those looks mark no
         `surface`, so overlap and text findings would be guesses.
Exit status: 0 no finding, 1 a finding, 2 a usage error.
"""

import json
import os
import pathlib
import struct
import sys

EDGE = 2.0
SEEN = 0.05
NEAR = 0.01


def png_size(path):
    """The width and the height of a PNG file, or None."""
    try:
        with open(path, "rb") as f:
            head = f.read(24)
    except OSError:
        return None
    if len(head) < 24 or head[:8] != b"\x89PNG\r\n\x1a\n":
        return None
    return struct.unpack(">II", head[16:24])


def opacity(node, by_id):
    """The opacity of a node times that of its parents."""
    level = 1.0
    seen = set()
    while node is not None and node["id"] not in seen:
        seen.add(node["id"])
        if node.get("visible") is False:
            return 0.0
        level *= node.get("opacity", 1.0)
        node = by_id.get(node.get("parent"))
    return level


def inside(a, b, slack=0.0):
    """Whether rect a lies inside rect b, b grown by slack."""
    return (
        a[0] >= b[0] - slack - NEAR
        and a[1] >= b[1] - slack - NEAR
        and a[0] + a[2] <= b[0] + b[2] + slack + NEAR
        and a[1] + a[3] <= b[1] + b[3] + slack + NEAR
    )


def overlap(a, b):
    """Whether two rects share an area."""
    return (
        a[0] < b[0] + b[2] - NEAR
        and b[0] < a[0] + a[2] - NEAR
        and a[1] < b[1] + b[3] - NEAR
        and b[1] < a[1] + a[3] - NEAR
    )


def work_area(output):
    """The work area of an output in its own logical pixels."""
    area = output.get("default_work_area") or [0, 0, 0, 0]
    if area == [0, 0, 0, 0]:
        area = output.get("work_area") or [0, 0, 0, 0]
    if area[2] <= 0 or area[3] <= 0:
        return [0, 0, output["size"][0], output["size"][1]]
    px, py = output["position"]
    return [area[0] - px, area[1] - py, area[2], area[3]]


def shown(nodes, owners):
    """The nodes of the owners that show, with their rect per output."""
    by_id = {n["id"]: n for n in nodes}
    out = []
    for node in nodes:
        meta = node.get("meta", {})
        if node.get("owner") not in owners or meta.get("probe"):
            continue
        if node.get("kind") not in ("rect", "image", "text"):
            continue
        if not node.get("rects") or opacity(node, by_id) < SEEN:
            continue
        out.append(node)
    return out


def light(node):
    """Whether an image is the glow or the shadow around a surface."""
    return node["id"].endswith(("-glow", "-shadow"))


def surfaces_on(nodes, output, screen):
    """The helper surfaces on an output: (id, rect, meta)."""
    flat = []
    for node in nodes:
        rect = node["rects"].get(output)
        if rect is None or node["kind"] == "text" or rect[2] <= 0 or rect[3] <= 0:
            continue
        meta = node.get("meta", {})
        if meta.get("surface") == "backdrop" or light(node):
            continue
        if "surface" not in meta and inside([0, 0, *screen], rect):
            continue
        flat.append((node["id"], rect, meta))
    marked = [s for s in flat if "surface" in s[2]]
    free = [
        s
        for s in flat
        if "surface" not in s[2] and not any(inside(s[1], m[1]) for m in marked)
    ]
    loose = [s for s in free if not any(o is not s and inside(s[1], o[1]) for o in free)]
    return marked + loose


def audit(outputs, nodes, owners):
    """The findings, one line each."""
    findings = []
    visible = shown(nodes, owners)
    for output in outputs:
        name, scale = output["name"], output.get("scale", 1.0)
        area = work_area(output)
        surfaces = surfaces_on(visible, name, output["size"])
        for i, (a, ra, ma) in enumerate(surfaces):
            if not inside(ra, area):
                findings.append(f"outside: {name}: {a} {fmt(ra)} leaves the work area {fmt(area)}")
            for b, rb, mb in surfaces[i + 1 :]:
                if not overlap(ra, rb):
                    continue
                holds = inside(ra, rb) and ma.get("in") == mb.get("surface")
                held = inside(rb, ra) and mb.get("in") == ma.get("surface")
                legacy = "surface" not in ma or "surface" not in mb
                if holds or held or (legacy and (inside(ra, rb) or inside(rb, ra))):
                    continue
                findings.append(f"overlap: {name}: {a} {fmt(ra)} and {b} {fmt(rb)}")
        for node in visible:
            rect = node["rects"].get(name)
            if rect is None:
                continue
            if node["kind"] == "image":
                findings.extend(image_findings(node, rect, name, scale))
            if node["kind"] == "text":
                findings.extend(text_findings(node, rect, name, area, surfaces))
    return findings


def fmt(rect):
    """A rect as x,y wxh with at most one decimal."""
    return "{:g},{:g} {:g}x{:g}".format(*(round(v, 1) for v in rect))


def image_findings(node, rect, output, scale):
    """An image that does not show its file pixel for pixel."""
    path = node.get("image")
    size = png_size(path) if path and not node.get("slice") else None
    if size is None:
        return []
    want = [size[0] / scale, size[1] / scale]
    findings = []
    if abs(rect[2] - want[0]) > 0.5 / scale or abs(rect[3] - want[1]) > 0.5 / scale:
        findings.append(
            f"image: {output}: {node['id']} shows {pathlib.Path(path).name} "
            f"of {size[0]}x{size[1]} px at {rect[2]:g}x{rect[3]:g} at scale {scale:g}"
        )
    for v in rect[:2]:
        if abs(v * scale - round(v * scale)) > NEAR:
            findings.append(
                f"image: {output}: {node['id']} starts between pixels at {fmt(rect)}, scale {scale:g}"
            )
            break
    return findings


def text_findings(node, rect, output, area, surfaces):
    """A text that leaves its surface or the work area."""
    text = node.get("text", "").strip()
    if not text:
        return []
    label = f"{node['id']} {text[:40]!r} {fmt(rect)}"
    if not inside(rect, area):
        return [f"outside: {output}: {label} leaves the work area {fmt(area)}"]
    centre = [rect[0] + rect[2] / 2, rect[1] + rect[3] / 2]
    under = [
        s
        for s in surfaces
        if s[1][0] <= centre[0] <= s[1][0] + s[1][2] and s[1][1] <= centre[1] <= s[1][1] + s[1][3]
    ]
    if not under:
        return [f"text: {output}: {label} stands on no surface"]
    sid, srect, _ = min(under, key=lambda s: s[1][2] * s[1][3])
    room = [srect[0] + EDGE, srect[1] + EDGE, srect[2] - 2 * EDGE, srect[3] - 2 * EDGE]
    if inside(rect, room):
        return []
    return [f"text: {output}: {label} leaves {sid} {fmt(srect)}"]


def main():
    """Audit the files of the command line."""
    if len(sys.argv) < 3:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    outputs = json.loads(pathlib.Path(sys.argv[1]).read_text())
    nodes = json.loads(pathlib.Path(sys.argv[2]).read_text())
    owners = set(sys.argv[3:]) or {"construct", "assistant"}
    kinds = os.environ.get("AUDIT_KINDS")
    findings = audit(outputs, nodes, owners)
    if kinds:
        findings = [f for f in findings if f.split(":", 1)[0] in kinds.split(",")]
    for line in findings:
        print(f"FAIL: {line}")
    if not findings:
        print(f"layout audit: {len(shown(nodes, owners))} nodes, no finding")
    sys.exit(1 if findings else 0)


if __name__ == "__main__":
    main()
