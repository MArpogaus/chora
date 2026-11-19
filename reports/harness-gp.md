## r1

Head: 66f384b4 on claude/harness-gp (red 37911502, fixes 66f384b4). Gate: `--only pre-commit,ci-check,red-first,identity,gate-lib` green; base merged (already up to date).
- gp-1: cells-for without `--head` adds `git ls-files --others --exclude-standard`; test: cells-for-selftest "untracked gate script" (new tests/gate-x.sh picks its cell).
- gp-2: a change to tests/merge-gate.sh or tests/gate-lib* picks gate-lib; test: selftest case "merge-gate picks gate-lib".
- gp-3: CONTRIBUTING.md line 295 says the pick is advisory and the train runs the full gate (docs, no test).
