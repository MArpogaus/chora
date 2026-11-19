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


def notice_acts(s, slot=None):
    """chora-notify runs beside the look, as its unit does, with the folder
    `slot` of the look (assets and notify.yaml) when it has one. A notice
    that a plugin sends through `notify.show` shows in a card inside the
    output and clear of the helper (I9), and goes when dismissed."""
    ready(s)
    if "notify" not in s.clients():
        assets = slot or REPO / "plugins/chora-notify/assets"
        argv = [str(s.bin / "chora-notify"), "--assets", str(assets)]
        if slot:
            argv += ["--config", str(slot / "notify.yaml")]
        s.spawn(argv, "chora-notify.log")
        s.until(lambda: s.state("notify") is not None, 30, "chora-notify")
    sent = {}

    def show(s):
        call = {
            "client": "notify",
            "method": "notify.show",
            "params": {"app": s.label, "summary": NOTICE, "body": "the text", "expire_ms": 0},
        }
        sent.update(s.core.call("methods.call", call))
        sent["id"] = shown_id(s, NOTICE)

    def dismiss(s):
        notice = shown_before(sent)
        call = {"client": "notify", "method": "notify.close", "params": {"id": notice}}
        s.core.call("methods.call", call)
        closed(s, notice)

    s.act("a notice shows", [show])
    s.act("a notice is dismissed", [dismiss])


def shown_id(s, summary):
    """Wait until chora-notify shows a notice with the summary (`shown` of
    its `state.get`) and answer its id; RuntimeError after 10 s."""
    found = {}

    def check():
        notify = s.state("notify") or {}
        ids = [n["id"] for n in notify.get("shown", []) if n.get("summary") == summary]
        if ids:
            found["id"] = ids[0]
        return bool(ids)

    s.until(check, 10, f"notice {summary!r} in shown of notify")
    return found["id"]


def closed(s, notice):
    """Wait until chora-notify lists the id as closed and shows it no more;
    RuntimeError after 10 s."""

    def check():
        notify = s.state("notify") or {}
        shown = [n.get("id") for n in notify.get("shown", [])]
        return notice in notify.get("closed", []) and notice not in shown

    s.until(check, 10, f"notice {notice} in closed of notify")


def shown_before(sent):
    """The id of the notice that an earlier act showed; RuntimeError, which
    fails only this act, when that act failed."""
    if "id" not in sent:
        raise RuntimeError("no notice shown by the act before")
    return sent["id"]
