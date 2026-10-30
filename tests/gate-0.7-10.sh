#!/bin/bash
# Gate check 10 of 0.7: the battle workflows drive the session with
# choractl wait, focus, the rectangles of queries and plugin methods,
# and without pixel clicks. The guard greps the workflows; W3 is exempt
# because dragging pointers is its subject. The run replays the two
# window workflows W1 and W2 headless and fails on new findings. The
# full round W1 to W8 stays with the battle driver.
#
# Usage: tests/gate-0.7-10.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl chora-tiling chora-xp chora-probe; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
cd "$repo" || fail "no repo"
findings=tests/battle/findings.md
hits=$(grep -lE 'pointer_move|\bclick\b' tests/battle/w*.sh |
    grep -v 'w3\.sh' || :)
test -z "$hits" || fail "these workflows still click at pixels: $hits"
touch "$findings" # ignored by git, so CI starts without it
before=$(grep -c '^## ' "$findings" || :)
for w in 1 2; do
    run=/tmp/gate10-w$w.md
    RUN=$run timeout 1200 bash "tests/battle/w$w.sh" >"$run.out" 2>&1 ||
        fail "w$w ended with an error, see $run and $run.out"
    after=$(grep -c '^## ' "$findings" || :)
    test "$after" = "$before" ||
        fail "w$w left new findings: $(sed -n "/^## /p" "$findings" |
            tail -n +$((before + 1)) | tr '\n' ';')"
    before=$after
done
ok "W1 and W2 pass with wait, focus, rects and methods, no pixel clicks"
