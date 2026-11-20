# harness-k

## r1

Branch claude/harness-k at 0dfdad8d (from 79c8d2b5). Gate `--jobs 3 --only
pre-commit,identity,gate-lib,ci-check,build,test,clippy`: green (ci-check
104 s, build 131 s, test 193 s).

| Item | State on head | Red | Fix | Cell |
|---|---|---|---|---|
| ig-1 merge-gate-selftest in gate-lib | closed (line of gate-lib runs it; selftest checks it) | - | - | gate-lib |
| ig-2 MERGE_GATE_IDENTITY printed | closed (identity prints the override; selftest checks it) | - | - | identity |
| TRAIN78 cut-check first | open | 11fe8147 | 87f17914: ci-check moved before build, cut-check and its selftest first in it | gate-lib, ci-check |
| H-mkdir | open | 5311405d (tests/mkdir-runtime.sh, 9 guards cases; whole tree 1 hit) | 41f50a5b waybar.sh:38 made 700 (also `$style`); b7d2a63a first attempt, 79f9a5c9 hook and CONTRIBUTING line | pre-commit |
| F24R-1 grab-gap callee | open | 1c1961fa (nested.rs passes) | 0dfdad8d: a first grab that is a same-file callee is read recursively | ci-check |

Notes: the mkdir check tracks variables set from XDG_RUNTIME_DIR in the same
file; only `mkdir -p` is flagged. Whole tree after the fix: 0 hits.
Not built: a callee in another file is still not read (documented).
