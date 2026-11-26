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
# per_id_rule_from (seconds since the epoch) is the committer date of the
# commit that brought the per id rule: a commit committed before it is
# checked by the rule before it (a test in the fix or before it, not one per
# id). The hook always sees a commit made now.
# The words that look like an id but are none (utf-8, sha-256, gate-0.8)
# are skipped, and the hook cuts its message file at the scissors line.
#
# The merge gate also runs red_first_run (below): see tests/merge-gate.sh.
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

per_id_rule_from=1795446505

finding_ids() {
    tr -c 'A-Za-z0-9.-' '\n' <<<"$1" | grep -v '\.[0-9]' | tr . '\n' |
        sed 's/\([0-9]\)-/\1\n/g' | grep -xE '[a-z]{1,6}-[0-9]+[a-z]?' | grep -vE '^(utf|sha|base|ipv|gate|gtk)-' | sort -u
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

fix_ids() {
    [ "$(git log -1 --format=%ct "$1")" -ge "$per_id_rule_from" ] || return 0
    commit_ids "$1"
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
    missing=$(missing_ids "$1" "$2")
    [ -n "$missing" ] || return 0
    for id in $missing; do
        echo "red-first: $3 misses a test commit before it that names $id"
    done
    return 1
}

hook_id_verdict() {
    provable_base || return 0
    id_verdict "$1" "$(branch_test_ids)" "this commit"
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
        notes=$(hook_id_verdict "$ids") && {
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

rust_tests_added() {
    git show --format= -U0 "$1" -- '*.rs' | awk '
        /^\+\+\+ b\// { file = substr($0, 7); pend = 0; next }
        /^\+[[:space:]]*#\[(tokio::)?test/ { pend = 1; next }
        pend && /^\+.*fn [A-Za-z_0-9]+/ {
            match($0, /fn [A-Za-z_0-9]+/)
            print file "\t" substr($0, RSTART + 3, RLENGTH - 3)
            pend = 0
        }'
}

crate_of() {
    local dir manifest name
    dir=$(dirname "$2")
    while :; do
        manifest=Cargo.toml
        [ "$dir" = . ] || manifest=$dir/Cargo.toml
        name=$(git show "$1:$manifest" 2>/dev/null | sed -n '/^\[package\]/,/^\[/s/^name *= *"\(.*\)".*/\1/p' | head -1)
        [ -z "$name" ] || {
            echo "$name"
            return 0
        }
        [ "$dir" != . ] || return 1
        dir=$(dirname "$dir")
    done
}

has_shell_test() {
    git show --format= --name-only "$1" | grep -Ev "$helper_files" | grep -Eq '^tests/gate-[^/]*\.sh$|^tests/[^/]*-selftest\.sh$|^tests/test_[^/]*\.py$'
}

shares_id() {
    local id
    for id in $1; do
        grep -qxF "$id" <<<"$2" && return 0
    done
    return 1
}

qualified() {
    local crate=$1 name
    shift
    for name in "$@"; do
        printf ' %s::%s' "$crate" "$name"
    done
}

first_error_file() {
    awk '/^error/ && !/^error: could not compile/ { e = 1 } e && /^ *--> / { sub(/^ *--> /, ""); sub(/:[0-9]+:[0-9]+.*$/, ""); print; exit }' <<<"$1"
}

names_file() {
    local f
    for f in $2; do
        [ "$f" = "$1" ] || [ "${f%/"$1"}" != "$f" ] && return 0
    done
    return 1
}

run_named() {
    local crate=$1 files=$2 out name path bfile failed=() passed=()
    shift 2
    out=$(cargo test -p "$crate" --tests --no-fail-fast -- "$@" 2>&1) || true
    grep -q '^error: could not compile' <<<"$out" && ! grep -q '^test result:' <<<"$out" && {
        bfile=$(first_error_file "$out")
        if [ -n "$bfile" ] && names_file "$bfile" "$files"; then
            echo "red $crate::(build)"
        else
            echo "unproven $crate: build error outside the added test, not run"
        fi
        return 0
    }
    for name in "$@"; do
        while read -r path; do
            [ "$path" = "$name" ] || [ "${path%::"$name"}" != "$path" ] || continue
            grep -qF "test $path ... FAILED" <<<"$out" && failed+=("$name") || passed+=("$name")
        done < <(sed -n 's/^test \(.*\) \.\.\. \(ok\|FAILED\)$/\1/p' <<<"$out")
    done
    [ "${#failed[@]}" = 0 ] || echo "red$(qualified "$crate" "${failed[@]}")"
    [ "${#failed[@]}" != 0 ] || [ "${#passed[@]}" = 0 ] || echo "pass$(qualified "$crate" "${passed[@]}")"
    [ "$((${#failed[@]} + ${#passed[@]}))" != 0 ] || echo "unproven $crate: none of the tests ran on the parent, not run"
}

rf_wt=

rf_cleanup() {
    [ -z "$rf_wt" ] || {
        git worktree remove --force "$rf_wt/wt" 2>/dev/null
        rm -rf "$rf_wt"
        rf_wt=
    }
}

red_first_run() {
    local prev rc
    prev=$(trap -p INT TERM PIPE)
    trap 'rf_cleanup; exit 130' INT
    trap 'rf_cleanup; exit 143' TERM
    trap 'rf_cleanup; exit 141' PIPE
    red_first_range "$@"
    rc=$?
    rf_cleanup
    trap - INT TERM PIPE
    [ -z "$prev" ] || eval "$prev"
    return "$rc"
}

red_first_range() {
    local base=$1 cap=10 shas=() fixes=() i j sha subject prev=-1 ids paired c file name crate wt bad=0 body
    local proven=0 notrun=0 edits
    local -A names=() files=()
    local line verdict gate runs passes
    git rev-parse --verify -q "$base" >/dev/null || return 1
    mapfile -t shas < <(git rev-list --reverse --no-merges --topo-order "$base..HEAD")
    for i in "${!shas[@]}"; do
        is_fix "$(git log -1 --format=%s "${shas[i]}")" && fixes+=("$i")
    done
    if [ "${#fixes[@]}" -gt "$cap" ]; then
        echo "red-first: run capped to the last $cap of ${#fixes[@]} fix commits"
        fixes=("${fixes[@]: -$cap}")
    fi
    [ "${#fixes[@]}" -gt 0 ] || return 0
    rf_wt=$(mktemp -d "${TMPDIR:-/tmp}/red-first-run.XXXXXX") || return 1
    wt=$rf_wt
    git worktree add -q --detach "$wt/wt" HEAD || return 1
    export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$(pwd)/target}
    for i in "${fixes[@]}"; do
        sha=${shas[i]}
        subject=$(git log -1 --format=%s "$sha")
        body=$(git log -1 --format=%b "$sha")
        grep -Eq '^No test: .+' <<<"$body" && continue
        ids=$(fix_ids "$sha")
        paired=()
        prev=-1
        for ((j = 0; j < i; j++)); do
            is_fix "$(git log -1 --format=%s "${shas[j]}")" || continue
            [ -z "$ids" ] || shares_id "$ids" "$(fix_ids "${shas[j]}")" || continue
            prev=$j
        done
        for ((j = prev + 1; j < i; j++)); do
            c=${shas[j]}
            commit_changes_test "$c" || continue
            git merge-base --is-ancestor "$c" "$sha^" || continue
            [ -z "$ids" ] || shares_id "$ids" "$(commit_ids "$c")" || continue
            paired+=("$c")
        done
        names=()
        files=()
        gate=0
        edits=0
        for c in "${paired[@]}"; do
            if has_shell_test "$c"; then
                gate=1
            elif [ -z "$(rust_tests_added "$c")" ]; then
                edits=1
            fi
            while IFS=$'\t' read -r file name; do
                [ -n "$file" ] || continue
                crate=$(crate_of "$c" "$file") || continue
                names[$crate]+="$name "
                files[$crate]+="$file "
            done < <(rust_tests_added "$c")
        done
        if [ "${#names[@]}" -gt 0 ]; then
            git -C "$wt/wt" checkout -q --detach "$sha^" 2>/dev/null || {
                echo "red-first: $(git log -1 --format=%h "$sha") checkout of the parent failed, not run"
                notrun=$((notrun + 1))
                continue
            }
            runs=0
            passes=0
            for crate in "${!names[@]}"; do
                # shellcheck disable=SC2086
                line=$(cd "$wt/wt" && run_named "$crate" "${files[$crate]}" ${names[$crate]})
                verdict=${line%% *}
                case $verdict in
                red)
                    runs=1
                    echo "red-first: $(git log -1 --format=%h "$sha")${line#red} red on the parent"
                    ;;
                pass)
                    passes=1
                    echo "red-first: $(git log -1 --format=%h "$sha")${line#pass} passes on the parent"
                    ;;
                unproven)
                    echo "red-first: $(git log -1 --format=%h "$sha") ${line#unproven }"
                    ;;
                esac
            done
            if [ "$runs" = 1 ]; then
                proven=$((proven + 1))
            elif [ "$passes" = 1 ] && [ "$gate" = 1 ]; then
                echo "red-first: $(git log -1 --format=%h "$sha") gate cell too, not run"
                notrun=$((notrun + 1))
            elif [ "$passes" = 1 ]; then
                echo "RED-FIRST NOT RED: $(git log -1 --format='%h %s' "$sha")"
                bad=1
            else
                notrun=$((notrun + 1))
            fi
        elif [ "$gate" = 1 ] || has_shell_test "$sha"; then
            echo "red-first: $(git log -1 --format=%h "$sha") gate cell, not run"
            notrun=$((notrun + 1))
        elif [ "$edits" = 1 ]; then
            echo "red-first: $(git log -1 --format=%h "$sha") test commit edits tests, not run"
            notrun=$((notrun + 1))
        elif [ -n "$(rust_tests_added "$sha")" ]; then
            echo "red-first: $(git log -1 --format=%h "$sha") test in the fix commit, not run"
            notrun=$((notrun + 1))
        else
            echo "red-first: $(git log -1 --format=%h "$sha") no test function found, not run"
            notrun=$((notrun + 1))
        fi
    done
    echo "red-first: $proven proven red, $notrun not run"
    return "$bad"
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then
    [ $# -eq 1 ] || {
        echo "usage: $0 MSG_FILE" >&2
        exit 2
    }
    red_first_hook "$1"
fi
