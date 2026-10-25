#!/bin/bash
# Coverage of the deprecated core program start (D42): `choractl spawn`
# asks the core, needs the grant `process.spawn`, and with `--capture`
# waits for the end and prints the output kept for `process.wait`. The
# methods stay in the core until T14 removes them; then this file, the
# step in tests/agent.sh and the exclusion in tests/legacy-callers.sh go
# away. Sourced by tests/agent.sh, which provides bin, ctl, out and fail.
if ctl spawn true 2>"$out/spawn-denied.err"; then fail "a spawn without grant worked"; fi
grep -q "needs a grant" "$out/spawn-denied.err" || fail "denied: $(cat "$out/spawn-denied.err")"
"$bin/choractl" grant agent process.spawn || fail "the grant failed"
ctl spawn --capture -- sh -c 'echo out; exit 3' >"$out/captured.json" ||
    fail "the captured spawn failed"
jq -e '.running == false and .status == 3 and .stdout == "out\n"' "$out/captured.json" >/dev/null ||
    fail "the captured spawn: $(cat "$out/captured.json")"
"$bin/choractl" revoke agent process.spawn || fail "the revoke failed"
