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
    s.spawn([str(s.bin / "chora-voice")], "voice.log", FAKE_ACP_ASK_AFTER=str(asked(s)))
    s.spawn([str(s.bin / "chora-assistant")], "assistant.log")
    s.until(lambda: {"voice", "assistant"} <= s.clients(), 40, "voice and the presenter connect")
    s.until(lambda: s.state("assistant") is not None, 30, "the presenter answers state.get")


def asked(s):
    """The file that lets the stand-in helper ask its question: until it
    exists the helper works (FAKE_ACP_ASK_AFTER of fake-acp.sh)."""
    return pathlib.Path(s.env["XDG_CONFIG_HOME"]) / "demo-ask"


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
        "the helper works",
        [agent("agent_say", name="coder", text="Schreib notes.txt."), helper_is("working")],
        expect=Expect(helper="working"),
        timeout=30,
    )
    s.act(
        "the helper asks",
        [lambda s: asked(s).touch(), helper_is("question")],
        expect=Expect(helper="answered"),
        timeout=30,
    )
    s.act(
        "the helper answers",
        [click_node("hud-card-yes"), helper_is("done")],
        expect=Expect(helper="answered"),
        timeout=30,
    )
    s.act("the chat of the helper opens", [presenter("chat.show")], timeout=30)
    s.act("the chat of the helper shuts", [presenter("chat.hide")], timeout=30)
    s.act(
        "the helper stops",
        [agent("agent_stop", name="coder")],
        expect=Expect(helper="no_helper"),
        timeout=30,
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
    another way than its cap. A row that joins keys or words with a slash
    counts as one text for each part."""
    spec_words, spec_caps = keysheet_spec()
    texts = [part for line in texts for part in line.split(" / ")]
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


def keysheet_acts(s, sheet, close="Escape", drawn_in_state=False):
    """The key list opens and its drawn texts hold the words and the
    spelling of kit::keysheet (KIT3): one wording per action of every
    look and one spelling per key, in all four looks. A look that paints the
    list into an image node (xp) tells its texts in `state.get`; the others
    draw text nodes of the scene."""
    name = f"the key list of {s.label} speaks the keysheet"
    after = s.act(
        name,
        [key("Super+F1")],
        opens=[sheet],
    )
    if after is not None:
        if drawn_in_state:
            runs = (s.state(s.label) or {}).get("texts") or []
            texts = [r["text"] for r in runs if r["node"].startswith(sheet)]
        else:
            query = {"selector": f"owner:{s.label} kind:text"}
            nodes = s.core.call("scene.query", query) or []
            texts = [n.get("text") for n in nodes if n.get("visible") and n.get("text")]
        s.record(s.label, f"act:{name}", keysheet_breaches(texts), name)
    s.act(f"{close} shuts the key list of the keysheet", [key(close)], closes=[sheet])
