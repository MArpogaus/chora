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
from harness import REPO, Expect, key

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
    remote_acts(s)
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


def remote(name):
    """A cue: a key pressed and released as a remote does, with no
    modifier. input.inject takes it as a press of a person, since the
    session runs with --test-injected-press-is-real."""
    return lambda s: s.inject(
        [
            {"key": {"key": name, "state": "pressed"}},
            {"key": {"key": name, "state": "released"}},
        ]
    )


def page_is(s, scene):
    """A check: mc shows the page `scene` (`scene.list` of mc names it)."""

    def verify(_after):
        shown = s.call("mc", "scene.list").get("current")
        return [] if shown == scene else [f"mc shows the page {shown!r}, not {scene!r}"]

    return verify


def remote_acts(s):
    """mc-32: in dark mc the keys of a remote turn the page: Right and
    Left alone turn it, the binding Super+Home goes to the first page. A
    plain Home is no key of mc (mc.yaml: Right, Left and Escape alone)."""
    s.act("mc turns dark", [lambda s: s.call("mc", "settings.set", {"theme": "dark"})])
    s.act(
        "the page is the first one",
        [key("Super+Home")],
        expect=Expect(verify=page_is(s, "agenda")),
    )
    s.act(
        "Right of a remote turns the page in dark mc",
        [remote("Right")],
        expect=Expect(verify=page_is(s, "morning")),
    )
    s.act(
        "a second Right of a remote turns on",
        [remote("Right")],
        expect=Expect(verify=page_is(s, "agents")),
    )
    s.act(
        "Left of a remote turns back",
        [remote("Left")],
        expect=Expect(verify=page_is(s, "morning")),
    )
    s.act(
        "Super+Home of a remote goes to the first page",
        [key("Super+Home")],
        expect=Expect(verify=page_is(s, "agenda")),
    )
