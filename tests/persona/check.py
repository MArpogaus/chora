#!/usr/bin/python3 -I
"""The checker of a persona session: the invariants of the acts and the
end-state contract of each task, read from the session and never shown to
the persona. It joins the session of tests/persona/session.sh as the client
persona-check (screen.capture and methods.call, no input).

Usage: PERSONA_ENV=FILE python3 -I tests/persona/check.py COMMAND
  start                  start the run: the first snapshot (round.sh runs it)
  begin                  take the snapshot a task is held against: the
                         windows open when the task starts; the driver runs
                         it before each task. `end` reads the newest begin,
                         or `start` when there is none
  after ACTION           run the invariants on what ACTION changed since the
                         last snapshot and append the breaches to
                         $PERSONA_DIR/breaches.jsonl; act.py runs it after
                         each action when PERSONA_CHECK=1. I4 does not run
                         (a persona declares no open) and every disconnect
                         counts as declared (I14 still reads panics and
                         failed effects)
  end [--look L] [--probe FILE] TASK...
                         the contract of each task against the snapshot of
                         `start`: T1..T7, X1, X2, L1..L3, M1, M2, C1..C3;
                         T4 takes its part as T4:volume|clock|screenshot|
                         lock|power. Prints `task T pass|FAIL: why` and the
                         breaches of the run outside tests/acts/known-red.toml;
                         exit status 1 when one is red.

A task passes when its contract and the run's breaches are clean; the
persona's own word (reached) is the third part and is not read here.
L is the look the task runs on (default: the first flavor); FILE is the
`--report` output of a chora-probe window (T5).
"""

import contextlib
import dataclasses
import json
import os
import pathlib
import re
import sys
import time
import traceback

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tests" / "acts"))
sys.path.insert(0, str(ROOT / "tests" / "acts" / "looks"))

import common  # noqa: E402
import harness  # noqa: E402
import invariants  # noqa: E402
from harness import Act, Expect, Session, Snapshot, wait_quiet  # noqa: E402

SKIPPED = {"I4"}
QUIET = 10
SHEETS = {
    "xp": "xp-dlg-help",
    "tiling": "tiling-keys",
    "mc": "mc-keys",
    "construct": "construct-help",
}
LETTERS = dict(
    zip(
        "qwertyuiopasdfghjklzxcvbnm",
        [16, 17, 18, 19, 20, 21, 22, 23, 24, 25]
        + [30, 31, 32, 33, 34, 35, 36, 37, 38]
        + [44, 45, 46, 47, 48, 49, 50],
        strict=True,
    )
)
CODES = LETTERS | {"Space": 57, "Return": 28, "Escape": 1, "F1": 59, "Print": 210, "Tab": 15, "/": 53}
POWER = r"(systemctl .*(suspend|poweroff|reboot|hibernate|halt|kexec)|loginctl terminate)"


def restore(raw):
    """A Snapshot from its JSON."""
    for key in ("toplevels", "wallpaper", "clients"):
        raw[key] = set(raw[key])
    return Snapshot(**raw)


def dump(snap):
    """The JSON of a Snapshot."""
    return json.dumps(dataclasses.asdict(snap), default=sorted)


class Run:
    """The files of the run in $PERSONA_DIR and the connection to the session."""

    def __init__(self, env_file):
        self.s = Session.attach(env_file)
        self.dir = self.s.work
        self.state_dir = self.dir / "check"
        self.state_dir.mkdir(exist_ok=True)
        self.calls = pathlib.Path(self.s.env["PERSONA_CALLS"])

    def snapshot(self, marks=None):
        """A snapshot once quiet; the logs since `marks` (path: size)."""
        self.s.log_marks = {pathlib.Path(k): v for k, v in (marks or {}).items()}
        return self.s.snapshot(wait_quiet(self.s.states, QUIET), self.s.watch.drain())

    def marks(self):
        """The end of every log now."""
        self.s.mark_logs()
        return {str(k): v for k, v in self.s.log_marks.items()}

    def load(self, name):
        """The saved snapshot `name` and its log marks, None when absent."""
        path = self.state_dir / f"{name}.json"
        if not path.exists():
            return None
        saved = json.loads(path.read_text())
        return restore(saved["snap"]), saved["marks"], saved["calls"]

    def save(self, name, snap, marks):
        """Keep a snapshot, the log marks and the length of the stand-in log."""
        calls = len(self.calls.read_text().splitlines())
        text = f'{{"snap": {dump(snap)}, "marks": {json.dumps(marks)}, "calls": {calls}}}'
        (self.state_dir / f"{name}.json").write_text(text)

    def observe(self, action, snap):
        """What the contracts read later: what was open, the helper, and the
        breaches of the key list while it was open."""
        look = self.s.label
        open_ids = sorted(invariants.open_ids(snap))
        seen = {
            "t": time.time(),
            "action": action,
            "open": open_ids,
            "helper": {k: (v or {}).get("helper") for k, v in snap.states.items()},
            "toplevels": sorted(snap.toplevels),
            "keysheet": None,
        }
        if SHEETS[look] in open_ids:
            seen["keysheet"] = common.keysheet_breaches(self.sheet_texts(look, snap))
        with open(self.state_dir / "history.jsonl", "a", encoding="utf-8") as f:
            f.write(json.dumps(seen) + "\n")

    def sheet_texts(self, look, snap):
        """The words the key list draws, as keysheet_acts reads them."""
        sheet = SHEETS[look]
        if look == "xp":
            runs = (snap.states.get(look) or {}).get("texts") or []
            return [r["text"] for r in runs if r["node"].startswith(sheet)]
        nodes = [n for n in snap.nodes if n.get("owner") == look and n.get("kind") == "text"]
        return [n.get("text") for n in nodes if n.get("visible") and n.get("text")]

    def history(self, since):
        """The observations since `since` (a time)."""
        path = self.state_dir / "history.jsonl"
        lines = path.read_text().splitlines() if path.exists() else []
        return [h for h in map(json.loads, lines) if h["t"] >= since]

    def breaches(self):
        """The recorded breaches, [(invariant, text)], outside known-red.toml."""
        path = self.dir / "breaches.jsonl"
        lines = path.read_text().splitlines() if path.exists() else []
        rows = {(r["look"], r["id"]) for r in harness.known_red()}
        found = [json.loads(line) for line in lines]
        return [b for b in found if (b["look"], b["invariant"]) not in rows]


def after(run, action):
    """Run the invariants on what an action changed; breaches.jsonl only."""
    saved = run.load("last")
    marks = saved[1] if saved else None
    try:
        now = run.snapshot(marks)
    except (harness.NotQuiet, RuntimeError, OSError) as err:
        stamp = {"time": round(time.time(), 3), "action": action, "look": run.s.label}
        line = stamp | {"invariant": "check", "breach": f"{type(err).__name__}: {err}"}
        with open(run.dir / "breaches.jsonl", "a", encoding="utf-8") as f:
            f.write(json.dumps(line) + "\n")
        return
    if saved is None:
        run.save("start", now, run.marks())
    else:
        before = saved[0]
        act = Act(action, disconnects=sorted(before.clients))
        lines = []
        for key, check in invariants.ALL.items():
            if key not in SKIPPED:
                lines += [{"invariant": key, "breach": text} for text in check(before, now, act)]
        if lines:
            evidence = run.dir / "evidence"
            evidence.mkdir(exist_ok=True)
            n = len(list(evidence.glob("*.png"))) + 1
            with contextlib.suppress(RuntimeError, OSError, KeyError):
                _, png = run.s.capture(run.s.outputs()[0]["name"], 1280)
                (evidence / f"{n:03d}-{action}.png").write_bytes(png)
            stamp = {"time": round(time.time(), 3), "action": action, "look": run.s.label}
            with open(run.dir / "breaches.jsonl", "a", encoding="utf-8") as f:
                f.writelines(json.dumps(stamp | line) + "\n" for line in lines)
    run.observe(action, now)
    run.save("last", now, run.marks())


def new_windows(before, after_):
    """The toplevels that came since `before`, as their nodes."""
    nodes = after_.by_id()
    return [nodes[sid] for sid in after_.toplevels - before.toplevels if sid in nodes]


@dataclasses.dataclass
class Ctx:
    """What a contract reads besides the two snapshots."""

    look: str
    hist: list
    calls: list
    probe: str | None
    keys: list
    entries: list

    def opened(self, sheet):
        """Whether `sheet` was open in some observation."""
        return any(sheet in h["open"] for h in self.hist)

    def called(self, pattern):
        """Whether a stand-in call line matches `pattern`; the polls of the
        looks (`wpctl get-volume`, `systemctl --user is-system-running`)
        are lines too, so a pattern names the action."""
        return any(re.match(pattern, line) for line in self.calls)


def verify_new_window(ctx, before):
    """A new toplevel of the look that has the focus."""

    def verify(after_):
        new = new_windows(before, after_)
        if not any(n.get("owner") == ctx.look and n.get("focused") for n in new):
            return [f"no new focused window of {ctx.look}; new: {[n['id'] for n in new]}"]
        return []

    return verify


def verify_sheet(ctx):
    """The key list was open, spoke the keysheet while open and is shut."""
    sheet = SHEETS[ctx.look]

    def verify(_after):
        if not ctx.opened(sheet):
            return [f"the key list {sheet} was never open"]
        said = [h["keysheet"] for h in ctx.hist if h["keysheet"] is not None]
        return [b for bad in said[-1:] for b in bad]

    return verify


def verify_helper(ctx):
    """The helper worked and is idle again, and an agent entry answers."""

    def verify(after_):
        words = [(h["helper"] or {}).get(ctx.look) for h in ctx.hist]
        out = []
        if "working" not in words:
            out.append(f"the helper never worked: {words}")
        if not any(e.get("role") == "agent" for e in ctx.entries):
            out.append("no agent entry answers in the transcript of the helper")
        if words[-1:] != ["idle"]:
            out.append(f"the helper ends {words[-1:]}, not idle")
        holder = (after_.states.get(ctx.look) or {}).get("focus", {}).get("holder")
        if holder != ctx.look:
            out.append(f"the focus is with {holder}, not back with {ctx.look}")
        return out

    return verify


def verify_system(ctx, part):
    """The stand-in of the part got its call; a power menu that was
    cancelled left nothing open and called no systemctl."""
    want = {
        "volume": r"wpctl set-(volume|mute)",
        "screenshot": r"grim ",
        "lock": r"(swaylock|loginctl lock-session)",
    }

    def verify(after_):
        if part == "clock":
            clock = [n["id"] for n in after_.nodes if "clock" in str(n.get("id"))]
            return [] if clock else ["no clock node in the scene"]
        if part == "power":
            return [f"systemctl was called: {line}" for line in ctx.calls if re.match(POWER, line)]
        return [] if ctx.called(want[part]) else [f"no call of {want[part]} in PERSONA_CALLS"]

    return verify


def verify_probe(ctx):
    """The probe window got no key of a Super chord the persona pressed."""

    def verify(after_):
        if not ctx.probe:
            return ["no --probe FILE: the report of chora-probe window --report"]
        if not pathlib.Path(ctx.probe).exists():
            return [f"no report {ctx.probe}: round.sh probe NAME starts the probe"]
        if not any((n.get("surface") or {}).get("app_id") == "chora-probe" for n in after_.nodes):
            return ["no chora-probe window in the scene: the probe died or never showed"]
        got = {
            line.split()[1]
            for line in pathlib.Path(ctx.probe).read_text(encoding="utf-8").splitlines()
            if line.startswith("key ")
        }
        stray = [c for c in ctx.keys if str(c) in got]
        return [f"the probe window got the key {c} of a Super chord" for c in stray]

    return verify


def verify_no_window_lost(ctx, before):
    """No window lost: every window that was open at the begin, or that came
    while the task ran, is still there. A window opened during the task is
    no breach."""
    seen = set(before.toplevels)
    for h in ctx.hist:
        seen |= set(h.get("toplevels") or [])

    def verify(after_):
        lost = sorted(seen - after_.toplevels)
        return [f"windows lost {lost}"] if lost else []

    return verify


def verify_same_windows(before):
    """No window lost: the same toplevels, each owned and shown."""

    def verify(after_):
        lost = sorted(before.toplevels - after_.toplevels)
        stray = sorted(after_.toplevels - before.toplevels)
        return [f"windows lost {lost}, new {stray}"] if lost or stray else []

    return verify


def verify_shown_focused(ctx):
    """Every window of the look is shown again and the look holds the focus."""

    def verify(after_):
        state = after_.states.get(ctx.look) or {}
        hidden = [w["surface"] for w in state.get("windows") or [] if not w.get("shown")]
        out = [f"hidden windows {hidden}"] if hidden else []
        if (state.get("focus") or {}).get("holder") != ctx.look:
            out.append(f"the focus is not with {ctx.look}")
        return out

    return verify


def verify_changed(before, what):
    """What `what(snapshot)` reads differs from the start."""
    return lambda after_: (
        [] if what(after_) != what(before) else [f"nothing changed: still {what(before)!r}"]
    )


def shown_scene(snap, look="mc"):
    """The scene that mc shows."""
    return [
        w.get("scene") for w in (snap.states.get(look) or {}).get("windows", []) if w.get("shown")
    ]


def notice_ids(snap):
    """The ids of the notices that the snapshot shows, closed or accepted."""
    block = snap.blocks.get("notify") or {}
    ids = set()
    for key in ("shown", "closed", "accepted"):
        for item in block.get(key) or []:
            ids.add(item.get("id") if isinstance(item, dict) else item)
    return ids


def verify_notice(before):
    """A notice that is new against `before`, or the words No helper."""

    def verify(after_):
        if any("No helper" in str(n) for n in after_.nodes):
            return []
        if notice_ids(after_) - notice_ids(before):
            return []
        return ["neither a new notice nor the words No helper"]

    return verify


def contracts(task, ctx, before):
    """The Expect object of a task; None for a task that does not exist."""
    name, _, part = task.partition(":")
    same = verify_same_windows(before)
    table = {
        "T1": Expect(open=[], verify=verify_new_window(ctx, before)),
        "T2": Expect(open=[], verify=verify_sheet(ctx)),
        "T3": Expect(verify=verify_helper(ctx)),
        "T4": Expect(open=[], verify=verify_system(ctx, part)),
        "T5": Expect(verify=verify_probe(ctx)),
        "T6": Expect(verify=verify_notice(before)),
        "T7": Expect(verify=same),
        "X1": Expect(focus=ctx.look, verify=verify_shown_focused(ctx)),
        "X2": Expect(
            open=[],
            verify=verify_changed(
                before,
                lambda a: [w.get("rect") for w in a.states.get(ctx.look, {}).get("windows", [])],
            ),
        ),
        "L1": Expect(gone=1, focus=ctx.look),
        "L2": Expect(focus=ctx.look),
        "L3": Expect(focus=ctx.look, verify=verify_shown_focused(ctx)),
        "M1": Expect(verify=verify_changed(before, shown_scene)),
        "M2": Expect(open=[], focus="mc"),
        "C1": Expect(cube={}),
        "C2": Expect(verify=verify_no_window_lost(ctx, before)),
        "C3": Expect(focus=ctx.look),
    }
    return table.get(name)


def parse_chords(actions):
    """The evdev codes of the last key of each Super chord in the actions."""
    codes = []
    for entry in actions:
        combo = (entry.get("args") or {}).get("combo", "")
        if entry.get("action") == "key" and combo.startswith("Super+"):
            code = CODES.get(combo.rsplit("+", 1)[-1])
            if code:
                codes.append(code)
    return codes


def baseline(run):
    """The snapshot a task is held against: the newest `begin`, else `start`."""
    return "begin" if run.load("begin") else "start"


def end(run, args):
    """Check the tasks against the snapshot of `start`."""
    look, probe, tasks = run.s.label, None, []
    it = iter(args)
    for a in it:
        if a == "--look":
            look = next(it)
        elif a == "--probe":
            probe = next(it)
        else:
            tasks.append(a)
    name = baseline(run)
    saved = run.load(name)
    if saved is None:
        sys.exit("check.py: no `start` yet")
    before, _marks, n_calls = saved
    since = (run.state_dir / f"{name}.json").stat().st_mtime
    now = run.snapshot()
    log = pathlib.Path(run.s.env["PERSONA_LOG"])
    actions = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
    voice = pathlib.Path(run.s.env["PERSONA_VOICE"]) / "entries.json"
    entries = json.loads(voice.read_text()) if voice.exists() else []
    ctx = Ctx(
        look,
        run.history(since),
        run.calls.read_text().splitlines()[n_calls:],
        probe,
        parse_chords([a for a in actions if a["time"] >= since]),
        entries,
    )
    bad = 0
    breaches = [b for b in run.breaches() if b["time"] >= since]
    for task in tasks:
        expect = contracts(task, ctx, before)
        if expect is None:
            sys.exit(f"check.py: no contract for {task}")
        mismatches = expect.check(before, now)
        bad += bool(mismatches)
        print(f"task {task} {'FAIL: ' + '; '.join(mismatches) if mismatches else 'pass'}")
    for b in breaches:
        print(f"breach {b['invariant']} in {b['action']}: {b['breach']}")
    print(f"breaches {len(breaches)}")
    return 1 if bad or breaches else 0


def main():
    """Dispatch the command."""
    path = os.environ.get("PERSONA_ENV")
    if not path or len(sys.argv) < 2:
        sys.exit(__doc__)
    command, args = sys.argv[1], sys.argv[2:]
    run = Run(path)
    try:
        if command in ("start", "begin"):
            now = run.snapshot()
            run.save(command, now, run.marks())
            run.save("last", now, run.marks())
            if command == "start":
                (run.state_dir / "begin.json").unlink(missing_ok=True)
            return 0
        if command == "after" and args:
            try:
                after(run, args[0])
            except Exception:  # noqa: BLE001
                with open(run.state_dir / "errors.log", "a", encoding="utf-8") as f:
                    f.write(traceback.format_exc())
            return 0
        if command == "end":
            return end(run, args)
        sys.exit(__doc__)
    finally:
        run.s.__exit__(None, None, None)


if __name__ == "__main__":
    sys.exit(main())
