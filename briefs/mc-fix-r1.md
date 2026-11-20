# mc-fix-r1 (RC 2 persona round 4, shared bar T1 by mouse; stability): mc-47, mc-48, mc-49 MAJOR candidates, P2-CHECK

Model: sonnet, effort high (named findings with captures; verify then a cell each)
Lane: cloud, branch claude/mc-fix from dac835dd (mc-remote, holds mc-p2-fix; both in train 85). One round, stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true` and `cargo check --workspace`; a failure: send_message the PM "mc-fix BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/mc.md rows mc-47 to mc-50; reports/p2-r2-mc.md "## r2"; reports/p2-plan.md "PM ruling 11-20 06:00 UTC: M2"; backlog.md P2-CHECK; feedback/2026-11-20-p2r2-sofa.md, -nurse.md and media/2026-11-20/p2r2-*/ (breaches.jsonl, actions.jsonl). On the branch: plugins/chora-mc, the kit list (run: fuzzel in mc), tests/gate-mc-sofa.sh, tests/gate-mc-handover.sh, tests/persona/check.py, harness.py wait_quiet.
Items (severity order; a fix needs its red gate test commit first; a candidate that does not reproduce: close with the replay as evidence, no code):
1. mc-47: a click on a row of the kit list on mc starts that program (as Return does). Cell: `mc-sofa launcher-click-row` (click the Foot row, a new window holds focus). Compare with the launcher-click cell of xp and tiling; reuse, do not copy.
2. mc-49: xp+mc never goes quiet on node mc-s-agenda-panels-today. Find who writes it and why. If mc rewrites it with no change: fix it (write only on a change; stability) with a red test that counts writes over 5 s idle. If the harness waits wrong: fix the harness. Then run `check.py start` on xp+mc 3 times; all quiet.
3. mc-48: replay sofa actions up to 18 (Super+a opens the chat) and read the node list: stale mc-r0-* beside mc-r1-* (remove the old set on redraw, red test first) or a checker misread (fix check.py I5).
4. P2-CHECK (checker only): M1 passes on any page change during the task; T6 accepts "Nothing found"; T1 and T4 ignore the key list the persona opened itself; the wpctl stand-in keeps the set value. A test per change in the persona tests.
Do not touch mc-50 (MINOR, rules.md 6) or the M2 ruling.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,keys,chat-keys,doors,mc-paths,mc-sofa,mc-handover,launcher-click,layout`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: red commits before fixes, gate cells pass, captures in reports/mc-fix/, reports/mc-fix.md "## r1" on claude/session-log (per item: red sha, fix sha or replay evidence, gate cell, capture); ledger rows mc-47 to mc-49 state `fixed <sha>` or `not a bug <evidence>`.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/mc-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
