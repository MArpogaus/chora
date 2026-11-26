
## r1

- Base 07eb539f (tour-launch r1) merged with 6daaaab8 and b1a6465c; head 06c3f7cc6; red 199005519, fix 06c3f7cc6.
- rtl1-1: ensure_notify waits (0.2 s checks of notify state.get, NOTIFY_WAIT 15 s) and stops with one line.
- rtl1-2: tests/tool-version.sh is the one --version parse (launch.py, merge-gate tool_minimums).
- rhm1-1: docs step prints SKIP with the reason, FAIL when docs or mkdocs.yml changed without mkdocs.
- Diffstat: 5 files, 71 insertions, 18 deletions (launch.py, merge-gate.sh, merge-gate-selftest.sh, test_tour.py, tool-version.sh).
- Gate: CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2: all steps passed (docs SKIP, mkdocs not installed, docs unchanged), EXIT 0; test_tour 98 passed; merge-gate-selftest ok.

## Review r1 (PM, 65 lines, 4583836c..06c3f7cc)
- Red 19900551 before fix 06c3f7cc: yes. rtl1-1 (ensure_notify waits on state.get, 15 s, one error line): fixed. rtl1-2 (one version parse, tests/tool-version.sh, used by merge-gate preflight and launch.py): fixed. rhm1-1 (docs NOT RUN passed silently): now SKIP with reason, FAIL when docs or mkdocs.yml changed; selftest asserts no NOT RUN left.
- Lane gate: full, all PASS, docs SKIP; test_tour 98 pass.
- rth1-1 MINOR (harness): mkdocs is not in the cloud environment, so docs SKIP on every run and `--push` refuses without `--allow-skip docs`. Trains from now pass `--allow-skip docs` (known host gap); backlog: mkdocs in the environment setup.
Verdict (computed): no open MAJOR -> MERGE-OK 06c3f7cc. Harness: own train after t149, before t150.

## r2

Base 06c3f7cc (tour-harness r1) + merge of a93bc75b; head f396ebe63.
Commits: red 87b373406 (test: REPO is the root of the repo holding launch.py; no fixed checkout literal), fix f396ebe63 (launch.py REPO from `__file__`, TOUR_REPO overrides).
Diffstat: demo/tour/launch.py 2 lines, tests/test_tour.py +27.
pytest tests/test_tour.py in /tmp/claude-0/th2: 100 passed. Gate in that worktree (--jobs 2 --allow-skip docs): all steps passed, docs skipped (mkdocs not installed); EXIT 0.

## Review r2 (PM, 08:55 UTC)
Base a93bc75b (merged 19348b95), head f396ebe6; r2 adds red 87b37340 (2 tests) then fix f396ebe6 (launch.py, 1 line).
- rth2-1 fixed: REPO = TOUR_REPO or the repository root of launch.py (parents[2]); the no-literal test greps demo/ and tests/ for the fixed checkout path.
- Red first holds: 87b37340 is before the fix; its grep test fails on the base (worker note: the REPO test passes only in the main checkout).
- rth2-n1 NIT: SCRATCH still defaults to a fixed /tmp/claude-0/tour; harmless for the gate.
- Worker gate: all steps PASS, docs skipped (no mkdocs, docs unchanged).
Verdict (computed): no open MAJOR, MERGE-OK f396ebe6. Harness branch: own train t153 right after t152, before other trains; then the running lanes merge the session branch and train runs drop TOUR_REPO.
