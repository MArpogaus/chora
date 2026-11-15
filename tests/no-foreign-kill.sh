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
# The hook reads commands, not text: comments, here-documents, the
# literal text of quoted strings (also over several lines), arithmetic
# and echo, printf or for lists without a command substitution are
# skipped; a command substitution counts, quoted or not. A line continued
# with `\` is read as one. The options are read from the tool name on, so
# the `-s` of kill or timeout is no session. A pgrep whose pids the next
# three lines filter by /proc/PID/environ (as session_pids does) passes.
# A name followed by another name (`need pgrep pkill`) is no call. A
# first argument held in a variable (`pgrep $scope -x foot`) cannot be
# read, so it only warns.
#
# Usage: tests/no-foreign-kill.sh FILE...   (pre-commit passes the files)
set -u
status=0
for file in "$@"; do
    found=$(awk '
        function push(context) { stack[++depth] = context }
        function code(s,    out, i, c, top) {
            out = ""
            for (i = 1; i <= length(s); i++) {
                c = substr(s, i, 1)
                top = stack[depth]
                if (top == "sq") {
                    if (c == "\047") { depth--; out = out c } else if (keep) out = out c
                    continue
                }
                if (top == "dq") {
                    if (c == "\\") { if (keep) out = out substr(s, i, 2); i++; continue }
                    if (c == "\"") { depth--; out = out c; continue }
                    if (substr(s, i, 2) == "$(") { push("sub"); out = out "$("; i++; continue }
                    if (c == "`") { push("bq"); out = out c; continue }
                    if (c == "$" || keep) out = out c
                    continue
                }
                if (c == "\\") { out = out substr(s, i, 2); i++; continue }
                if (c == "\047" || c == "\"") {
                    keep = out ~ /(^|[^<])<<-?[[:space:]]*$/
                    push(c == "\"" ? "dq" : "sq")
                    out = out c
                    continue
                }
                if (c == "#" && (i == 1 || substr(s, i - 1, 1) ~ /[[:space:]]/)) break
                if (substr(s, i, 2) == "$(") { push("sub"); out = out "$("; i++; continue }
                if (c == "(") { push("par"); out = out c; continue }
                if (c == ")") { if (top == "sub" || top == "par") depth--; out = out c; continue }
                if (c == "`") { if (top == "bq") depth--; else push("bq"); out = out c; continue }
                out = out c
            }
            gsub(/\(\(.*\)\)/, "(())", out)
            return out
        }
        function environ_filter(n,    k) {
            for (k = n + 1; k <= n + 3 && k <= NR; k++)
                if (raw[k] !~ /^[[:space:]]*#/ && raw[k] ~ /\/proc\/.*environ/) return 1
            return 0
        }
        function verdict(command, n,    rest, word, args) {
            if (command !~ /\$\(|`/ && command ~ /^[[:space:]]*(echo|printf|for)([[:space:]]|$)/) return ""
            rest = command
            while (match(rest, /(^|[^[:alnum:]_-])(pkill|killall|pidof|pgrep)[[:space:]]+[^[:space:]>)]/)) {
                rest = substr(rest, RSTART + RLENGTH - 1)
                word = rest
                sub(/[[:space:]].*/, "", word)
                if (word ~ /^(pkill|killall|pidof|pgrep)$/) continue
                args = rest
                sub(/[)`].*/, "", args)
                if (args ~ /(^|[[:space:]])-[[:lower:]]*[PsgF]|(^|[[:space:]])--(parent|session|pgroup|pidfile)/) continue
                if (substr(command, 1, length(command) - length(rest)) ~ /pgrep[[:space:]]+$/ && environ_filter(n)) continue
                if (word ~ /^"?\$[^(]/) return "W"
                return "F"
            }
            return ""
        }
        { raw[NR] = $0 }
        END {
            stack[0] = "top"
            for (n = 1; n <= NR; n++) {
                if (heredoc != "") {
                    body = raw[n]
                    sub(/^\t+/, "", body)
                    if (body == heredoc) heredoc = ""
                    continue
                }
                start = n
                line = code(raw[n])
                while (line ~ /\\$/ && n < NR) {
                    sub(/\\$/, " ", line)
                    line = line code(raw[++n])
                }
                if (match(line, /(^|[^<])<<-?[[:space:]]*["\047\\]?[A-Za-z_][A-Za-z0-9_]*/)) {
                    heredoc = substr(line, RSTART, RLENGTH)
                    sub(/^[^<]*<<-?[[:space:]]*["\047\\]?/, "", heredoc)
                }
                parts = split(line, command, /[;|&]/)
                for (i = 1; i <= parts; i++) {
                    found = verdict(command[i], n)
                    if (found == "") continue
                    print found " " FILENAME ":" start ":" raw[start]
                    break
                }
            }
        }' "$file")
    [ -z "$found" ] && continue
    warnings=$(grep '^W ' <<<"$found" | cut -c3-)
    [ -n "$warnings" ] && printf 'warning: the scope of this lookup is in a variable; check it by hand:\n%s\n' "$warnings"
    grep -q '^F ' <<<"$found" || continue
    status=1
    grep '^F ' <<<"$found" | cut -c3-
done
if [ "$status" != 0 ]; then
    echo "a test script names its processes by pid, parent (-P), session (-s), group (-g),"
    echo "pid file (-F) or session_pids of tests/gate-lib.sh (its own runtime directory):"
    echo "another session's processes carry the same names."
fi
exit "$status"
