#!/bin/bash
# The red-first rule of tests/merge-gate.sh, run by the commit-msg hook:
# a `fix` or `feat` commit needs a test change in it (the staged files) or
# in a commit before it on the branch (RED_FIRST_BASE..HEAD, without
# merges), else a body line `No test: <reason>`. Other types and merge
# messages pass. A test is what the red-first step of tests/merge-gate.sh
# counts; tests/merge-gate.sh sources this file for `changes_test`.
#
# Usage: tests/red-first.sh MSG_FILE
#   RED_FIRST_BASE  the base of the branch (default
#                   origin/claude/autonomous-dev-session-3w-i33pg7); without
#                   that ref only the staged files count
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

commit_changes_test() {
    changes_test "$(git show --format= --name-only "$1")" "$(git show --format= -U0 "$1" -- '*.rs')"
}

branch_has_test() {
    local base=${RED_FIRST_BASE:-origin/claude/autonomous-dev-session-3w-i33pg7} sha
    git rev-parse -q --verify HEAD >/dev/null || return 1
    git rev-parse -q --verify "$base^{commit}" >/dev/null || return 1
    for sha in $(git rev-list --no-merges "$base..HEAD"); do
        commit_changes_test "$sha" && return 0
    done
    return 1
}

red_first_hook() {
    local message subject
    message=$(grep -v '^#' "$1") || true
    subject=$(grep -m1 . <<<"$message")
    needs_test "$subject" || return 0
    grep -Eq '^No test: .+' <<<"$message" && return 0
    changes_test "$(git diff --cached --name-only)" "$(git diff --cached -U0 -- '*.rs')" && return 0
    branch_has_test && return 0
    echo "RED-FIRST MISSING: $subject"
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
