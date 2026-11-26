# harness-minor

## r1

Base 5646b8fd, head b1a6465c2 on claude/harness-minor-r1 (4 files, +85 -10: tests/merge-gate-selftest.sh, tests/merge-gate.sh, tests/persona/act.py, tests/persona/test_act_wait.py).

- raw1-1 FIXED. Red a8b55c73 (CheckOrder in test_act_wait.py: a slow-starting key check, then a look; the look's check ran first, `['look','key']`), fix 82422d89. Each check takes a ticket (checks.ticket) when its action ends and waits under the chain flock until checks.chain.done reaches ticket-1 (give-up after 4 x CHECK_TIMEOUT if an earlier one was killed). A look still does not wait.
- rsl1-1 NOT FIXED, no commit. chora-construct has no `--output` flag (`chora-construct --help`: --config, --print-config, --overlay), so startup_of passing `${look_output[@]}` would make clap refuse the start; the output needs a construct product change (no product code in this lane). No persona round uses construct as the joining flavor. Stays backlog, product side.
- rcd1-1 FIXED. Red fd9c07bb (merge-gate-selftest: no `docs` step in `--list`), fix b1a6465c. New gate step `docs` after pre-commit: `mkdocs build --strict --quiet` into a temp dir; without mkdocs it prints `NOT RUN: mkdocs is not installed` and exits 0. Gate here: docs PASS 2 s (mkdocs and mkdocs-material installed with pip for this run).
- rv9j-4 ALREADY FIXED in the base, no commit, no red. demo/tour/chapters/handover.py has tiling_companions (swaybg and waybar on the tiling output, 671b426e, 632ea8bd waits and raises); the report read an older head. Captures of the chapter handover on 671b426e^ and on 5646b8fd, act 05-01 right screen (h1 is this act): media/2026-11-25/harness-minor-r1/05-01-tiling-before.jpg (pure black, 0,0 extrema, 9 KB) and 05-01-tiling-after.jpg (bar and wallpaper, 15 KB). Chapter handover on the head: 11 acts, 11 green, 0 red.

Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no step filter, no --push) on b1a6465c2: all steps PASS, no foreign flake. Last lines: `tiling-disc PASS 72s`, `e2e PASS 284s`, `all steps passed; no --push, nothing is pushed`. `pgrep -fa 'chora --headless'` empty.

## Review r1 (PM, 00:27 UTC)
Base 5646b8fd, head b1a6465c, 4 files +85 -10 (tests/persona/act.py, test_act_wait.py, tests/merge-gate.sh, merge-gate-selftest.sh). PM review: under 200 lines.
Checks: red a8b55c73 before fix 82422d89 (raw1-1); red fd9c07bb before fix b1a6465c (rcd1-1). Tickets are taken in check() under the pending lock, so in action order; run_check waits for done >= ticket-1 and polls 50 ms with the chain lock released; a killed check is passed after TURN_LIMIT (240 s); done is max(ticket, done). Docs step: strict build in a temp dir, removed after. Gate on the head: all PASS (report).
Findings:
- rhm1-1 NIT, tests/merge-gate.sh docs step: without mkdocs it prints NOT RUN and passes, so a container without mkdocs never checks the docs; the CI docs job still builds strict. Backlog.
- rsl1-1 stays open (product: chora-construct has no --output); backlog.
Verdict (computed): 0 MAJOR, MERGE-OK. Harness branch: its own train (t148) right after t147, then the running lanes merge the session branch.
