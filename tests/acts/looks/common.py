"""Acts that every look runs."""

import pathlib

from harness import Expect, key, program


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


KEYSHEET = pathlib.Path(__file__).resolve().parents[2] / "keysheet" / "words.table"

EVERY_LOOK = [
    "keys",
    "chat",
    "lock",
    "power",
    "screenshot",
    "volume:up",
    "volume:down",
    "volume:mute",
    "launcher",
]


def keysheet_spec():
    """The English words by action and the spelling of the keys, from the
    table that kit::keysheet is held to (tests/keysheet/words.table)."""
    words, caps = {}, {}
    for line in KEYSHEET.read_text().splitlines():
        fields = line.split("\t")
        if line.startswith("words\t"):
            words[fields[1]] = fields[2]
        elif line.startswith("cap\t"):
            caps[fields[1]] = fields[2]
    return words, caps


def keysheet_breaches(texts, words=None, caps=None):
    """What a key list that draws `texts` gets wrong: a wording of an action
    of every look that is not the one of kit::keysheet, and a key spelled
    another way than its cap."""
    spec_words, spec_caps = keysheet_spec()
    words = words or {name: spec_words[name] for name in EVERY_LOOK}
    caps = caps or spec_caps
    breaches = [
        f"the key list does not say {english!r} for {name}"
        for name, english in words.items()
        if english not in texts
    ]
    raw = {key for key, cap in caps.items() if len(key) > 1 and cap != key} | {"Enter"}
    for line in texts:
        parts = line.replace(" ", "").split("+")
        spelled = [p for p in parts if p in raw and (len(parts) > 1 or line in raw)]
        breaches += [f"the key {p!r} is not spelled as its cap in {line!r}" for p in spelled]
    return breaches


def keysheet_acts(s, sheet, close="Escape"):
    """The key list opens and its drawn texts hold the words and the
    spelling of kit::keysheet (KIT3): one wording per action of every
    look and one spelling per key, in all four looks."""
    name = f"the key list of {s.label} speaks the keysheet"
    after = s.act(
        name,
        [key("Super+F1")],
        opens=[sheet],
    )
    if after is not None:
        nodes = s.core.call("scene.query", {"selector": f"owner:{s.label} kind:text"}) or []
        texts = [n.get("text") for n in nodes if n.get("visible") and n.get("text")]
        s.record(s.label, f"act:{name}", keysheet_breaches(texts), name)
    s.act(f"{close} shuts the key list of the keysheet", [key(close)], closes=[sheet])
