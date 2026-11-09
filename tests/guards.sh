#!/usr/bin/env bash
# Prove the pre-commit guards red and green in a temporary git repository.
# Usage: tests/guards.sh   (run by hand; exits 1 if a guard misbehaves)
set -u
tests_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

new_repo() {
  rm -rf "${work:?}/repo"
  mkdir "$work/repo"
  git -C "$work/repo" init -q
  git -C "$work/repo" config user.email guards@example.com
  git -C "$work/repo" config user.name guards
}

verdict() {
  local label=$1 want=$2 status=$3
  local got=pass
  [ "$status" -ne 0 ] && got=fail
  if [ "$got" = "$want" ]; then
    echo "ok   $label ($want)"
  else
    echo "FAIL $label (wanted $want, exit $status)"
    failures=$((failures + 1))
  fi
}

comment_case() {
  local label=$1 want=$2 path=$3 base=$4 added=$5
  new_repo
  cd "$work/repo" || exit 1
  mkdir -p "$(dirname "$path")"
  if [ -n "$base" ]; then
    printf '%b' "$base" >"$path"
    git add "$path"
    git commit -q -m base
  fi
  printf '%b' "$added" >>"$path"
  git add "$path"
  python3 -I "$tests_dir/no-inline-comments.py" >/dev/null
  verdict "no-inline-comments: $label" "$want" $?
}

comment_case "rust line comment" fail a.rs "" '// explain\nfn a() {}\n'
comment_case "rust trailing comment" fail a.rs "" 'fn a() {} // explain\n'
comment_case "rust doc comment" pass a.rs "" '/// docs\n//! crate docs\nfn a() {}\n'
comment_case "rust SAFETY comment" pass a.rs "" '// SAFETY: checked\nfn a() {}\n'
comment_case "rust url in string" pass a.rs "" 'const U: &str = "http://x.org";\n'
comment_case "rust slashes in string" pass a.rs "" 'const U: &str = "a // b";\n'
comment_case "rust url after colon" pass a.rs "" 'fn a() { open(http://x.org); }\n'
comment_case "rust old comment untouched" pass a.rs '// old\nfn a() {}\n' 'fn b() {}\n'
comment_case "shell comment line" fail a.sh '#!/bin/sh\n# header\n\nls\n' '# explain\nls\n'
comment_case "shell trailing comment" fail a.sh "" 'ls # explain\n'
comment_case "shell header and shebang" pass a.sh "" '#!/bin/sh\n# usage: a.sh\n# more usage\nls\n'
comment_case "shell directive" pass a.sh '#!/bin/sh\nls\n' '# shellcheck disable=SC2086 reason\nls \x24x\n'
comment_case "shell hash in quotes" pass a.sh '#!/bin/sh\nls\n' 'echo "a # b" '"'"'c # d'"'"'\n'
comment_case "shell length expansion" pass a.sh '#!/bin/sh\nls\n' 'echo "\x24{#x}" \x24#\n'
comment_case "python comment line" fail a.py "" 'x = 1\n# explain\n'
comment_case "python trailing comment" fail a.py "" 'x = 1  # explain\n'
comment_case "python header" pass a.py "" '#!/usr/bin/env python3\n# usage: a.py\nx = 1\n'
comment_case "python noqa and type" pass a.py 'x = 1\n' 'import os  # noqa: F401\ny = []  # type: list\n'
comment_case "python hash in string" pass a.py 'x = 1\n' 's = "#a"\n'
comment_case "other files unchecked" pass notes.txt "" '# explain\n'

[ "$failures" -eq 0 ] && echo "all guards behave" || echo "$failures guard check(s) failed"
[ "$failures" -eq 0 ]
