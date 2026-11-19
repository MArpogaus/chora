"""The acts of construct. The door acts: the key list (tests/doors keys-key, keys-alias).

The cube acts (arch plan 1.2, WM7a) run the hosted form: the construct
hosts the cube, one xp manages its four faces, and the acts read the cube
through the role `cube.list` (D71), never through construct's own state.
"""

import os
import signal
import time

from common import door_acts, window_acts
from harness import LOOKS, Expect, key, program

MANAGER = "xp"
FACE_PREFIXES = ("construct-cube", "construct-look")


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


def step_back(s):
    """A cue: stand back from the front face."""
    s.call("construct", "view.step_back")


def construct_shown(after):
    """The visible nodes of the construct outside the cube and its look:
    at 1:1 the front face alone shows."""
    return [
        f"the construct shows {n['id']} at 1:1"
        for n in after.nodes
        if n.get("owner") == "construct"
        and n.get("visible") is not False
        and (n.get("opacity") is None or n.get("opacity") > 0.0)
        and n.get("kind") in ("text", "image", "rect", "surface", "shader")
        and not str(n.get("id")).startswith(FACE_PREFIXES)
    ]


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


def kill_construct_mid_turn(s):
    """A cue: a slow turn, construct killed in its middle, then started
    again as the flavor's unit would; the xp of the cube ends with it."""
    pid = pid_of(s, "chora-construct")
    s.call("construct", "cube.turn", {"by": 1, "ms": 3000})
    time.sleep(1.0)
    os.kill(pid, signal.SIGKILL)
    s.until(lambda: not {"construct", MANAGER} & s.clients(), 20, "construct and xp gone")
    s.looks.remove(MANAGER)
    command, extra = LOOKS["construct"]
    s.spawn([str(s.bin / command[0]), *command[1:]], "construct-again.log", **extra)
    s.until(lambda: s.state("construct") is not None, 30, "construct again")


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
        "a drag on a face turns the cube",
        [step_back, drag(780, 80)],
        expect=Expect(cube={"front": 1, "turning": False, "mode": "navigate"}),
    )
    s.act(
        "back at 1:1 no construct surface is left",
        [key("Return")],
        expect=Expect(cube={"front": 1, "mode": "use"}, verify=construct_shown),
    )
    s.act(
        "construct killed mid-turn loses no window",
        [lambda s: s.call("construct", "cube.turn", {"to": 0}), kill_construct_mid_turn],
        disconnects=["construct", MANAGER],
        expect=Expect(gone=0),
    )
