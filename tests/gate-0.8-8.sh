#!/bin/bash
# Gate check 8 of 0.8: the revoke binding of the safety layer removes
# the capabilities of all clients. The test is the unit test of T8 in
# crates/chora/src/safety.rs.
#
# Usage: tests/gate-0.8-8.sh
set -u
. "$(dirname "$0")/gate-lib.sh"
need cargo
out=$(cd "$repo" && cargo test -p chora --lib --locked \
    -- the_revoke_binding_takes_the_capabilities_of_all_clients 2>&1) ||
    fail "the revoke test: $(tail -n 15 <<<"$out")"
grep -q "test result: ok. 1 passed" <<<"$out" || fail "$(tail -n 15 <<<"$out")"
ok "the fixed revoke binding takes the capabilities of all clients"
