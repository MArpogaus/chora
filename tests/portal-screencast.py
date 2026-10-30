"""Asks xdg-desktop-portal for a screencast of an output, as a browser does.

Prints the PipeWire node of the stream and exits 0, or prints the failed
step and exits 1. Needs PyGObject and a session bus with the portal.
"""

import sys

from gi.repository import Gio, GLib

PORTAL = "org.freedesktop.portal.Desktop"
PATH = "/org/freedesktop/portal/desktop"
bus = Gio.bus_get_sync(Gio.BusType.SESSION)
sender = bus.get_unique_name()[1:].replace(".", "_")
loop = GLib.MainLoop()


def request(method, signature, args, options):
    """Call a method of the ScreenCast portal and wait for its Response."""
    token = f"t{method}"
    options = {k: GLib.Variant(*v) for k, v in options.items()}
    options["handle_token"] = GLib.Variant("s", token)
    answer = {}

    def on_response(_bus, _sender, _path, _iface, _signal, params):
        answer["code"], answer["results"] = params.unpack()
        loop.quit()

    sub = bus.signal_subscribe(
        PORTAL,
        "org.freedesktop.portal.Request",
        "Response",
        f"{PATH}/request/{sender}/{token}",
        None,
        Gio.DBusSignalFlags.NO_MATCH_RULE,
        on_response,
    )
    call = GLib.Variant(signature, (*args, options))
    bus.call_sync(
        PORTAL, PATH, "org.freedesktop.portal.ScreenCast", method, call,
        None, Gio.DBusCallFlags.NONE, 10000, None,
    )  # fmt: skip
    GLib.timeout_add_seconds(20, loop.quit)
    loop.run()
    bus.signal_unsubscribe(sub)
    if answer.get("code") != 0:
        sys.exit(f"{method}: {answer or 'no response'}")
    return answer["results"]


created = request("CreateSession", "(a{sv})", (), {"session_handle_token": ("s", "s1")})
session = created["session_handle"]
# types 1: a monitor
request("SelectSources", "(oa{sv})", (session,), {"types": ("u", 1)})
streams = request("Start", "(osa{sv})", (session, ""), {}).get("streams")
if not streams:
    sys.exit(f"Start gave no stream: {streams}")
print(f"node {streams[0][0]} {streams[0][1]}")
