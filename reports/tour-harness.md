
## r1

- Base 07eb539f (tour-launch r1) merged with 6daaaab8 and b1a6465c; head 06c3f7cc6; red 199005519, fix 06c3f7cc6.
- rtl1-1: ensure_notify waits (0.2 s checks of notify state.get, NOTIFY_WAIT 15 s) and stops with one line.
- rtl1-2: tests/tool-version.sh is the one --version parse (launch.py, merge-gate tool_minimums).
- rhm1-1: docs step prints SKIP with the reason, FAIL when docs or mkdocs.yml changed without mkdocs.
- Diffstat: 5 files, 71 insertions, 18 deletions (launch.py, merge-gate.sh, merge-gate-selftest.sh, test_tour.py, tool-version.sh).
- Gate: CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2: all steps passed (docs SKIP, mkdocs not installed, docs unchanged), EXIT 0; test_tour 98 passed; merge-gate-selftest ok.
