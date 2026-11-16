# ci-fix r1 (rc 4, 5, 3)
Model: sonnet, effort medium (two small harness and packaging fixes)
Branch: claude/ci-fix, new from the session branch ca2804b4.

## Task
CI on ca2804b4 (PR #106) failed twice:
1. arch job: `missing: usr/share/chora/huds/tiling/hud.yaml`. dist/arch/PKGBUILD:104-108 installs
   huds/neutral, mc, construct but not plugins/chora-assistant/huds/tiling/* (Cargo.toml:49 and
   dist/build-arch.sh:28 have it). Red first: a fast static gate tests/gate-package-lists.sh
   (no build) that fails when a path in build-arch.sh's expected list has no install rule in
   dist/arch/PKGBUILD or no asset in crates/chora/Cargo.toml; add it to tests/merge-gate.sh
   (step `package-lists`, header line union). Commit it red, then fix PKGBUILD.
2. F31 (backlog.md): tests/xp.sh cleanup (63-71) kills and then rm -rf $XDG_RUNTIME_DIR without
   waiting; an exiting client writes into it ("Directory not empty", CI e2e). Wait for the killed
   pids (bounded, e.g. 5 s) before rm; check the other tests/*.sh with the same pattern and fix
   them alike. Red first where a deterministic test is possible; else say so in the report.

## Done when
- tests/gate-package-lists.sh red on ca2804b4, green after the fix; xp step passes 3 runs.
- tests/merge-gate.sh (no --push) passes once.
- reports/ci-fix-r1.md on claude/session-log: commits, cells, runs. Push the branch, hand back
  to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with one send_message.
No other scope. Author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>, no
attribution lines, never --no-verify. Read dev-rules.md and conventions.md once.
