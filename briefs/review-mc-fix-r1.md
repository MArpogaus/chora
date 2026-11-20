# review mc-fix r1 (RC 2 persona round 4, stability: mc-48, mc-49, P2-CHECK): seeded review of claude/mc-fix before its merge

Model: sonnet, effort high (plugin and test code, named findings; rules.md 7: the builder never reviews itself)
Lane: cloud, read-only on code: claude/mc-fix head b4104cae, base dac835dd (8 files, +270 -14). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No full merge gate (the lane ran 12 gate steps green).
Step 0 (no setup script): `git fetch --unshallow || true`; `git fetch origin claude/session-log claude/mc-fix`; `cargo check --workspace`; a failure: send_message the PM "review mc-fix BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/mc-fix-r1.md (done criteria), reports/mc-fix.md "## r1", findings/mc.md rows mc-47 to mc-49; then `git diff dac835dd..b4104cae`.
Checks, each with its command and files:
1) Red first: c3432638 fails on its parent and passes at 55f75baa; 137437c5 fails at 55f75baa and passes at bb3de644 (`cargo test -p chora-mc` per sha in a scratch worktree; name the test).
2) mc-49, lifecycle.rs and build.rs: a removed node no longer keeps mc busy. Can busy now clear too early (a node removed while a write to another node runs)? Is the busy set keyed so a rename or rebuild cannot leak an entry?
3) mc-48, main.rs and state.rs: `state.get` reports `fade:<node>` busy for the 250 ms cross-fade. Does busy end when the fade ends, also when a second fade starts mid-fade or the region is removed mid-fade? No timer leak on exit.
4) P2-CHECK, tests/persona/check.py and session.sh: M1 passes on any page change during the task; T6 accepts "Nothing found"; T1 and T4 ignore a key list the persona opened itself; the wpctl stand-in keeps the set value. Does any change make the checker pass a real breach (a check that can no longer fail)? Each change has a test in test_acts.py or persona-selftest.sh that fails without it.
5) mc-47 closed as "not a bug of mc" (fuzzel 1.9.2 ignores the click): read the evidence; say whether the mc-sofa cell that was dropped should come back once fuzzel >= 1.11 is on PATH (the PM verifies that on the train).
6) `cargo test -p chora-mc`, `python3 -m pytest tests/test_acts.py -q`, `bash tests/persona-selftest.sh`, `cargo clippy --workspace --all-targets -- -D warnings`; no file over 1,500 lines (main.rs!).
Known, not findings: gate-tiling-apps-click / launcher-click red in lane containers (ti-1, the PM checks it with fuzzel 1.11.1).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/mc-fix.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
