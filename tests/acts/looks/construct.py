"""The acts of construct. The door acts come with the adapters (NET2)."""

from common import window_acts


def run(s):
    """A window opens as a pane and closes. The keys stay with the walk
    until a flight into the pane, so the act does not ask for the focus."""
    window_acts(s, focused=None)
