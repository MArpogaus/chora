"""The acts of mc. The door acts: the key list (tests/doors keys-key, keys-alias)."""

from common import (
    closed,
    door_acts,
    helper_acts,
    keysheet_acts,
    notice_acts,
    shown_before,
    shown_id,
    window_acts,
)
from harness import REPO

PUSHED = "Pushed through mc"


def run(s):
    """A window opens and closes; the key list opens and shuts by
    Super+F1. mc reports no `open` yet, so its ids are the ones `kit::wm::open`
    (WM3) gives."""
    window_acts(s)
    door_acts(s, "mc-keys", "Super+F1")
    keysheet_acts(s, "mc-keys", "Super+F1")
    notice_acts(s, REPO / "plugins/chora-mc/notify")
    pushed = {}
    s.act("notify.push of mc shows in chora-notify", [lambda s: push(s, pushed)])
    s.act("notify.clear of mc dismisses it", [lambda s: clear(s, pushed)])
    helper_acts(s)


def push(s, pushed):
    """Call `notify.push` of mc, which other plugins call, and wait for the
    notice in `shown` of chora-notify."""
    call = {
        "client": s.label,
        "method": "notify.push",
        "params": {"title": PUSHED, "body": "the text"},
    }
    s.core.call("methods.call", call)
    pushed["id"] = shown_id(s, PUSHED)


def clear(s, pushed):
    """Call `notify.clear` of mc and wait for the id in `closed` of
    chora-notify."""
    notice = shown_before(pushed)
    s.core.call("methods.call", {"client": s.label, "method": "notify.clear", "params": {}})
    closed(s, notice)
