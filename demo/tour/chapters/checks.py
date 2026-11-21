"""The checks that the chapters share (imports only: the rows of xp, tiling and handover pass them as
`expect=` or `verify=`). A check on a log file or on the rows of fuzzel judges what its act added: it
is made when the row is declared, which is before the act runs, and skips what the files held then."""

import dataclasses, os, shutil

from harness import Expect

LETTER = "/home/user/notes/letter.txt"


@dataclasses.dataclass
class Diff(Expect):
    """An `Expect` that reads both snapshots as well: `delta(before, after)` answers the mismatches
    (the read before the act is the state that the act changed)."""

    delta: object = None

    def check(self, before, after):
        return super().check(before, after) + (self.delta(before, after) if self.delta else [])


def nodes_of(after, *ids):
    """The ids that the scene has as visible nodes."""
    have = {n.get("id") for n in after.nodes if n.get("visible", True)}
    return [i for i in ids if i in have]


def shown(*ids):
    """A check: every id is a visible node."""
    return lambda after: [f"no node {i}" for i in ids if i not in nodes_of(after, i)]


def gone(*ids):
    """A check: no id is a visible node."""
    return lambda after: [f"node {i} is still there" for i in nodes_of(after, *ids)]


def volume_is(percent, state_file):
    """A check: the mixer of the session (the stand-in wpctl keeps `volume mute`) holds the level."""

    def check(after):
        got = round(float(open(state_file).read().split()[0]) * 100)
        return [] if got == percent else [f"the mixer is at {got} percent, expected {percent}"]

    return check


def letter_is(text, path=LETTER):
    """A check: the file that the stand-in editor writes holds the typed text."""

    def check(after):
        got = open(path).read().rstrip("\n")
        return [] if got == text else [f"letter.txt holds {got!r}, expected {text!r}"]

    return check


def both(*checks):
    """A check: the mismatches of every check."""
    return lambda after: [m for check in checks for m in check(after)]


def logged(path, needle):
    """A check: a line that the act added to the log file has the text (a call that the fake voice or
    fuzzel got)."""
    skip = len(open(path).read().splitlines()) if os.path.exists(path) else 0

    def check(after):
        lines = open(path).read().splitlines()[skip:] if os.path.exists(path) else []
        return [] if any(needle in line for line in lines) else [f"no new line with {needle!r} in {os.path.basename(path)}"]

    return check


def moved(look="xp"):
    """A delta: the rect of a window of the look differs from the read before (X2)."""
    rects = lambda s: [w.get("rect") for w in (s.states.get(look) or {}).get("windows", [])]
    return lambda before, after: [] if rects(before) != rects(after) else [f"no window of {look} moved: {rects(after)}"]


def notice_is_new(before, after):
    """A delta: chora-notify has a notice that the read before did not know (T6, I9)."""

    def ids(snap):
        block = snap.blocks.get("notify") or {}
        return {n.get("id") if isinstance(n, dict) else n for key in ("shown", "closed", "accepted", "waiting") for n in block.get(key) or []}

    return [] if ids(after) - ids(before) else ["no new notice in chora-notify"]


def fuzzel_log(directory, stub):
    """A `fuzzel` in front of the real one on the PATH of the session: it keeps the rows that a
    launcher or menu of the look gives it (`fuzzel.rows.N`) and runs the real fuzzel on them."""
    real = shutil.which("fuzzel")
    open(f"{stub}/fuzzel", "w").write(f"""#!/bin/sh
d={directory}; n=$(cat $d/fuzzel.n 2>/dev/null || echo 0); n=$((n+1)); echo $n >$d/fuzzel.n
case " $* " in *" --dmenu "*) tee $d/fuzzel.rows.$n | exec {real} "$@" ;; *) exec {real} "$@" ;; esac
""")
    os.chmod(f"{stub}/fuzzel", 0o755)


def calls_of(directory, skip=0):
    """The rows that each call of fuzzel got, from the call after `skip`."""
    names = sorted((n for n in os.listdir(directory) if n.startswith("fuzzel.rows.")), key=lambda n: int(n.rsplit(".", 1)[1]))
    return [open(os.path.join(directory, n)).read().splitlines() for n in names[skip:]]


def rows_have(directory, labels):
    """A check: a call of fuzzel that the act made got every label as a row (the power menu: Lock,
    Shut down, ...)."""
    skip = len(calls_of(directory))

    def check(after):
        calls = calls_of(directory, skip)
        return [] if any(set(labels) <= set(rows) for rows in calls) else [f"no menu with the rows {labels}: {[c[:6] for c in calls]}"]

    return check


def first_rows(directory, skip=0):
    """The first row that each call of the Run box got from xp, from the call after `skip`."""
    return [rows[:1] for rows in calls_of(directory, skip)]


def box_says(directory, text):
    """A check: a call that the act made listed `Nothing found for "text"` as its first row (the
    stand-in of fuzzel keeps what it gets)."""
    want = f'Nothing found for "{text}"'
    skip = len(calls_of(directory))
    return lambda after: [] if [want] in first_rows(directory, skip) else [f"the Run box never said {want!r}: {first_rows(directory, skip)}"]
