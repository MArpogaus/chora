"""The acts of xp. The door acts: the key list (Help and Support on its
keys page), the Start menu by its button and the dialog Turn Off Computer
by its key (tests/doors keys-key, keys-alias, power-key; the Start button
is the door of launcher-door and power-door)."""

from common import door_acts, notice_acts, window_acts
from harness import Expect, click, drag_title, key, program

HELP = "xp-dlg-help"
MENU = "xp-menu"
POWER = "xp-session"


def run(s):
    """A window opens and closes, then the doors open and shut."""
    window_acts(s)
    drag_acts(s)
    door_acts(s, HELP)
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
