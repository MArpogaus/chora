#!/bin/bash
# Selftest of tests/grab-gap-check.py: a start_grabs that reaches end_grabs
# with no grab call between them is an ungrab_all in disguise (GR1, GR2),
# and the check names its file and line; a path with a grab between them
# passes.
#
# Usage: tests/grab-gap-check-selftest.sh
set -euo pipefail
here=$(dirname "$0")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
printf 'fn a() {\n    rt.start_grabs();\n    rt.end_grabs()?;\n}\n' >"$work/gap.rs"
printf 'fn b() {\n    rt.start_grabs();\n    self.grab_all()\n        .and_then(|()| rt.end_grabs())\n}\n' >"$work/ok.rs"
printf 'fn c() {\n    rt.start_grabs();\n    rt.grab(k, 1)?;\n    rt.end_grabs()\n}\n' >"$work/ok2.rs"
python3 "$here/grab-gap-check.py" "$work/ok.rs" "$work/ok2.rs" || { echo "FAIL: a grab between passes"; exit 1; }
if out=$(python3 "$here/grab-gap-check.py" "$work/gap.rs" 2>&1); then
    echo "FAIL: the gap passed"
    exit 1
fi
grep -q 'gap.rs:2' <<<"$out" || { echo "FAIL: no file and line: $out"; exit 1; }
echo "grab-gap-check-selftest: ok"
