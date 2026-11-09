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

session_case() {
  local label=$1 want=$2 path=$3
  new_repo
  cd "$work/repo" || exit 1
  mkdir -p "$(dirname "$path")"
  printf 'x\n' >"$path"
  git add "$path"
  "$tests_dir/no-session-files.sh" >/dev/null 2>&1
  verdict "no-session-files: $label" "$want" $?
}

session_case "plans/autonomy" fail plans/autonomy/plan.md
session_case "reports" fail reports/day1.md
session_case "reflections" fail reflections/a.md
session_case "media" fail media/shot.png
session_case "drafts" fail drafts/a.md
for name in charter state ledger friction answers questions backlog HANDOFF; do
  session_case "top-level $name.md" fail "$name.md"
done
session_case "docs/reports is code" pass docs/reports/a.md
session_case "plans/other" pass plans/other/a.md
session_case "nested state.md" pass docs/state.md
session_case "source file" pass src/main.rs

message_case() {
  local label=$1 want=$2 text=$3
  printf '%b' "$text" >"$work/message"
  python3 -I "$tests_dir/commit-message-length.py" "$work/message" >/dev/null 2>&1
  verdict "commit-message-length: $label" "$want" $?
}

subject_68=$(printf 's%.0s' $(seq 68))
subject_69=$(printf 's%.0s' $(seq 69))
body_72=$(printf 'b%.0s' $(seq 72))
body_73=$(printf 'b%.0s' $(seq 73))
long_url="https://example.org/$(printf 'u%.0s' $(seq 100))"

message_case "subject of 68" pass "$subject_68\n"
message_case "subject of 69" fail "$subject_69\n"
message_case "body line of 72" pass "short\n\n$body_72\n"
message_case "body line of 73" fail "short\n\n$body_73\n"
message_case "long URL line" pass "short\n\n$long_url\n"
message_case "line starting with a URL" pass "short\n\n$long_url and $body_73\n"
message_case "long comment line" pass "short\n\n# $body_73\n"
message_case "long line after a URL line" fail "short\n\n$long_url\n$body_73\n"

[ "$failures" -eq 0 ] && echo "all guards behave" || echo "$failures guard check(s) failed"
[ "$failures" -eq 0 ]
