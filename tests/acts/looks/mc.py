"""The acts of mc. The door acts: the key list (tests/doors keys-key, keys-alias)."""

from common import door_acts, notice_acts, window_acts
from harness import REPO

PUSHED = "Pushed through mc"


def run(s):
    """A window opens and closes; the key list opens and shuts by
    Super+F1. mc reports no `open` yet, so its ids are the ones `kit::wm::open`
    (WM3) gives."""
    window_acts(s)
    door_acts(s, "mc-keys", "Super+F1")
    notice_acts(s, REPO / "plugins/chora-mc/notify")
    s.act("notify.push of mc shows in chora-notify", [push])
    s.act("notify.clear of mc dismisses it", [clear])


def push(s):
    """Call `notify.push` of mc, which other plugins call."""
    call = {
        "client": s.label,
        "method": "notify.push",
        "params": {"title": PUSHED, "body": "the text"},
    }
    s.core.call("methods.call", call)


def clear(s):
    """Call `notify.clear` of mc."""
    s.core.call("methods.call", {"client": s.label, "method": "notify.clear", "params": {}})
