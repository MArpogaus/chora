"""The acts of tiling. The door acts: the key list (tests/doors keys-key, keys-alias)."""

from common import door_acts, helper_acts, window_acts


def run(s):
    """A window opens and closes; the key list opens and shuts."""
    window_acts(s)
    door_acts(s, "tiling-keys", "Escape")
    helper_acts(s)
