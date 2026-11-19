"""The acts of construct. The door acts: the key list (tests/doors keys-key, keys-alias).

The cube acts (arch plan 1.2, WM7a) run the hosted form: the construct
hosts the cube, one xp manages its four faces, and the acts read the cube
through the role `cube.list` (D71), never through construct's own state.
"""

import os
import signal
import time

from common import door_acts, window_acts
from harness import LOOKS, Expect, key, load_png_rows, program, wait

MANAGER = "xp"
HINT_GONE = 4.0


def run(s):
    """A window opens as a pane and closes. The keys stay with the walk
    until a flight into the pane, so the act does not ask for the focus."""
    window_acts(s, focused=None)
    door_acts(s, "construct-help", "Escape")
    cube_acts(s)


def open_cube(s):
    """A cue: cube.open, then wait until the xp of the faces answers."""
    s.call("construct", "cube.open")
    s.looks.append(MANAGER)
    s.until(lambda: s.state(MANAGER) is not None, 30, "the xp of the cube")
    s.until(lambda: all(f["health"] == "up" for f in s.cube().get("faces", [])), 30, "faces up")


def pager(desk):
    """A cue: a click on the button of `desk` in the pager of the xp
    taskbar of the front face."""

    def cue(s):
        listing = s.cube()
        if "error" in listing:
            raise RuntimeError(listing["error"])
        front = listing["faces"][listing["front"]]["output"]
        button = f"xp-pager-{desk}-{front}"
        nodes = s.core.call("scene.query", {"selector": f"id:{button}"}) or []
        shown = [n for n in nodes if n.get("visible", True)]
        if not shown:
            raise RuntimeError(f"no pager button {button}: xp shows no pager on a face")
        x, y, w, h = shown[0]["rect"]
        s.inject([{"pointer_move": {"position": [x + w / 2, y + h / 2]}}, {"click": {}}])

    return cue


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
    pixel, so no construct node is left over the face."""

    def verify(after):
        listing = after.cube or {}
        if "faces" not in listing:
            return [f"the cube role: {listing.get('error')}"]
        face = listing["faces"][listing["front"]]["output"]
        screen = s.outputs()[0]["name"]
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
            if max(abs(ra[i + k] - rb[i + k]) for k in range(3)) > 8
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
    """A cue: a slow turn, construct killed in its middle, then started
    again as the flavor's unit would; the xp of the cube ends with it, and
    its windows come back to a look after the reset of the core."""
    pid = pid_of(s, "chora-construct")
    taken = s.call("construct", "cube.turn", {"by": 1, "ms": 3000})
    if taken is not True:
        raise RuntimeError(f"the slow turn was not taken: {taken!r}")
    time.sleep(1.0)
    if s.cube().get("turning") is not True:
        raise RuntimeError(f"the cube is not turning before the kill: {s.cube()!r}")
    os.kill(pid, signal.SIGKILL)
    s.until(lambda: not {"construct", MANAGER} & s.clients(), 20, "construct and xp gone")
    s.looks.remove(MANAGER)
    command, extra = LOOKS["construct"]
    s.spawn([str(s.bin / command[0]), *command[1:]], "construct-again.log", **extra)
    s.until(lambda: s.state("construct") is not None, 30, "construct again")
    s.until(lambda: all_with_a_look(s), 30, "every window back with a look")


def all_with_a_look(s):
    """Whether a look of the session owns every toplevel."""
    surfaces = s.core.call("scene.query", {"selector": "kind:surface"}) or []
    return all(n.get("owner") in s.looks for n in surfaces)


def cube_acts(s):
    """The cube turns by key, pager and drag, comes back to 1:1 with no
    construct surface left, and a construct killed mid-turn loses no
    window."""
    s.wallpaper = {o["name"] for o in s.outputs()}
    s.act(
        "the cube opens with one xp on four faces",
        [open_cube],
        expect=Expect(cube={"faces": 4, "front": 0, "turning": False, "mode": "use"}),
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
        "the key of the step back stands back",
        [lambda s: s.call("construct", "cube.turn", {"to": 0}), wait(0.8), key("Alt+Escape")],
        expect=Expect(cube={"front": 0, "turning": False, "mode": "navigate"}),
    )
    s.act(
        "a drag on a face turns the cube",
        [drag(780, 80)],
        expect=Expect(cube={"front": 1, "turning": False, "mode": "navigate"}),
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
    s.act(
        "construct killed mid-turn loses no window",
        [kill_construct_mid_turn],
        disconnects=["construct", MANAGER],
        expect=Expect(gone=0),
    )
