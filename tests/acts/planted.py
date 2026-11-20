"""The live selftest of the act harness: in a real session of mc, each
invariant passes on a sound act and fails on a planted breach. I5 and I10
read the text boxes of `scene.boxes`: two overlapping text nodes and a raw
entry break them.

I7, I3, I8 (two helper surfaces) and I14 are broken in the session: a
black rect over the screen, the keyboard on a node of the harness, two
nodes marked as helper surfaces, a client that leaves. I1, I4 and I8
(working after the answer) need a look that misreports, so their breach
is planted in the answer of `state.get` that the harness reads; quiet is
planted the same way, with a `busy` that never ends. mc's real state has
no `open` yet, so the sound baseline gets it through the same filter.
"""

import contextlib
import json
import os
import pathlib
import subprocess
import tempfile
import types

from harness import REPO, Core, Expect, Session, known_red, program

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


def text(nid, words, at):
    """A text node of the harness under the overlay."""
    return {
        "create": {
            "id": nid,
            "kind": "text",
            "parent": f"screen:{OUT}",
            "props": {"text": words, "font_size": 16, "transform": {"translate": at}},
        }
    }


def overprint(s):
    """A cue: two lines of text on top of each other (j1-4)."""
    apply(
        s,
        text("planted-text-1", "No helper", [400, 300]),
        text("planted-text-2", "Volume 40%", [420, 304]),
    )


def raw_entry(s):
    """A cue: a launcher entry shown raw."""
    apply(s, text("planted-raw", "org.gnome.TextEditor.desktop", [400, 360]))


def rect(nid, at, size, color, meta=None):
    """A rect node of the harness under the overlay."""
    props = {"size": size, "color": color, "transform": {"translate": at}}
    if meta:
        props["meta"] = meta
    return {"create": {"id": nid, "kind": "rect", "parent": f"screen:{OUT}", "props": props}}


def coloured(nid, words, at, size, color):
    """A text node of a colour, in a font size."""
    node = text(nid, words, at)
    node["create"]["props"] |= {"font_size": size, "color": color}
    return node


def faint_text(s):
    """A cue: grey text on a nearly grey panel (contrast about 1.1:1)."""
    apply(
        s,
        rect("planted-faint-bg", [400, 400], [300, 40], [0.5, 0.5, 0.5, 1]),
        coloured("planted-faint", "Next step", [410, 408], 16, [0.56, 0.56, 0.56, 1]),
    )


def small_text(s):
    """A cue: white text of 10 px on a dark panel (contrast sound)."""
    apply(
        s,
        rect("planted-small-bg", [400, 460], [300, 40], [0.05, 0.05, 0.05, 1]),
        coloured("planted-small", "Next step", [410, 468], 10, [1, 1, 1, 1]),
    )


def odd_gaps(s):
    """A cue: two helper surfaces 13 px apart and two tiles 7 px apart."""
    surf = {"surface": "planted"}
    apply(
        s,
        rect("planted-gap-a", [100, 100], [100, 60], [0.2, 0.2, 0.2, 1], surf),
        rect("planted-gap-b", [213, 100], [100, 60], [0.2, 0.2, 0.2, 1], surf),
        rect("tb-planted-1", [100, 300], [100, 60], [0.2, 0.2, 0.2, 1]),
        rect("tb-planted-2", [207, 300], [100, 60], [0.2, 0.2, 0.2, 1]),
    )


def agent_word(s):
    """A cue: a text with the word of the builders."""
    apply(s, coloured("planted-agent", "Ask the agent", [400, 520], 16, [1, 1, 1, 1]))


def remove(s, *ids):
    """Remove nodes of the harness."""
    apply(s, *({"remove": {"id": i}} for i in ids))


def filtered(s, change):
    """Plant a change into the states that the harness reads."""

    def state_filter(_look, state):
        return change(dict(state))

    s.state_filter = state_filter


def persona_check(must):
    """tests/persona/check.py in a persona session of mc: a sound action
    leaves nothing in breaches.jsonl, a planted black screen leaves an I7
    line there."""
    persona = REPO / "tests" / "persona"
    with tempfile.TemporaryDirectory(prefix="planted-persona-") as work:
        env = dict(
            os.environ,
            PERSONA_DIR=work,
            CHORA_BIN=os.environ.get("BIN", str(REPO / "target" / "debug")),
        )
        session = subprocess.Popen(
            [str(persona / "session.sh"), "mc"],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            start_new_session=True,
        )
        env_file = pathlib.Path(work) / "session.env"
        try:
            if not session.stdout.readline().startswith("PERSONA_ENV="):
                raise RuntimeError("the persona session did not start")
            sock = next(
                line.split("=", 1)[1]
                for line in env_file.read_text().splitlines()
                if line.startswith("CHORA_SOCKET=")
            )
            check = env | {"PERSONA_ENV": str(env_file)}

            def after(action):
                subprocess.run(
                    [str(persona / "check.py"), "after", action],
                    env=check,
                    check=True,
                    capture_output=True,
                    timeout=60,
                )
                path = pathlib.Path(work) / "breaches.jsonl"
                return (
                    [json.loads(line) for line in path.read_text().splitlines()]
                    if path.exists()
                    else []
                )

            subprocess.run([str(persona / "check.py"), "start"], env=check, check=True, timeout=60)
            listed = {(r["look"], r["id"]) for r in known_red()}
            sound = [b for b in after("sound") if (b["look"], b["invariant"]) not in listed]
            must("persona sound", sound, fail=False)
            planted = Core(sock, name="planted-persona")
            black_screen(
                types.SimpleNamespace(core=planted, outputs=lambda: planted.call("output.list"))
            )
            found = [b for b in after("planted-black") if b["invariant"] == "I7"]
            must("persona I7 planted", found, fail=True)
            planted.close()
        finally:
            session.terminate()
            with contextlib.suppress(subprocess.TimeoutExpired):
                session.wait(timeout=20)


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
        sound_keys = ("I1", "I2", "I3", "I4", "I5", "I6", "I7", "I8", "I10", "I12", "I14")
        for key in (*sound_keys, "act:a window opens"):
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
        must("I5 two texts overprint", cell(s, "overprint", "I5", [overprint]), fail=True)
        remove(s, "planted-text-1", "planted-text-2")
        must("I10 a raw entry shows", cell(s, "raw", "I10", [raw_entry]), fail=True)
        remove(s, "planted-raw")

        must("I6 faint text", cell(s, "faint", "I6", [faint_text]), fail=True)
        remove(s, "planted-faint-bg", "planted-faint")
        must("I6 small text", cell(s, "small", "I6", [small_text]), fail=True)
        remove(s, "planted-small-bg", "planted-small")
        must("I2 odd gaps", cell(s, "gaps", "I2", [odd_gaps]), fail=True)
        remove(s, "planted-gap-a", "planted-gap-b", "tb-planted-1", "tb-planted-2")
        must("I10 the word agent", cell(s, "agent", "I10", [agent_word]), fail=True)
        remove(s, "planted-agent")
        apply(s, rect("planted-fixed", [900, 40], [100, 60], [0.2, 0.2, 0.2, 1], {"surface": "p"}))
        move = lambda _s: apply(s, {"update": {"id": "planted-fixed", "props": {"transform": {"translate": [903, 40]}}}})
        must("I2 held, a fixed surface moves", cell(s, "moved", "I2", [move], held=0.3), fail=True)
        must("I2 held, a fixed surface stays", cell(s, "stays", "I2", held=0.3), fail=False)
        remove(s, "planted-fixed")

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
    persona_check(must)
    return checks
