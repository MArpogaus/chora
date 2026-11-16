# Review ci-fix r1 (claude/ci-fix cd98d2c8), sonnet, 11-16 21:5x: FIX

Commits ok (red first, author, no attribution). package-lists red at d7be70b9, green at HEAD; gate-lib-selftest ok at HEAD.

- rv-1 MAJOR: the kill-then-rm race remains in tests/gate-0.8-4.sh:18 (`$runtime` is XDG_RUNTIME_DIR; runs in CI, .github/workflows/ci.yaml). Same shape: tests/gate-0.7-4.sh:16. Proof: `grep -n "trap.*kill.*rm " tests/*.sh`.
- rv-2 MAJOR: tests/packaging-files.py:68 scans flavors/ and plugins/*/hud only, so plugins/*/huds/* (and other shipped plugin data) are never compared; gate-package-lists.sh only checks paths already in build-arch.sh's hand list.
- rv-3 MINOR: gate-package-lists.sh:36 accepts any Cargo asset with the right dest dir, never matches source; PKGBUILD check is a regex.
- rv-4 MINOR: remove_after_exit sends TERM only; no KILL escalation after 5 s (headless-session.sh:62 does escalate).
- rv-5 MINOR: xp.sh:63-70 waits only for pids known at kill time.
- rv-6 NIT: gate-lib-selftest.sh:139 kill -KILL non-zero if gone.
