#!/usr/bin/env bash
# pre-commit hook: a test script must not signal or pick a process by its
# name alone. Other sessions and the parallel steps of the merge gate
# (--jobs) run the same programs on the same machine, and `pkill`,
# `killall`, `pidof` and a `pgrep -x` or `pgrep -f` without a parent (-P),
# a session (-s) or a process group (-g) reach them. session_pids of
# tests/gate-lib.sh finds the programs of the own runtime directory. tests/last-words.sh once took the newest
# `chora` of the machine and sent it SIGSEGV; the core of a gate of
# another session died, and that gate said "no core".
#
# Usage: tests/no-foreign-kill.sh FILE...   (pre-commit passes the files)
set -u
status=0
for file in "$@"; do
    found=$(awk '
        /^[[:space:]]*#/ { next }
        {
            parts = split($0, command, /[;|&]/)
            for (i = 1; i <= parts; i++) {
                named = command[i] ~ /(^|[^[:alnum:]_-])(pkill|killall|pidof)([^[:alnum:]_-]|$)/ ||
                    command[i] ~ /(^|[^[:alnum:]_-])pgrep[[:space:]](.*[[:space:]])?-[[:alpha:]]*[xf]/
                scoped = command[i] ~ /[[:space:]]-(P|s|g)([[:space:]]|$)/
                if (named && !scoped) {
                    print FILENAME ":" FNR ":" $0
                    break
                }
            }
        }' "$file")
    [ -z "$found" ] && continue
    status=1
    echo "$found"
done
if [ "$status" != 0 ]; then
    echo "a test script names its processes by pid, parent (-P), session (-s), group (-g)"
    echo "or session_pids of tests/gate-lib.sh (its own runtime directory):"
    echo "another session's processes carry the same names."
fi
exit "$status"
