#!/usr/bin/env bash
# pre-commit hook: a shell script of flavors/ or plugins/ that makes a
# directory under $XDG_RUNTIME_DIR with `mkdir -p` needs `-m 700`: with the
# default umask the directory is readable by other users of the machine
# (the hint and pid files of the plugins live there). A path is under the
# runtime directory when the line names XDG_RUNTIME_DIR or a variable that
# was set from it (`runtime=${XDG_RUNTIME_DIR:-...}`, `pids=$runtime/x`), in
# the same file. Comment lines are skipped. Only `mkdir -p` is flagged: a
# plain `mkdir` fails on an existing path and is no race for a shared one.
#
# Usage: tests/mkdir-runtime.sh FILE...   (pre-commit passes the files)
set -u
status=0
for file in "$@"; do
    found=$(awk '
        /^[[:space:]]*#/ { next }
        {
            line = $0
            if (match(line, /^[[:space:]]*(export[[:space:]]+|local[[:space:]]+)?[A-Za-z_][A-Za-z0-9_]*=/)) {
                name = substr(line, RSTART, RLENGTH)
                sub(/^[[:space:]]*(export[[:space:]]+|local[[:space:]]+)?/, "", name)
                sub(/=$/, "", name)
                rhs = substr(line, RSTART + RLENGTH)
                if (rhs ~ /XDG_RUNTIME_DIR/) tainted[name] = 1
                else for (v in tainted)
                    if (rhs ~ ("\\$\\{?" v "([^A-Za-z0-9_]|$)")) { tainted[name] = 1; break }
            }
            if (line !~ /(^|[^[:alnum:]_-])mkdir[[:space:]]/) next
            if (line !~ /mkdir[[:space:]]+(-[[:alpha:]]+[[:space:]]+)*-[[:alpha:]]*p/) next
            if (line ~ /-m[[:space:]]*0?700|-[[:alpha:]]*m[[:space:]]*0?700|--mode[= ][[:space:]]*0?700/) next
            hit = (line ~ /XDG_RUNTIME_DIR/)
            if (!hit) for (v in tainted)
                if (line ~ ("\\$\\{?" v "([^A-Za-z0-9_]|$)")) { hit = 1; break }
            if (hit) print FILENAME ":" NR ":" $0
        }' "$file")
    [ -z "$found" ] && continue
    status=1
    echo "$found"
done
if [ "$status" != 0 ]; then
    echo "mkdir -p under \$XDG_RUNTIME_DIR needs -m 700 (and"
    echo "# shellcheck disable=SC2174 above it): other users must not read it."
fi
exit "$status"
