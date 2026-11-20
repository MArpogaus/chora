"""The first invariants of the act harness (design 3.3 of the arch plan).

Each invariant is a function of the snapshot before an act, the snapshot
after it and the act, and answers the breaches as sentences; an empty
list is a pass. They read only the snapshots (harness.Snapshot), so the
selftest tests/test_acts.py plants a breach without a session.

I1   every toplevel is attached, owned by one look and in that look's
     `windows`; a `view` is not an owner
I3   once quiet, the core focus is a shown window of the one look that
     says it holds the focus
I4   `open` after the act is `open` before plus what the act declared
I7   the work area of an output whose look draws the wallpaper shows
     more than one colour
I8   the helper is not `working` after its answer event; one helper
     surface per output
I14  during the act: no `effect.failed`, no panic line, no disconnect
     the act did not declare

The box invariants read `scene.boxes` (Snapshot.boxes) and the `texts`
that looks and chora-notify report in `state.get`:

I2   a shown window has a size and lies at least 32 px inside its
     output, except declared floats; the gaps of helper surfaces (hud-audit.py)
     and of tiles (tiling-audit.py) are whole units; in a held act a helper
     surface stays on its pixel
I5   no two text boxes of visible nodes or reported runs overlap on one
     output without an opaque node between
I6   a text box lies inside its panel; its text reads at least 14 px
     (a state word may be smaller) and has the contrast of the guide on the
     capture: 4.5:1, 7:1 for a key line or a button (png-contrast.py)
I9   a notice chora-notify accepted during the act is in its `shown`,
     `waiting` or `closed`; one that shows has a box inside the output,
     not under the chat
I10  no visible text matches a raw entry (`.desktop`, `%U`, a reverse-DNS
     id, a path of a binary), says "agent" or "HUD", or spells a key as a
     long name that tests/keysheet/words.table gives a cap (Escape, not Esc)
I12  no bar or panel box over a fullscreen window; the held window on
     top; a modal above its parent
I13  the projected corners of neighbouring cube faces meet within 1 px;
     held in a turn, a face that shows has a neighbour that shows with
     corners (the faces that face away have no box)

Limits: the texts are text nodes and the runs that chora-draw reports;
text of outside programs (waybar, mako) and SVG text in a picture are not
seen. Images count as not opaque, so a menu image over a label does not
hide it for I5. I12 sees a held window only where a look reports `held`
in its `windows`, which acts snapshot once quiet. I13 reads the faces
as the acts hold them, at rest and halfway through a turn.
"""

import importlib.util
import pathlib
import re

TESTS = pathlib.Path(__file__).resolve().parent.parent
PANIC = re.compile(r"panicked at|^thread '.*' panicked|RUST_BACKTRACE=")
HELPER_IDS = {"hud-chat"}
HELPER_APP_IDS = {"chora-chat"}


def looks_with_state(snap):
    """The looks of the snapshot that answered `state.get`."""
    return {look: state for look, state in snap.states.items() if state is not None}


def windows_of(state):
    """The surfaces of the `windows` of a state, by id."""
    return {w.get("surface"): w for w in state.get("windows") or [] if w.get("surface")}


def missing_keys(snap, *keys):
    """A breach for each look that answers `state.get` without a key."""
    return [
        f"{look} missing key {key}: its state.get does not answer it"
        for look, state in sorted(looks_with_state(snap).items())
        for key in keys
        if key not in state
    ]


def i1(_before, after, _act):
    """Every toplevel is attached, owned by one look and in its windows."""
    out = [f"{look} answers no state.get" for look, s in after.states.items() if s is None]
    states = looks_with_state(after)
    out += missing_keys(after, "windows")
    if out:
        return out
    owned = {look: windows_of(state) for look, state in states.items()}
    for look, state in states.items():
        for view in state.get("views") or []:
            if view.get("surface") in owned[look]:
                out.append(f"{look} lists {view.get('surface')} as a view and as its window")
    nodes = after.by_id()
    for sid in sorted(after.toplevels):
        node = nodes.get(sid)
        if node is None or node.get("parent") is None:
            out.append(f"the toplevel {sid} is detached")
            continue
        owner = node.get("owner")
        holders = sorted(look for look, windows in owned.items() if sid in windows)
        if owner not in states:
            out.append(f"the toplevel {sid} is owned by {owner}, which is no look")
        elif holders != [owner]:
            out.append(f"the toplevel {sid} of {owner} is in the windows of {holders or 'no look'}")
    return out


def i3(_before, after, _act):
    """The core focus is a shown window of the one look that holds it."""
    states = looks_with_state(after)
    if gaps := missing_keys(after, "focus"):
        return gaps
    claims = sorted(
        look for look, s in states.items() if (s.get("focus") or {}).get("holder") == look
    )
    if len(claims) > 1:
        return [f"{' and '.join(claims)} each say they hold the focus"]
    claimer = claims[0] if claims else None
    focused = [n for n in after.nodes if n.get("focused")]
    surfaces = [n for n in focused if n.get("kind") == "surface"]
    nodes = [n for n in focused if n.get("kind") != "surface"]
    target = nodes[0] if nodes else surfaces[0] if surfaces else None
    if target is None:
        named = claimer and (states[claimer].get("focus") or {}).get("surface")
        return [f"{claimer} says it holds {named}, the core focuses nothing"] if named else []
    owner = target.get("owner")
    if claimer is None:
        return [f"the core focuses {target['id']} of {owner}, no look says it holds the focus"]
    if owner != claimer:
        return [f"the core focuses {target['id']} of {owner}, {claimer} says it holds the focus"]
    if nodes:
        return []
    sid = target["id"]
    window = windows_of(states[claimer]).get(sid)
    if window is None or not window.get("shown"):
        return [f"the focused {sid} is no shown window of {claimer}"]
    named = (states[claimer].get("focus") or {}).get("surface")
    if named not in (None, sid):
        return [f"{claimer} says it focuses {named}, the core focuses {sid}"]
    return []


def open_ids(snap):
    """The ids of what is open in all looks."""
    return {o.get("id") for s in looks_with_state(snap).values() for o in s.get("open") or []}


def i4(before, after, act):
    """What is open after the act is what was open plus the declared."""
    was, now = open_ids(before), open_ids(after)
    opens, closes = set(act.opens), set(act.closes)
    if gaps := missing_keys(after, "open"):
        return gaps
    out = [f"{i} opened, the act did not declare it" for i in sorted(now - was - opens)]
    out += [f"{i} was declared open and is not" for i in sorted(opens - now)]
    out += [f"{i} closed, the act did not declare it" for i in sorted(was - now - closes)]
    return out


def i7(_before, after, _act):
    """The work area of a wallpaper output has more than one colour."""
    out = []
    for output in sorted(after.wallpaper):
        count = after.colours.get(output)
        if count is None:
            out.append(f"no capture of the work area of {output}")
        elif count <= 1:
            out.append(f"the work area of {output} shows one colour")
    return out


def helper_surfaces(node):
    """Whether a node is a surface of a helper: the chat of the HUD host,
    a node a look marks with meta `helper: surface`, or a chat window."""
    surface = node.get("surface") or {}
    return (
        node.get("id") in HELPER_IDS
        or (node.get("meta") or {}).get("helper") == "surface"
        or surface.get("app_id") in HELPER_APP_IDS
    )


def i8(_before, after, _act):
    """The helper is not working after its answer; one helper surface per output."""
    out = []
    said = [
        (e.get("data") or {}).get("state") for e in after.events if e.get("event") == "voice.state"
    ]
    states = looks_with_state(after) | {
        name: state for name, state in after.blocks.items() if state and "helper" in state
    }
    if said and states and not any("helper" in s for s in states.values()):
        out += [
            f"{look} missing key helper: the helper answered and no look reports it"
            for look in sorted(states)[:1]
        ]
    if said and said[-1] != "working":
        out += [
            f"{look} says the helper works after its answer ({said[-1]})"
            for look, s in states.items()
            if s.get("helper") == "working"
        ]
    helpers = {n["id"] for n in after.nodes if helper_surfaces(n)}
    if helpers and after.boxes is None:
        return [*out, "missing boxes: no scene.boxes answer for the helper surfaces"]
    per_output = {}
    for output, entry in sorted((after.boxes or {}).items()):
        for box in entry.get("boxes") or []:
            if box.get("node") in helpers and box["node"] not in per_output.get(output, []):
                per_output.setdefault(output, []).append(box["node"])
    out += [
        f"{output} shows {len(ids)} helper surfaces: {', '.join(ids)}"
        for output, ids in sorted(per_output.items())
        if len(ids) > 1
    ]
    return out


def i14(before, after, act):
    """No failed effect, no panic line, no undeclared disconnect."""
    out = [
        f"effect.failed: {str(e.get('error'))[:200]}"
        for e in after.events
        if e.get("event") == "effect.failed"
    ]
    for name, text in sorted(after.logs.items()):
        lines = [line for line in text.splitlines() if PANIC.search(line)]
        out += [f"{name} log: {line[:200]}" for line in lines[:3]]
    gone = before.clients - after.clients - set(act.disconnects)
    out += [f"{client} disconnected, the act did not declare it" for client in sorted(gone)]
    return out


MARGIN = 32
SEAM = 1.0
HELD_SLACK = 0.5
LEAST_PX = 14
STATE_WORD = re.compile(r"state")
KEY_LINE = re.compile(r"(?:^|[-_])(?:key|keycap|button|btn)(?:[-_]|$)")
READ = 4.5
READ_KEY = 7.0
LEADING = 0.25
RAW = [
    (re.compile(r"\.desktop\b"), "a .desktop file"),
    (re.compile(r"%[UuFfick]\b"), "a field code of Exec"),
    (
        re.compile(r"\b(?:org|com|net|io|de|ch|dev|app)\.[A-Za-z0-9_-]+\.[A-Za-z0-9_.-]+"),
        "a reverse-DNS id",
    ),
    (re.compile(r"(?:^|\s)/(?:usr/)?(?:local/)?s?bin/\S+"), "the path of a binary"),
]
WORDS = re.compile(r"\b(?:agents?|HUD)\b", re.I)
FACE = re.compile(r"-cube-face-(\d+)$")


def boxes_on(snap):
    """The boxes by output, bottom first; {} without a boxes answer."""
    return {o: e.get("boxes") or [] for o, e in (snap.boxes or {}).items()}


def node_boxes(snap):
    """{output: {node: (index, box)}} of the boxes of nodes."""
    return {
        o: {b["node"]: (i, b) for i, b in enumerate(boxes) if b.get("node")}
        for o, boxes in boxes_on(snap).items()
    }


def overlap(a, b, least=1.0):
    """The intersection [x, y, w, h] of two rects when both sides exceed
    `least`, else None."""
    x, y = max(a[0], b[0]), max(a[1], b[1])
    w = min(a[0] + a[2], b[0] + b[2]) - x
    h = min(a[1] + a[3], b[1] + b[3]) - y
    return [x, y, w, h] if w > least and h > least else None


def inside(inner, outer, slack=1.0):
    """Whether a rect lies inside another, with a slack in pixels."""
    return (
        inner[0] >= outer[0] - slack
        and inner[1] >= outer[1] - slack
        and inner[0] + inner[2] <= outer[0] + outer[2] + slack
        and inner[1] + inner[3] <= outer[1] + outer[3] + slack
    )


def reported_texts(snap):
    """Every run that a look or a block reports, as (reporter, run)."""
    sources = {**snap.blocks, **looks_with_state(snap)}
    return [
        (name, run)
        for name, state in sorted(sources.items())
        if state
        for run in state.get("texts") or []
    ]


def plain(m):
    """Whether a column-major 4x4 matrix is a 2D scale and translation."""
    flat = (1, 2, 3, 4, 6, 7, 8, 9, 11, 14)
    return all(abs(m[i]) < 1e-9 for i in flat) and m[0] > 0 and m[5] > 0


def matmul(a, b):
    """The product a * b of two column-major 4x4 matrices."""
    return [sum(a[k * 4 + r] * b[c * 4 + k] for k in range(4)) for c in range(4) for r in range(4)]


def matrix_of(node):
    """The matrix of the transform of a node; identity without one."""
    m = (node.get("transform") or {}).get("matrix")
    return m if m and len(m) == 16 else [1.0 if i % 5 == 0 else 0.0 for i in range(16)]


def placement(snap, nodes, output, box):
    """The matrix of a node on an output before any clip: the transforms
    of its ancestors and its own, and the camera of the output for a node
    of the scene. None when it is not a plain scale and translation."""
    chain = []
    node = nodes.get(box.get("node"))
    while node is not None:
        chain.append(matrix_of(node))
        node = nodes.get(node.get("parent"))
    matrix = [1.0 if i % 5 == 0 else 0.0 for i in range(16)]
    if box.get("layer") == "scene":
        info = next((o for o in snap.outputs if o.get("name") == output), {})
        matrix = (info.get("camera") or {}).get("matrix") or matrix
    for m in reversed(chain):
        matrix = matmul(matrix, m)
    return matrix if plain(matrix) else None


def clipped(rect, box):
    """A rect cut to a box; None when nothing is left."""
    return overlap(rect, box, 0.0)


def run_rect(snap, nodes, output, box, run):
    """The screen rect of a run of a picture: through the placement of its
    node before any clip, cut to the box afterwards, so a picture that a
    clip shows in part keeps its runs where they are. A node that does not
    place as a plain scale and translation, or whose placement does not
    hold the box, scales the run with its box against its size."""
    size = (nodes.get(run["node"]) or {}).get("size") or box["rect"][2:]
    x, y, w, h = run["rect"]
    m = placement(snap, nodes, output, box)
    if m is not None and size[0] and size[1]:
        whole = [m[12], m[13], m[0] * size[0], m[5] * size[1]]
        if inside(box["rect"], whole):
            rect = [m[12] + x * m[0], m[13] + y * m[5], w * m[0], h * m[5]]
            return clipped(rect, box["rect"])
    sx = box["rect"][2] / size[0] if size[0] else 1.0
    sy = box["rect"][3] / size[1] if size[1] else 1.0
    return [box["rect"][0] + x * sx, box["rect"][1] + y * sy, w * sx, h * sy]


def screen_texts(snap):
    """{output: [(index, label, text, rect)]}: the text boxes of text nodes
    and the reported runs, mapped through the placement of their node."""
    nodes = snap.by_id()
    placed = node_boxes(snap)
    out = {}
    for output, boxes in boxes_on(snap).items():
        items = out.setdefault(output, [])
        for i, box in enumerate(boxes):
            if box.get("kind") == "text" and box.get("text"):
                items.append((i, box["node"], box["text"], box["rect"]))
        for _name, run in reported_texts(snap):
            index, box = placed[output].get(run.get("node"), (None, None))
            if box is None:
                continue
            rect = run_rect(snap, nodes, output, box, run)
            if rect is not None:
                items.append((index, run["node"], run.get("text", ""), rect))
    return out


def tool(name):
    """A script of tests/ by file name (its name has a dash)."""
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), TESTS / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def fixed_surfaces(snap):
    """{output: {node: rect}}: the helper surfaces (meta `surface`, no
    scrim) that sit on the screen; the faces of a cube turn and are no
    part of it."""
    nodes = snap.by_id()
    out = {}
    for output, boxes in boxes_on(snap).items():
        for b in boxes:
            meta = (nodes.get(b.get("node")) or {}).get("meta") or {}
            if meta.get("surface") not in (None, "backdrop") and not FACE.search(b["node"]):
                out.setdefault(output, {})[b["node"]] = b["rect"]
    return out


def gap_breaches(after):
    """The gaps of helper surfaces and of tiles that are no whole unit."""
    out = []
    hud, tiling = tool("hud-audit"), tool("tiling-audit")
    for output, rects in sorted(fixed_surfaces(after).items()):
        out += hud.gap_findings(output, rects, hud.GAP_UNIT)
    for output, boxes in sorted(boxes_on(after).items()):
        tiles = [{"id": b["node"], "rects": {output: b["rect"]}} for b in boxes if b.get("node")]
        out += tiling.gap_findings(tiles, output)
    return out


def held_breaches(before, after):
    """A helper surface that moved in a held act: screen-fixed nodes stay
    on their pixel while the camera moves."""
    was = fixed_surfaces(before)
    return [
        f"on {output} the screen-fixed {node} moved from {was[output][node]} to {rect} "
        "in a held act"
        for output, rects in sorted(fixed_surfaces(after).items())
        for node, rect in sorted(rects.items())
        if node in was.get(output, {})
        and any(abs(a - b) > HELD_SLACK for a, b in zip(was[output][node], rect))
    ]


def i2(before, after, act):
    """A shown window has a size and lies 32 px inside its output; the
    gaps of surfaces and tiles are whole units; held, surfaces stay put."""
    out = gap_breaches(after)
    if act.held:
        out += held_breaches(before, after)
    placed = node_boxes(after)
    for look, state in sorted(looks_with_state(after).items()):
        for w in state.get("windows") or []:
            if not w.get("shown") or w.get("minimized") or w.get("floating"):
                continue
            output, sid = w.get("output"), w.get("surface")
            if output is None or output not in placed:
                continue
            size = (after.boxes[output] or {}).get("size") or [0, 0]
            _, box = placed[output].get(sid, (None, None))
            if box is None or box["rect"][2] <= 0 or box["rect"][3] <= 0:
                out.append(f"{look} shows {sid} on {output}, which has no box of a size there")
            elif not overlap(box["rect"], [0, 0, *size], MARGIN):
                out.append(f"{look} shows {sid} at {box['rect']}, not 32 px inside {output}")
    return out


def covered(boxes, low, high, rect):
    """Whether an opaque box between two indices covers a rect."""
    return any(b.get("opaque") and inside(rect, b["rect"], 0.0) for b in boxes[low + 1 : high + 1])


def i5(_before, after, _act):
    """No two text boxes overlap on one output without an opaque node
    between. The line box of a text node holds its leading, so lines
    whose boxes share a quarter of the smaller height or less pass."""
    out = []
    boxes = boxes_on(after)
    for output, items in sorted(screen_texts(after).items()):
        for n, (ia, la, ta, ra) in enumerate(items):
            for ib, lb, tb, rb in items[n + 1 :]:
                where = overlap(ra, rb)
                if where is None or where[3] <= LEADING * min(ra[3], rb[3]):
                    continue
                low, high = sorted((ia, ib))
                if low != high and covered(boxes[output], low, high, where):
                    continue
                out.append(f"on {output} {ta!r} ({la}) overlaps {tb!r} ({lb}) at {where}")
    return out


def text_sizes(snap):
    """{(label, text): px}: the font size of each text box and reported run."""
    sizes = {
        (b["node"], b["text"]): b["font_size"]
        for boxes in boxes_on(snap).values()
        for b in boxes
        if b.get("kind") == "text" and b.get("text") and b.get("font_size")
    }
    for _name, run in reported_texts(snap):
        if run.get("px"):
            sizes[(run.get("node"), run.get("text", ""))] = run["px"]
    return sizes


def guide_breaches(snap):
    """Text that is too small, and text of too little contrast on the capture."""
    out = []
    sizes = text_sizes(snap)
    contrast = tool("png-contrast")
    for output, items in sorted(screen_texts(snap).items()):
        scale, rows = snap.shots.get(output) or (None, None)
        for _i, label, words, rect in items:
            px = sizes.get((label, words))
            if px is not None and px < LEAST_PX and not STATE_WORD.search(label):
                out.append(f"on {output} {words!r} ({label}) is {px:g} px, the guide wants {LEAST_PX} px")
            if rows is None or rect[2] < 1 or rect[3] < 1:
                continue
            x, y = max(int(rect[0] * scale), 0), max(int(rect[1] * scale), 0)
            w = min(int((rect[0] + rect[2]) * scale), len(rows[0]) // 4) - x
            h = min(int((rect[1] + rect[3]) * scale), len(rows)) - y
            if w < 2 or h < 2:
                continue
            ratio, back, fore = contrast.contrast_rows(rows, x, y, w, h)
            want = READ_KEY if KEY_LINE.search(label) else READ
            if back != fore and ratio < want:
                out.append(
                    f"on {output} {words!r} ({label}) has the contrast {ratio:.2f}:1 "
                    f"({fore} on {back}), the guide wants {want:g}:1"
                )
    return out


def i6(_before, after, _act):
    """A text box lies inside its panel: a run inside its picture, a text
    node inside the box of its nearest ancestor that has one; its size and
    contrast are the guide's."""
    out = guide_breaches(after)
    nodes = after.by_id()
    for name, run in reported_texts(after):
        size = (nodes.get(run.get("node")) or {}).get("size")
        if size and not inside(run["rect"], [0, 0, *size]):
            out.append(f"{name}: {run.get('text')!r} at {run['rect']} leaves its picture {size}")
    for output, placed in sorted(node_boxes(after).items()):
        for node, (_, box) in placed.items():
            if box.get("kind") != "text":
                continue
            parent = (nodes.get(node) or {}).get("parent")
            while parent and parent not in placed:
                parent = (nodes.get(parent) or {}).get("parent")
            if parent and not inside(box["rect"], placed[parent][1]["rect"]):
                out.append(
                    f"on {output} {box.get('text')!r} at {box['rect']} leaves its panel "
                    f"{parent} {placed[parent][1]['rect']}"
                )
    return out


def i9(before, after, _act):
    """A notice that showed during the act has a box inside the output,
    not under a helper surface."""
    state = after.blocks.get("notify")
    if not state:
        return []
    if gaps := [
        f"notify missing key {key}: its state.get does not answer it"
        for key in ("shown", "accepted", "waiting", "closed")
        if key not in state
    ]:
        return gaps
    was = {n.get("id") for n in (before.blocks.get("notify") or {}).get("shown") or []}
    old = set((before.blocks.get("notify") or {}).get("accepted") or [])
    kept = {n.get("id") for n in state["shown"]} | set(state["waiting"]) | set(state["closed"])
    out = [
        f"the notice {i} was accepted during the act and never shown: "
        "it is not in shown, waiting or closed"
        for i in state["accepted"]
        if i not in old and i not in kept
    ]
    helpers = {n["id"] for n in after.nodes if helper_surfaces(n)}
    for notice in state["shown"]:
        if notice.get("id") in was:
            continue
        shown = [
            (o, i, b, boxes)
            for o, boxes in boxes_on(after).items()
            for i, b in enumerate(boxes)
            if b.get("node") == notice.get("node")
        ]
        if not shown:
            out.append(f"the notice {notice.get('summary')!r} has no box on any output")
        for output, i, box, boxes in shown:
            size = after.boxes[output].get("size") or [0, 0]
            if not inside(box["rect"], [0, 0, *size]):
                out.append(f"the notice {notice.get('summary')!r} leaves {output}: {box['rect']}")
            out += [
                f"the notice {notice.get('summary')!r} is under the helper {b['node']} on {output}"
                for b in boxes[i + 1 :]
                if b.get("node") in helpers and overlap(box["rect"], b["rect"])
            ]
    return out


def long_key_names():
    """The keys whose cap in tests/keysheet/words.table is not their name."""
    caps = {}
    for line in (TESTS / "keysheet" / "words.table").read_text().splitlines():
        fields = line.split("\t")
        if fields[0] == "cap" and len(fields) > 2 and fields[1] != fields[2]:
            caps[fields[1]] = fields[2]
    return caps


def i10(_before, after, _act):
    """No visible text is a raw launcher entry, says "agent" or "HUD", or
    spells a key otherwise than the word table."""
    out = []
    caps = long_key_names()
    for output, items in sorted(screen_texts(after).items()):
        for _i, label, text, _rect in items:
            out += [
                f"on {output} {label} shows {what}: {text!r}" for rx, what in RAW if rx.search(text)
            ]
            if found := WORDS.search(text):
                out.append(f"on {output} {label} says {found.group(0)!r}, a word of the builders: {text!r}")
            if text.strip() in caps:
                out.append(
                    f"on {output} {label} spells the key {text!r}, the word table says "
                    f"{caps[text.strip()]!r}"
                )
    return out


def bars_over_fullscreen(after, look, window, state):
    """The bars and panels over a fullscreen window: a layer surface above
    the scene or a node of a look, above the window and over its box. A
    window attached to it, such as a dialog, and the open entries of the
    look for the window are no bars."""
    looks = set(after.states)
    own = {o.get("id") for o in state.get("open") or [] if o.get("by") == window.get("surface")}
    out = []
    for output, nodes in sorted(node_boxes(after).items()):
        i, box = nodes.get(window.get("surface"), (None, None))
        if box is None:
            continue
        for b in boxes_on(after)[output][i + 1 :]:
            if b.get("kind") == "surface" or b.get("node") in own:
                continue
            bar = b.get("layer") in ("top", "shell_overlay") or b.get("owner") in looks
            if bar and overlap(box["rect"], b["rect"]):
                name = b.get("node") or b.get("namespace")
                by = b.get("owner") or b.get("layer")
                out.append(f"{name} of {by} lies over the fullscreen {window['surface']} of {look}")
    return out


def modals_below(after, look, state):
    """The open modals of a look that lie below the window they are for."""
    out = []
    for entry in state.get("open") or []:
        parent = entry.get("by")
        if not entry.get("modal") or not parent:
            continue
        for output, nodes in sorted(node_boxes(after).items()):
            if entry.get("id") in nodes and parent in nodes:
                if nodes[entry["id"]][0] < nodes[parent][0]:
                    out.append(f"the modal {entry['id']} of {look} lies below {parent} on {output}")
    return out


BELOW_HELD = {None, "background", "normal"}


def covers_held(after, look, window):
    """The boxes of the scene over the held window of a look: a window, or
    a node of another client in an order below `held` (D89). Its own
    frame and the bars of the overlay order are no cover."""
    out = []
    for output, nodes in sorted(node_boxes(after).items()):
        i, box = nodes.get(window.get("surface"), (None, None))
        if box is None:
            continue
        for b in boxes_on(after)[output][i + 1 :]:
            if b.get("layer") != "scene" or b.get("order") not in BELOW_HELD:
                continue
            if b.get("owner") == look and b.get("kind") != "surface":
                continue
            if overlap(box["rect"], b["rect"]):
                out.append(
                    f"{b.get('node')} of {b.get('owner')} lies over the held "
                    f"{window['surface']} of {look} on {output}"
                )
    return out


def i12(_before, after, _act):
    """No bar or panel box over a fullscreen window; the held window on
    top; a modal above its parent."""
    out = []
    for look, state in sorted(looks_with_state(after).items()):
        for w in state.get("windows") or []:
            if w.get("fullscreen"):
                out += bars_over_fullscreen(after, look, w, state)
            if w.get("held"):
                out += covers_held(after, look, w)
        out += modals_below(after, look, state)
    return out


def edges(corners):
    """The four edges of a quad as pairs of corners."""
    return [(corners[k], corners[(k + 1) % 4]) for k in range(4)]


def near(a, b):
    """Whether two points lie within SEAM px."""
    return abs(a[0] - b[0]) <= SEAM and abs(a[1] - b[1]) <= SEAM


def shares_edge(a, b):
    """Whether two quads share an edge: both corners of an edge of one lie
    on both corners of an edge of the other, in either direction."""
    return any(
        (near(p, r) and near(q, s)) or (near(p, s) and near(q, r))
        for p, q in edges(a)
        for r, s in edges(b)
    )


def ring_breaches(output, owner, faces, count, held):
    """The breaches of the faces of one cube on one output: a face meets
    the next one; held in a turn, a neighbour that has a box has corners
    and a face that shows does not show alone. A closed cube hides the faces
    that face away, so a neighbour without a box is no breach (co-21):
    halfway through a turn two faces show and the other two face away."""
    out = []
    for k in sorted(k for k, face in faces.items() if face.get("corners")):
        face = faces[k]
        ring = sorted({(k + 1) % count, (k - 1) % count} - {k})
        for n in ring if held else []:
            other = faces.get(n)
            if other is not None and not other.get("corners"):
                out.append(
                    f"on {output} the face {other['node']} has no corners beside {face['node']}"
                )
        if held and not any(faces.get(n, {}).get("corners") for n in ring):
            names = " nor ".join(f"{owner}-cube-face-{n}" for n in ring)
            out.append(f"on {output} the face {face['node']} shows alone in a turn: neither {names}")
        other = faces.get((k + 1) % count)
        if other and other.get("corners") and not shares_edge(face["corners"], other["corners"]):
            out.append(
                f"on {output} the faces {face['node']} and {other['node']} do not meet: "
                f"{[[round(v, 1) for v in c] for c in face['corners']]} and "
                f"{[[round(v, 1) for v in c] for c in other['corners']]}"
            )
    return out


def i13(_before, after, act):
    """Neighbouring cube faces that show share an edge within 1 px; in an
    act that holds the cube in a turn, a face next to one that shows has
    a box with corners too."""
    out = []
    for output, boxes in sorted(boxes_on(after).items()):
        cubes = {}
        for b in boxes:
            found = FACE.search(b.get("node") or "")
            if found:
                cubes.setdefault(b.get("owner"), {})[int(found.group(1))] = b
        for owner, faces in sorted(cubes.items(), key=lambda c: str(c[0])):
            out += ring_breaches(output, owner, faces, max(4, 1 + max(faces)), act.held)
    return out


ALL = {
    "I1": i1,
    "I2": i2,
    "I3": i3,
    "I4": i4,
    "I5": i5,
    "I6": i6,
    "I7": i7,
    "I8": i8,
    "I9": i9,
    "I10": i10,
    "I12": i12,
    "I13": i13,
    "I14": i14,
}
