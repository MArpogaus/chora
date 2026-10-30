#!/bin/bash
# Gate check 5 of 0.8: a subscription that starts during a transaction
# misses no event and sees no event twice; 1000 transactions check the
# sequence numbers. The test is the unit test of T2 in
# crates/chora/src/ipc.rs.
#
# Usage: tests/gate-0.8-5.sh
set -u
. "$(dirname "$0")/gate-lib.sh"
need cargo
out=$(cd "$repo" && cargo test -p chora --lib --locked \
    -- a_subscription_answers_with_a_snapshot_and_numbers_its_events --nocapture 2>&1) ||
    fail "the sequence test: $(tail -n 15 <<<"$out")"
grep -q "test result: ok. 1 passed" <<<"$out" || fail "$(tail -n 15 <<<"$out")"
ok "the snapshot and 1000 transactions keep one gapless sequence per connection"
