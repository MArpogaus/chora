"""The acts of construct. The door acts: the key list (tests/doors keys-key, keys-alias)."""

from common import door_acts, window_acts


def run(s):
    """A window opens as a pane and closes. The keys stay with the walk
    until a flight into the pane, so the act does not ask for the focus."""
    window_acts(s, focused=None)
    door_acts(s, "construct-help", "Escape")
