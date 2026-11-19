"""The acts of xp. The door acts: the key list (Help and Support on its
keys page), the Start menu by its button and the dialog Turn Off Computer
by its key (tests/doors keys-key, keys-alias, power-key; the Start button
is the door of launcher-door and power-door)."""

from common import door_acts, keysheet_acts, window_acts
from harness import Expect, click, key

HELP = "xp-dlg-help"
MENU = "xp-menu"
POWER = "xp-session"


def run(s):
    """A window opens and closes, then the doors open and shut."""
    window_acts(s)
    door_acts(s, HELP)
    keysheet_acts(s, HELP, drawn_in_state=True)
    s.act("the Start button opens the Start menu", [click(50, 785)], opens=[MENU],
          expect=Expect(open=[MENU]))
    s.act("Escape shuts the Start menu", [key("Escape")], closes=[MENU], expect=Expect(open=[]))
    s.act("Super+Shift+e opens Turn Off Computer", [key("Super+Shift+e")], opens=[POWER],
          expect=Expect(open=[POWER]))
    s.act("Escape shuts Turn Off Computer", [key("Escape")], closes=[POWER],
          expect=Expect(open=[]))
