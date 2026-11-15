#!/usr/bin/env bash
# pre-commit hook: a test script must not signal or pick a process by its
# name alone. Other sessions and the parallel steps of the merge gate
# (--jobs) run the same programs on the same machine, and `pkill`,
# `killall`, `pidof` and a `pgrep` without a parent (-P), a session (-s),
# a process group (-g) or a pid file (-F) reach them. session_pids of
# tests/gate-lib.sh finds the programs of the own runtime directory.
# tests/last-words.sh once took the newest `chora` of the machine and
# sent it SIGSEGV; the core of a gate of another session died, and that
# gate said "no core".
#
# The hook reads commands, not text: comment lines, here-documents,
# quoted strings and echo or printf commands are skipped, and a line
# continued with `\` is read as one. A pgrep whose pids the next three
# lines filter by /proc/PID/environ (as session_pids does) passes.
#
# Usage: tests/no-foreign-kill.sh FILE...   (pre-commit passes the files)
set -u
status=0
for file in "$@"; do
    found=$(awk '
        { raw[NR] = $0 }
        END {
            for (n = 1; n <= NR; n++) {
                line = raw[n]
                if (heredoc != "") {
                    body = line
                    sub(/^\t+/, "", body)
                    if (body == heredoc) heredoc = ""
                    continue
                }
                if (line ~ /^[[:space:]]*#/) continue
                start = n
                while (line ~ /\\$/ && n < NR) {
                    sub(/\\$/, " ", line)
                    line = line raw[++n]
                }
                if (match(line, /(^|[^<])<<-?[[:space:]]*["\047]?[A-Za-z_][A-Za-z0-9_]*/)) {
                    heredoc = substr(line, RSTART, RLENGTH)
                    sub(/^[^<]*<<-?[[:space:]]*["\047]?/, "", heredoc)
                }
                gsub(/"([^"\\]|\\.)*"/, "\"\"", line)
                gsub(/\047[^\047]*\047/, "\047\047", line)
                parts = split(line, command, /[;|&]/)
                for (i = 1; i <= parts; i++) {
                    if (command[i] ~ /^[[:space:]]*(echo|printf)([[:space:]]|$)/) continue
                    if (command[i] !~ /(^|[^[:alnum:]_-])(pkill|killall|pidof|pgrep)[[:space:]]+[^[:space:]>)]/) continue
                    if (command[i] ~ /[[:space:]]-[[:lower:]]*[PsgF]|[[:space:]]--(parent|session|pgroup|pidfile)/) continue
                    if (command[i] ~ /pgrep/ && environ_filter(n)) continue
                    print FILENAME ":" start ":" raw[start]
                    break
                }
            }
        }
        function environ_filter(n,    k) {
            for (k = n + 1; k <= n + 3 && k <= NR; k++)
                if (raw[k] ~ /\/proc\/.*environ/) return 1
            return 0
        }' "$file")
    [ -z "$found" ] && continue
    status=1
    echo "$found"
done
if [ "$status" != 0 ]; then
    echo "a test script names its processes by pid, parent (-P), session (-s), group (-g),"
    echo "pid file (-F) or session_pids of tests/gate-lib.sh (its own runtime directory):"
    echo "another session's processes carry the same names."
fi
exit "$status"
