#!/bin/bash
# Gate check 5 of 0.7: the Quadlet unit dist/quadlet/chora-agent@.container
# connects through its client directory after a restart of chora, with no
# change to the unit. This needs podman, a running user systemd, an
# installed chora@.service and the image localhost/chora-agent:latest
# with a choractl inside. None of it exists in this sandbox, so the
# answer here is SKIP.
#
# Usage: tests/gate-0.7-5.sh   INSTANCE: the chora flavor, main by default
set -u
. "$(dirname "$0")/gate-lib.sh"
need podman
systemctl --user is-system-running >/dev/null 2>&1 ||
    skip "no answering user systemd; Quadlet units need one"
test -f "$HOME/.config/containers/systemd/chora-agent@.container" ||
    skip "the Quadlet example is not installed under ~/.config/containers/systemd"
test -f "$HOME/.config/systemd/user/chora@.service" ||
    skip "chora@.service is not installed for the user"
podman image exists localhost/chora-agent:latest ||
    skip "no image localhost/chora-agent:latest; build it with a choractl inside"
# On a host with all of it: start chora, make the client socket of the
# agent, start the unit, restart chora, run choractl clients up again,
# and let the untouched unit connect once more.
instance=${INSTANCE:-main}
systemctl --user start "chora@$instance" || fail "chora@$instance did not start"
echo 'agent: {grants: [screen.capture]}' | "$bin/choractl" clients up - >/dev/null ||
    fail "the client socket of agent"
systemctl --user start "chora-agent@$instance" || fail "the unit did not start"
systemctl --user restart "chora@$instance" || fail "chora did not restart"
sleep 3
echo 'agent: {grants: [screen.capture]}' | "$bin/choractl" clients up - >/dev/null ||
    fail "clients up after the restart"
systemctl --user is-active --quiet "chora-agent@$instance" ||
    fail "the untouched unit stayed inactive"
ok "the Quadlet unit connects through its client directory after a restart"
