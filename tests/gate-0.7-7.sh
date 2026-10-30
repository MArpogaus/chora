#!/bin/bash
# Gate check 7 of 0.7: the core has no code that forks a process except
# the startup command in crates/chora/src/process.rs. The test is the
# unit test of that file.
#
# Usage: tests/gate-0.7-7.sh
set -u
. "$(dirname "$0")/gate-lib.sh"
out=$(cd "$repo" && cargo test -p chora --lib --locked \
    -- the_core_forks_nothing_but_its_startup_command 2>&1) ||
    fail "the fork test: $(tail -n 15 <<<"$out")"
grep -q "test result: ok. 1 passed" <<<"$out" || fail "$(tail -n 15 <<<"$out")"
ok "the core forks nothing but its startup command"
