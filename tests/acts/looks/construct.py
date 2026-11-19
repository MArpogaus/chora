"""The acts of construct. The door acts: the key list (tests/doors keys-key, keys-alias).
The cube acts: the cube opens with xp on four faces and stands back, so
I13 reads the corners of its faces at rest (co-21)."""

from common import door_acts, window_acts

FACES = 4


def method(name, params=None):
    """A cue: call a method of the construct."""
    return lambda s: s.core.call(
        "methods.call", {"client": "construct", "method": name, "params": params or {}}
    )


def faces_up(s):
    """A cue: wait until the four virtual outputs of the faces are up and
    xp, their window manager, shows a desktop on each."""
    s.until(
        lambda: len([o for o in s.outputs() if o.get("virtual")]) == FACES
        and len((s.state("xp") or {}).get("desks") or []) == FACES,
        80,
        "four faces with xp",
    )


def run(s):
    """A window opens as a pane and closes. The keys stay with the walk
    until a flight into the pane, so the act does not ask for the focus.
    Then the cube opens and stands back."""
    window_acts(s, focused=None)
    door_acts(s, "construct-help", "Escape")
    s.act("the cube opens", [method("cube.open"), faces_up], timeout=60)
    s.act("the cube stands back", [method("view.step_back")], timeout=30)
