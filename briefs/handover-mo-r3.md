# handover-mo r3 (rc 2, shared RC bar "a visible way to start"): train 99b red at the agent cell

Model: sonnet, effort low (one test script; cause known from the train log)
Lane: cloud, claude/handover-mo at 60314da1. Cost at most 0.3 %; stop at 150 k context. One area: gate scripts that start chora-tiling.
Why: train 99b (1c599c74 + guide-checks + ti-29 + mc-minor + handover-mo 60314da1) passed every cell up to `agent`, which failed: "2. tiling.get_layout and tiling.set_layout switch the layout / FAIL: timeout: ys_are 14 407". tests/agent.sh:70 and :336 start chora-tiling without `--no-companions`; since 60314da1 bare chora-tiling starts the bar, which moves the windows (rhm-1 fix). The `e2e` cell after it did not run.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "handover-mo BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Merge origin/claude/autonomous-dev-session-3w-i33pg7 (the newest session head; it may hold train 99c).
2) Reproduce: `tests/merge-gate.sh --jobs 1 --only build,agent,e2e` red at agent.
3) Fix KISS: pass `--no-companions` where tests/agent.sh (and the e2e script, if it fails the same way) starts chora-tiling and asserts geometry; no product change. Grep every merge-gate cell script that starts chora-tiling and asserts window positions without `--no-companions`; name any you change.
4) Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,build,agent,e2e,tiling-audit,tiling-look,doors` green. A red cell you did not cause: name it with its log line; do not rerun to green. Poll in the same turn; never end a turn to wait.
Done when: agent and e2e green on the merge, gate green.
Report: reports/handover-mo.md "## r3" on claude/session-log (shas, cause with file:line, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/handover-mo and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
