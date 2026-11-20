#!/bin/bash
# Selftest of the step 14 diagnostics of tests/nested.sh: a failing
# wait_for of not_black prints the pixel, the top color, the tail of
# probe.log and the lock lines of the core, so that one failed run says
# why. Runs without a compositor: the screen functions are stubs.
#
# Usage: tests/nested-diag-selftest.sh
set -euo pipefail
here=$(dirname "$0")
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT
printf 'take-lock: client 7 holds the lock\nunrelated line\n' >"$out/probe.log"
printf 'INFO session lock: takeover by client 7\nINFO unrelated\nWARN lock surface late\n' >"$out/chora.log"
pixel() { echo "0 0 0"; }
top_color() { echo "12 34 56"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
# shellcheck source=tests/nested-diag.sh
. "$here/nested-diag.sh"
text=$(not_black_diag)
for want in "pixel 100,100 = 0 0 0" "top color = 12 34 56" "take-lock: client 7 holds the lock" \
    "session lock: takeover by client 7" "lock surface late"; do
    grep -qF -- "$want" <<<"$text" || fail "the diagnostics lack: $want
$text"
done
grep -qF "unrelated line" <<<"$text" || fail "the probe.log tail lacks its last line"
if grep -qF "INFO unrelated" <<<"$text"; then fail "a line without lock is in the core lines"; fi
echo "nested-diag-selftest: ok"
