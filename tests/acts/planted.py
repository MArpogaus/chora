"""The live selftest of the act harness: in a real session of mc, each
invariant passes on a sound act and fails on a planted breach.

I7, I3, I8 (two helper surfaces) and I14 are broken in the session: a
black rect over the screen, the keyboard on a node of the harness, two
nodes marked as helper surfaces, a client that leaves. I1, I4 and I8
(working after the answer) need a look that misreports, so their breach
is planted in the answer of `state.get` that the harness reads; quiet is
planted the same way, with a `busy` that never ends. mc's real state has
no `open` yet, so the sound baseline gets it through the same filter.
"""

import contextlib

from harness import Core, Expect, Session, program

OUT = "HEADLESS-1"


def cell(s, name, invariant, cues=(), **kw):
    """Run an act with fresh results; the breaches of one invariant. A
    cell of an invariant whose act broke answers the act's failure under
    `act:`, so a broken act never stands in for a breach."""
    s.results.clear()
    s.act(name, cues, **kw)
    key = (s.label, invariant)
    broken = s.results.get((s.label, f"act:{name}"), [])
    if broken and not invariant.startswith("act:"):
        raise RuntimeError(f"the act {name!r} of the cell {invariant} broke: {broken[0]}")
    return s.results.get(key, [])


def apply(s, *commands):
    """A transaction of the harness client."""
    s.core.call("scene.apply", {"commands": list(commands)})


def black_screen(s):
    """A cue: an opaque black rect over the whole output."""
    w, h = s.outputs()[0]["size"]
    apply(
        s,
        {
            "create": {
                "id": "planted-black",
                "kind": "rect",
                "parent": f"screen:{OUT}",
                "props": {"size": [w, h], "color": [0, 0, 0, 1]},
            }
        },
    )


def focus_own_node(s):
    """A cue: the keyboard goes to a node of the harness."""
    apply(
        s,
        {
            "create": {
                "id": "planted-focus",
                "kind": "rect",
                "parent": f"screen:{OUT}",
                "props": {"size": [4, 4], "color": [1, 0, 0, 1]},
            }
        },
        {"focus_node": {"node": "planted-focus"}},
    )


def two_helpers(s):
    """A cue: two nodes that say they are helper surfaces, on one output."""
    apply(
        s,
        *(
            {
                "create": {
                    "id": f"planted-helper-{i}",
                    "kind": "rect",
                    "parent": f"screen:{OUT}",
                    "props": {
                        "size": [40, 40],
                        "color": [0.3, 0.3, 0.3, 1],
                        "meta": {"helper": "surface"},
                    },
                }
            }
            for i in (1, 2)
        ),
    )


def remove(s, *ids):
    """Remove nodes of the harness."""
    apply(s, *({"remove": {"id": i}} for i in ids))


def filtered(s, change):
    """Plant a change into the states that the harness reads."""

    def state_filter(_look, state):
        return change(dict(state))

    s.state_filter = state_filter


def run(results):
    """Run the planted cells; results maps the cell to its failures."""
    checks = []

    def must(name, breaches, fail):
        ok = bool(breaches) == fail
        checks.append(name)
        results[("selftest", name)] = (
            []
            if ok
            else [f"expected {'a breach' if fail else 'no breach'}, got {breaches or 'none'}"]
        )

    with Session("mc") as s:
        filtered(s, lambda st: st | {"open": []})
        s.act(
            "a window opens",
            [program("chora-probe", "window", "40", "120", "200")],
            expect=Expect(new_window={"app_id": "chora-probe", "owner": "mc", "focused": True}),
        )
        sound = dict(s.results)
        for key in ("I1", "I3", "I4", "I7", "I8", "I14", "act:a window opens"):
            must(f"sound {key}", sound.get(("mc", key), []), fail=False)

        must("I7 black screen", cell(s, "black", "I7", [black_screen]), fail=True)
        remove(s, "planted-black")
        must(
            "I3 focus on a node of the harness", cell(s, "focus", "I3", [focus_own_node]), fail=True
        )
        apply(s, {"focus_node": {}})
        remove(s, "planted-focus")
        must("I8 two helper surfaces", cell(s, "helpers", "I8", [two_helpers]), fail=True)
        remove(s, "planted-helper-1", "planted-helper-2")

        planted = Core(s.env["CHORA_SOCKET"], name="planted")
        s.until(lambda: "planted" in s.clients(), 5, "planted client")
        must(
            "I14 declared disconnect",
            cell(s, "declared", "I14", [lambda _s: planted.close()], disconnects=["planted"]),
            fail=False,
        )
        planted = Core(s.env["CHORA_SOCKET"], name="planted")
        s.until(lambda: "planted" in s.clients(), 5, "planted client")
        must(
            "I14 undeclared disconnect",
            cell(s, "undeclared", "I14", [lambda _s: planted.close()]),
            fail=True,
        )

        must("sound again I1", cell(s, "plain", "I1"), fail=False)
        dropped = []

        def drop(state):
            if dropped:
                state["windows"] = []
            return state

        filtered(s, drop)
        must(
            "I1 a window missing from its look",
            cell(s, "dropped", "I1", [lambda _s: dropped.append(1)]),
            fail=True,
        )
        opened = []
        filtered(s, lambda st: st | {"open": [{"id": "planted-menu"}] if opened else []})
        must(
            "I4 an undeclared open", cell(s, "open", "I4", [lambda _s: opened.append(1)]), fail=True
        )
        opened.clear()
        must(
            "I4 a declared open",
            cell(s, "declared open", "I4", [lambda _s: opened.append(1)], opens=["planted-menu"]),
            fail=False,
        )

        voice = Core(s.env["CHORA_SOCKET"], name="voice")
        filtered(s, lambda st: st | {"helper": "working"})
        must(
            "I8 working after the answer",
            cell(
                s,
                "answered",
                "I8",
                [
                    lambda _s: voice.call(
                        "events.emit", {"event": "voice.state", "data": {"state": "idle"}}
                    )
                ],
            ),
            fail=True,
        )
        with contextlib.suppress(OSError):
            voice.close()

        busy = []
        filtered(s, lambda st: st | {"busy": ["planted-animation"] if busy else []})
        must(
            "quiet times out",
            cell(s, "busy", "act:busy", [lambda _s: busy.append(1)], timeout=2),
            fail=True,
        )
        s.state_filter = None
    return checks
