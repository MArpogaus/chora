#!/bin/bash
# Self-test of tools/cut-check on a scratch workspace: a plugin to plugin
# dependency fails; an asset outside the crate, a foreign plugin name, an
# extra workspace dependency and a grown copy row are reported; a copy row
# at max 0 that still matches fails.
#
# Usage: tests/cut-check-selftest.sh
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
tool=$repo/tools/cut-check
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

fail() {
    echo "FAIL: $*"
    failures=$((failures + 1))
}
run() { "$tool" --root "$work" >"$work/out" 2>&1; }
want_line() { grep -q "$1" "$work/out" || fail "$2: no line matching '$1': $(cat "$work/out")"; }

crate() {
    local dir=$1 name=$2
    mkdir -p "$work/$dir/src"
    printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2021"\n' "$name" >"$work/$dir/Cargo.toml"
    echo "fn main() {}" >"$work/$dir/src/main.rs"
}

printf '[workspace]\nmembers = ["plugins/*", "crates/*"]\nresolver = "2"\n' >"$work/Cargo.toml"
mkdir -p "$work/tools"
crate crates/chora-kit chora-kit
crate plugins/alpha alpha
crate plugins/beta beta
printf '[dependencies]\n' >>"$work/plugins/alpha/Cargo.toml"

run || fail "clean workspace fails: $(cat "$work/out")"
grep -q '^ok: cut-check' "$work/out" || fail "clean workspace prints no ok"

printf 'chora-kit = { path = "../../crates/chora-kit" }\n' >>"$work/plugins/alpha/Cargo.toml"
run || fail "shared dependency blocks: $(cat "$work/out")"

printf 'beta = { path = "../beta" }\n' >>"$work/plugins/alpha/Cargo.toml"
run && fail "plugin to plugin dependency passes"
want_line '^FAIL: cut-check: deps: alpha depends on beta' "deps"
sed -i '/^beta = /d' "$work/plugins/alpha/Cargo.toml"

printf 'other = { path = "../../crates/other" }\n' >>"$work/plugins/alpha/Cargo.toml"
crate crates/other other
run || fail "extra workspace dependency blocks"
want_line '^REPORT: cut-check: packaging: alpha depends on workspace crate other' "packaging"
sed -i '/^other = /d' "$work/plugins/alpha/Cargo.toml"

echo 'const A: &str = include_str!("../../../README.md");' >>"$work/plugins/alpha/src/main.rs"
echo 'fn x() { let _ = "beta"; }' >>"$work/plugins/alpha/src/main.rs"
run || fail "asset and name reports block"
want_line '^REPORT: cut-check: assets: alpha .*README.md' "assets"
want_line '^REPORT: cut-check: names: alpha names beta' "names"

printf 'fn y() { copied(); }\nfn z() { copied(); }\n' >>"$work/plugins/beta/src/main.rs"
printf '[[row]]\nname = "copied"\npattern = "copied"\nallow = []\nmax = 1\n' >"$work/tools/copies.toml"
run || fail "grown copy row blocks"
want_line '^REPORT: cut-check: copies: copied: grew to 2, max 1' "copies grown"
sed -i 's/^max = 1/max = 0/' "$work/tools/copies.toml"
run && fail "copy row at max 0 passes"
want_line '^FAIL: cut-check: copies: copied' "copies at 0"

[ "$failures" = 0 ] || exit 1
echo "ok: cut-check-selftest"
