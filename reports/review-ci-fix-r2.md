# Review ci-fix r2 (claude/ci-fix fbcf6055), PM, 11-16 22:3x: MERGE-OK

r2 delta cd98d2c8..fbcf6055 read in full. Red first: 3954ee20 before 6bcc9bcb, 4178e3bc before fbcf6055; author Marcel, no attribution.
- rv-1 closed: gate-0.8-4.sh:19, gate-0.7-4.sh:17 use remove_after_exit (worker: gate-0.8-4 3/3, no runtime dir left; 0.7-4 needs podman, not run).
- rv-4 closed: proc-lib.sh remove_after_exit sends KILL after the budget; self-test case (TERM ignored) red then green.
- rv-2 closed: packaging-files.py scans plugins/*/hud*, assets, collectors (165 files, 0 gaps); packaging-files-selftest.sh in ci-check.
- NIT: --pkgbuild parsing is positional only (argv exactly 3). Not worth a round.
Full merge-gate passed in one run (worker). Train 33 verifies.
