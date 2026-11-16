# harness-rc r2

- Commit 3641d5e on claude/harness-rc. MAJOR: ci.yaml e2e job runs tests/gate-tiling-bar-early-signal.sh after the Apps cells; merge-gate reads it through e2e-sequence.sh. The e2e step log shows it as cell 25 and passes.
- MINOR: waybar.sh traps USR1, USR2 and RTMIN+1 by name in the parent before the fork and resets them after the pid file is written; the shebang is now bash (dash cannot trap RTMIN+1 by name).
- merge-gate: red-first, pre-commit, gate-lib, build, test, clippy, ci-check PASS; 2d-guard FAILED here ("Needed a single revision", a ref missing in this container, not caused by the change); `--only e2e` PASS (298 s).
- The session-log branch has unrelated history, so I did not merge it.
