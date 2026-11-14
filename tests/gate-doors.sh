#!/bin/bash
# The shared doors (rc 1): every look has the same keys and one visible
# door per action. One cell per look and action; the cells and what each
# asks are in tests/doors/lib.sh, and tests/doors/LOOK.sh answers them for
# a look in a headless session, with stand-ins for swaylock, loginctl,
# systemctl, wpctl, grim and notify-send. The gate prints the table
# action x look and fails on a cell that is not `pass`.
#
# Usage: tests/gate-doors.sh [LOOK...]   BIN: directory with the binaries
#   LOOK  xp, tiling, mc, construct (default all)
#   ONLY  a regular expression of cell names; the table is then not full
set -u
. "$(dirname "$0")/gate-lib.sh"
. "$(dirname "$0")/doors/lib.sh"
looks=${*:-xp tiling mc construct}
out=$(mktemp -d /tmp/gate-doors-out.XXXXXX)
tmp_dirs+=("$out")
trap gate_cleanup EXIT
for look in $looks; do
    [ -f "$repo/tests/doors/$look.sh" ] || {
        echo "no tests/doors/$look.sh" >"$out/$look.log"
        continue
    }
    bash "$repo/tests/doors/$look.sh" >"$out/$look.log" 2>&1
done
bad=0
# shellcheck disable=SC2086 # one word per look
printf '\n%-16s' cell
# shellcheck disable=SC2086
printf ' %-9s' $looks
printf '\n'
for id in $doors_cells; do
    [ -z "${ONLY:-}" ] || [[ $id =~ ^(${ONLY})$ ]] || continue
    printf '%-16s' "$id"
    for look in $looks; do
        got=$(sed -n "s/^cell $look $id //p" "$out/$look.log" | tail -n 1)
        [ "${got:-missing}" = pass ] || bad=$((bad + 1))
        printf ' %-9s' "${got:-missing}"
    done
    printf '\n'
done
if [ "$bad" != 0 ]; then
    for look in $looks; do
        echo "--- $look: last lines of its log"
        grep -v '^cell ' "$out/$look.log" | tail -n 8
    done
    fail "$bad cells are not pass"
fi
ok "every look has every key and door"
