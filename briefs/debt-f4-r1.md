# debt-f4-r1 (F4: workarounds 13 > RC 9, hygiene 16 > RC 14; reports/debt-2026-11-20.md sections 2 and 6): close six named rows with proof

Model: sonnet, effort high (harness and gate work across several scripts; the PM reviews)
Lane: cloud, branch claude/debt-f4 from 79c8d2b5 (session branch, train 81). One round, stop at 250 k. Cost about 1 %. A harness branch: it gets its own train.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace` and `cargo build --workspace`; a failure: send_message the PM "debt-f4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/debt-2026-11-20.md sections 2, 6, 7 (rows B4, N5, H8, H10, H11, H12), backlog.md DB10, docs D90 and D91 in docs/decisions.md on the branch. Skills: git -C /home/user/agent_skills show origin/dev:skills/git-conventions/SKILL.md (section "Rules for agents", for H8).
Scope: tests/ (gate scripts, merge-gate.sh), README.md, docs/dev-rules.md (or the file that holds the dev rules), docs/known-issues.md. Do not touch crates/, plugins/, services/, flavors/ (lanes co-p2-fix, voice-v12, flakes-j and lint-down own those areas; voice-v12 closes H4, not you). Merge origin/claude/autonomous-dev-session-3w-i33pg7 before the gate.
Rows, each with a red check first where a check is possible:
1. B4: the six construct scripts outside the gate (gate-construct-k1, k2a, 3d, mcpane, nested, windows). Run each 3 times. 3/3 green: a new cell `construct-extra` in the default set. Red: rerun alone once; a real fault is a finding row in findings/construct.md with the exact line and the script stays out with a known-red line; a script that tests a removed feature: delete it and name the commit that removed the feature.
2. N5 (backlog DB10): the hud gate pointer workaround; show with a run whether Tab works without it; drop it if so, else a finding.
3. H8: README "Use of LLM coding tools" gets the exact wording of git-conventions "Rules for agents".
4. H10: tests/gate-mc-handover.sh within_ms 1000 to a margin derived from 10 runs (p95 times 2, at most 500); the pull step-back gets an assert.
5. H11 (D90): merge-gate.sh fails at start under 6 GB free on / and prints df; a drift check (session branch head vs origin) prints a warning. Red test in tests/gate-lib (or the merge-gate selftest) first.
6. H12: the dev rules gain "a merge that touches IPC or security adds a D entry".
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,gate-lib,build,test,clippy,layout,hud,construct-extra,mc-handover` (cells that exist); poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: each of the six rows is closed with a sha and a passing cell, or left open with a finding and its run output; reports/debt-f4.md "## r1" on claude/session-log gives the new workaround and hygiene counts by the commands of reports/debt-2026-11-20.md section 7.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/debt-f4 and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
