#!/bin/bash
# The one command that decides whether a merge into the session branch
# may be pushed. It runs the steps in order, each without a pipe and with
# its whole output in its own log, and stops at the first step that failed.
#
# One gate runs at a time: the gate and the gate scripts take the lock
# of tests/gate-lock.sh, and a second one prints a line and waits.
#
# Steps: red-first, pre-commit, gate-lib, build, test, clippy, 2d-guard, xp, matrix, virtual, xp-faces, hud,
# layout, chat-keys, last-words, cube, pull, winit, wm-pane. `layout` audits
# the layout of the construct HUD in each state. `chat-keys` opens the
# chat in tiling with Super+a and types into its field. `matrix` ends with the click that
# chooses a window manager for a new window; `virtual` hands a window over between window
# managers on a screen and on virtual outputs; `xp-faces` runs one xp on
# four virtual outputs; `cube` shows that xp on the faces of a cube in the
# construct; `pull` drags a window out of a face of that cube and back.
# `winit` runs a virtual output with xp in a chora nested on the winit
# backend; `wm-pane` runs xp and tiling each in a pane of the construct.
# `red-first` checks the range BASE..HEAD (commits that are not merges, oldest
# first): each `fix` or `feat` commit needs a commit of the range that adds
# or changes a test file (a path under tests/, a test_*, *_test or *_tests
# file, or an added #[test] line) before it or in it. Else the step prints
# RED-FIRST MISSING with the subject and fails, unless the commit body has
# a line `No test: <reason>`. `pre-commit` runs the hooks over the same range.
# A step fails when its exit code is not 0, when its log has a line that
# starts with `FAIL:` or `MISMATCH`, or when it has the line
# `test result: FAILED`. One line per step: name, PASS or FAIL, seconds,
# log path. A failed step prints the last 30 lines of its log.
#
# Usage: tests/merge-gate.sh [--only STEP,...] [--worker STEP,...] [--push BRANCH]
#                            [--base REF]
#   --only STEP,...  run only these steps; never pushes
#   --worker STEP,...  the lane mode: build, test, clippy and the steps
#                    named here (`--worker xp,hud`; `--worker ''` for none);
#                    never pushes
#   --base REF       the base of the merged range (default
#                    origin/claude/autonomous-dev-session-3w-i33pg7)
#   --push BRANCH    after every step passed, `git push origin HEAD:BRANCH`;
#                    refuses when the work tree has uncommitted changes
#   -h, --help       this text
#   MERGE_GATE_STEPS  file with one `name|command` per line, instead of the
#                     steps above (the self-test uses it)
#   MERGE_GATE_RUNS   directory for the run directories (default: /tmp)
#   CHORA_GATE_LOCK   the lock file of the gates (see tests/gate-lock.sh)
# Exit status: 0 all steps passed, 1 a step failed or the push was
# refused, 2 a usage error.
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)

usage() { sed -n '2,/^set -u/p' "$0" | sed '$d; s/^# \{0,1\}//'; }
die() {
    echo "merge-gate: $*" >&2
    exit 2
}

only=""
worker=""
worker_mode=""
push_branch=""
base=origin/claude/autonomous-dev-session-3w-i33pg7
red_check=""
while [ $# -gt 0 ]; do
    case $1 in
    -h | --help)
        usage
        exit 0
        ;;
    --only)
        [ $# -ge 2 ] || die "--only needs a list of steps"
        only=$2
        shift 2
        ;;
    --worker)
        [ $# -ge 2 ] || die "--worker needs a list of steps (may be empty)"
        worker=$2
        worker_mode=1
        shift 2
        ;;
    --base)
        [ $# -ge 2 ] || die "--base needs a ref"
        base=$2
        shift 2
        ;;
    --red-first-check)
        red_check=1
        shift
        ;;
    --push)
        [ $# -ge 2 ] || die "--push needs a branch"
        push_branch=$2
        shift 2
        ;;
    *) die "unknown argument $1" ;;
    esac
done

red_first() {
    local missing=0 seen=0 sha subject body diff
    git -C "$repo" rev-parse --verify -q "$base" >/dev/null || die "no base ref $base"
    while read -r sha; do
        [ -n "$sha" ] || continue
        subject=$(git -C "$repo" log -1 --format=%s "$sha")
        diff=$(git -C "$repo" show --format= --name-only "$sha")
        if grep -Eq '(^|/)tests/|(^|/)test_[^/]*$|_tests?\.[a-z]+$' <<<"$diff" ||
            git -C "$repo" show --format= -U0 "$sha" | grep -Eq '^\+[[:space:]]*#\[(tokio::)?test'; then
            seen=1
        fi
        grep -Eq '^(fix|feat)(\([^)]*\))?!?:' <<<"$subject" || continue
        [ "$seen" = 0 ] || continue
        body=$(git -C "$repo" log -1 --format=%b "$sha")
        grep -Eq '^No test: .+' <<<"$body" && continue
        echo "RED-FIRST MISSING: $(git -C "$repo" log -1 --format='%h %s' "$sha")"
        missing=1
    done < <(git -C "$repo" rev-list --reverse --no-merges "$base..HEAD")
    [ "$missing" = 0 ] && echo "red-first: every fix and feat commit has a test before it or in it"
    return "$missing"
}

if [ -n "$red_check" ]; then
    red_first
    exit $?
fi

default_steps() {
    cat <<STEPS
red-first|tests/merge-gate.sh --red-first-check --base "$base"
pre-commit|pre-commit run --from-ref "$base" --to-ref HEAD
gate-lib|tests/gate-lib-selftest.sh
build|cargo build --workspace
test|cargo test --workspace
clippy|cargo clippy --workspace --all-targets -- -D warnings
2d-guard|tests/2d-guard.sh 8742dee
xp|tests/xp.sh "$repo/target/debug"
matrix|tests/gate-handover-matrix.sh && tests/gate-give-holders.sh && BIN=$repo/target/debug tests/gate-click-routes.sh
virtual|BIN=$repo/target/debug tests/gate-handover-virtual.sh
xp-faces|BIN=$repo/target/debug tests/gate-xp-faces.sh
hud|BIN=$repo/target/debug tests/gate-hud.sh
layout|BIN=$repo/target/debug tests/gate-construct-layout.sh
chat-keys|BIN=$repo/target/debug tests/gate-chat-keys.sh
last-words|CHORA_BIN=$repo/target/debug tests/last-words.sh
cube|BIN=$repo/target/debug tests/gate-construct-cube.sh
pull|BIN=$repo/target/debug tests/gate-construct-pull.sh
winit|BIN=$repo/target/debug tests/gate-virtual-winit.sh
wm-pane|BIN=$repo/target/debug tests/gate-construct-wm-pane.sh
STEPS
}

if [ -n "${MERGE_GATE_STEPS:-}" ]; then
    steps=$(cat "$MERGE_GATE_STEPS") || die "cannot read $MERGE_GATE_STEPS"
else
    steps=$(default_steps)
fi

step_names=()
step_commands=()
while IFS='|' read -r name command; do
    [ -n "$name" ] || continue
    step_names+=("$name")
    step_commands+=("$command")
done <<<"$steps"

selected() {
    [ -z "$only" ] && return 0
    case ",$only," in *",$1,"*) return 0 ;; esac
    return 1
}

if [ -n "$worker_mode" ]; then
    [ -z "$only" ] || die "--worker and --only exclude each other"
    [ -z "$push_branch" ] || die "--worker never pushes"
    only="build,test,clippy${worker:+,$worker}"
fi

if [ -n "$only" ]; then
    IFS=',' read -ra wanted <<<"$only"
    for want in "${wanted[@]}"; do
        found=0
        for name in "${step_names[@]}"; do
            [ "$name" != "$want" ] || found=1
        done
        [ "$found" = 1 ] || die "unknown step $want"
    done
fi

if [ -n "$push_branch" ] && [ -z "$only" ]; then
    dirty=$(cd "$repo" && git status --porcelain) || die "git status failed"
    if [ -n "$dirty" ]; then
        echo "merge-gate: the work tree has uncommitted changes; nothing runs" >&2
        exit 1
    fi
fi

. "$repo/tests/gate-lock.sh"
gate_lock merge-gate || die "cannot take the gate lock"

runs=${MERGE_GATE_RUNS:-/tmp}
mkdir -p "$runs" || die "no run directory"
run_dir=$(mktemp -d "$runs/merge-gate.XXXXXX") || die "no run directory"
echo "run directory: $run_dir"

child=""
stop_child() {
    [ -z "$child" ] || kill "$child" 2>/dev/null
    wait "$child" 2>/dev/null
}
on_signal() {
    stop_child
    echo "merge-gate: interrupted" >&2
    exit 1
}
trap on_signal INT TERM
trap stop_child EXIT

log_failed() {
    grep -Eq '^(FAIL:|MISMATCH|test result: FAILED)' "$1"
}

for i in "${!step_names[@]}"; do
    name=${step_names[$i]}
    selected "$name" || continue
    log=$run_dir/$name.log
    started=$SECONDS
    (cd "$repo" && exec bash -c "${step_commands[$i]}") >"$log" 2>&1 &
    child=$!
    wait "$child"
    code=$?
    child=""
    result=PASS
    if [ "$code" != 0 ] || log_failed "$log"; then
        result=FAIL
    fi
    printf '%-10s %s %5ss %s\n' "$name" "$result" "$((SECONDS - started))" "$log"
    if [ "$result" = FAIL ]; then
        echo "exit code $code; last 30 lines of $log:"
        tail -n 30 "$log"
        exit 1
    fi
done

if [ -n "$only" ]; then
    echo "partial run: nothing is pushed"
    exit 0
fi
if [ -z "$push_branch" ]; then
    echo "all steps passed; no --push, nothing is pushed"
    exit 0
fi
cd "$repo" && git push origin "HEAD:$push_branch"
