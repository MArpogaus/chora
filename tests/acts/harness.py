"""Act contracts (design 3.2 of the arch plan): a session of one or more
looks, acts that run cues and compare what changed with what they expect,
and the invariants of invariants.py after every act.

    with Session("xp") as s:
        s.act("open a window", cues=[program("chora-probe", "window", "40")],
              expect=Expect(new_window={"app_id": "chora-probe", "owner": "xp",
                                        "focused": True}))

`act` runs the cues, waits until every `busy` of every look is empty
(quiet; a timeout fails the act), reads `state.get` of each look and of
the blocks (chora-notify), `scene.query` for the nodes and `scene.boxes`
for what each output shows, compares with `expect` and runs every
invariant. A
mismatch keeps the act, the state diff and a still in the run folder.
An act declares what it leaves open (`opens`), what it closes
(`closes`) and the clients it ends on purpose (`disconnects`).

Results go by (look, id), the id an invariant (`I1`) or an act
(`act:NAME`); `judge` holds them against tests/acts/known-red.toml: a
listed failure does not block, a listed pass blocks (the row must go),
an unlisted failure blocks.
"""

import base64
import contextlib
import dataclasses
import importlib.util
import json
import os
import pathlib
import queue
import shutil
import signal
import socket
import subprocess
import tempfile
import threading
import time
import tomllib

import invariants

REPO = pathlib.Path(__file__).resolve().parents[2]
KNOWN_RED = REPO / "tests" / "acts" / "known-red.toml"
EVENTS = ["effect.failed", "effects.dropped", "focus.changed", "surface.created", "voice.*"]
SETTLE = 0.4
QUIET_GAP = 0.2
TERM_WAIT = 8
CAPTURE_SIDE = 320


def load_png_rows():
    """The PNG decoder of tests/png-pixel.py (its name has a dash)."""
    spec = importlib.util.spec_from_file_location("png_pixel", REPO / "tests" / "png-pixel.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.rows


@dataclasses.dataclass
class Snapshot:
    """What the harness reads at one moment of a session."""

    states: dict
    nodes: list
    toplevels: set
    colours: dict
    wallpaper: set
    clients: set
    events: list = dataclasses.field(default_factory=list)
    logs: dict = dataclasses.field(default_factory=dict)
    outputs: list = dataclasses.field(default_factory=list)
    boxes: dict | None = None
    blocks: dict = dataclasses.field(default_factory=dict)

    def by_id(self):
        """The nodes by id."""
        return {n.get("id"): n for n in self.nodes}

    def windows(self):
        """Every toplevel with its node and the look whose state lists it."""
        listed = {}
        for look, state in self.states.items():
            for w in (state or {}).get("windows") or []:
                listed.setdefault(w.get("surface"), (look, w))
        nodes = self.by_id()
        return {sid: (nodes.get(sid, {}), *listed.get(sid, (None, {}))) for sid in self.toplevels}


@dataclasses.dataclass
class Act:
    """An act: its name, what it opens, closes and disconnects on purpose,
    and whether it reads a look held in the middle of its motion."""

    name: str
    opens: list = dataclasses.field(default_factory=list)
    closes: list = dataclasses.field(default_factory=list)
    disconnects: list = dataclasses.field(default_factory=list)
    held: bool = False


def app_id(node):
    """The app id of a surface node."""
    return (node.get("surface") or {}).get("app_id")


@dataclasses.dataclass
class Expect:
    """What an act expects to change; a field left at None is not checked.

    new_window  one new toplevel whose fields match: `app_id`, `title`,
                `owner` (the client that owns its node), `focused`, and
                any field of its entry in the look's `windows`
    gone        the number of toplevels that went away
    focus       the look that says it holds the focus once quiet
    open        the ids that are open afterwards, in any look
    helper      the `helper` word of every look that reports one and of
                the presenter chora-assistant while it runs
    """

    new_window: dict | None = None
    gone: int | None = None
    focus: str | None = None
    open: list | None = None
    helper: str | None = None

    def check(self, before, after):
        """The mismatches between the expectation and the two snapshots."""
        out = []
        new = after.toplevels - before.toplevels
        if self.new_window is not None:
            out += self.check_new(after, new)
        if self.gone is not None and len(before.toplevels - after.toplevels) != self.gone:
            gone = sorted(before.toplevels - after.toplevels)
            out.append(f"expected {self.gone} windows gone, got {gone}")
        if self.focus is not None:
            holders = [
                look
                for look, s in after.states.items()
                if s and (s.get("focus") or {}).get("holder") == look
            ]
            if holders != [self.focus]:
                out.append(f"expected the focus with {self.focus}, holders {holders}")
        if self.open is not None and sorted(invariants.open_ids(after)) != sorted(self.open):
            out.append(
                f"expected open {sorted(self.open)}, got {sorted(invariants.open_ids(after))}"
            )
        if self.helper is not None:
            words = {
                look: s.get("helper") for look, s in after.states.items() if s and "helper" in s
            }
            if "assistant" in after.clients:
                words["assistant"] = ((after.blocks or {}).get("assistant") or {}).get("helper")
            if not words or any(w != self.helper for w in words.values()):
                out.append(f"expected the helper {self.helper}, got {words}")
        return out

    def check_new(self, after, new):
        """The mismatches of the one new window."""
        if len(new) != 1:
            return [f"expected one new window {self.new_window}, got {sorted(new)}"]
        sid = next(iter(new))
        node, _look, entry = after.windows()[sid]
        got = {
            "app_id": app_id(node),
            "title": (node.get("surface") or {}).get("title"),
            "owner": node.get("owner"),
            "focused": bool(node.get("focused")),
        } | entry
        return [
            f"the new window {sid}: {key} is {got.get(key)!r}, expected {want!r}"
            for key, want in self.new_window.items()
            if got.get(key) != want
        ]


def work_rect(output):
    """The work area of an output in its own logical pixels: the zone of
    the output, `default_work_area`, in scene coordinates of the default
    camera, which moves it by the position of the output (a virtual output
    lies far off the screens). `work_area` follows the camera of the
    output, so a look that moves the camera would move this crop."""
    px, py = output.get("position") or [0, 0]
    x, y, w, h = output.get("default_work_area") or [px, py, *output["size"]]
    return [x - px, y - py, w, h]


class NotQuiet(Exception):
    """The looks did not get quiet within the timeout."""


def busy_of(states):
    """Every running `busy` entry, as LOOK:ENTRY; NotQuiet for a look that
    answers `state.get` without the key."""
    for look, s in sorted(states.items()):
        if s is not None and "busy" not in s:
            raise NotQuiet(f"{look} missing key busy: its state does not say when it is quiet")
    return [f"{look}:{b}" for look, s in sorted(states.items()) if s for b in s["busy"] or []]


def wait_quiet(read, timeout, sleep=time.sleep, now=time.monotonic):
    """Read the states until two reads in a row, QUIET_GAP s apart, show no
    look busy; NotQuiet at the timeout."""
    start = now()
    empty = 0
    while True:
        states = read()
        busy = busy_of(states)
        empty = 0 if busy else empty + 1
        if empty >= 2:
            return states
        if busy and now() - start >= timeout:
            raise NotQuiet(f"still busy after {timeout} s: {', '.join(busy)}")
        sleep(QUIET_GAP if not busy else 0.1)


@dataclasses.dataclass
class Verdict:
    """The judgement of a run against the known-red list."""

    blocking: list
    red: list
    passed: list


def known_red(path=KNOWN_RED):
    """The rows of the known-red list."""
    with open(path, "rb") as f:
        return tomllib.load(f).get("red", [])


def judge(results, rows, full=False):
    """Hold the results, {(look, id): [breach, ...]}, against the rows.

    A row needs `look`, `id`, `finding` and `slice`; an optional `match`
    must occur in every breach text of the cell. With `full`, a row
    that did not run blocks too: the list names only what runs.
    """
    listed = {}
    for row in rows:
        missing = [k for k in ("look", "id", "finding", "slice") if not row.get(k)]
        if missing:
            raise ValueError(f"known-red row {row} lacks {', '.join(missing)}")
        listed[(row["look"], row["id"])] = row
    blocking, red, passed = [], [], []
    for key, breaches in sorted(results.items()):
        row = listed.get(key)
        if row and breaches and row.get("match"):
            row = row if all(row["match"] in b for b in breaches) else None
        if breaches and row:
            red.append(key)
        elif breaches:
            blocking.append(f"FAIL: {key[0]} {key[1]}: {breaches[0]}")
        elif row:
            blocking.append(
                f"FAIL: {key[0]} {key[1]} passes but is listed in known-red.toml "
                f"({row['finding']}); remove the row"
            )
        else:
            passed.append(key)
    if full:
        blocking += [
            f"FAIL: known-red row {k[0]} {k[1]} did not run"
            for k in sorted(listed)
            if k not in results and k[0] in {r[0] for r in results}
        ]
    return Verdict(blocking, red, passed)


class Core:
    """A JSON-RPC connection to the core, one message per line."""

    def __init__(self, path, name=None, events=None):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(path)
        self.file = self.sock.makefile("rwb")
        self.next_id = 0
        self.events = queue.Queue()
        self.answers = queue.Queue()
        self.reader = threading.Thread(target=self.read, daemon=True)
        self.reader.start()
        if name:
            self.call("hello", {"name": name})
        if events is not None:
            self.call("events.subscribe", {"events": events})

    def read(self):
        """Sort the lines into answers and events until the socket ends."""
        for line in self.file:
            message = json.loads(line)
            if message.get("method") == "event":
                self.events.put(message.get("params") or {})
            else:
                self.answers.put(message)
        self.answers.put(None)

    def call(self, method, params=None, timeout=10):
        """Call a method; RuntimeError with the error of the core."""
        self.next_id += 1
        request = {"jsonrpc": "2.0", "id": self.next_id, "method": method}
        if params is not None:
            request["params"] = params
        self.file.write(json.dumps(request).encode() + b"\n")
        self.file.flush()
        while True:
            answer = self.answers.get(timeout=timeout)
            if answer is None:
                raise RuntimeError(f"the core closed the connection during {method}")
            if answer.get("id") == self.next_id:
                break
        if "error" in answer:
            raise RuntimeError(f"{method}: {answer['error'].get('message')}")
        return answer.get("result")

    def drain(self):
        """The events that arrived since the last drain; RuntimeError when
        the core closed the connection, so no event goes unseen."""
        if not self.reader.is_alive():
            raise RuntimeError("the core closed the event connection")
        out = []
        with contextlib.suppress(queue.Empty):
            while True:
                out.append(self.events.get_nowait())
        return out

    def close(self):
        """End the connection."""
        with contextlib.suppress(OSError):
            self.sock.shutdown(socket.SHUT_RDWR)
        self.sock.close()


LOOKS = {
    "xp": (["chora-xp", "--theme", str(REPO / "flavors/xp/theme")], {}),
    "mc": (["chora-mc", "--manage-windows"], {}),
    "tiling": (["chora-tiling"], {"CHORA_FLAVOR_DIR": str(REPO / "flavors/tiling")}),
    "construct": (
        ["chora-construct"],
        {"CHORA_FLAVOR_DIR": str(REPO / "flavors/construct"), "CHORA_FRAME_BUDGET_US": "500000"},
    ),
}
WALLPAPER = {"xp", "mc", "construct"}
BLOCKS = ["notify", "assistant"]


class Session:
    """A headless session with a look as its startup command and the
    other looks beside it; the context ends every program it started.

    looks      the looks, the first one starts with the core
    size       the output, as tests/headless-session.sh takes it
    wallpaper  the outputs whose work area I7 checks; by default every
               output when the first look draws the wallpaper (tiling
               leaves it to swaybg)
    """

    def __init__(self, *looks, size="1280x800", wallpaper=None, results=None, runs=None):
        self.looks = list(looks)
        self.size = size
        self.wallpaper = wallpaper
        self.label = self.looks[0]
        self.results = {} if results is None else results
        self.runs = pathlib.Path(runs or tempfile.mkdtemp(prefix="acts-"))
        self.bin = pathlib.Path(os.environ.get("BIN", REPO / "target" / "debug"))
        self.procs = []
        self.log_marks = {}
        self.state_filter = None

    def __enter__(self):
        self.work = pathlib.Path(tempfile.mkdtemp(prefix="acts-session-"))
        try:
            return self.setup()
        except BaseException:
            self.__exit__(None, None, None)
            raise

    def setup(self):
        """Start the core and the looks; the caller ends them on a failure."""
        home = self.work / "home"
        for d in ("config", "home", "data", "state", "pictures"):
            (self.work / d).mkdir()
        self.env = dict(
            os.environ,
            XDG_CONFIG_HOME=str(self.work / "config"),
            HOME=str(home),
            XDG_DATA_HOME=str(self.work / "data"),
            XDG_STATE_HOME=str(self.work / "state"),
            XDG_PICTURES_DIR=str(self.work / "pictures"),
            CHORA_BIN=str(self.bin),
            PATH=f"{self.bin}:{os.environ.get('PATH', '')}",
            CHORA_TEST_ACCEPT_INJECTED_KEYS="1",
        )
        command, extra = LOOKS[self.looks[0]]
        env = dict(self.env, CHORA_TEST_FLAGS="--test-injected-press-is-real", **extra)
        env_file = self.work / "env"
        with open(env_file, "w") as out:
            self.core_proc = subprocess.Popen(
                [
                    str(REPO / "tests/headless-session.sh"),
                    self.size,
                    "--",
                    str(self.bin / command[0]),
                    *command[1:],
                ],
                stdout=out,
                stderr=subprocess.STDOUT,
                env=env,
                start_new_session=True,
            )
        self.procs.append(self.core_proc)
        values = self.wait_env(env_file)
        self.env.update(values)
        self.runtime = pathlib.Path(values["XDG_RUNTIME_DIR"])
        for look in self.looks[1:]:
            command, extra = LOOKS[look]
            self.spawn([str(self.bin / command[0]), *command[1:]], f"{look}.log", **extra)
        subprocess.run(
            [
                str(self.bin / "choractl"),
                "grant",
                "acts",
                "screen.capture",
                "input.inject",
                "methods.call",
            ],
            env=self.env,
            check=True,
            capture_output=True,
        )
        self.core = Core(values["CHORA_SOCKET"], name="acts")
        self.watch = Core(values["CHORA_SOCKET"], name="acts-events", events=EVENTS)
        self.until(lambda: set(self.looks) <= self.clients(), 30, "the looks connect")
        self.until(lambda: self.outputs(), 10, "an output")
        return self

    def __exit__(self, *_exc):
        for core in (getattr(self, "watch", None), getattr(self, "core", None)):
            if core:
                core.close()
        for proc in reversed(self.procs):
            with contextlib.suppress(ProcessLookupError, PermissionError):
                os.killpg(proc.pid, signal.SIGTERM)
        end = time.monotonic() + TERM_WAIT
        for proc in reversed(self.procs):
            with contextlib.suppress(subprocess.TimeoutExpired):
                proc.wait(timeout=max(0.1, end - time.monotonic()))
        for proc in reversed(self.procs):
            with contextlib.suppress(ProcessLookupError, PermissionError):
                os.killpg(proc.pid, signal.SIGKILL)
            with contextlib.suppress(subprocess.TimeoutExpired):
                proc.wait(timeout=2)
        shutil.rmtree(self.work, ignore_errors=True)

    def wait_env(self, env_file):
        """The KEY=VALUE lines of tests/headless-session.sh."""
        for _ in range(120):
            text = env_file.read_text()
            if "CHORA_SOCKET=" in text:
                time.sleep(0.2)
                text = env_file.read_text()
                return dict(
                    line.split("=", 1)
                    for line in text.splitlines()
                    if "=" in line and line.split("=", 1)[0].isupper()
                )
            if self.core_proc.poll() is not None:
                break
            time.sleep(0.5)
        raise RuntimeError(f"no core: {env_file.read_text()[-400:]}")

    def until(self, check, seconds, what):
        """Wait until the check holds; RuntimeError after the seconds."""
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            with contextlib.suppress(RuntimeError, OSError):
                if check():
                    return
            time.sleep(0.25)
        raise RuntimeError(f"no {what} within {seconds} s")

    def spawn(self, argv, log, **env):
        """Start a program of the session with its log in the work folder."""
        out = open(self.work / log, "ab")  # noqa: SIM115
        proc = subprocess.Popen(
            argv,
            stdout=out,
            stderr=subprocess.STDOUT,
            env=dict(self.env, **env),
            start_new_session=True,
        )
        out.close()
        self.procs.append(proc)
        return proc

    def clients(self):
        """The connected client names."""
        return {c.get("name") for c in self.core.call("clients.list") or []}

    def outputs(self):
        """The outputs."""
        return self.core.call("output.list") or []

    def state(self, look):
        """The answer of `state.get` of a look; None when it has none."""
        try:
            state = self.core.call("methods.call", {"client": look, "method": "state.get"})
        except RuntimeError:
            return None
        return self.state_filter(look, state) if self.state_filter else state

    def states(self):
        """The states of the looks."""
        return {look: self.state(look) for look in self.looks}

    def blocks(self, clients):
        """The states of the connected blocks that are no look."""
        return {b: self.state(b) for b in BLOCKS if b in clients and b not in self.looks}

    def inject(self, events):
        """Synthetic input through the core."""
        self.core.call("input.inject", {"events": events})

    def capture(self, output, side=CAPTURE_SIDE):
        """A capture of an output: width, height, scale and the PNG bytes."""
        shot = self.core.call("screen.capture", {"output": output, "max_side": side})
        return shot, base64.b64decode(shot["png"])

    def colours(self, output):
        """The number of colours in the work area of an output."""
        shot, png = self.capture(output["name"])
        x, y, w, h = work_rect(output)
        scale = shot["width"] / output["size"][0]
        x0, y0 = int(x * scale), int(y * scale)
        x1, y1 = int((x + w) * scale), int((y + h) * scale)
        seen = set()
        for r, row in enumerate(load_png_rows()(png)):
            if y0 <= r < y1:
                seen.update(bytes(row[i : i + 3]) for i in range(x0 * 4, x1 * 4, 4))
        return len(seen)

    def mark_logs(self):
        """Remember the end of every log, so a snapshot reads what follows."""
        self.log_marks = {p: p.stat().st_size for p in self.logs()}

    def logs(self):
        """The logs of the core and of the programs of the session."""
        return [*self.runtime.glob("*.log"), *self.work.glob("*.log")]

    def new_log_text(self):
        """What the logs got since the mark, by file name."""
        out = {}
        for path in self.logs():
            with open(path, "rb") as f:
                f.seek(self.log_marks.get(path, 0))
                out[path.stem] = f.read().decode(errors="replace")
        return out

    def snapshot(self, states=None, events=()):
        """Read the session; the states when the caller has them."""
        nodes = self.core.call("scene.query", {"selector": "*"}) or []
        outputs = self.outputs()
        wallpaper = self.wallpaper
        if wallpaper is None:
            wallpaper = {o["name"] for o in outputs} if self.looks[0] in WALLPAPER else set()
        colours = {o["name"]: self.colours(o) for o in outputs if o["name"] in wallpaper}
        clients = self.clients()
        return Snapshot(
            states=self.states() if states is None else states,
            nodes=nodes,
            toplevels={n["id"] for n in nodes if n.get("kind") == "surface"},
            colours=colours,
            wallpaper=set(wallpaper),
            clients=clients,
            events=list(events),
            logs=self.new_log_text(),
            outputs=outputs,
            boxes=self.core.call("scene.boxes"),
            blocks=self.blocks(clients),
        )

    def act(
        self,
        name,
        cues=(),
        expect=None,
        opens=(),
        closes=(),
        disconnects=(),
        timeout=10,
        look=None,
        held=None,
    ):
        """Run an act, record its result and those of the invariants. With
        `held` seconds the act reads the session that long after its cues
        without waiting for quiet: a look that animates is read in the
        middle of its motion, which the next act waits out."""
        act = Act(name, list(opens), list(closes), list(disconnects), held is not None)
        look = look or self.label
        try:
            before = self.snapshot(wait_quiet(self.states, timeout))
            self.watch.drain()
            self.mark_logs()
            for cue in cues:
                cue(self)
            time.sleep(SETTLE if held is None else held)
            states = self.states() if held is not None else wait_quiet(self.states, timeout)
            after = self.snapshot(states, self.watch.drain())
        except (NotQuiet, RuntimeError, OSError, queue.Empty) as err:
            self.record(look, f"act:{name}", [f"{type(err).__name__}: {err}"], name)
            return None
        mismatches = expect.check(before, after) if expect else []
        self.record(look, f"act:{name}", mismatches, name, before, after)
        for key, check in invariants.ALL.items():
            self.record(look, key, check(before, after, act), name, before, after)
        return after

    def record(self, look, key, breaches, act, before=None, after=None):
        """Keep the first breaches of a cell, with the act, a still and the diff."""
        cell = self.results.setdefault((look, key), [])
        if not breaches or cell:
            return
        evidence = self.runs / f"{look}-{key.replace(':', '-').replace(' ', '_')}"
        evidence.mkdir(parents=True, exist_ok=True)
        if before and after:
            diff = {"before": before.states, "after": after.states}
            (evidence / "states.json").write_text(json.dumps(diff, indent=1))
        with contextlib.suppress(RuntimeError, OSError, KeyError):
            _, png = self.capture(self.outputs()[0]["name"], 1280)
            (evidence / "still.png").write_bytes(png)
        cell += [f"in act {act!r}: {b} (evidence {evidence})" for b in breaches]


def key(keys):
    """A cue: a shortcut such as Super+r."""
    return lambda s: s.inject([{"shortcut": {"keys": keys}}])


def text(value):
    """A cue: typed text; a newline presses Return."""
    return lambda s: s.inject([{"text": {"text": value}}])


def click(x, y, button=272):
    """A cue: a click at a point of the first output."""
    return lambda s: s.inject(
        [{"pointer_move": {"position": [x, y]}}, {"click": {"button": button}}]
    )


def program(name, *args, log=None):
    """A cue: start a program of the build; its window comes within 10 s."""

    def cue(s):
        count = len([n for n in s.core.call("scene.query", {"selector": "kind:surface"}) or []])
        s.spawn([str(s.bin / name), *args], log or f"{name}.log")
        s.until(
            lambda: len(s.core.call("scene.query", {"selector": "kind:surface"}) or []) > count,
            10,
            f"window of {name}",
        )

    return cue


def wait(seconds):
    """A cue: let time pass, for a look that animates without a `busy`."""
    return lambda _s: time.sleep(seconds)
