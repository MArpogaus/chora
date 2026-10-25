#!/bin/bash
# Gate check 7 of 0.7: the core has no code that forks a process except
# the startup command. T14 (D42) removed the program start from the core;
# the startup command lives in crates/chora/src/process.rs alone. The
# unit test `the_core_forks_nothing_but_its_startup_command` mirrors this
# check.
#
# Usage: tests/gate-0.7-7.sh
set -u
. "$(dirname "$0")/gate-lib.sh"
cd "$repo" || fail "no repo"
# The IPC methods that started a program, and the fork and exec sites of
# the production code. Test modules are out; awk leaves a file at its
# first cfg(test).
methods=$(git grep -nE '"(process\.(spawn|wait)|plugins\.restart)"' -- crates/chora/src || :)
forks=$(git ls-files 'crates/chora/src/*.rs' |
    grep -v 'src/process.rs' |
    xargs awk '
    /#\[cfg\(test\)\][[:space:]]*(pub[[:space:]]+)?mod[[:space:]]+(tests|testing)/ { nextfile }
    /#\[cfg\(test\)\]/ { pending=1; next }
    pending && /^(pub[[:space:]]+)?mod[[:space:]]+(tests|testing)/ { nextfile }
    { pending=0 }
    /Command::new|posix_spawn|execve?\(|fork\(/ { print FILENAME ":" FNR }')
# process.rs is exempt: it holds exactly the startup command of the core.
test -z "$methods" ||
    fail "the core still starts programs: $(tr '\n' ' ' <<<"$methods")"
test -z "$forks" ||
    fail "fork sites outside the startup command: $(tr '\n' ' ' <<<"$forks")"
ok "the core forks nothing but its startup command"
