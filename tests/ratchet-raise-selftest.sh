#!/bin/bash
# Self-test of tests/ratchet-raise.sh, the commit-msg hook: a staged change
# that raises a count in tools/lint-ratchet.list, tools/size-ratchet.list or
# tools/sleep-ratchet.list needs a `ratchet-raise: <reason>` line in the message; a lower count or an
# unchanged list needs nothing.
#
# Usage: tests/ratchet-raise-selftest.sh
set -u
repo=$(cd "$(dirname "$0")/.." && pwd)
hook=$repo/tests/ratchet-raise.sh
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

case_run() {
    local label=$1 want=$2 old=$3 new=$4 msg=$5 list=${6:-tools/lint-ratchet.list}
    rm -rf "${work:?}/r"
    mkdir -p "$work/r/tools"
    (
        cd "$work/r" || exit 1
        git init -q
        printf '# c\nallow %s\n' "$old" >"$list"
        git add tools
        git -c user.name=t -c user.email=t@e commit -q -m base
        printf '# c\nallow %s\n' "$new" >"$list"
        git add tools
        printf '%b' "$msg" >"$work/msg"
        bash "$hook" "$work/msg" >/dev/null 2>&1
    )
    local status=$? got=pass
    [ "$status" -ne 0 ] && got=fail
    if [ "$got" = "$want" ]; then
        echo "ok   $label ($want)"
    else
        echo "FAIL $label (wanted $want, exit $status)"
        failures=$((failures + 1))
    fi
}

case_run "raised count, no line" fail 5 6 'chore: x\n'
case_run "raised count with the line" pass 5 6 'chore: x\n\nratchet-raise: new allow in a test\n'
case_run "lower count" pass 5 4 'chore: x\n'
case_run "same count" pass 5 5 'chore: x\n'
for sleep_list in tools/size-ratchet.list tools/sleep-ratchet.list; do
    case_run "$sleep_list raised, no line" fail 5 6 'chore: x\n' "$sleep_list"
    case_run "$sleep_list raised with the line" pass 5 6 'chore: x\n\nratchet-raise: why\n' "$sleep_list"
    case_run "$sleep_list lower" pass 5 4 'chore: x\n' "$sleep_list"
done
[ "$failures" -eq 0 ]
