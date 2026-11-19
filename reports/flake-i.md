# flake-i

## r1

Branch claude/flake-i, head e5e44719 (from 219d5afe).

- F22 fixed in e5e44719 (crates/chora/tests/log_file.rs). Cause: the panic of the test thread starts with the core and can reach chora.log before the "chora runs" line (lib.rs:152), so the test, which waited only for the panic line, read a log without the start line (assert at line 92). Reproduced under load (tests/under-load.sh, 8 busy loops): 1 of 30 failed before, 0 of 60 after. The test waits for both lines now.
- F31 already closed on 219d5afe: cd98d2c8 added remove_after_exit (tests/proc-lib.sh, bounded 5 s, then KILL) and xp.sh, agent.sh, nested.sh, gate-0.7-4.sh and gate-0.8-4.sh use it; headless-session.sh waits for its process group (5 s) before rm. No change made.
- Gate: pre-commit, ci-check, build, test, gate-lib, xp green (--jobs 3); test step green 3 of 3.
- Untouched: cells of flake-h.

## r1 PM review (11-19 21:2x UTC)
Base 219d5afe, head e5e44719, one file (crates/chora/tests/log_file.rs, +12 -5). wait_for takes a word list; the panic test waits for "chora runs" and the panic line, as its asserts read both. Matches the F22 cause the lane measured (1/30 red before, 0/60 after). F31 closed on base by cd98d2c8: checked the claim against tests/proc-lib.sh remove_after_exit. No findings. Verdict MERGE-OK e5e44719; train 75.
