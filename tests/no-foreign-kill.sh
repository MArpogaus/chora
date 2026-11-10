#!/usr/bin/env bash
# pre-commit hook: a test script must not signal or pick a process by its
# name alone. Other sessions run their own cores on the same machine, and
# `pkill`, `killall` and a `pgrep -x chora` without a parent (-P) or a
# session (-s) reach them. tests/last-words.sh once took the newest
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
                named = command[i] ~ /(^|[^[:alnum:]_-])(pkill|killall)([^[:alnum:]_-]|$)/ ||
                    command[i] ~ /(^|[^[:alnum:]_-])pgrep[[:space:]].*-x chora([^-[:alnum:]_]|$)/
                scoped = command[i] ~ /[[:space:]]-(P|s)([[:space:]]|$)/
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
    echo "a test script names its processes by pid, or by parent (-P) or session (-s):"
    echo "another session's processes carry the same names."
fi
exit "$status"
