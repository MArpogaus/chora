# xp-tidy r2 (shared bar, xp): PM review r1 MINOR and the two items left open in r1

Model: sonnet, effort medium (small xp fixes with named files; routine)
Lane: cloud, branch claude/xp-tidy at bd6fed1e. Stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "xp-tidy BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/xp-tidy.md "## r1" and "## PM review r1".
Items, in order:
1. pr3-1 MINOR plugins/chora-xp/src/wm/dispatch.rs:75: a look that exports window.give after its connected event (started after xp) must not stay gray in Send To. Red first: a unit test in tests_moved.rs (connected event, then the client gains the method; open the system menu; the look is listed). Fix: refresh at menu open when the cache is empty or stale, at most once per open; keep pr1-1 (no clients.list per press while the cache is fresh).
2. pr3-3 NIT: a failing clients.list warns once per failure streak, not at each client event; extend a_failed_clients_list_warns_once.
3. The e2e cell for a configured terminal editor (r1 "Not done"): with editor set in chora-xp.yaml, the Start menu entry opens it; add it to an existing xp gate, no new copies (F2).
Each fix: its red test commit first (format the test and keep subjects short so the hooks accept it), then the fix. Not in scope: the DB8 refactors.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,xp,xp-faces,xp-notepad,launcher-click,matrix`. Poll the gate in the same turn; never end a turn to wait.
Done when: each item red then green with shas; gate PASS lines; reports/xp-tidy.md "## r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/xp-tidy and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
