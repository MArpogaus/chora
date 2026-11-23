#!/bin/bash
# The red-first rule of tests/merge-gate.sh, run by the commit-msg hook:
# a `fix` or `feat` commit needs a test change in it (the staged files) or
# in a commit before it on the branch (RED_FIRST_BASE..HEAD, without
# merges), else a body line `No test: <reason>`. Other types, merges
# (MERGE_HEAD) and merge messages pass. A test is what the red-first step
# of tests/merge-gate.sh counts; tests/merge-gate.sh sources this file.
# The hook refuses only what it can prove: when the base ref resolves and
# base..HEAD has no test. Without the base ref (a shallow or single-branch
# clone), on an unborn HEAD or on the base branch itself (its base..HEAD
# holds only the unpushed commits) it prints a note and lets the commit
# pass; the merge gate checks the range again.
#
# A `fix` commit that names finding ids (a subject or body word like sh-1,
# mc-55 or rtd-2: lowercase letters, a hyphen, digits, an optional letter)
# needs, for each id, a commit before it on the branch (or this commit
# itself) that names the same id and changes a test: a test that another
# id brought does not count. A fix that names no id keeps the rule above.
#
# Usage: tests/red-first.sh MSG_FILE
#   RED_FIRST_BASE  the base of the branch (default
#                   origin/claude/autonomous-dev-session-3w-i33pg7)
# Exit status: 0 the commit may go in, 1 RED-FIRST MISSING.

test_files='(^|/)tests/.*\.rs$|^tests/gate-[^/]*\.sh$|^tests/[^/]*-selftest\.sh$|^tests/test_[^/]*\.py$'
helper_files='^tests/gate-(lib|lock)\.sh$'

changes_test() {
    grep -Ev "$helper_files" <<<"$1" | grep -Eq "$test_files" ||
        grep -Eq '^\+[[:space:]]*#\[(tokio::)?test' <<<"$2"
}

needs_test() {
    grep -Eq '^(fix|feat)(\([^)]*\))?!?:' <<<"$1"
}

is_fix() {
    grep -Eq '^fix(\([^)]*\))?!?:' <<<"$1"
}

finding_ids() {
    tr -c 'A-Za-z0-9.-' '\n' <<<"$1" | grep -v '\.[0-9]' | tr . '\n' |
        grep -xE '[a-z]{1,6}-[0-9]+[a-z]?' | grep -vE '^(utf|sha|base|ipv|gate|gtk)-' | sort -u
}

missing_ids() {
    local id
    for id in $1; do
        grep -qxF "$id" <<<"$2" || echo "$id"
    done
}

commit_changes_test() {
    changes_test "$(git show --format= --name-only "$1")" "$(git show --format= -U0 "$1" -- '*.rs')"
}

commit_ids() {
    finding_ids "$(git log -1 --format=%B "$1")"
}

on_base_branch() {
    local base_ref branch
    base_ref=$(git rev-parse -q --symbolic-full-name "$1")
    branch=$(git symbolic-ref -q HEAD) || return 1
    [ "$base_ref" = "$branch" ] || [ "${base_ref#refs/remotes/*/}" = "${branch#refs/heads/}" ]
}

provable_base() {
    rf_base=${RED_FIRST_BASE:-origin/claude/autonomous-dev-session-3w-i33pg7}
    if ! git rev-parse -q --verify "$rf_base^{commit}" >/dev/null || ! git rev-parse -q --verify HEAD >/dev/null; then
        echo "red-first: no base ref $rf_base or no commit yet, so nothing to prove; the merge gate checks the range"
        return 1
    fi
    if on_base_branch "$rf_base"; then
        echo "red-first: on the base branch $rf_base, base..HEAD holds only unpushed commits; the merge gate checks the range"
        return 1
    fi
}

branch_verdict() {
    local sha
    provable_base || return 0
    for sha in $(git rev-list --no-merges "$rf_base..HEAD"); do
        commit_changes_test "$sha" && return 0
    done
    return 1
}

branch_test_ids() {
    local sha
    for sha in $(git rev-list --no-merges "$rf_base..HEAD"); do
        commit_changes_test "$sha" && commit_ids "$sha"
    done
}

id_verdict() {
    local missing id
    provable_base || return 0
    missing=$(missing_ids "$1" "$(branch_test_ids)")
    [ -n "$missing" ] || return 0
    for id in $missing; do
        echo "red-first: no commit before this one names $id and changes a test"
    done
    return 1
}

red_first_hook() {
    local message subject ids notes
    [ -e "$(git rev-parse --git-path MERGE_HEAD)" ] && return 0
    message=$(sed '/^# -* >8 -*$/,$d' "$1" | grep -v '^#') || true
    subject=$(grep -m1 . <<<"$message")
    needs_test "$subject" || return 0
    grep -Eq '^No test: .+' <<<"$message" && return 0
    changes_test "$(git diff --cached --name-only)" "$(git diff --cached -U0 -- '*.rs')" && return 0
    ids=""
    ! is_fix "$subject" || ids=$(finding_ids "$message")
    if [ -n "$ids" ]; then
        notes=$(id_verdict "$ids") && {
            [ -z "$notes" ] || echo "$notes"
            return 0
        }
        echo "RED-FIRST MISSING: $subject"
        echo "$notes"
    else
        branch_verdict && return 0
        echo "RED-FIRST MISSING: $subject"
    fi
    echo "red-first: stage the test that fails without this change, commit it first, or add a body line 'No test: <reason>'"
    return 1
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then
    [ $# -eq 1 ] || {
        echo "usage: $0 MSG_FILE" >&2
        exit 2
    }
    red_first_hook "$1"
fi
