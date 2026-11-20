#!/usr/bin/env bash
# pre-commit hook: run tests/guards.sh and refuse a commit when it is red,
# except a commit that only adds guard cases to tests/guards.sh: that is
# the red commit of a test-first change (the cases fail until the guard is
# fixed in the next commit). Any other staged file, a removed line or an
# added line that is no `*_case` call keeps the refusal.
#
# Usage: tests/guards-hook.sh   (GUARDS_SCRIPT: the guard test to run)
set -u
tests_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
script=${GUARDS_SCRIPT:-$tests_dir/guards.sh}
bash "$script" && exit 0
status=$?
staged=$(git diff --cached --name-only)
[ "$staged" = tests/guards.sh ] || exit "$status"
other=$(git diff --cached -U0 --no-color --no-ext-diff -- tests/guards.sh |
  grep -Ev '^(\+\+\+|---|@@|diff |index |new file|old mode|new mode)' |
  grep -Ev '^\+[a-z_]+_case[[:space:]]')
if [ -n "$other" ]; then
  exit "$status"
fi
echo "guards.sh is red, but the commit only adds guard cases: red commit accepted"
exit 0
