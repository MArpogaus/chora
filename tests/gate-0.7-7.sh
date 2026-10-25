#!/bin/bash
# Gate check 7 of 0.7: the core has no code that forks a process except
# the startup command. Until T14 removes process.spawn, process.wait and
# plugins.restart from the core, this check must answer FAIL, as
# plans/rest-0.7-0.8.md states.
#
# Usage: tests/gate-0.7-7.sh
set -u
. "$(dirname "$0")/gate-lib.sh"
cd "$repo" || fail "no repo"
# The IPC methods that start a program, and the fork and exec sites of
# the production code. Test modules are out; awk leaves a file at its
# first cfg(test).
methods=$(git grep -nE '"(process\.(spawn|wait)|plugins\.restart)"' -- crates/chora/src || :)
forks=$(git ls-files 'crates/chora/src/*.rs' | xargs awk '
    /#\[cfg\(test\)\]/ { nextfile }
    /Command::new|posix_spawn|execve?\(/ { print FILENAME ":" FNR }')
# T14 will leave exactly the startup command; until then every hit is
# one that must leave the core.
test -z "$methods" ||
    fail "the core still starts programs: $(tr '\n' ' ' <<<"$methods")"
test -z "$forks" ||
    fail "fork sites outside the startup command: $(tr '\n' ' ' <<<"$forks")"
ok "the core forks nothing but its startup command"
