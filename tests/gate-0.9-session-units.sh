#!/bin/bash
# Part of gate check 8 of 0.9: the units of the session and of every
# flavor pass `systemd-analyze --user verify`, and the session entries
# pass `desktop-file-validate`. The units name their programs in /usr; a
# copy of each unit takes them from this tree instead, so the check needs
# no installed package. The programs of other packages (waybar, mako,
# swaybg, swayidle, kanshi) must be installed.
#
# Usage: tests/gate-0.9-session-units.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need systemd-analyze
need desktop-file-validate
need_bins chora chora-tiling chora-xp

desktop-file-validate "$repo"/dist/session/*.desktop || fail "a session entry is not valid"
ok "the session entries are valid"

units=$(mktemp -d /tmp/gate-units.XXXXXX)
trap 'rm -rf "$units"' EXIT
# A template is verified as one instance.
cp "$repo/dist/session/chora@.service" "$units/chora@tiling.service"
cp "$repo"/dist/session/*.target "$repo"/flavors/*/*.service "$repo"/flavors/*/*.target "$units/"
sed -i -e "s|/usr/bin/chora|$bin/chora|g" \
    -e "s|/usr/libexec/chora/chora-xwayland|$repo/dist/session/chora-xwayland|" "$units"/*
# The flavor units find each other in the copy; the trailing colon keeps
# the units of systemd and of the other packages. verify only warns about
# an unknown key, so any message fails.
log=$(SYSTEMD_UNIT_PATH=$units: XDG_RUNTIME_DIR=${XDG_RUNTIME_DIR:-$units} \
    systemd-analyze --user verify "$units"/* 2>&1) && [ -z "$log" ] ||
    fail "a unit is not valid: $log"
ok "the units of the session and of the flavors are valid"
