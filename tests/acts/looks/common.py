"""Acts that every look runs."""

import pathlib

from harness import REPO, Expect, click, key, program


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


def voice_call(method, params=None):
    """A cue: call a method of the voice agent."""
    return lambda s: s.core.call(
        "methods.call", {"client": "voice", "method": method, "params": params or {}}
    )


def agent(tool, **params):
    """A cue: a tool of the voice agent for its helpers."""
    return voice_call("agent", {"tool": tool, **params})


def presenter(method):
    """A cue: call a method of the helper presenter chora-assistant."""
    return lambda s: s.core.call(
        "methods.call", {"client": "assistant", "method": method, "params": {}}
    )


def helper_up(s):
    """A cue: start chora-voice with the stand-in helper of
    services/voice/tests/fake-acp.sh and the project demo (as
    tests/doors/lib.sh doors_voice), then the presenter chora-assistant."""
    config = pathlib.Path(s.env["XDG_CONFIG_HOME"])
    demo = config / "demo"
    (config / "chora").mkdir(parents=True, exist_ok=True)
    demo.mkdir(exist_ok=True)
    (config / "chora" / "chora-voice.yaml").write_text(
        f"""capture_cmd: [sh, -c, exec sleep 1000]
playback_cmd: [sh, -c, exec cat >/dev/null]
asr_url: ws://127.0.0.1:9
tts_url: ws://127.0.0.1:9
llm_url: http://127.0.0.1:9/v1
earcons: false
indicator: false
unsandboxed: true
agent_cmd: [sh, {REPO / "services/voice/tests/fake-acp.sh"}]
agent_projects: {{demo: {demo}}}
"""
    )
    s.spawn([str(s.bin / "chora-voice")], "voice.log")
    s.spawn([str(s.bin / "chora-assistant")], "assistant.log")
    s.until(lambda: {"voice", "assistant"} <= s.clients(), 40, "voice and the presenter connect")
    s.until(lambda: s.state("assistant") is not None, 30, "the presenter answers state.get")


def click_node(node):
    """A cue: wait until the node has a box, then click its middle (the
    core of the acts counts an injected press as one of the person)."""

    def box(s):
        for entry in (s.core.call("scene.boxes") or {}).values():
            for b in entry.get("boxes") or []:
                if b.get("node") == node:
                    return b["rect"]
        return None

    def cue(s):
        s.until(lambda: box(s), 20, f"a box of {node}")
        x, y, w, h = box(s)
        click(x + w / 2, y + h / 2)(s)

    return cue


def helper_is(state):
    """A cue: wait until voice says that the helper coder is in the state."""

    def said(s):
        agents = (
            s.core.call("methods.call", {"client": "voice", "method": "state", "params": {}}) or {}
        ).get("agents") or []
        return any(a.get("name") == "coder" and a.get("state") == state for a in agents)

    return lambda s: s.until(lambda: said(s), 30, f"coder {state}")


def helper_acts(s):
    """I8 in every look (KIT7): one helper state, the one of the presenter,
    in every look and in chora-assistant, through a turn of a helper: no
    helper, idle, working, a question and the answer (never working after
    it), the chat as the one helper surface of the output, no helper again."""
    ready(s)
    s.act("the helper presenter starts", [helper_up], expect=Expect(helper="no_helper"), timeout=60)
    s.act(
        "a helper starts",
        [agent("agent_start", project="demo", name="coder"), helper_is("idle")],
        expect=Expect(helper="idle"),
        timeout=30,
    )
    s.act(
        "the helper asks",
        [agent("agent_say", name="coder", text="Schreib notes.txt."), helper_is("question")],
        expect=Expect(helper="answered"),
        timeout=30,
    )
    s.act("the helper answers", [click_node("hud-card-yes"), helper_is("done")], timeout=30)
    s.act("the chat of the helper opens", [presenter("chat.show")], timeout=30)
    s.act("the chat of the helper shuts", [presenter("chat.hide")], timeout=30)
    s.act(
        "the helper stops",
        [agent("agent_stop", name="coder")],
        expect=Expect(helper="no_helper"),
        timeout=30,
    )
