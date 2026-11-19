"""The acts of construct. The door acts: the key list (tests/doors keys-key, keys-alias).
The cube acts: the cube opens with xp on four faces and stands back, so
I13 reads the corners of its faces at rest and held halfway through a turn
(co-21)."""

from common import door_acts, helper_acts, window_acts
from harness import Expect, key, program

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
    window opens the window list, without a flight into a pane first
    (known red under WM2)."""
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
    tab_after_a_new_window(s)
    helper_acts(s)
    s.act("the cube opens", [method("cube.open"), faces_up], timeout=60)
    s.act("the cube stands back", [method("view.step_back")], timeout=30)
    s.act(
        "the cube is held halfway through a turn",
        [method("cube.turn", {"by": 1, "ms": TURN_MS})],
        timeout=30,
        held=HELD,
    )
    s.act("the cube comes to rest", [], timeout=30)
