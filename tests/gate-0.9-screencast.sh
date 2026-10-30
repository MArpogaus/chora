#!/bin/bash
# Screen sharing (D37): with the portal files of dist/portal, a browser
# asks xdg-desktop-portal for a screencast, the portal picks
# xdg-desktop-portal-wlr for XDG_CURRENT_DESKTOP=chora, and its stream of
# an output appears in PipeWire. The chooser of the flavor (fuzzel) is
# swapped for `head -n 1`, which takes the first output of the list.
#
# Usage: tests/gate-0.9-screencast.sh   BIN: directory with the binaries
# Environment: LIBEXEC (default /usr/libexec) with xdg-desktop-portal and
# xdg-desktop-portal-wlr, PORTALS (default /usr/share/xdg-desktop-portal/
# portals) with wlr.portal.
set -u
. "$(dirname "$0")/gate-lib.sh"
libexec=${LIBEXEC:-/usr/libexec}
portals=${PORTALS:-/usr/share/xdg-desktop-portal/portals}
need dbus-run-session
need pipewire
test -x "$libexec/xdg-desktop-portal-wlr" || skip "xdg-desktop-portal-wlr is not installed here"
test -x "$libexec/xdg-desktop-portal" || skip "xdg-desktop-portal is not installed here"
python3 -c 'import gi' 2>/dev/null || skip "PyGObject is not installed here"
need_bins chora choractl
session_start 640x480

# The portal files of the tree in directories of their own; a private
# runtime dir keeps PipeWire away from the host. With
# XDG_DESKTOP_PORTAL_DIR, the portal reads its .conf from there too.
temp_config
work=$XDG_CONFIG_HOME
mkdir -p "$work/xdg-desktop-portal-wlr" "$work/portals" "$work/run"
chmod 700 "$work/run"
cp "$repo/dist/portal/chora-portals.conf" "$work/portals/"
sed 's/^chooser_cmd=.*/chooser_cmd=head -n 1/' "$repo/dist/portal/xdg-desktop-portal-wlr" \
    >"$work/xdg-desktop-portal-wlr/chora"
# A second backend that also implements ScreenCast and Screenshot and
# names chora in UseIn: a fallback without chora-portals.conf picks it,
# so the log check below proves that the portal reads the file. It needs
# no process: the portal never asks it for a screencast.
cp "$portals/wlr.portal" "$work/portals/"
printf '%s\n' '[portal]' 'DBusName=org.freedesktop.impl.portal.desktop.gtk' \
    'Interfaces=org.freedesktop.impl.portal.ScreenCast;org.freedesktop.impl.portal.Screenshot;' \
    'UseIn=chora' >"$work/portals/gtk.portal"
export XDG_DESKTOP_PORTAL_DIR=$work/portals \
    XDG_CURRENT_DESKTOP=chora PIPEWIRE_RUNTIME_DIR=$work/run LIBEXEC=$libexec

# shellcheck disable=SC2016 # expands in the inner shell
answer=$(timeout 60 dbus-run-session -- bash -c '
    pipewire >"$0/pipewire.log" 2>&1 &
    "$LIBEXEC/xdg-desktop-portal-wlr" -l DEBUG >"$0/xdpw.log" 2>&1 &
    sleep 1
    "$LIBEXEC/xdg-desktop-portal" -r -v >"$0/portal.log" 2>&1 &
    sleep 1
    python3 "$1"
    status=$?
    kill $(jobs -p) 2>/dev/null
    exit $status' "$work" "$repo/tests/portal-screencast.py" 2>&1)
status=$?
[ "$status" = 0 ] || fail "no screencast ($status): $answer
xdpw: $(tail -n 20 "$work/xdpw.log")
portal: $(tail -n 5 "$work/portal.log")
pipewire: $(tail -n 5 "$work/pipewire.log")"
for line in "Using portal configuration file '$work/portals/chora-portals.conf' for desktop 'chora'" \
    "Using wlr.portal for org.freedesktop.impl.portal.ScreenCast (interface specific config)"; do
    grep -qF "$line" "$work/portal.log" ||
        fail "the portal log has no line \"$line\": $(grep -i 'using' "$work/portal.log")"
done
ok "the portal read chora-portals.conf, picked wlr over gtk and gave the stream of an output as PipeWire $answer"
