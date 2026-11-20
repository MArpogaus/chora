#!/bin/bash
# The one command that decides whether a merge into the session branch
# may be pushed. It runs the steps in order, each without a pipe and with
# its whole output in its own log, and stops at the first step that failed.
#
# One gate runs at a time: the gate and the gate scripts take the lock
# of tests/gate-lock.sh, and a second one prints a line and waits.
#
# Steps: red-first, identity, pre-commit, gate-lib, ci-check, build, test, clippy, 2d-guard, package-deps, package-lists, xp, matrix, virtual, xp-faces, xp-paperclip, xp-fields, xp-question, hud,
# layout, tiling-audit, tiling-look, tiling-empty, chat-keys, doors, acts, last-words, cube, pull, placed, construct-hud, construct-extra, out, winit, wm-pane, pane-keys, walk, keys, pixels, mc-paths, mc-sofa, mc-column, mc-handover, launcher-click, voice-acp, voice-events, agent, e2e. `layout` audits
# the layout of the construct HUD in each state. `tiling-empty` shows that an output and a pane of tiling with no window have the bar and the wallpaper, also with `--with-companions` alone. `chat-keys` opens the
# chat in tiling with Super+a and types into its field. `doors` asks every
# look for the shared keys and one visible door per action. `acts` runs the act contracts of
# tests/acts with the invariants I1 to I14 (no I11 yet) on state.get and scene.boxes against tests/acts/known-red.toml. `matrix` ends with the click that
# chooses a window manager for a new window; `virtual` hands a window over between window
# managers on a screen and on virtual outputs; `xp-faces` runs one xp on
# four virtual outputs; `cube` shows that xp on the faces of a cube in the
# construct; `pull` drags a window out of a face of that cube and back; `out` turns the stood-back cube by a drag on a window, shows its keys and closes it.
# `winit` runs a virtual output with xp in a chora nested on the winit
# backend; `wm-pane` runs xp and tiling each in a pane of the construct; `pane-keys` gives the keys to the pane that a click chose; `walk` taps the walk keys of the
# construct and reads the pose of the camera at exact times; `keys` types text
# into the space, starts the terminal and the launcher, closes panes and
# checks the grabs after a flight into a pane.  `pixels` reads the labels, a pane
# beside a cube and the 2x step-in from captures. `mc-paths` runs the cells of the main paths of chora-mc. `mc-sofa` runs the cells of the sofa and kitchen scenarios of chora-mc. `mc-column` runs the margin column of chora-mc at 1280x720 and 1920x1080. `mc-handover` runs the cells of a window that goes to mc. `launcher-click` clicks a row of the Apps launcher of tiling and of the Programs launcher of the construct with the hands of a persona (act.py) and asks for the program of that row. `construct-extra` runs the construct gates K1 (a pane at 1:1), mcpane, nested, k2a, windows and 3d (K5, direct input on the 3D base). `e2e` runs the scripts of the e2e job of
# .github/workflows/ci.yaml in its order (tests/e2e-sequence.sh); a skip fails the step unless tests/e2e-skips.txt lists it.
# `voice-acp` runs the ACP sub-agent smoke test (a question that edits a file is answered by a HUD button, not by a spoken yes), `voice-events` the voice.state events, and `agent` the agent interface in a headless sway (tests/agent.sh).
# `identity` fails when any commit of BASE..HEAD, merges included, has an author or
# a committer other than the identity in `expected_identity` below (a lane hook has
# re-authored commits as another identity); it prints one FAIL line with the sha, the
# role and the identity found for each.
# `red-first` checks the range BASE..HEAD (commits that are not merges, oldest
# first, in topological order): each `fix` or `feat` commit needs a commit
# of the range that changes a test before it or in it, so a test may come in
# the same commit as the fix. A test is a Rust file under a tests/ directory,
# a Rust file with an added #[test] line, tests/gate-*.sh (not gate-lib.sh
# or gate-lock.sh), tests/*-selftest.sh or tests/test_*.py; helper scripts
# such as tests/gate-lib.sh and tests/ci-local.sh do not count (the rule
# lives in tests/red-first.sh, which the commit-msg hook runs). Else the step
# prints RED-FIRST MISSING with the subject and fails, unless the commit body
# has a line `No test: <reason>`. `pre-commit` runs the hooks over the same
# range, with SKIP=cargo-clippy,lychee,vale as CI does, and fails when
# pre-commit is not installed.
# `ci-check` also runs tools/size-ratchet (a file over 1,500 production lines may not grow) and
# tools/cut-check (blocking: a plugin depending on a plugin or an extra workspace crate, an asset or
# link that leaves its crate, a name of another plugin beyond tools/cut-known.toml; it reports the
# new match arms on a kit enum since the merge base), each with its self-test.
# A step that passed but whose log has a line `SKIP: <reason>` did not check what it
# is for (a tool missing or too old, a cell that does not run): its line says SKIP, not
# PASS, and the summary after the steps lists each SKIP step with its reasons. A train
# stays honest: `--push` refuses while any step is SKIP unless the step is named by
# `--allow-skip` (a host gap the owner knows, e.g. `--allow-skip launcher-click`).
# A step fails when its exit code is not 0, when its log has a line that
# starts with `FAIL:` or `MISMATCH`, or when it has the line
# `test result: FAILED`. One line per step: name, PASS or FAIL, seconds,
# log path. A failed step prints the last 30 lines of its log.
# With --jobs N, the steps after 2d-guard run up to N at once: each gate
# script has its own sockets and runtime directory and judges its
# programs by pid (the hook no-foreign-kill), and the steps mostly wait,
# so N=3 on 4 CPUs keeps their times. red-first to clippy and 2d-guard (a
# cold cargo build of its base) run one at a time. A line is printed when
# a step ends; after a failed step no further step starts and the running
# ones are stopped: each step runs in its own process group, and the gate ends
# the whole group of each running step on exit (TERM, KILL after 5 s) and waits.
#
# Usage: tests/merge-gate.sh [--only STEP,...] [--worker STEP,...] [--push BRANCH]
#                            [--base REF] [--jobs N] [--allow-skip STEP,...] [--red-first-check]
#                            [--identity-check] [--list] [--commands]
#   --only STEP,...  run only these steps; never pushes
#   --worker STEP,...  the lane mode: build, test, clippy and the steps
#                    named here (`--worker xp,hud`; `--worker ''` for none);
#                    never pushes
#   --base REF       the base of the merged range (default
#                    origin/claude/autonomous-dev-session-3w-i33pg7)
#   --push BRANCH    after every step passed, `git push origin HEAD:BRANCH`;
#                    refuses when the work tree has uncommitted changes
#   --allow-skip STEP,...  steps that may be SKIP when --push runs; a SKIP
#                    step not named here makes --push refuse
#   --jobs N         run up to N steps after 2d-guard at once (default 1)
#   --red-first-check  run only the red-first check, without the lock
#   --identity-check run only the identity check, without the lock
#   --list           print the step names, one per line
#   --commands       print `name|command` of each step, one per line
#   -h, --help       this text
#   MERGE_GATE_IDENTITY  the expected identity, instead of expected_identity
#   MERGE_GATE_NESTED  set by the self-test's own gate-lib run: the gate-lib
#                     step then skips merge-gate-selftest.sh (no recursion)
#   MERGE_GATE_STEPS  file with one `name|command` per line, instead of the
#                     steps above (the self-test uses it)
#   MERGE_GATE_RUNS   directory for the run directories (default: /tmp)
#   CHORA_GATE_LOCK   the lock file of the gates (see tests/gate-lock.sh)
# A run that starts has at least 6 GB free on / (D90): under that the gate prints df and exits 1.
# A shallow clone fails at once (git fetch --unshallow first, R18).
# When the session branch of BASE exists here, the gate fetches it and warns when its head differs
# from the head on origin (a drift of the container).
# Exit status: 0 all steps passed, 1 a step failed or the push was
# refused, 2 a usage error.
set -u

expected_identity="Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>"

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
id_check=""
list_steps=""
list_commands=""
jobs_max=1
allow_skip=""
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
    --identity-check)
        id_check=1
        shift
        ;;
    --list)
        list_steps=1
        shift
        ;;
    --commands)
        list_commands=1
        shift
        ;;
    --jobs)
        [[ ${2:-} =~ ^[1-9][0-9]*$ ]] || die "--jobs needs a number above 0"
        jobs_max=$2
        shift 2
        ;;
    --allow-skip)
        [ $# -ge 2 ] || die "--allow-skip needs a list of steps"
        allow_skip=$2
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

. "$repo/tests/red-first.sh"

red_first() {
    local missing=0 seen=0 sha subject body
    git -C "$repo" rev-parse --verify -q "$base" >/dev/null || die "no base ref $base"
    while read -r sha; do
        [ -n "$sha" ] || continue
        subject=$(git -C "$repo" log -1 --format=%s "$sha")
        (cd "$repo" && commit_changes_test "$sha") && seen=1
        needs_test "$subject" || continue
        [ "$seen" = 0 ] || continue
        body=$(git -C "$repo" log -1 --format=%b "$sha")
        grep -Eq '^No test: .+' <<<"$body" && continue
        echo "RED-FIRST MISSING: $(git -C "$repo" log -1 --format='%h %s' "$sha")"
        missing=1
    done < <(git -C "$repo" rev-list --reverse --no-merges --topo-order "$base..HEAD")
    [ "$missing" = 0 ] && echo "red-first: every fix and feat commit has a test before it or in it"
    return "$missing"
}

identity() {
    local want=${MERGE_GATE_IDENTITY:-$expected_identity} bad=0 sha author committer
    git -C "$repo" rev-parse --verify -q "$base" >/dev/null || die "no base ref $base"
    [ -z "${MERGE_GATE_IDENTITY:-}" ] || echo "identity: MERGE_GATE_IDENTITY overrides the expected identity: $want"
    while IFS=$'\t' read -r sha author committer; do
        [ -n "$sha" ] || continue
        if [ "$author" != "$want" ]; then
            echo "FAIL: $sha author is $author, want $want"
            bad=1
        fi
        if [ "$committer" != "$want" ]; then
            echo "FAIL: $sha committer is $committer, want $want"
            bad=1
        fi
    done < <(git -C "$repo" log --format='%h%x09%an <%ae>%x09%cn <%ce>' "$base..HEAD")
    [ "$bad" = 0 ] && echo "identity: every commit of $base..HEAD has the identity $want"
    return "$bad"
}

if [ -n "$red_check" ]; then
    red_first
    exit $?
fi
if [ -n "$id_check" ]; then
    identity
    exit $?
fi

default_steps() {
    cat <<STEPS
red-first|tests/merge-gate.sh --red-first-check --base "$base"
identity|tests/merge-gate.sh --identity-check --base "$base"
pre-commit|command -v pre-commit >/dev/null || { echo "FAIL: pre-commit is not installed"; exit 1; }; SKIP=cargo-clippy,lychee,vale pre-commit run --from-ref "$base" --to-ref HEAD
gate-lib|{ tests/gate-lib-selftest.sh && tests/under-load-selftest.sh && tests/cells-for-selftest.sh && tests/nested-diag-selftest.sh; } & a=\$!; [ x\${MERGE_GATE_NESTED:-} != x ] || tests/merge-gate-selftest.sh; b=\$?; wait \$a && [ \$b = 0 ]
ci-check|tools/cut-check --base "$(git merge-base "$base" HEAD)" && tests/cut-check-selftest.sh && tests/grab-gap-check.py && tests/grab-gap-check-selftest.sh && tests/agent-sandbox.sh && tests/gate-0.7-9.sh && tests/gate-0.8-5.sh && tests/gate-0.8-8.sh && tests/gate-0.8-plugin-runtime.sh && tests/packaging-files.py && tests/flavor-notify.py && tests/screenshot-notice.sh && tests/tiling-scripts-selftest.sh && tests/packaging-files-selftest.sh && tools/size-ratchet --base "$(git merge-base "$base" HEAD)" && tests/size-ratchet-selftest.sh
build|cargo build --workspace --locked
test|cargo test --workspace --locked
clippy|cargo clippy --workspace --all-targets --locked -- -D warnings
2d-guard|tests/2d-guard.sh 8742dee
package-deps|tests/gate-package-deps.sh
package-lists|tests/gate-package-lists.sh
xp|tests/xp.sh "$repo/target/debug"
matrix|tests/gate-handover-matrix.sh && tests/gate-give-holders.sh && BIN=$repo/target/debug tests/gate-click-routes.sh && BIN=$repo/target/debug tests/gate-injected-presses.sh
virtual|BIN=$repo/target/debug tests/gate-handover-virtual.sh
xp-faces|BIN=$repo/target/debug tests/gate-xp-faces.sh
xp-paperclip|BIN=$repo/target/debug tests/gate-xp-paperclip.sh
xp-fields|BIN=$repo/target/debug tests/gate-xp-fields.sh
xp-question|BIN=$repo/target/debug tests/gate-xp-question.sh
xp-notepad|BIN=$repo/target/debug tests/gate-xp-notepad.sh && BIN=$repo/target/debug tests/gate-xp-notepad-fallback.sh && BIN=$repo/target/debug tests/gate-xp-notepad-gui-only.sh
hud|BIN=$repo/target/debug tests/gate-hud.sh
layout|BIN=$repo/target/debug tests/gate-construct-layout.sh
tiling-audit|BIN=$repo/target/debug tests/gate-tiling-audit.sh
tiling-look|BIN=$repo/target/debug tests/gate-tiling-wobbly.sh && BIN=$repo/target/debug tests/gate-tiling-bar-palette.sh && BIN=$repo/target/debug tests/gate-tiling-bar-spec.sh && BIN=$repo/target/debug tests/gate-tiling-look-r2.sh && BIN=$repo/target/debug tests/gate-tiling-power.sh && BIN=$repo/target/debug tests/gate-tiling-print-notice.sh
tiling-empty|BIN=$repo/target/debug tests/gate-tiling-empty-output.sh
chat-keys|BIN=$repo/target/debug tests/gate-chat-keys.sh
doors|BIN=$repo/target/debug tests/gate-doors.sh
acts|BIN=$repo/target/debug tests/gate-acts.sh
last-words|CHORA_BIN=$repo/target/debug tests/last-words.sh
cube|BIN=$repo/target/debug tests/gate-construct-cube.sh
pull|BIN=$repo/target/debug tests/gate-construct-pull.sh
placed|BIN=$repo/target/debug tests/gate-construct-placed.sh
construct-hud|BIN=$repo/target/debug tests/gate-construct-hud.sh
construct-extra|BIN=$repo/target/debug tests/gate-construct-k1.sh && BIN=$repo/target/debug tests/gate-construct-mcpane.sh && BIN=$repo/target/debug tests/gate-construct-nested.sh && BIN=$repo/target/debug tests/gate-construct-k2a.sh && BIN=$repo/target/debug tests/gate-construct-windows.sh && BIN=$repo/target/debug tests/gate-construct-3d.sh
out|BIN=$repo/target/debug tests/gate-construct-out.sh
winit|BIN=$repo/target/debug tests/gate-virtual-winit.sh
wm-pane|BIN=$repo/target/debug tests/gate-construct-wm-pane.sh
pane-keys|BIN=$repo/target/debug tests/gate-construct-pane-keys.sh
walk|BIN=$repo/target/debug tests/gate-construct-walk.sh
keys|BIN=$repo/target/debug tests/gate-construct-keys.sh && BIN=$repo/target/debug tests/gate-probe-hangup.sh
pixels|tests/gate-construct-pixels.sh
mc-paths|BIN=$repo/target/debug tests/gate-mc-paths.sh
mc-sofa|BIN=$repo/target/debug tests/gate-mc-sofa.sh
mc-column|BIN=$repo/target/debug tests/gate-mc-column.sh
mc-handover|BIN=$repo/target/debug tests/gate-mc-handover.sh
launcher-click|BIN=$repo/target/debug tests/gate-launcher-click.sh
voice-acp|BIN=$repo/target/debug tests/voice-acp.sh
voice-events|BIN=$repo/target/debug tests/voice-events.sh
agent|out=\$(mktemp -d /tmp/chora-agent-gate.XXXXXX) && tests/agent.sh "$repo/target/debug" "\$out" && rm -rf "\$out"
e2e|tests/e2e-sequence.sh "$repo/target/debug"
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

if [ -n "$list_commands" ]; then
    printf '%s\n' "$steps"
    exit 0
fi

if [ -n "$list_steps" ]; then
    printf '%s\n' "${step_names[@]}"
    exit 0
fi

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

min_free_gb=6
check_free_space() {
    local free_kb
    free_kb=$(df -Pk / | awk 'NR == 2 {print $4}')
    [ "${free_kb:-0}" -ge $((min_free_gb * 1024 * 1024)) ] && return 0
    df -h /
    echo "merge-gate: only $((${free_kb:-0} / 1024 / 1024)) GB free on /, $min_free_gb GB are needed (D90); free space and run again" >&2
    return 1
}

warn_on_drift() {
    local branch=${base#origin/} local_sha origin_sha
    local_sha=$(git -C "$repo" rev-parse --verify -q "refs/heads/$branch") || return 0
    [ -n "$local_sha" ] || return 0
    timeout 30 git -C "$repo" fetch -q origin "$branch" 2>/dev/null ||
        echo "merge-gate: WARNING: cannot fetch origin $branch, so the drift check uses the last fetch"
    origin_sha=$(git -C "$repo" rev-parse --verify -q "$base") || return 0
    [ "$local_sha" = "$origin_sha" ] ||
        echo "merge-gate: WARNING: drift: $branch is at ${local_sha:0:8}, $base at ${origin_sha:0:8} (D90)"
}

check_full_history() {
    [ "$(git -C "$repo" rev-parse --is-shallow-repository)" != true ] && return 0
    echo "merge-gate: this is a shallow clone; run git fetch --unshallow and start again (R18)" >&2
    return 1
}

check_free_space || exit 1
check_full_history || exit 1
warn_on_drift

. "$repo/tests/gate-lock.sh"
gate_lock merge-gate || die "cannot take the gate lock"

runs=${MERGE_GATE_RUNS:-/tmp}
mkdir -p "$runs" || die "no run directory"
run_dir=$(mktemp -d "$runs/merge-gate.XXXXXX") || die "no run directory"
echo "run directory: $run_dir"

declare -A running=()
declare -A started=()
skipped_steps=()
skipped_reasons=()
group_alive() {
    kill -0 -- "-$1" 2>/dev/null
}
stop_children() {
    local pid tries alive
    for pid in "${!running[@]}"; do
        kill -TERM -- "-$pid" 2>/dev/null
    done
    for ((tries = 0; tries < 50; tries++)); do
        alive=0
        for pid in "${!running[@]}"; do
            ! group_alive "$pid" || alive=1
        done
        [ "$alive" = 1 ] || break
        sleep 0.1
    done
    for pid in "${!running[@]}"; do
        ! group_alive "$pid" || kill -KILL -- "-$pid" 2>/dev/null
    done
    for pid in "${!running[@]}"; do
        wait "$pid" 2>/dev/null
    done
    running=()
}
on_signal() {
    stop_children
    echo "merge-gate: interrupted" >&2
    exit 1
}
trap on_signal INT TERM
trap stop_children EXIT

log_reasons() {
    sed -n 's/^SKIP: //p' "$1" | paste -sd ';' - | sed 's/;/; /g'
}

log_failed() {
    grep -Eq '^(FAIL:|MISMATCH|test result: FAILED)' "$1"
}

start_step() {
    local log=$run_dir/${step_names[$1]}.log
    (cd "$repo" && exec setsid bash -c "${step_commands[$1]}") >"$log" 2>&1 &
    running[$!]=$1
    started[$!]=$SECONDS
}

wait_step() {
    local pid code i name log result=PASS
    wait -n -p pid "${!running[@]}"
    code=$?
    i=${running[$pid]}
    unset "running[$pid]"
    name=${step_names[$i]}
    log=$run_dir/$name.log
    if [ "$code" != 0 ] || log_failed "$log"; then
        result=FAIL
    fi
    if [ "$result" = PASS ] && grep -q '^SKIP: ' "$log"; then
        result=SKIP
        skipped_steps+=("$name")
        skipped_reasons+=("$(log_reasons "$log")")
    fi
    printf '%-10s %-4s %5ss %s\n' "$name" "$result" "$((SECONDS - started[$pid]))" "$log"
    if [ "$result" = FAIL ]; then
        stop_children
        echo "exit code $code; last 30 lines of $log:"
        tail -n 30 "$log"
        exit 1
    fi
}

serial_steps=",red-first,identity,pre-commit,gate-lib,build,test,clippy,2d-guard,"
for i in "${!step_names[@]}"; do
    selected "${step_names[$i]}" || continue
    if [ "$jobs_max" = 1 ] || [[ $serial_steps == *",${step_names[$i]},"* ]]; then
        while [ "${#running[@]}" -gt 0 ]; do wait_step; done
        start_step "$i"
        wait_step
        continue
    fi
    while [ "${#running[@]}" -ge "$jobs_max" ]; do wait_step; done
    start_step "$i"
done
while [ "${#running[@]}" -gt 0 ]; do wait_step; done

for i in "${!skipped_steps[@]}"; do
    echo "skipped: ${skipped_steps[$i]} (${skipped_reasons[$i]})"
done

if [ -n "$only" ]; then
    echo "partial run: nothing is pushed"
    exit 0
fi
if [ -z "$push_branch" ]; then
    echo "all steps passed; no --push, nothing is pushed"
    exit 0
fi
refused=()
for name in "${skipped_steps[@]}"; do
    case ",$allow_skip," in *",$name,"*) ;; *) refused+=("$name") ;; esac
done
if [ "${#refused[@]}" -gt 0 ]; then
    refused_list=$(IFS=,; echo "${refused[*]}")
    echo "merge-gate: refusing to push: ${refused[*]} skipped; pass --allow-skip $refused_list to push with a host gap you know" >&2
    exit 1
fi
cd "$repo" && git push origin "HEAD:$push_branch"
