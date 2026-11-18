"""The first invariants of the act harness (design 3.3 of the arch plan).

Each invariant is a function of the snapshot before an act, the snapshot
after it and the act, and answers the breaches as sentences; an empty
list is a pass. They read only the snapshots (harness.Snapshot), so the
selftest tests/test_acts.py plants a breach without a session.

I1   every toplevel is attached, owned by one look and in that look's
     `windows`; a `view` is not an owner
I3   once quiet, the core focus is a shown window of the one look that
     says it holds the focus
I4   `open` after the act is `open` before plus what the act declared
I7   the work area of an output whose look draws the wallpaper shows
     more than one colour
I8   the helper is not `working` after its answer event; one helper
     surface per output
I14  during the act: no `effect.failed`, no panic line, no disconnect
     the act did not declare
"""

import re

PANIC = re.compile(r"panicked at|^thread '.*' panicked|RUST_BACKTRACE=")
HELPER_IDS = {"hud-chat"}
HELPER_APP_IDS = {"chora-chat"}


def looks_with_state(snap):
    """The looks of the snapshot that answered `state.get`."""
    return {look: state for look, state in snap.states.items() if state is not None}


def windows_of(state):
    """The surfaces of the `windows` of a state, by id."""
    return {w.get("surface"): w for w in state.get("windows") or [] if w.get("surface")}


def i1(_before, after, _act):
    """Every toplevel is attached, owned by one look and in its windows."""
    out = [f"{look} answers no state.get" for look, s in after.states.items() if s is None]
    states = looks_with_state(after)
    if out:
        return out
    owned = {look: windows_of(state) for look, state in states.items()}
    for look, state in states.items():
        for view in state.get("views") or []:
            if view.get("surface") in owned[look]:
                out.append(f"{look} lists {view.get('surface')} as a view and as its window")
    nodes = after.by_id()
    for sid in sorted(after.toplevels):
        node = nodes.get(sid)
        if node is None or node.get("parent") is None:
            out.append(f"the toplevel {sid} is detached")
            continue
        owner = node.get("owner")
        holders = sorted(look for look, windows in owned.items() if sid in windows)
        if owner not in states:
            out.append(f"the toplevel {sid} is owned by {owner}, which is no look")
        elif holders != [owner]:
            out.append(f"the toplevel {sid} of {owner} is in the windows of {holders or 'no look'}")
    return out


def i3(_before, after, _act):
    """The core focus is a shown window of the one look that holds it."""
    states = looks_with_state(after)
    claims = sorted(
        look for look, s in states.items() if (s.get("focus") or {}).get("holder") == look
    )
    if len(claims) > 1:
        return [f"{' and '.join(claims)} each say they hold the focus"]
    claimer = claims[0] if claims else None
    focused = [n for n in after.nodes if n.get("focused")]
    surfaces = [n for n in focused if n.get("kind") == "surface"]
    nodes = [n for n in focused if n.get("kind") != "surface"]
    target = nodes[0] if nodes else surfaces[0] if surfaces else None
    if target is None:
        named = claimer and (states[claimer].get("focus") or {}).get("surface")
        return [f"{claimer} says it holds {named}, the core focuses nothing"] if named else []
    owner = target.get("owner")
    if claimer is None:
        return [f"the core focuses {target['id']} of {owner}, no look says it holds the focus"]
    if owner != claimer:
        return [f"the core focuses {target['id']} of {owner}, {claimer} says it holds the focus"]
    if nodes:
        return []
    sid = target["id"]
    window = windows_of(states[claimer]).get(sid)
    if window is None or not window.get("shown"):
        return [f"the focused {sid} is no shown window of {claimer}"]
    named = (states[claimer].get("focus") or {}).get("surface")
    if named not in (None, sid):
        return [f"{claimer} says it focuses {named}, the core focuses {sid}"]
    return []


def open_ids(snap):
    """The ids of what is open in all looks."""
    return {o.get("id") for s in looks_with_state(snap).values() for o in s.get("open") or []}


def i4(before, after, act):
    """What is open after the act is what was open plus the declared."""
    was, now = open_ids(before), open_ids(after)
    opens, closes = set(act.opens), set(act.closes)
    out = [f"{i} opened, the act did not declare it" for i in sorted(now - was - opens)]
    out += [f"{i} was declared open and is not" for i in sorted(opens - now)]
    out += [f"{i} closed, the act did not declare it" for i in sorted(was - now - closes)]
    return out


def i7(_before, after, _act):
    """The work area of a wallpaper output has more than one colour."""
    out = []
    for output in sorted(after.wallpaper):
        count = after.colours.get(output)
        if count is None:
            out.append(f"no capture of the work area of {output}")
        elif count <= 1:
            out.append(f"the work area of {output} shows one colour")
    return out


def helper_surfaces(node):
    """Whether a node is a surface of a helper: the chat of the HUD host,
    a node a look marks with meta `helper: surface`, or a chat window."""
    surface = node.get("surface") or {}
    return (
        node.get("id") in HELPER_IDS
        or (node.get("meta") or {}).get("helper") == "surface"
        or surface.get("app_id") in HELPER_APP_IDS
    )


def i8(_before, after, _act):
    """The helper is not working after its answer; one helper surface per output."""
    out = []
    said = [
        (e.get("data") or {}).get("state") for e in after.events if e.get("event") == "voice.state"
    ]
    if said and said[-1] != "working":
        out += [
            f"{look} says the helper works after its answer ({said[-1]})"
            for look, s in looks_with_state(after).items()
            if s.get("helper") == "working"
        ]
    per_output = {}
    for node in after.nodes:
        if node.get("visible") is False or not helper_surfaces(node):
            continue
        for output in node.get("rects") or {}:
            per_output.setdefault(output, []).append(node["id"])
    out += [
        f"{output} shows {len(ids)} helper surfaces: {', '.join(ids)}"
        for output, ids in sorted(per_output.items())
        if len(ids) > 1
    ]
    return out


def i14(before, after, act):
    """No failed effect, no panic line, no undeclared disconnect."""
    out = [
        f"effect.failed: {str(e.get('error'))[:200]}"
        for e in after.events
        if e.get("event") == "effect.failed"
    ]
    for name, text in sorted(after.logs.items()):
        lines = [line for line in text.splitlines() if PANIC.search(line)]
        out += [f"{name} log: {line[:200]}" for line in lines[:3]]
    gone = before.clients - after.clients - set(act.disconnects)
    out += [f"{client} disconnected, the act did not declare it" for client in sorted(gone)]
    return out


ALL = {"I1": i1, "I3": i3, "I4": i4, "I7": i7, "I8": i8, "I14": i14}
