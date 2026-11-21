
## r1

- red 27248ad4: selftest puts an rm wrapper (execs the next rm on PATH) first on PATH; the nopc/nobin tools are linked through it, so the selftest hangs (rc=124 at 300s).
- fix fd8d6d0a: `real_tool` helper (first non-script of `type -aP`) at both ln lines. No change to ci-local.sh or merge-gate.sh.
- check: "a wrapper first on PATH does not change the nobin results".
- selftest: 170 ok, 0 FAIL, rc=0. gate --jobs 1 --only pre-commit,gate-lib: pre-commit PASS, gate-lib PASS (88s).
- head: fd8d6d0a

## Review r1 (PM)
Base 598a34ff..fd8d6d0a, one file (tests/merge-gate-selftest.sh, +19 -2). Red 27248ad4 before fix fd8d6d0a.
- Done criterion (selftest green with a wrapper first on PATH): met by the lane's run (170 ok); the train gate reruns it.
- rgp-1 NIT: the new `check` line only asserts the wrapper is first on PATH; the real guard is that the nobin case no longer hangs.
- rgp-2 NIT: `PATH="$wrap:$PATH"` stays set for all later selftest cases; `for f in $(type -aP ...)` splits paths with spaces.
Verdict (computed): no BLOCK or MAJOR -> MERGE-OK. Into t98.
