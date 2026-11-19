"""Acts that every look runs."""

import pathlib

from harness import REPO, Expect, key, program


def ready(s):
    """Wait until every look answers `state.get`: a look connects before
    it exports its methods, and the first act must not read that gap."""
    s.until(lambda: all(s.state(look) is not None for look in s.looks), 30, "state.get")


def window_acts(s, focused=True):
    """A window of chora-probe opens, owned by the look and focused when
    `focused` (None: not checked), and goes when its program ends."""
    ready(s)
    want = {"app_id": "chora-probe", "owner": s.label}
    if focused is not None:
        want["focused"] = focused
    s.act(
        "a window opens",
        [program("chora-probe", "window", "40", "120", "200")],
        expect=Expect(new_window=want),
    )
    s.act("the window closes", [lambda s: s.procs.pop().terminate()], expect=Expect(gone=1))


def door_acts(s, sheet, close="Escape"):
    """The key list opens by Super+F1 and by Super+/ as the open entry
    `sheet` of the look and shuts by `close` (the door cells keys-key and
    keys-alias of tests/doors as acts)."""
    for keys, name in (("Super+F1", "keys"), ("Super+slash", "keys alias")):
        s.act(
            f"the {name} key opens the key list",
            [key(keys)],
            opens=[sheet],
            expect=Expect(open=[sheet]),
        )
        s.act(
            f"{close} shuts the key list of the {name} key",
            [key(close)],
            closes=[sheet],
            expect=Expect(open=[]),
        )


NOTICE = "Probe notice"


def notice_acts(s):
    """chora-notify runs beside the look, as its unit does. A notice that
    a plugin sends through `notify.show` shows in a card inside the
    output and clear of the helper (I9), and goes when dismissed."""
    ready(s)
    if "notify" not in s.clients():
        assets = REPO / "plugins/chora-notify/assets"
        s.spawn([str(s.bin / "chora-notify"), "--assets", str(assets)], "chora-notify.log")
        s.until(lambda: s.state("notify") is not None, 30, "chora-notify")
    sent = {}

    def show(s):
        call = {
            "client": "notify",
            "method": "notify.show",
            "params": {"app": s.label, "summary": NOTICE, "body": "the text", "expire_ms": 0},
        }
        sent.update(s.core.call("methods.call", call))

    def dismiss(s):
        call = {"client": "notify", "method": "notify.close", "params": {"id": sent["id"]}}
        s.core.call("methods.call", call)

    s.act("a notice shows", [show], notices=[NOTICE])
    s.act("a notice is dismissed", [dismiss], dismissed=[NOTICE])
