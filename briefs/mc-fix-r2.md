# mc-fix-r2 (RC 2 persona round 4; train 87 blocker): merge the session branch into claude/mc-fix and resolve its conflicts

Model: sonnet, effort high (a merge of two checker changes; both intents must survive)
Lane: cloud, branch claude/mc-fix at b4104cae (MERGE-OK 08:43, Review r1 in reports/mc-fix.md). One round, stop at 150 k. Cost about 0.4 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`, `git fetch origin claude/autonomous-dev-session-3w-i33pg7` and `cargo check --workspace`; a failure: send_message the PM "mc-fix BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Why: b4104cae conflicts with the session head 9973d0eb (train 85c, holds xp-p2r3) in tests/persona/check.py (the snapshot row: `toplevels` vs `page`; contracts T1, T2, T4, M1, M2) and tests/persona/session.sh (the wpctl stand-in). The PM found it with `git merge-tree --write-tree 9973d0eb b4104cae`.
Read on claude/session-log: reports/mc-fix.md "## r1" and "## Review r1"; reports/xp-p2r3.md (what xp-p2r3 changed in check.py and session.sh and why).
Items:
1. Merge origin/claude/autonomous-dev-session-3w-i33pg7 (9973d0eb or later) into claude/mc-fix with a merge commit. Keep both intents: the row keeps `toplevels` and `page`; T1 and T4 keep `open=ctx.left` of xp-p2r3 and the key-list check of mc-fix (verify_only_keys_open); M1 keeps the page change of mc-fix; M2 keeps `open=ctx.left`; T2 per xp-p2r3 unless mc-fix's test says otherwise (name it); the wpctl stand-in is one script that keeps the set value and the mute state. No other change.
2. Both test sets pass: `python3 -I -m pytest tests/test_acts.py -q`, `bash tests/persona-selftest.sh` (after `cargo build --workspace`).
Do not fix review r1 findings (rv-1..rv-5 are backlog MC-FIX-RV) and do not touch mc-50.
Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,mc-paths,mc-sofa,mc-handover`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Known, not findings: launcher-click and gate-tiling-apps-click red in lane containers (ti-1).
Done when: the merge sha on claude/mc-fix, tests and gate cells pass, reports/mc-fix.md "## r2" on claude/session-log (merge sha, each conflict hunk and how it was kept, test counts).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/mc-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
