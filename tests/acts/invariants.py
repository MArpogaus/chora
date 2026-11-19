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
I9   a notice the act sent is in `shown` of chora-notify and its card lies
     inside the output, clear of the helper surfaces; one the act
     dismissed is gone
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


def missing_keys(snap, *keys):
    """A breach for each look that answers `state.get` without a key."""
    return [
        f"{look} missing key {key}: its state.get does not answer it"
        for look, state in sorted(looks_with_state(snap).items())
        for key in keys
        if key not in state
    ]


def i1(_before, after, _act):
    """Every toplevel is attached, owned by one look and in its windows."""
    out = [f"{look} answers no state.get" for look, s in after.states.items() if s is None]
    states = looks_with_state(after)
    out += missing_keys(after, "windows")
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
    if gaps := missing_keys(after, "focus"):
        return gaps
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
    if gaps := missing_keys(after, "open"):
        return gaps
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
    states = looks_with_state(after)
    if said and states and not any("helper" in s for s in states.values()):
        out += [
            f"{look} missing key helper: the helper answered and no look reports it"
            for look in sorted(states)[:1]
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
        if "rects" not in node:
            out.append(f"the helper surface {node.get('id')} missing key rects")
        for output in node.get("rects") or {}:
            per_output.setdefault(output, []).append(node["id"])
    out += [
        f"{output} shows {len(ids)} helper surfaces: {', '.join(ids)}"
        for output, ids in sorted(per_output.items())
        if len(ids) > 1
    ]
    return out


def inside(rect, size):
    """Whether a rectangle x, y, w, h lies inside an output of `size`."""
    x, y, w, h = rect
    return x >= 0 and y >= 0 and w > 0 and h > 0 and x + w <= size[0] and y + h <= size[1]


def overlap(a, b):
    """Whether two rectangles x, y, w, h share area."""
    return a[0] < b[0] + b[2] and b[0] < a[0] + a[2] and a[1] < b[1] + b[3] and b[1] < a[1] + a[3]


def card_boxes(snap, root):
    """The box of the card on each output: the union of the rectangles of
    the node `root` and of its descendants; None without the node."""
    nodes = snap.by_id()
    if root not in nodes:
        return None
    boxes = {}
    pending = [root]
    while pending:
        node = nodes.get(pending.pop(), {})
        pending += node.get("children") or []
        for output, (x, y, w, h) in (node.get("rects") or {}).items():
            x0, y0, x1, y1 = boxes.get(output, (x, y, x + w, y + h))
            boxes[output] = (min(x0, x), min(y0, y), max(x1, x + w), max(y1, y + h))
    return {o: [x0, y0, x1 - x0, y1 - y0] for o, (x0, y0, x1, y1) in boxes.items()}


def i9(_before, after, act):
    """The notices of the act show in a card inside the output and clear
    of the helper; the dismissed ones are gone."""
    if not act.notices and not act.dismissed:
        return []
    if after.notify is None:
        return ["the act sent a notice and no chora-notify answers state.get"]
    shown = {n.get("summary") for n in after.notify.get("shown") or []}
    out = [
        f"the notice {s!r} is not in the shown of chora-notify"
        for s in act.notices
        if s not in shown
    ]
    out += [
        f"the notice {s!r} is still shown after its dismissal" for s in act.dismissed if s in shown
    ]
    if not act.notices or out:
        return out
    boxes = card_boxes(after, after.notify.get("card"))
    if boxes is None:
        return ["chora-notify shows the notice and draws no card"]
    sizes = {o.get("name"): o.get("size") for o in after.outputs}
    helpers = [n for n in after.nodes if n.get("visible") is not False and helper_surfaces(n)]
    for output, rect in sorted(boxes.items()):
        if not inside(rect, sizes.get(output) or [0, 0]):
            out.append(f"the card lies outside {output}: {rect}")
        out += [
            f"the card {rect} lies over {h.get('id')} {h['rects'][output]}"
            for h in helpers
            if output in (h.get("rects") or {}) and overlap(rect, h["rects"][output])
        ]
    if not boxes:
        out.append("the card of the notice has no box on any output")
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


ALL = {"I1": i1, "I3": i3, "I4": i4, "I7": i7, "I8": i8, "I9": i9, "I14": i14}
