# Brief ci-fix r2 (rc 4, 5)

Model: sonnet, effort medium (two small harness fixes from review)
Branch: claude/ci-fix (continue from cd98d2c8). Review: reports/review-ci-fix-r1.md.

Tasks, each red test commit first:
1. rv-1: tests/gate-0.8-4.sh:18 and tests/gate-0.7-4.sh:16 kill then rm -rf without waiting. Source tests/proc-lib.sh and use remove_after_exit. Add KILL escalation after the 5 s budget in remove_after_exit (rv-4), with a self-test case in gate-lib-selftest.sh (a pid that ignores TERM).
2. rv-2: widen tests/packaging-files.py shipped() to every shipped plugin data dir (plugins/*/hud, plugins/*/huds, and any other dir whose files Cargo.toml or PKGBUILD install). Red test: it must flag the tiling HUD at 7455399c^ (d7be70b9), green at HEAD.
Not in scope: rv-3, rv-5, rv-6 (backlog).

Done when: both red then green; gate-0.8-4 runs (or its cleanup self-test passes) 3 times; full merge-gate passes in one run (run it with run_in_background, no wrapper timeout); reports/ci-fix-r2.md on claude/session-log; hand back by send_message.
