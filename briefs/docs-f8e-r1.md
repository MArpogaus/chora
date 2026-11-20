# docs-f8e r1 (F8 docs and records; CM-R1): kit swallow line, D118, kept nodes during the resume grace

Model: sonnet, effort medium (two doc lines, one decision record, one pinning unit test; routine)
Lane: cloud, branch claude/docs-f8e from ab4cbb4f (co-p2-fix, MERGE-OK, train 85). One round, stop at 250 k. Cost about 0.4 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, then `cargo check --workspace`; a failure or a missing tool: send_message the PM "docs-f8e r1 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09); keep the brief's identity, never ask.
Read on claude/session-log: reports/audit-f.md "## r3" (F8 risk), reports/core-minors.md (item 5 and "PM review r1"), backlog.md CM-R1; on the branch: CONTRIBUTING.md (kit module list, "A new module adds its line"), docs/decisions.md (D96-D117 format), crates/chora-kit/src/swallow.rs, docs/api.md "Window managers" (resume, ~l.965).
Items (own commit each):
1. DOCS-F8e: CONTRIBUTING.md kit module list gets the `swallow` line (what lives there, when to use it: a face or pane manager grabs the Super chords it binds no action for, `Runtime::grab_swallowed`, after its own grabs, before `end_grabs`). docs commit.
2. DOCS-F8e: docs/decisions.md D118, the swallow of unbound Super chords by the manager of a face or pane (context co-p2-fix, alternatives: core swallows, each look copies; cost line), same table form as D117. "Accepted by the PM after review r2 of claude/co-p2-fix, train 85". docs commit.
3. CM-R1: docs/api.md Window managers, resume: one sentence that kept nodes stay shown during the grace time (no hide, no flicker on a quick restart); a pinning unit test in crates/chora/src/ipc.rs tests (a window manager with `resume` disconnects; within the grace its nodes are still in `scene.query` with visible true; after the resume hello they are unchanged). Test commit, then the doc commit. If the test is red on the head, stop and report (that would be a behaviour finding, not a fix of this lane).
Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib`; all PASS. A red cell you did not cause: rerun it alone once and name it.
Done when: each item names its commit (and the test for 3); gate green; reports/docs-f8e.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines), same turn.
Rules: push claude/docs-f8e and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
