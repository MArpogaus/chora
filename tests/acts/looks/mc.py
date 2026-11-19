"""The acts of mc. The door acts: the key list (tests/doors keys-key, keys-alias)."""

from common import door_acts, keysheet_acts, window_acts


def run(s):
    """A window opens and closes; the key list opens and shuts by
    Super+F1. mc reports no `open` yet, so its ids are the ones `kit::wm::open`
    (WM3) gives."""
    window_acts(s)
    door_acts(s, "mc-keys", "Super+F1")
    keysheet_acts(s, "mc-keys", "Super+F1")
