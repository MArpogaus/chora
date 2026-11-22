"""The acts of construct. The door acts: the key list (tests/doors keys-key, keys-alias).

The cube acts (arch plan 1.2, WM7a) run the hosted form: the construct
hosts the cube, one xp manages its four faces, and the acts read the cube
through the role `cube.list` (D71), never through construct's own state.
The cube opens once; after the pull act it stands back and I13 reads the
corners of its faces held halfway through a turn and at rest (co-21).
A construct killed mid-turn leaves the windows with the xp of the faces,
which goes flat on the screen within 3 s; a construct started again
reopens the cube around that xp, and its windows go back to the faces.
Without construct the pager of xp switches its desks itself.
"""

import os
import signal
import subprocess
import time

from common import door_acts, helper_acts, keysheet_acts, notice_acts, window_acts
from harness import LOOKS, Expect, key, load_png_rows, program, wait
from invariants import boxes_on

MANAGER = "xp"
HUD_HOST = "assistant"
HINT_GONE = 4.0
FLAT_WITHIN = 3.0
GRACE = 15.0

LIST = "construct-list"
FACES = 4
TURN_MS = 4000
HELD = 2.0


def method(name, params=None):
    """A cue: call a method of the construct."""
    return lambda s: s.core.call(
        "methods.call", {"client": "construct", "method": name, "params": params or {}}
    )


def faces_up(s):
    """A cue: wait until the four virtual outputs of the faces are up and
    xp, their window manager, shows a desktop on each."""
    s.until(
        lambda: (
            len([o for o in s.outputs() if o.get("virtual")]) == FACES
            and len((s.state("xp") or {}).get("desks") or []) == FACES
        ),
        80,
        "four faces with xp",
    )


def press_the_pane(s):
    """Press on the pane of the newest window of chora-probe and drag a
    little there and back, as the person does to point at a pane."""
    surfaces = s.core.call("scene.query", {"selector": "kind:surface"}) or []
    probe = [n["id"] for n in surfaces if (n.get("surface") or {}).get("app_id") == "chora-probe"]
    node = f"construct-pane-{probe[-1]}"
    s.until(lambda: box(s, node), 20, f"a box of {node}")
    x, y, w, h = box(s, node)
    px, py = x + w / 2, y + h / 2
    s.inject(
        [
            {"pointer_move": {"position": [px, py]}},
            {"button": {"state": "pressed"}},
            {"pointer_move": {"position": [px + 20, py]}},
            {"pointer_move": {"position": [px, py]}},
            {"button": {"state": "released"}},
        ]
    )


def box(s, node):
    """The rect of the first box of `node`, or None."""
    for entry in (s.core.call("scene.boxes") or {}).values():
        for b in entry.get("boxes") or []:
            if b.get("node") == node:
                return b["rect"]
    return None


def tab_after_a_new_window(s):
    """kit7-a2 (rw-5): after a press on a pane, Tab right after a new
    window opens the window list, without a flight into a pane first."""
    want = {"app_id": "chora-probe", "owner": s.label}
    window = [program("chora-probe", "window", "40", "120", "200")]
    s.act("a window opens to point at", window, expect=Expect(new_window=want))
    s.act("a press on its pane", [press_the_pane])
    s.act("a window opens for the list", window, expect=Expect(new_window=want))
    s.act(
        "Tab right after a new window opens the window list",
        [key("Tab")],
        opens=[LIST],
        expect=Expect(open=[LIST]),
    )
    s.act(
        "Escape shuts the window list",
        [key("Escape")],
        closes=[LIST],
        expect=Expect(open=[]),
    )
    s.act(
        "the windows of the list close",
        [lambda s: s.procs.pop().terminate(), lambda s: s.procs.pop().terminate()],
        expect=Expect(gone=2),
    )


def run(s):
    """A window opens as a pane and closes. The keys stay with the walk
    until a flight into the pane, so the act does not ask for the focus.
    Then the cube opens and stands back."""
    window_acts(s, focused=None)
    door_acts(s, "construct-help", "Escape")
    keysheet_acts(s, "construct-help", "Escape")
    tab_after_a_new_window(s)
    helper_acts(s)
    cube_acts(s)


def open_cube(s):
    """A cue: cube.open, then wait until the xp of the faces answers."""
    s.call("construct", "cube.open")
    s.looks.append(MANAGER)
    s.until(lambda: s.state(MANAGER) is not None, 30, "the xp of the cube")
    s.until(lambda: all(f["health"] == "up" for f in s.cube().get("faces", [])), 30, "faces up")
    faces_up(s)


def pager(desk):
    """A cue: a click on the button of `desk` in the pager of the xp
    taskbar of the front face."""

    def cue(s):
        listing = s.cube()
        if "error" in listing:
            raise RuntimeError(listing["error"])
        front = listing["faces"][listing["front"]]["output"]
        click_box(s, front, f"xp-pager-{desk}-{front}", "xp shows no pager on a face")

    return cue


def click_box(s, output, node, missing):
    """Click the middle of the box of `node` on `output`, which shows at
    1:1 on the screen."""
    boxes = ((s.core.call("scene.boxes") or {}).get(output) or {}).get("boxes") or []
    found = [b for b in boxes if b.get("node") == node]
    if not found:
        raise RuntimeError(f"no box of {node}: {missing}")
    x, y, w, h = found[-1]["rect"]
    s.inject([{"pointer_move": {"position": [x + w / 2, y + h / 2]}}, {"click": {}}])


def drag(x0, x1, y=400):
    """A cue: a drag of the left button from x0 to x1."""
    return lambda s: s.inject(
        [
            {"pointer_move": {"position": [x0, y]}},
            {"button": {"state": "pressed", "hold": True}},
            {"pointer_move": {"position": [(x0 + x1) / 2, y]}},
            {"pointer_move": {"position": [x1, y]}},
            {"button": {"state": "released"}},
        ]
    )


def modifier_drag(x0, x1, y=400):
    """A cue: a drag of the left button from x0 to x1 with Super held."""
    return lambda s: s.inject(
        [
            {"pointer_move": {"position": [x0, y]}},
            {"key": {"key": "Super_L", "state": "pressed"}},
            {"button": {"state": "pressed", "hold": True}},
            {"pointer_move": {"position": [(x0 + x1) / 2, y]}},
            {"pointer_move": {"position": [x1, y]}},
            {"button": {"state": "released"}},
            {"key": {"key": "Super_L", "state": "released"}},
        ]
    )


def plain(name):
    """A cue: a key pressed and released without a modifier."""
    return lambda s: s.inject(
        [
            {"key": {"key": name, "state": "pressed"}},
            {"key": {"key": name, "state": "released"}},
        ]
    )


def one_to_one(s):
    """A check: the screen shows the output of the front face pixel for
    pixel, so no construct node is left over the face. The boxes of the HUD
    host do not count: its voice island stays at the top right in a face
    (docs/construct-style.md, rule 22), nor does the resting chip Keys of a
    face (co-42)."""

    def verify(after):
        listing = after.cube or {}
        if "faces" not in listing:
            return [f"the cube role: {listing.get('error')}"]
        face = listing["faces"][listing["front"]]["output"]
        screen = s.outputs()[0]["name"]
        hud = [
            b["rect"]
            for b in boxes_on(after).get(screen, [])
            if b.get("owner") == HUD_HOST or str(b.get("node", "")).startswith("construct-hints")
        ]

        def free(x, y):
            return not any(r[0] <= x < r[0] + r[2] and r[1] <= y < r[1] + r[3] for r in hud)

        rows = load_png_rows()
        a = list(rows(s.capture(screen, 1280)[1]))
        b = list(rows(s.capture(face, 1280)[1]))
        if len(a) != len(b):
            return [f"{screen} has {len(a)} rows and {face} {len(b)}"]
        total = sum(len(r) // 4 for r in a)
        differ = [
            (i // 4, y)
            for y, (ra, rb) in enumerate(zip(a, b, strict=True))
            for i in range(0, min(len(ra), len(rb)), 4)
            if max(abs(ra[i + k] - rb[i + k]) for k in range(3)) > 8 and free(i // 4, y)
        ]
        if len(differ) > total // 1000:
            xs, ys = [x for x, _ in differ], [y for _, y in differ]
            box = [min(xs), min(ys), max(xs), max(ys)]
            return [
                f"{screen} differs from the face {face} in {len(differ)} of {total} "
                f"pixels, within {box}"
            ]
        return []

    return verify


def pid_of(s, name):
    """The process of the binary `name` in the session of `s`."""
    mark = f"XDG_RUNTIME_DIR={s.runtime}".encode()
    for pid in sorted(filter(str.isdigit, os.listdir("/proc")), key=int):
        try:
            argv = open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")
            env = open(f"/proc/{pid}/environ", "rb").read().split(b"\0")
        except OSError:
            continue
        if argv[0].endswith(name.encode()) and mark in env:
            return int(pid)
    raise RuntimeError(f"no process {name} in the session")


def probe_surface(s):
    """The surface of the window of chora-probe."""
    nodes = s.core.call("scene.query", {"selector": "kind:surface"}) or []
    found = [n["id"] for n in nodes if (n.get("surface") or {}).get("app_id") == "chora-probe"]
    if len(found) != 1:
        raise RuntimeError(f"expected one chora-probe window, got {found}")
    return found[0]


def pull_and_remove(s):
    """A cue: cube.pull makes the probe window a pane and answers its name;
    a second pull of the same window is refused, as it is off a face; the
    window goes back to the xp of the face once the pane is removed."""
    sid = probe_surface(s)
    try:
        pane = s.call("construct", "cube.pull", {"surface": sid}).get("pane")
    except RuntimeError as err:
        listed = s.call("construct", "window.list")
        raise RuntimeError(f"{err}; window.list {listed}; cube.list {s.cube()}") from err
    listed = s.call("construct", "construct.list").get("panes") or []
    if pane not in [p.get("name") for p in listed]:
        raise RuntimeError(f"the pane {pane!r} of the pull is not in construct.list")
    try:
        s.call("construct", "cube.pull", {"surface": sid})
    except RuntimeError:
        pass
    else:
        raise RuntimeError("a second pull of a window in a pane was not refused")
    s.call("construct", "pane.remove", {"name": pane})
    s.until(lambda: owner_of(s, sid) == MANAGER, 20, "the window back with the xp of the face")


def owner_of(s, sid):
    """The client that owns the node of the surface `sid`."""
    nodes = s.core.call("scene.query", {"selector": f"id:{sid}"}) or []
    return nodes[0].get("owner") if nodes else None


def kill_construct_mid_turn(s):
    """A cue: a slow turn, construct killed in its middle and not started
    again; within 3 s the xp of the faces holds every window on the
    screen, flat, and no window is without an owner."""
    pid = pid_of(s, "chora-construct")
    taken = s.call("construct", "cube.turn", {"by": 1, "ms": 3000})
    if taken is not True:
        raise RuntimeError(f"the slow turn was not taken: {taken!r}")
    time.sleep(1.0)
    if s.cube().get("turning") is not True:
        raise RuntimeError(f"the cube is not turning before the kill: {s.cube()!r}")
    os.kill(pid, signal.SIGKILL)
    s.looks.remove("construct")
    s.until(lambda: "construct" not in s.clients(), 5, "construct gone")
    s.until(lambda: flat_xp(s), FLAT_WITHIN, "every window with a flat xp on the screen")


def screens(s):
    """The names of the outputs that are not virtual."""
    return {o["name"] for o in s.outputs() if not o.get("virtual")}


def flat_xp(s):
    """Whether xp runs on the screen alone and owns every toplevel there."""
    state = s.state(MANAGER) or {}
    desks = {d.get("output") for d in state.get("desks") or []}
    surfaces = s.core.call("scene.query", {"selector": "kind:surface"}) or []
    windows = {w["surface"]: w for w in state.get("windows") or []}
    return (
        desks == screens(s)
        and bool(surfaces)
        and all(n.get("owner") == MANAGER for n in surfaces)
        and all(windows.get(n["id"], {}).get("output") in desks for n in surfaces)
    )


def restart_construct(s):
    """A cue: construct started again, as the flavor's unit would; it
    reopens the cube around the running xp, whose windows go back to the
    faces, and its start card does not say that nothing is open."""
    command, extra = LOOKS["construct"]
    s.spawn([str(s.bin / command[0]), *command[1:]], "construct-again.log", **extra)
    s.until(lambda: s.state("construct") is not None, 30, "construct again")
    s.looks.append("construct")
    s.until(lambda: len(s.cube().get("faces") or []) == FACES, 30, "the cube reopened")
    s.until(lambda: windows_on_faces(s), 30, "every window back on a face of xp")
    start = s.core.call("scene.query", {"selector": "id:construct-start"}) or []
    panes = s.call("construct", "construct.list").get("panes") or []
    if start and start[0].get("visible", True) and panes:
        raise RuntimeError(f"the start card shows beside the panes {panes}")


def windows_on_faces(s):
    """Whether xp owns every toplevel and each lies on a face."""
    faces = {f["output"] for f in s.cube().get("faces") or []}
    state = s.state(MANAGER) or {}
    windows = {w["surface"]: w for w in state.get("windows") or []}
    surfaces = s.core.call("scene.query", {"selector": "kind:surface"}) or []
    return bool(surfaces) and all(
        n.get("owner") == MANAGER and windows.get(n["id"], {}).get("output") in faces
        for n in surfaces
    )


def stop_construct(s):
    """A cue: construct ends for good; xp goes flat on the screen. The
    core keeps the nodes of construct for its grace time of 10 s, in
    which they still cover the screen; the cue waits for their end."""
    os.kill(pid_of(s, "chora-construct"), signal.SIGKILL)
    s.looks.remove("construct")
    s.until(lambda: "construct" not in s.clients(), 5, "construct gone")
    s.until(lambda: flat_xp(s), FLAT_WITHIN, "every window with a flat xp on the screen")
    kept = {"selector": "owner:construct"}
    s.until(lambda: not s.core.call("scene.query", kept), GRACE, "the kept nodes of construct")


def screen_pager(desk):
    """A cue: a click on the button of `desk` in the pager of the xp
    taskbar of the screen."""

    def cue(s):
        screen = sorted(screens(s))[0]
        click_box(s, screen, f"xp-pager-{desk}-{screen}", "no pager on the screen")

    return cue


def shown_desk(desk):
    """A check: xp shows the desk `desk` (counted from 1) on the screen."""

    def verify(after):
        state = after.states.get(MANAGER) or {}
        shown = [d.get("shown") for d in state.get("desks") or []]
        return [] if shown and all(d == [desk] for d in shown) else [f"xp shows the desks {shown}"]

    return verify


def stood_back_whole(s):
    """A check for co-18 and co-16: after a drag on the cube stood back the
    front face lies whole on the screen, the three chips still show the
    caps of their keys, and every window lies on a face with the xp of it."""
    chips = ["turn-left", "step-in", "turn-right"]

    def verify(after):
        out = []
        screen = s.outputs()[0]
        width, height = screen["size"]
        front = (after.cube or {}).get("front")
        rect = box(s, f"construct-cube-face-{front}")
        if rect is None:
            out.append(f"no box of the front face {front}")
        else:
            x, y, w, h = rect
            if x < 0 or y < 0 or x + w > width or y + h > height:
                out.append(f"the front face {front} leaves the screen: {rect}")
        shown = {
            n["id"]
            for n in s.core.call("scene.query", {"selector": "owner:construct"}) or []
            if n.get("visible", True) and "-keys-" in n["id"]
        }
        for chip in chips:
            cap = f"construct-hints-construct-mode-{chip}-keys-0"
            if cap not in shown:
                out.append(f"the chip {chip} has no key cap")
        if not windows_on_faces(s):
            out.append("a window is off the faces of xp")
        return out

    return verify


def one_holder(after):
    """`Expect.verify` for co-38: exactly one client says it holds the focus."""
    claims = sorted(
        look
        for look, st in after.states.items()
        if (st or {}).get("focus", {}).get("holder") == look
    )
    return (
        []
        if len(claims) == 1
        else [f"{' and '.join(claims) or 'no client'} say they hold the focus"]
    )


def pull_keep(s):
    """A cue: cube.pull makes the probe window a pane of the space, which
    stays; its name is kept for `pane_remove`."""
    s.pane = s.call("construct", "cube.pull", {"surface": probe_surface(s)}).get("pane")


def grab_any(s):
    """A cue: the grant that lets construct grab a key without Ctrl, Alt or
    Super, F1 among them; a step back and in makes it grab again."""
    subprocess.run(
        [str(s.bin / "choractl"), "grant", "construct", "input.grab-any"],
        env=s.env,
        check=True,
        capture_output=True,
    )
    s.call("construct", "view.step_back")
    s.call("construct", "view.step_in")


def activate_probe(s):
    """A cue: xp gives the keyboard to the window of chora-probe, as a
    click on it does; the window is the focus of the core."""
    sid = probe_surface(s)
    s.call(MANAGER, "activate", {"surface": sid})
    s.until(
        lambda: (s.state(MANAGER) or {}).get("focus", {}).get("surface") == sid,
        10,
        "xp focuses the window",
    )


def pane_remove(s):
    """A cue: the pane of `pull_keep` goes, its window back to the face."""
    s.call("construct", "pane.remove", {"name": s.pane})
    s.until(lambda: owner_of(s, probe_surface(s)) == MANAGER, 20, "the window back with xp")


def hint_rect(s, node):
    """The rect of the node `node` on the screen, or None."""
    for n in s.core.call("scene.query", {"selector": f"id:{node}"}) or []:
        for output, rect in (n.get("rects") or {}).items():
            if output == s.outputs()[0]["name"]:
                return rect
    return None


def stable_rect(s, node, timeout=10):
    """The rect of the hint `node` once two reads 0.25 s apart agree: the
    hints of a cube stood back are still drawn for a moment after a step in
    and move when the set of the hints changes."""
    seen = []

    def settled():
        seen.append(hint_rect(s, node))
        return len(seen) >= 2 and seen[-1] is not None and seen[-1] == seen[-2]

    s.until(settled, timeout, f"the hint {node} at rest on the screen")
    return seen[-1]


def cube_uses(s):
    """A cue: wait until the cube is stepped into a face (mode use)."""
    s.until(
        lambda: s.cube().get("mode") == "use" and not s.cube().get("turning"),
        10,
        "the cube steps into a face",
    )


def hint_click(node, dwell=0.0):
    """A cue: a click in the middle of the hint `node` on the screen; the
    pointer rests low first, which shows the hints of a face. `dwell` is
    the time between reading the rect and the click: a slow harness, in
    which the 3 s of the hints after a step in run out in between."""

    def cue(s):
        w, h = s.outputs()[0]["size"]
        s.inject([{"pointer_move": {"position": [w / 2, h - 10]}}])
        aim = stable_rect(s, node)
        time.sleep(dwell)
        for _ in range(3):
            x, y, bw, bh = aim
            s.inject([{"pointer_move": {"position": [x + bw / 2, y + bh / 2]}}])
            now = stable_rect(s, node)
            if now == aim:
                s.inject([{"click": {}}])
                return
            aim = now
        raise RuntimeError(f"the hint {node} moves away from the pointer")

    return cue


SHEET = "construct-help"
KEYS_HINT = "construct-hint-keys"


def press_hinted(node):
    """A cue: the key that the hint `node` names, pressed as a shortcut; its
    keycaps are read from the scene."""

    def cue(s):
        caps = []

        def read():
            caps.clear()
            while True:
                found = s.core.call(
                    "scene.query", {"selector": f"id:construct-hints-{node}-keys-{len(caps)}-text"}
                )
                text = (found or [{}])[0].get("text")
                if not text:
                    return bool(caps)
                caps.append(text)

        s.until(read, 5, f"the hint {node} names a key")
        s.inject([{"shortcut": {"keys": "+".join(caps)}}])

    return cue


def hint_shown(s):
    """A cue: the pointer rests low, which shows the hints of a face."""
    w, h = s.outputs()[0]["size"]
    s.inject([{"pointer_move": {"position": [w / 2, h - 10]}}])
    stable_rect(s, KEYS_HINT)


def rest_then_click(node):
    """A cue: wait past the 3 s of the hints of a face, then click the hint
    `node` where it stands, with no pointer rest low first (co-42)."""

    def cue(s):
        time.sleep(5.0)
        s.until(lambda: hint_rect(s, node), 5, f"the hint {node} after 5 s")
        x, y, bw, bh = hint_rect(s, node)
        s.inject([{"pointer_move": {"position": [x + bw / 2, y + bh / 2]}}, {"click": {}}])

    return cue


def keys_default_acts(s):
    """rck-1, rck-4: in a default session, with no grant, the hint Keys of a
    face and of the cube stood back names a key that opens the key list."""
    s.act(
        "Alt+Escape stands back for a face",
        [key("Alt+Escape")],
        expect=Expect(cube={"mode": "navigate"}),
    )
    s.act("Return steps into a face", [plain("Return")], expect=Expect(cube={"mode": "use"}))
    s.act("xp gives the window of a face the keyboard", [activate_probe])
    s.act(
        "the hint Keys of a face names a key that opens the key list",
        [
            key("Alt+Escape"),
            cube_navigates,
            plain("Return"),
            cube_uses,
            hint_shown,
            press_hinted("keys"),
        ],
        opens=[SHEET],
        expect=Expect(open=[SHEET]),
    )
    s.act("Escape shuts the key list of the hint", [plain("Escape")], closes=[SHEET])
    s.act(
        "co-42: the hint Keys of a face stays after 3 s and opens the key list by a click",
        [rest_then_click(KEYS_HINT)],
        opens=[SHEET],
        expect=Expect(open=[SHEET]),
    )
    s.act("Escape shuts the key list of the resting hint", [plain("Escape")], closes=[SHEET])
    s.act(
        "Alt+Escape stands back without a grant",
        [key("Alt+Escape")],
        expect=Expect(cube={"mode": "navigate"}),
    )
    s.act(
        "the hint Keys of the stood back cube names a key that opens the key list",
        [hint_shown, press_hinted("keys")],
        opens=[SHEET],
        expect=Expect(open=[SHEET]),
    )
    s.act("Escape shuts the key list of the cube hint", [plain("Escape")], closes=[SHEET])
    s.act("Return steps in", [plain("Return")], expect=Expect(cube={"mode": "use"}))


def cube_navigates(s):
    """A cue: wait until the cube stands back (mode navigate)."""
    s.until(
        lambda: s.cube().get("mode") == "navigate",
        10,
        "the cube stands back",
    )


def grab_revoke(s):
    """A cue: the grant of `grab_any` goes; a step back and in grabs anew."""
    subprocess.run(
        [str(s.bin / "choractl"), "revoke", "construct", "input.grab-any"],
        env=s.env,
        check=True,
        capture_output=True,
    )
    s.call("construct", "view.step_back")
    s.call("construct", "view.step_in")


def keys_acts(s):
    """co-38, co-40, co-41 (keys rc 2): after a pane was pulled and the cube
    stepped back one client holds the focus; Escape shuts the key list of
    the stood back cube and Return steps in; F1 opens the key list in a
    face and stood back, and the hint of its key opens it by a click."""
    keys_default_acts(s)
    s.act("construct may grab F1 without a modifier", [grab_any])
    s.act("xp gives the window the keyboard", [activate_probe])
    s.act(
        "a window is pulled to a pane of the space",
        [pull_keep],
        expect=Expect(cube={"mode": "use"}),
    )
    s.act(
        "Alt+Escape with a pane in the space leaves one client with the focus",
        [key("Alt+Escape")],
        expect=Expect(cube={"mode": "navigate"}, verify=one_holder),
    )
    s.act(
        "the pane goes back to its face while the cube stands back",
        [pane_remove],
        expect=Expect(cube={"mode": "navigate"}),
    )
    s.act("Super+F1 shows the key list of the stood back cube", [key("Super+F1")], opens=[SHEET])
    s.act(
        "Escape shuts the key list of the stood back cube",
        [plain("Escape")],
        closes=[SHEET],
        expect=Expect(open=[], cube={"mode": "navigate"}),
    )
    s.act(
        "Return steps in after the key list shut",
        [plain("Return")],
        expect=Expect(cube={"mode": "use"}),
    )
    s.act(
        "F1 opens the key list in a face", [plain("F1")], opens=[SHEET], expect=Expect(open=[SHEET])
    )
    s.act(
        "Escape shuts the key list of a face",
        [plain("Escape")],
        closes=[SHEET],
        expect=Expect(open=[]),
    )
    s.act(
        "the hint Keys of a face opens the key list by a click",
        [
            key("Alt+Escape"),
            cube_navigates,
            plain("Return"),
            cube_uses,
            hint_click(KEYS_HINT, dwell=3.5),
        ],
        opens=[SHEET],
        expect=Expect(open=[SHEET]),
    )
    s.act(
        "Escape shuts the key list opened by the hint",
        [plain("Escape")],
        closes=[SHEET],
        expect=Expect(open=[]),
    )
    s.act(
        "Alt+Escape stands back for the key hints",
        [key("Alt+Escape")],
        expect=Expect(cube={"mode": "navigate"}),
    )
    s.act(
        "F1 opens the key list of the stood back cube",
        [plain("F1")],
        opens=[SHEET],
        expect=Expect(open=[SHEET]),
    )
    s.act(
        "Escape shuts the key list of the cube again",
        [plain("Escape")],
        closes=[SHEET],
        expect=Expect(open=[]),
    )
    s.act(
        "the chip Keys of the stood back cube opens the key list by a click",
        [hint_click(KEYS_HINT)],
        opens=[SHEET],
        expect=Expect(open=[SHEET]),
    )
    s.act(
        "Escape shuts the key list opened by the chip",
        [plain("Escape")],
        closes=[SHEET],
        expect=Expect(open=[]),
    )
    s.act("Return steps in after the chip", [plain("Return")], expect=Expect(cube={"mode": "use"}))
    s.act(
        "the grant input.grab-any is revoked again",
        [grab_revoke],
        expect=Expect(cube={"mode": "use"}),
    )


def cube_acts(s):
    """The cube turns by key, pager and drag, comes back to 1:1 with no
    construct surface left, stands back and is read held in a turn and at
    rest, and a construct killed mid-turn loses no window."""
    s.wallpaper = {o["name"] for o in s.outputs()}
    s.act(
        "the cube opens with one xp on four faces",
        [open_cube],
        expect=Expect(cube={"faces": 4, "front": 0, "turning": False, "mode": "use"}),
        timeout=60,
    )
    s.act(
        "a window opens on the front face",
        [
            lambda s: s.inject([{"pointer_move": {"position": [640, 400]}}]),
            program("chora-probe", "window", "40", "120", "200"),
        ],
        expect=Expect(new_window={"app_id": "chora-probe", "owner": MANAGER}),
    )
    s.act(
        "the turn key turns the cube",
        [key("Ctrl+Alt+Right")],
        expect=Expect(cube={"front": 1, "turning": False, "mode": "use"}),
    )
    s.act(
        "a turn to a face turns the cube the short way",
        [lambda s: s.call("construct", "cube.turn", {"to": 0})],
        expect=Expect(cube={"front": 0, "turning": False}),
    )
    s.act(
        "the pager turns the cube",
        [pager(2)],
        expect=Expect(cube={"front": 2, "turning": False, "mode": "use"}),
    )
    s.act(
        "a drag with the modifier on the front face turns the cube",
        [lambda s: s.call("construct", "cube.turn", {"to": 0}), wait(0.8), modifier_drag(780, 80)],
        expect=Expect(cube={"front": 1, "turning": False, "mode": "use"}),
    )
    s.act(
        "the key of the step back stands back",
        [lambda s: s.call("construct", "cube.turn", {"to": 0}), wait(0.8), key("Alt+Escape")],
        expect=Expect(cube={"front": 0, "turning": False, "mode": "navigate"}),
    )
    s.act(
        "a drag on a face turns the cube",
        [drag(780, 80)],
        expect=Expect(
            cube={"front": 1, "turning": False, "mode": "navigate"},
            verify=stood_back_whole(s),
        ),
    )
    s.act(
        "a drag to the right on the stood back cube turns it back",
        [drag(500, 900)],
        expect=Expect(
            cube={"front": 0, "turning": False, "mode": "navigate"},
            verify=stood_back_whole(s),
        ),
    )
    s.act(
        "a second drag to the left turns it on again",
        [drag(780, 80)],
        expect=Expect(
            cube={"front": 1, "turning": False, "mode": "navigate"},
            verify=stood_back_whole(s),
        ),
    )
    s.act(
        "back at 1:1 no construct surface is left once the hint went",
        [plain("Return"), wait(HINT_GONE)],
        expect=Expect(cube={"front": 1, "mode": "use"}, verify=one_to_one(s)),
    )
    s.act(
        "a pulled window goes back to the xp of its face when its pane goes",
        [pull_and_remove],
        expect=Expect(gone=0, cube={"mode": "use"}),
    )
    keys_acts(s)
    s.act("the cube stands back", [method("view.step_back")], timeout=30)
    s.act(
        "the cube is held halfway through a turn",
        [method("cube.turn", {"by": 1, "ms": TURN_MS})],
        timeout=30,
        held=HELD,
    )
    s.act("the cube comes to rest", [], timeout=30)
    notice_acts(s)
    s.act(
        "construct killed mid-turn leaves the windows with a flat xp within 3 s",
        [kill_construct_mid_turn],
        disconnects=["construct"],
        expect=Expect(gone=0),
    )
    s.act(
        "construct started again reopens the cube with the windows on its faces",
        [restart_construct],
        expect=Expect(gone=0, cube={"faces": 4, "turning": False}),
        timeout=60,
    )
    s.act(
        "without construct the pager of xp switches the desk",
        [stop_construct, screen_pager(1)],
        disconnects=["construct"],
        expect=Expect(gone=0, verify=shown_desk(2)),
        timeout=30,
    )
