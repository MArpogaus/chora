"""The acts of xp. The door acts: the key list (Help and Support on its
keys page), the Start menu by its button and the dialog Turn Off Computer
by its key (tests/doors keys-key, keys-alias, power-key; the Start button
is the door of launcher-door and power-door). The session has a `wpctl`
that keeps its volume and mute and a `lowriter`, the first editor that
Notepad tries, that shows its window when an act lets it (`STAND_INS`), so
that what one act sets the next act reads back and a start takes as long
as the act wants."""

import contextlib
import os
import signal

from common import (
    click_node,
    door_acts,
    helper_acts,
    keysheet_acts,
    keysheet_breaches,
    notice_acts,
    notify_up,
    ready,
    window_acts,
)
from harness import REPO, Expect, click, drag_title, key, program

HELP = "xp-dlg-help"
MENU = "xp-menu"
POWER = "xp-session"
CONTEXT = "xp-ctx"
SPEAKER = "xp-speaker-HEADLESS-1"

WPCTL = """#!/bin/sh
state="$FAKE_STATE"
[ -f "$state" ] || echo "0.50 0" >"$state"
read -r volume mute <"$state"
case "$1" in
get-volume)
    if [ "$mute" = 1 ]; then echo "Volume: $volume [MUTED]"; else echo "Volume: $volume"; fi ;;
set-mute)
    case "$3" in 1) mute=1 ;; 0) mute=0 ;; toggle) mute=$((1 - mute)) ;; esac
    echo "$volume $mute" >"$state" ;;
set-volume)
    for last; do :; done
    case "$last" in
    *%+) volume=$(awk -v v="$volume" -v p="${last%\\%+}" 'BEGIN{v+=p/100; if (v>1) v=1; printf "%.2f", v}') ;;
    *%-) volume=$(awk -v v="$volume" -v p="${last%\\%-}" 'BEGIN{v-=p/100; if (v<0) v=0; printf "%.2f", v}') ;;
    *%) volume=$(awk -v p="${last%\\%}" 'BEGIN{printf "%.2f", p/100}') ;;
    esac
    echo "$volume $mute" >"$state" ;;
esac
"""
EDITOR = """#!/bin/sh
until [ -e @WORK@/fake-state.go ]; do sleep 0.2; done
rm -f @WORK@/fake-state.go
echo $$ >@WORK@/editor.pid
exec chora-probe window 40 120 200
"""
STAND_INS = {"wpctl": WPCTL, "lowriter": EDITOR}


def run(s):
    """A window opens and closes, then the doors open and shut. The
    helper acts come before Turn Off Computer, which Escape does not shut
    yet (known red), so that their click on the card closes no dialog."""
    window_acts(s)
    drag_acts(s)
    door_acts(s, HELP)
    keysheet_acts(s, HELP, drawn_in_state=True)
    help_acts(s)
    helper_acts(s)
    desktop_acts(s)
    volume_acts(s)
    starting_acts(s)
    s.act(
        "the Start button opens the Start menu",
        [click(50, 785)],
        opens=[MENU],
        expect=Expect(open=[MENU]),
    )
    s.act("Escape shuts the Start menu", [key("Escape")], closes=[MENU], expect=Expect(open=[]))
    s.act(
        "Super+Shift+e opens Turn Off Computer",
        [key("Super+Shift+e")],
        opens=[POWER],
        expect=Expect(open=[POWER]),
    )
    s.act("Escape shuts Turn Off Computer", [key("Escape")], closes=[POWER], expect=Expect(open=[]))
    notice_acts(s)


def drag_acts(s):
    """The Start menu does not stay open through a drag of a window (sd-h2):
    `open` after the drag is `open` before, less the menu the drag shuts. A
    press on the title bar shut it before WM3 (this cell guards that); the
    give and the drag out of a face did not (the cells of wm/state.rs)."""
    s.act("a window opens for the drag", [program("chora-probe", "window", "40", "120", "200")])
    s.act(
        "the Start button opens the Start menu before a drag",
        [click(50, 785)],
        opens=[MENU],
        expect=Expect(open=[MENU]),
    )
    s.act(
        "a drag of a window shuts the Start menu",
        [drag_title("xp", 120, 60)],
        closes=[MENU],
        expect=Expect(open=[]),
    )
    s.act("the dragged window closes", [lambda s: s.procs.pop().terminate()])


def windows_are(minimized):
    """A check of the windows of xp: each one is away, or none is."""

    def check(after):
        windows = after.states["xp"]["windows"]
        away = [w["surface"] for w in windows if bool(w.get("minimized")) != minimized]
        return (
            [f"windows not {'away' if minimized else 'back'}: {away}"]
            if away or not windows
            else []
        )

    return check


def desktop_acts(s):
    """Show the Desktop toggles (xp-31): Super+d puts the windows away,
    Super+d again brings the same ones back, and the task button of a
    window that is away brings it back."""
    ready(s)
    s.act(
        "a window opens for Show Desktop",
        [program("chora-probe", "window", "40", "120", "200")],
        expect=Expect(new_window={"app_id": "chora-probe"}),
    )

    def task_button(s):
        surface = s.state("xp")["windows"][-1]["surface"]
        click_node(f"xp-task-{surface}-HEADLESS-1")(s)

    s.act(
        "Super+d puts the windows away", [key("Super+d")], expect=Expect(verify=windows_are(True))
    )
    s.act(
        "a second Super+d brings them back",
        [key("Super+d")],
        expect=Expect(verify=windows_are(False)),
    )
    s.act(
        "Super+d puts the windows away again",
        [key("Super+d")],
        expect=Expect(verify=windows_are(True)),
    )
    s.act(
        "the task button brings the window back",
        [task_button],
        expect=Expect(verify=windows_are(False)),
    )
    s.act("the window of Show Desktop closes", [lambda s: s.procs.pop().terminate()])


def mixer(s):
    """The volume and the mute that the stand-in of wpctl holds."""
    with open(s.work / "fake-state") as f:
        volume, mute = f.read().split()
    return round(float(volume) * 100), mute == "1"


def tooltip_says(want):
    """A check: the tooltip on screen has these words."""

    def check(after):
        text = ((after.by_id().get("xp-tooltip") or {}).get("meta") or {}).get("text")
        return [] if text == want else [f"the tooltip says {text!r}, expected {want!r}"]

    return check


def mixer_is(percent, muted):
    """A check: the mixer holds a level and a mute."""

    def check(after):
        got = mixer(after_session[0])
        return [] if got == (percent, muted) else [f"the mixer is {got}"]

    return check


def mixer_settles(percent, muted):
    """A cue: the mixer takes a change in the background; wait for it."""

    def cue(s):
        s.until(lambda: mixer(s) == (percent, muted), 6, f"the mixer at {percent} {muted}")

    return cue


after_session = [None]


def hover(want):
    """A cue: the pointer rests on the speaker until its tooltip says what
    is wanted, or 8 s pass: the speaker reads the mixer every few seconds."""

    def says(s):
        node = s.core.call("scene.query", {"selector": "id:xp-tooltip"})
        return bool(node) and (node[0].get("meta") or {}).get("text") == want

    def cue(s):
        x, y = speaker_middle(s)
        s.inject(
            [{"pointer_move": {"position": [640, 300]}}, {"pointer_move": {"position": [x, y]}}]
        )
        with contextlib.suppress(RuntimeError):
            s.until(lambda: says(s), 8, f"the tooltip {want!r}")

    return cue


def speaker_middle(s):
    """The middle of the speaker in the tray."""
    x, y, w, h = next(
        iter(s.core.call("scene.query", {"selector": f"id:{SPEAKER}"})[0]["rects"].values())
    )
    return x + w / 2, y + h / 2


def volume_acts(s):
    """The volume by the mouse (xp-26, xp-32): a click on the speaker opens
    the volume popup, whose rows set the level and mute (rows 0 to 3: 100,
    75, 50, 25 percent, row 5: Mute); a level that a key set shows in the
    tooltip of the speaker."""
    ready(s)
    after_session[0] = s
    s.act(
        "the volume key raises the level",
        [key("XF86AudioRaiseVolume"), mixer_settles(55, False)],
        expect=Expect(verify=mixer_is(55, False)),
    )
    s.act(
        "the tooltip of the speaker says the level that the key set",
        [hover("Volume 55%")],
        expect=Expect(verify=tooltip_says("Volume 55%")),
    )
    s.act(
        "a click on the speaker opens the volume popup",
        [click_node(SPEAKER)],
        opens=[CONTEXT],
        expect=Expect(open=[CONTEXT]),
    )
    s.act(
        "a click on 75 percent sets the level and shuts the popup",
        [click_node("xp-ctx-0-1"), mixer_settles(75, False)],
        closes=[CONTEXT],
        expect=Expect(open=[], verify=mixer_is(75, False)),
    )
    s.act(
        "the tooltip of the speaker says the level that the popup set",
        [hover("Volume 75%")],
        expect=Expect(verify=tooltip_says("Volume 75%")),
    )
    s.act(
        "the speaker opens the volume popup again",
        [click_node(SPEAKER)],
        opens=[CONTEXT],
        expect=Expect(open=[CONTEXT]),
    )
    s.act(
        "Mute of the popup mutes",
        [click_node("xp-ctx-0-5"), mixer_settles(75, True)],
        closes=[CONTEXT],
        expect=Expect(open=[], verify=mixer_is(75, True)),
    )
    s.act(
        "the speaker opens the volume popup for Mute",
        [click_node(SPEAKER)],
        opens=[CONTEXT],
        expect=Expect(open=[CONTEXT]),
    )
    s.act(
        "Mute of the popup unmutes",
        [click_node("xp-ctx-0-5"), mixer_settles(75, False)],
        closes=[CONTEXT],
        expect=Expect(open=[], verify=mixer_is(75, False)),
    )
    s.act(
        "the speaker opens the volume popup to shut it",
        [click_node(SPEAKER)],
        opens=[CONTEXT],
        expect=Expect(open=[CONTEXT]),
    )
    s.act(
        "Escape shuts the volume popup", [key("Escape")], closes=[CONTEXT], expect=Expect(open=[])
    )


def item_node(s, text):
    """The node of the Start menu item that says `text`."""
    runs = (s.state("xp") or {}).get("texts") or []
    return next(r["node"] for r in runs if r["text"] == text and r["node"].startswith("xp-item-"))


def click_item(text):
    """A cue: click the item of the open Start menu that says `text`."""
    return lambda s: click_node(item_node(s, text))(s)


def help_texts(after):
    """The texts and their boxes of the right pane of Help, and the size
    of the pane."""
    runs = (after.states["xp"] or {}).get("texts") or []
    body = [r for r in runs if r["node"] == f"{HELP}-content-body"]
    node = after.by_id().get(f"{HELP}-content-body") or {}
    size = next(iter((node.get("rects") or {}).values()), [0, 0, 0, 0])
    return body, size


def pane_says(line):
    """A check: the pane of Help says `line`."""

    def check(after):
        body, _ = help_texts(after)
        return [] if any(r["text"] == line for r in body) else [f"the pane of Help lacks {line!r}"]

    return check


def key_table_whole(after):
    """A check: the key list of Help in its pane speaks the keysheet and
    every row of it ends inside the pane."""
    body, [_, _, _, height] = help_texts(after)
    texts = [r["text"] for r in body if r["rect"][0] >= 200]
    breaches = keysheet_breaches(texts)
    low = [r["text"] for r in body if r["rect"][1] + r["rect"][3] > height]
    return breaches + [f"the row {t!r} ends below the pane ({height})" for t in low]


def help_acts(s):
    """Help and Support by the Start menu (xp-11): it opens on its first
    topic with the right pane filled, the topic of the keys lists every row
    of the keysheet inside the window at 1280x800, and Escape shuts it."""
    ready(s)
    s.act(
        "the Start button opens the Start menu for Help",
        [click(50, 785)],
        opens=[MENU],
        expect=Expect(open=[MENU]),
    )
    s.act(
        "Help and Support of the Start menu opens Help on a filled pane",
        [click_item("Help and Support")],
        opens=[HELP],
        closes=[MENU],
        expect=Expect(
            open=[HELP],
            verify=pane_says("Click Start in the bottom left corner and click a program."),
        ),
    )
    s.act(
        "the topic Keyboard shortcuts lists every key inside the window",
        [click_node(f"{HELP}-content-c-topic-4")],
        expect=Expect(verify=key_table_whole),
    )
    s.act("Escape shuts Help", [key("Escape")], closes=[HELP], expect=Expect(open=[]))


NOTIFY_ASSETS = REPO / "flavors/xp/assets/notify"
STARTING = "Starting Notepad"


def starting_shown(after):
    """Whether chora-notify shows the balloon that Notepad starts."""
    shown = (after.blocks or {}).get("notify", {}).get("shown", [])
    return any(n.get("summary") == STARTING for n in shown)


def balloon_is(want):
    """A check: the balloon of the start of Notepad shows, or does not."""

    def check(after):
        return (
            [] if starting_shown(after) == want else [f"the balloon {STARTING!r} shows: {not want}"]
        )

    return check


def balloon_comes(want):
    """A cue: wait up to 12 s until the balloon shows (`want`) or goes."""

    def cue(s):
        def now():
            shown = (s.state("notify") or {}).get("shown", [])
            return any(n.get("summary") == STARTING for n in shown) == want

        with contextlib.suppress(RuntimeError):
            s.until(now, 12, f"the balloon {'there' if want else 'gone'}")

    return cue


def let_notepad_show(s):
    """A cue: the stand-in of the editor shows its window."""
    (s.work / "fake-state.go").touch()
    s.until(
        lambda: any(
            n.get("surface", {}).get("app_id") == "chora-probe"
            for n in s.core.call("scene.query", {"selector": "kind:surface"}) or []
        ),
        10,
        "the window of Notepad",
    )


def end_notepad(s):
    """A cue: the stand-in of the editor ends with its window."""
    os.kill(int((s.work / "editor.pid").read_text()), signal.SIGTERM)


def start_notepad(s, why):
    """The Start menu starts Notepad, whose window waits for the act."""
    s.act(
        f"the Start button opens the Start menu to start Notepad {why}",
        [click(50, 785)],
        opens=[MENU],
        expect=Expect(open=[MENU]),
    )
    s.act(
        f"a click on Notepad starts it {why}",
        [click_item("Notepad")],
        closes=[MENU],
        expect=Expect(open=[]),
    )
    s.act(
        f"the balloon says that Notepad starts {why}",
        [balloon_comes(True)],
        expect=Expect(verify=balloon_is(True)),
    )


def starting_acts(s):
    """The balloon that a program starts (xp-35) never covers a menu or a
    popup (I5, which every act holds) and goes when the window of the
    program shows, with or without the activation token."""
    notify_up(s, NOTIFY_ASSETS)
    start_notepad(s, "under the volume popup")
    s.act(
        "the volume popup opens while the balloon shows",
        [click_node(SPEAKER)],
        opens=[CONTEXT],
        expect=Expect(open=[CONTEXT]),
    )
    s.act(
        "Escape shuts the volume popup over the balloon",
        [key("Escape")],
        closes=[CONTEXT],
        expect=Expect(open=[]),
    )
    s.act(
        "the window of Notepad shows",
        [let_notepad_show, balloon_comes(False)],
        expect=Expect(new_window={"app_id": "chora-probe"}, verify=balloon_is(False)),
    )
    s.act("Notepad ends", [end_notepad], expect=Expect(gone=1))
    start_notepad(s, "to see it go")
    s.act(
        "the balloon goes when the window of Notepad shows",
        [let_notepad_show, balloon_comes(False)],
        expect=Expect(new_window={"app_id": "chora-probe"}, verify=balloon_is(False)),
    )
    s.act("the second Notepad ends", [end_notepad], expect=Expect(gone=1))
