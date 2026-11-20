#!/bin/bash
# Self-test of tools/cut-check on a scratch workspace: a plugin to plugin
# dependency (normal, dev or build, in a nested crate too), an extra
# workspace dependency, an asset outside the crate, a link that leaves the
# crate, a costume of another look in a huds folder, a foreign plugin name
# beyond the count of tools/cut-known.toml and a copy row at max 0 that
# still matches fail; a grown copy row and a count below its known max are
# reported; a new match arm on a shared enum is reported with --base.
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
run() {
    "$tool" --root "$work" >"$work/out" 2>&1
    local got=$?
    ! grep -q Traceback "$work/out" || fail "cut-check crashed: $(cat "$work/out")"
    return $got
}
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

printf '[dev-dependencies]\nbeta = { path = "../beta" }\n' >>"$work/plugins/alpha/Cargo.toml"
run && fail "plugin to plugin dev-dependency passes"
want_line '^FAIL: cut-check: deps: alpha depends on beta' "dev-deps"
sed -i '/^\[dev-dependencies\]/,$d' "$work/plugins/alpha/Cargo.toml"

printf '[build-dependencies]\nbeta = { path = "../beta" }\n' >>"$work/plugins/alpha/Cargo.toml"
run && fail "plugin to plugin build-dependency passes"
want_line '^FAIL: cut-check: deps: alpha depends on beta' "build-deps"
sed -i '/^\[build-dependencies\]/,$d' "$work/plugins/alpha/Cargo.toml"

crate plugins/alpha/nested alpha-nested
sed -i 's|"crates/\*"|"crates/*", "plugins/alpha/nested"|' "$work/Cargo.toml"
printf '[dependencies]\nbeta = { path = "../../beta" }\n' >>"$work/plugins/alpha/nested/Cargo.toml"
run && fail "plugin to plugin dependency of a nested crate passes"
want_line '^FAIL: cut-check: deps: alpha-nested depends on beta' "nested deps"
rm -rf "$work/plugins/alpha/nested"
sed -i 's|, "plugins/alpha/nested"||' "$work/Cargo.toml"

printf 'other = { path = "../../crates/other" }\n' >>"$work/plugins/alpha/Cargo.toml"
crate crates/other other
run && fail "extra workspace dependency passes"
want_line '^FAIL: cut-check: packaging: alpha depends on workspace crate other' "packaging"
sed -i '/^other = /d' "$work/plugins/alpha/Cargo.toml"

echo 'const A: &str = include_str!("../../../README.md");' >>"$work/plugins/alpha/src/main.rs"
run && fail "asset outside the crate passes"
want_line '^FAIL: cut-check: assets: alpha .*README.md' "assets"
printf '[[known]]\ncheck = "assets"\ncrate = "alpha"\nmax = 1\n' >"$work/tools/cut-known.toml"
run || fail "an asset breach within its known count blocks: $(cat "$work/out")"
want_line '^REPORT: cut-check: assets: alpha .*README.md' "assets known"
sed -i 's/^max = 1/max = 2/' "$work/tools/cut-known.toml"
run || fail "a count below its known max blocks"
want_line '^REPORT: cut-check: known: alpha assets: 1, lower max from 2' "assets lower max"
sed -i 's/^max = 2/max = 0/' "$work/tools/cut-known.toml"
run && fail "an asset breach above its known count passes"
rm "$work/tools/cut-known.toml"
sed -i '/include_str/d' "$work/plugins/alpha/src/main.rs"

mkdir -p "$work/plugins/alpha/huds/neutral" "$work/plugins/beta/hud"
echo "hud: 1" >"$work/plugins/alpha/huds/neutral/hud.yaml"
echo "hud: 1" >"$work/plugins/beta/hud/hud.yaml"
run || fail "the neutral costume and a costume folder of the look block: $(cat "$work/out")"
ln -s ../../beta/hud "$work/plugins/alpha/huds/beta"
run && fail "a link that leaves the crate passes"
want_line '^FAIL: cut-check: assets: alpha huds/beta is a link that leaves the crate' "link"
rm "$work/plugins/alpha/huds/beta"
mkdir "$work/plugins/alpha/huds/beta"
echo "hud: 1" >"$work/plugins/alpha/huds/beta/hud.yaml"
run && fail "a costume of another look in huds passes"
want_line '^FAIL: cut-check: assets: alpha huds/beta is not the neutral costume' "costume"
rm -rf "$work/plugins/alpha/huds" "$work/plugins/beta/hud"

echo 'fn x() { let _ = "beta"; }' >>"$work/plugins/alpha/src/main.rs"
run && fail "a foreign plugin name passes"
want_line '^FAIL: cut-check: names: alpha names beta on 1 lines' "names"
printf '[[known]]\ncheck = "names"\ncrate = "alpha"\nother = "beta"\nmax = 1\n' >"$work/tools/cut-known.toml"
run || fail "a name within its known count blocks: $(cat "$work/out")"
want_line '^REPORT: cut-check: names: alpha names beta on 1 lines' "names known"
sed -i 's/^max = 1/max = 3/' "$work/tools/cut-known.toml"
run || fail "a names count below its known max blocks"
want_line '^REPORT: cut-check: known: alpha names beta: 1, lower max from 3' "names lower max"
rm "$work/tools/cut-known.toml"
sed -i '/"beta"/d' "$work/plugins/alpha/src/main.rs"

printf 'fn y() { copied(); }\nfn z() { copied(); }\n' >>"$work/plugins/beta/src/main.rs"
printf '[[row]]\nname = "copied"\npattern = "copied"\nallow = []\nmax = 1\n' >"$work/tools/copies.toml"
run || fail "grown copy row blocks"
want_line '^REPORT: cut-check: copies: copied: grew to 2, max 1' "copies grown"
sed -i 's/^max = 1/max = 0/' "$work/tools/copies.toml"
run && fail "copy row at max 0 passes"
want_line '^FAIL: cut-check: copies: copied' "copies at 0"
rm "$work/tools/copies.toml"

mkdir -p "$work/demo/tour/chapters"
: >"$work/demo/tour/tour.py"
: >"$work/demo/tour/chapters/xp.py"
printf '[[row]]\nname = "tour copy"\nfiles = "(^|/)v[0-9]+_[a-z0-9]+[.]py$"\npaths = ["demo/tour"]\nmax = 0\n' >"$work/tools/copies.toml"
run || fail "the tour with no v*_ copy fails: $(cat "$work/out")"
: >"$work/demo/tour/v9_xp.py"
run && fail "a v9_ copy of a tour script passes"
want_line '^FAIL: cut-check: copies: tour copy: 1 left' "tour copy by name"
rm "$work/demo/tour/v9_xp.py" "$work/tools/copies.toml"
sed -i '/copied/d' "$work/plugins/beta/src/main.rs"

printf 'pub enum System { Quit, Zoom }\n' >"$work/crates/chora-kit/src/lib.rs"
printf 'enum Local { Zoom }\n' >>"$work/plugins/beta/src/main.rs"
printf '/target\n' >"$work/.gitignore"
git -C "$work" init -q
git -C "$work" add -A
git -C "$work" -c user.name=t -c user.email=t@t commit -qm base
printf 'fn arms(s: System) { match s {\n    System::Quit => {}\n    System::Zoom => {}\n} }\n' >>"$work/plugins/alpha/src/main.rs"
printf 'fn local(s: Local) { match s {\n    Local::Zoom => {}\n} }\n' >>"$work/plugins/beta/src/main.rs"
"$tool" --root "$work" >"$work/out" 2>&1 || fail "arms without a base block: $(cat "$work/out")"
! grep -q 'variants:' "$work/out" || fail "variants reported without --base: $(cat "$work/out")"
"$tool" --root "$work" --base HEAD >"$work/out" 2>&1 || fail "new arms with --base block: $(cat "$work/out")"
want_line '^REPORT: cut-check: variants: alpha acts on System::Quit (new arm in plugins/alpha/src/main.rs)' "arm Quit"
want_line '^REPORT: cut-check: variants: alpha acts on System::Zoom (new arm in plugins/alpha/src/main.rs)' "arm Zoom"
! grep -q 'Local::Zoom' "$work/out" || fail "an enum that is not in the kit is reported: $(cat "$work/out")"
git -C "$work" add -A
git -C "$work" -c user.name=t -c user.email=t@t commit -qm arms
"$tool" --root "$work" --base HEAD >"$work/out" 2>&1 || fail "no change with --base blocks"
! grep -q 'variants:' "$work/out" || fail "an arm of the base is reported: $(cat "$work/out")"

[ "$failures" = 0 ] || exit 1
echo "ok: cut-check-selftest"
