# probe-leak r1 (F5 no flaky tests: sh-30, leaked chora-probe busy-loops)

Model: sonnet, effort high (test harness and the probe client, a clear red; no new feature)
Lane: cloud, branch claude/probe-leak from d81c8293. One round, stop at 250 k. Cost about 0.6 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true` and `cargo check --workspace`; a failure: send_message the PM "probe-leak BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/shared.md row sh-30. Evidence: on the PM host 4 probes `target/debug/chora-probe window 200 200 200 --title Probe window --server-deco --report` from old trains ran 1.5-5.3 h at 75-91 % CPU each.
Items:
1. The probe exits when its compositor goes away: tests/probe/src/main.rs, the window mode with --report and dispatch_until (a hangup, POLLHUP/POLLERR, or a dispatch or read error ends the loop; no spin on a closed fd). Red first: a test (tests/ shell cell or a Rust test in tests/probe) that starts a headless chora, a probe window --report, kills chora, and fails when the probe still runs after 2 s or uses CPU; then the fix.
2. The gate leaves no probe: the cells that start chora-probe (grep tests/ for chora-probe: gate-tiling-motion, gate-construct-keys, -out, -placed, -walk, tiling-keys, agent.sh, persona-selftest and others) kill their probes on exit, best in the shared trap of the gate lib (one place, not per cell). Red first: tests/gate-lib-selftest.sh or a new selftest that runs a stub cell and finds a leftover probe; then the fix. Check with `pgrep -f chora-probe` empty after the gate cells below.
3. Report how many probes leaked in your container before your fix (pgrep at step 0 and after the gate at the base).
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first if it moved; `tests/merge-gate.sh --jobs 2 --only pre-commit,gate-lib,ci-check,build,test,clippy,keys,walk,placed,out,agent` green; poll it in the same turn; never end a turn to wait. Do not rerun to green.
Done when: items 1-3 with red and fix shas, gate cells pass, no probe left, reports/probe-leak.md "## r1" on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/probe-leak and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages; red test commit before each fix.
