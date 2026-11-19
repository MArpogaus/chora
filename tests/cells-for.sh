#!/bin/bash
# The gate cells a diff touches. Prints a comma list for the `--only` option
# of tests/merge-gate.sh; it always holds ci-check.
#
# 1. always pre-commit, ci-check, build, test, clippy;
# 2. each cell whose command, or a tests/ file it names (and the files those
#    name, so the libs they source), is a changed file;
# 3. each cell whose command or such a file names the binary of a changed crate
#    (plugins/chora-X -> chora-X, services/voice -> chora-voice);
#    a change to tests/merge-gate.sh or tests/gate-lib* also picks gate-lib;
# 4. a change to crates/chora, crates/chora-kit, crates/chora-scene, or to a
#    Cargo.toml or Cargo.lock at the root picks every cell.
# A cfg(test)-only Rust change picks cells like any other: correctness over speed.
#
# Usage: tests/cells-for.sh [--base REF] [--head REF]
#   --base REF  default origin/claude/autonomous-dev-session-3w-i33pg7
#   --head REF  compare REF with the base; default: the work tree against
#               the merge base of the base and HEAD, plus untracked files
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
base=origin/claude/autonomous-dev-session-3w-i33pg7
head=""
while [ $# -gt 0 ]; do
    case $1 in
    --base)
        base=${2:?--base needs a ref}
        shift 2
        ;;
    --head)
        head=${2:?--head needs a ref}
        shift 2
        ;;
    -h | --help)
        sed -n '2,/^set -u/p' "$0" | sed '$d; s/^# \{0,1\}//'
        exit 0
        ;;
    *)
        echo "cells-for: unknown argument $1" >&2
        exit 2
        ;;
    esac
done

cd "$repo" || exit 2
if [ -n "$head" ]; then
    changed=$(git diff --name-only "$base" "$head") || exit 2
else
    mb=$(git merge-base "$base" HEAD) || {
        echo "cells-for: no base $base" >&2
        exit 2
    }
    changed=$(git diff --name-only "$mb") || exit 2
    changed+=$'\n'$(git ls-files --others --exclude-standard)
fi

cmds=$(tests/merge-gate.sh --commands --base "$base") || exit 2

fixed=(pre-commit ci-check build test clippy)
all=0
bins=()
while IFS= read -r f; do
    case $f in
    crates/chora/* | crates/chora-kit/* | crates/chora-scene/* | Cargo.toml | Cargo.lock) all=1 ;;
    plugins/chora-*/*)
        f=${f#plugins/}
        bins+=("${f%%/*}")
        ;;
    services/*/*)
        f=${f#services/}
        bins+=("chora-${f%%/*}")
        ;;
    crates/*/*)
        f=${f#crates/}
        bins+=("${f%%/*}")
        ;;
    esac
done <<<"$changed"

files_of() {
    grep -Eo '(tests|tools)/[A-Za-z0-9._/-]+|[A-Za-z0-9._-]+\.(sh|py)' <<<"$1" | while read -r t; do
        case $t in
        */*) echo "$t" ;;
        *) [ -f "tests/$t" ] && echo "tests/$t" ;;
        esac
    done | sort -u
}

picked=()
while IFS='|' read -r name cmd; do
    [ -n "$name" ] || continue
    if [ "$all" = 1 ]; then
        picked+=("$name")
        continue
    fi
    seen=$'\n'
    todo=$(files_of "$cmd")
    text=$cmd
    while [ -n "$todo" ]; do
        next=""
        while IFS= read -r t; do
            [ -n "$t" ] || continue
            case $seen in *$'\n'"$t"$'\n'*) continue ;; esac
            seen+="$t"$'\n'
            if [ -f "$t" ] && [ "$t" != tests/merge-gate.sh ]; then
                body=$(cat "$t")
                text+=$'\n'$body
                next+=$(files_of "$body")$'\n'
            fi
        done <<<"$todo"
        todo=$next
    done
    hit=0
    while IFS= read -r f; do
        [ -n "$f" ] || continue
        case $seen in *$'\n'"$f"$'\n'*)
            hit=1
            break
            ;;
        esac
        while IFS= read -r t; do
            [ -n "$t" ] && [ "${f#"$t"/}" != "$f" ] && hit=1
        done <<<"$seen"
        [ "$hit" = 1 ] && break
    done <<<"$changed"
    if [ "$hit" = 0 ]; then
        for b in "${bins[@]+"${bins[@]}"}"; do
            if grep -Eq "(^|[^A-Za-z0-9_-])$b([^A-Za-z0-9_-]|\$)" <<<"$text"; then
                hit=1
                break
            fi
        done
    fi
    if [ "$hit" = 0 ] && [ "$name" = gate-lib ] && grep -Eq '^tests/(merge-gate\.sh|gate-lib)' <<<"$changed"; then
        hit=1
    fi
    [ "$hit" = 1 ] && picked+=("$name")
done <<<"$cmds"

out=()
for n in "${fixed[@]}" "${picked[@]+"${picked[@]}"}"; do
    case " ${out[*]-} " in *" $n "*) ;; *) out+=("$n") ;; esac
done
(
    IFS=,
    echo "${out[*]}"
)
