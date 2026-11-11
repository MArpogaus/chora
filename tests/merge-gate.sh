#!/bin/bash
# The one command that decides whether a merge into the session branch
# may be pushed. It runs the steps in order, each without a pipe and with
# its whole output in its own log, and stops at the first step that failed.
#
# Steps: build, test, clippy, 2d-guard, xp, matrix, virtual, xp-faces, hud,
# last-words, cube, pull. `matrix` ends with the click that chooses a
# window manager for a new window; `virtual` hands a window over between window
# managers on a screen and on virtual outputs; `xp-faces` runs one xp on
# four virtual outputs; `cube` shows that xp on the faces of a cube in the
# construct; `pull` drags a window out of a face of that cube and back.
# A step fails when its exit code is not 0, when its log has a line that
# starts with `FAIL:` or `MISMATCH`, or when it has the line
# `test result: FAILED`. One line per step: name, PASS or FAIL, seconds,
# log path. A failed step prints the last 30 lines of its log.
#
# Usage: tests/merge-gate.sh [--only STEP,...] [--push BRANCH]
#   --only STEP,...  run only these steps; never pushes
#   --push BRANCH    after every step passed, `git push origin HEAD:BRANCH`;
#                    refuses when the work tree has uncommitted changes
#   -h, --help       this text
#   MERGE_GATE_STEPS  file with one `name|command` per line, instead of the
#                     steps above (the self-test uses it)
#   MERGE_GATE_RUNS   directory for the run directories (default: /tmp)
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
push_branch=""
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
    --push)
        [ $# -ge 2 ] || die "--push needs a branch"
        push_branch=$2
        shift 2
        ;;
    *) die "unknown argument $1" ;;
    esac
done

default_steps() {
    cat <<STEPS
build|cargo build --workspace
test|cargo test --workspace
clippy|cargo clippy --workspace --all-targets -- -D warnings
2d-guard|tests/2d-guard.sh 8742dee
xp|tests/xp.sh "$repo/target/debug"
matrix|tests/gate-handover-matrix.sh && tests/gate-give-holders.sh && BIN=$repo/target/debug tests/gate-click-routes.sh
virtual|BIN=$repo/target/debug tests/gate-handover-virtual.sh
xp-faces|BIN=$repo/target/debug tests/gate-xp-faces.sh
hud|BIN=$repo/target/debug tests/gate-hud.sh
last-words|CHORA_BIN=$repo/target/debug tests/last-words.sh
cube|BIN=$repo/target/debug tests/gate-construct-cube.sh
pull|BIN=$repo/target/debug tests/gate-construct-pull.sh
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
