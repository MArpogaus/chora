# harness r5 (claude/harness 5f0977f1)

Closes review-harness-r4 M1 and m1. Base merge: origin/claude/autonomous-dev-session-3w-i33pg7
was already contained (no merge commit).

## Commits
- c6d39f8c test(gate): an allow with a trailing comment in the runtime grep
- 761ea365 fix(gate): close an allow in the runtime grep before a comment
- 5f0977f1 fix(guards): pass kill tools in any order after need or command -v

## 1. MAJOR M1, runtime grep window (tests/gate-0.8-plugin-runtime.sh:21)
The `lint` window now closes on `)]` followed by optional spaces and an optional `// ...`.
Red (c6d39f8c, merge-gate-selftest rc 1):
  FAIL: plugin runtime: an allow with a trailing comment hides nothing after it: got '0', want '1'
  FAIL: plugin runtime: an allowed item with a trailing comment covers only itself: got '0', want '1'
Green (761ea365): merge-gate-selftest rc 0, all 14 plugin runtime cases ok.
Today's plugins/ tree (one-line allow whose window stayed open past its line):
before 27 spans, 4085 lines skipped; after 0 spans, 0 lines. (The r4 review counted 20/1217
on an older tree with its own span.awk.) The gate on the tree passes after the fix (rc 0),
so no string call was hidden. The three allowed sites are still excused, hits:
chora-mc testing.rs:140-141, chora-tiling main.rs:2323,2328, chora-xp wm.rs:4318-4319.

## 2. MINOR m1, no-foreign-kill (tests/no-foreign-kill.sh:70)
A command `need ...` or `command -v ...` holding only plain words (letters, digits, `_./>-`)
gives no verdict, so tool names pass in any order. A later `pkill chora` is still refused.
Red: guards.sh against the old hook, 2 FAIL (+ the nested-run FAIL that follows):
  FAIL no-foreign-kill: several tools, the kill tool not last (wanted pass, exit 1)
  FAIL no-foreign-kill: a check for several tools, the kill tool not last (wanted pass, exit 1)
Green: "all guards behave", rc 0.
Deviation: test and fix share 5f0977f1; the guards pre-commit hook refused the red-only test
commit (3 guard checks failed), same as b822a958/7ef5fe1f in r3/r4.

## Evidence
tests/merge-gate.sh (full, serial): 28 steps PASS, no SKIP, "all steps passed", rc 0.
tests/ci-local.sh: msrv, docs, pre-commit, deny, machete, packaging PASS, rc 0.

## Open
Backlog R13 as briefed (escaped quotes, here-doc substitutions, eval / bash -c, red-first on
other tracking names, rt/b). `#[allow(...)] stmt;` on one line (review m5) still keeps the
window open; not rustfmt layout, clippy catches conn::connect.
