#!/bin/bash
# commit-msg hook: a staged change that raises a count in
# tools/lint-ratchet.list or tools/size-ratchet.list needs a
# `ratchet-raise: <reason>` line in the message. $1 is the message file.
# Falling counts and new lines with a lower value need nothing.
set -u
lists="tools/lint-ratchet.list tools/size-ratchet.list"
counts() { sed 's/#.*//' | awk 'NF >= 2 { print $1, $2 }'; }
raised=""
for list in $lists; do
    git diff --cached --quiet -- "$list" && continue
    old=$(git show "HEAD:$list" 2>/dev/null | counts)
    new=$(git show ":$list" 2>/dev/null | counts)
    while read -r name count; do
        [ -n "$name" ] || continue
        was=$(awk -v k="$name" '$1 == k { print $2 }' <<<"$old")
        if [ -z "$was" ] || [ "$count" -gt "$was" ]; then
            raised="$raised $list:$name"
        fi
    done <<<"$new"
done
[ -z "$raised" ] && exit 0
grep -Eq '^ratchet-raise: .+' "$1" && exit 0
echo "ratchet raised without a 'ratchet-raise: <reason>' line:$raised" >&2
exit 1
