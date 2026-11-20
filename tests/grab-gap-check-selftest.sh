#!/bin/bash
# Selftest of tests/grab-gap-check.py: a start_grabs that reaches end_grabs
# with no grab call between them is an ungrab_all in disguise (GR1, GR2),
# and the check names its file and line; a path with a grab between them
# passes. A callee of the same file that returns early or grabs only inside
# an if or a match fails: its caller reaches end_grabs with no grab on that
# branch. A grab in a loop passes (an empty loop grabs nothing, which no
# check can know) and so does a callee in another file (not read).
#
# Usage: tests/grab-gap-check-selftest.sh
set -euo pipefail
here=$(dirname "$0")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
printf 'fn a() {\n    rt.start_grabs();\n    rt.end_grabs()?;\n}\n' >"$work/gap.rs"
printf 'fn b() {\n    rt.start_grabs();\n    self.grab_all()\n        .and_then(|()| rt.end_grabs())\n}\n' >"$work/ok.rs"
printf 'fn c() {\n    rt.start_grabs();\n    rt.grab(k, 1)?;\n    rt.end_grabs()\n}\n' >"$work/ok2.rs"
printf 'fn d() {\n    rt.start_grabs();\n    self.grab_all()?;\n    rt.end_grabs()\n}\nfn grab_all(&self) {\n    if self.keys.is_empty() {\n        return Ok(());\n    }\n    rt.grab(k, 1)\n}\n' >"$work/skip.rs"
printf 'fn e() {\n    rt.start_grabs();\n    self.grab_all()?;\n    rt.end_grabs()\n}\nfn grab_all(&self) {\n    if self.on {\n        rt.grab(k, 1)?;\n    }\n    Ok(())\n}\n' >"$work/branch.rs"
printf 'fn f() {\n    rt.start_grabs();\n    self.grab_all()?;\n    rt.end_grabs()\n}\nfn grab_all(&self) {\n    for k in &self.keys {\n        rt.grab(k, 1)?;\n    }\n    Ok(())\n}\n' >"$work/loop.rs"
printf 'fn g() {\n    rt.start_grabs();\n    self.grab_all()?;\n    rt.end_grabs()\n}\nfn grab_all(&self) {\n    self.grab_one()\n}\nfn grab_one(&self) {\n    if self.on {\n        rt.grab(k, 1)?;\n    }\n    Ok(())\n}\n' >"$work/nested.rs"
printf 'fn h() {\n    rt.start_grabs();\n    self.grab_all()?;\n    rt.end_grabs()\n}\nfn grab_all(&self) {\n    self.grab_one()\n}\nfn grab_one(&self) {\n    rt.grab(k, 1)\n}\n' >"$work/nested_ok.rs"
python3 "$here/grab-gap-check.py" "$work/ok.rs" "$work/ok2.rs" "$work/nested_ok.rs" || { echo "FAIL: a grab between passes"; exit 1; }
if out=$(python3 "$here/grab-gap-check.py" "$work/gap.rs" 2>&1); then
    echo "FAIL: the gap passed"
    exit 1
fi
grep -q 'gap.rs:2' <<<"$out" || { echo "FAIL: no file and line: $out"; exit 1; }
python3 "$here/grab-gap-check.py" "$work/loop.rs" || { echo "FAIL: a grab in a loop passes"; exit 1; }
for f in skip branch nested; do
    if out=$(python3 "$here/grab-gap-check.py" "$work/$f.rs" 2>&1); then
        echo "FAIL: the callee that skips its grab on a branch passed ($f)"
        exit 1
    fi
    grep -q "$f.rs:2" <<<"$out" || { echo "FAIL: no file and line ($f): $out"; exit 1; }
done
echo "grab-gap-check-selftest: ok"
