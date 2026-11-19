#!/bin/bash
# Self-test of tests/cells-for.sh: in a temp git repo with a stub
# merge-gate.sh, a change to a plugin picks the cells that name its binary,
# a change to a gate script or a lib it sources picks its cells, a change
# to the core picks every cell, and a docs-only change picks the fixed five.
#
# Usage: tests/cells-for-selftest.sh
# shellcheck disable=SC2016 # the stub scripts hold a literal $0
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
fails=0
fail() { echo "FAIL: $*"; fails=1; }

[ -x "$repo/tests/cells-for.sh" ] || { echo "FAIL: tests/cells-for.sh is missing"; exit 1; }

g=(git -c user.name=t -c user.email=t@example.org -c commit.gpgsign=false)
cd "$work" || exit 1
git init -q -b main . || exit 1
mkdir -p tests plugins/chora-notify plugins/chora-xp services/voice crates/chora docs
cp "$repo/tests/cells-for.sh" tests/
cat >tests/merge-gate.sh <<'STUB'
#!/bin/bash
[ "${1:-}" = --commands ] || exit 2
cat <<'STEPS'
red-first|tests/merge-gate.sh --red-first-check
identity|tests/merge-gate.sh --identity-check
pre-commit|pre-commit run
gate-lib|tests/gate-lib-selftest.sh
build|cargo build
test|cargo test
clippy|cargo clippy
ci-check|tests/ci-a.sh
xp|tests/xp.sh target/debug
hud|BIN=x tests/gate-hud.sh
chat|BIN=x tests/gate-chat.sh
STEPS
STUB
chmod +x tests/merge-gate.sh tests/cells-for.sh
echo '. "$(dirname "$0")/hud-lib.sh"' >tests/gate-hud.sh
echo 'chora-voice --x' >tests/gate-chat.sh
echo 'chora-notify prints' >tests/xp.sh
echo 'x' >tests/hud-lib.sh
echo 'a' >tests/ci-a.sh
echo 'a' >tests/gate-lib-selftest.sh
echo a >plugins/chora-notify/a.rs
echo a >plugins/chora-xp/a.rs
echo a >services/voice/a.rs
echo a >crates/chora/a.rs
echo a >docs/a.md
"${g[@]}" add -A && "${g[@]}" commit -qm base || exit 1

fixed=pre-commit,ci-check,build,test,clippy
case_() {
    local name=$1 file=$2 want=$3 got
    git checkout -q -B work main
    echo change >>"$file"
    "${g[@]}" commit -qam "$name"
    got=$(tests/cells-for.sh --base main | tr ',' '\n' | sort | paste -sd,)
    want=$(tr ',' '\n' <<<"$want" | sort | paste -sd,)
    [ "$got" = "$want" ] || fail "$name: got $got, want $want"
}
case_ "notify picks xp" plugins/chora-notify/a.rs "$fixed,xp"
case_ "hud script picks hud" tests/gate-hud.sh "$fixed,hud"
case_ "hud lib picks hud" tests/hud-lib.sh "$fixed,hud"
case_ "voice picks chat" services/voice/a.rs "$fixed,chat"
case_ "xp plugin picks nothing extra" plugins/chora-xp/a.rs "$fixed"
case_ "docs only" docs/a.md "$fixed"
all=$(tests/merge-gate.sh --commands | cut -d'|' -f1 | paste -sd,)
case_ "core picks all" crates/chora/a.rs "$all"

[ "$fails" = 0 ] && echo "cells-for-selftest: ok"
exit "$fails"
