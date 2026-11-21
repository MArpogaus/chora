#!/usr/bin/env bash
# Selftest of tools/hooks/pre-commit (identity) and tools/hooks/commit-msg (trailers) on claude/session-log.
# Usage: bash tools/test_identity_hooks.sh
set -u
here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
ME_NAME="Marcel Arpogaus"
ME_MAIL="38564291+MArpogaus@users.noreply.github.com"
failures=0
git init -q "$work/r"
git -C "$work/r" checkout -q -b claude/session-log

verdict() {
    local label=$1 want=$2 status=$3 got=pass
    [ "$status" -ne 0 ] && got=fail
    if [ "$got" = "$want" ]; then
        echo "ok   $label ($want)"
    else
        echo "FAIL $label (wanted $want, exit $status)"
        failures=$((failures + 1))
    fi
}

ident_case() {
    local label=$1 want=$2 branch=$3 an=$4 am=$5 cn=$6 cm=$7
    git -C "$work/r" checkout -q -B "$branch"
    (
        cd "$work/r" || exit 1
        export GIT_AUTHOR_NAME=$an GIT_AUTHOR_EMAIL=$am GIT_COMMITTER_NAME=$cn GIT_COMMITTER_EMAIL=$cm
        bash "$here/hooks/pre-commit" >/dev/null 2>&1
    )
    verdict "pre-commit: $label" "$want" $?
}

ident_case "the owner as author and committer" pass claude/session-log "$ME_NAME" "$ME_MAIL" "$ME_NAME" "$ME_MAIL"
ident_case "another author" fail claude/session-log Claude noreply@anthropic.com "$ME_NAME" "$ME_MAIL"
ident_case "another committer" fail claude/session-log "$ME_NAME" "$ME_MAIL" Claude noreply@anthropic.com
ident_case "the owner's name with another mail" fail claude/session-log "$ME_NAME" other@example.com "$ME_NAME" "$ME_MAIL"
ident_case "another identity on another branch" pass claude/other Claude noreply@anthropic.com Claude noreply@anthropic.com

msg_case() {
    local label=$1 want=$2 branch=$3 text=$4
    git -C "$work/r" checkout -q -B "$branch"
    printf '%b' "$text" >"$work/msg"
    (cd "$work/r" && bash "$here/hooks/commit-msg" "$work/msg" >/dev/null 2>&1)
    verdict "commit-msg: $label" "$want" $?
}

msg_case "a plain message" pass claude/session-log 'log: x\n'
msg_case "a Co-Authored-By line" fail claude/session-log 'log: x\n\nCo-Authored-By: C <c@e>\n'
msg_case "a Claude-Session line" fail claude/session-log 'log: x\n\nClaude-Session: https://x\n'
msg_case "a trailer on another branch" pass claude/other 'log: x\n\nClaude-Session: https://x\n'
[ "$failures" -eq 0 ]
