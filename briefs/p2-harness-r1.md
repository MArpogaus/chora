# p2-harness-r1 (RC 2 persona round 4, P2 item 2 prep): construct round harness, verify "XP did not start"

Model: sonnet, effort high (harness root causes across round.sh, act.py and a cube child start)
Lane: cloud, branch claude/p2-harness from b97369d1 (session branch, train 80). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "p2-harness BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-r1-construct.md "## r1" (driver slips, harness reds), feedback/2026-11-20-p2r1-{explorer,max,demo}.md, media/2026-11-20/persona-*/ logs, reports/p2-plan.md sections 1, 2. On the branch: tests/persona/ (round.sh, act.py, check.py, README.md), plugins/chora-construct cube start of the xp child.
Items (red first for each: a check that fails on b97369d1, committed alone, then the fix):
1. "XP did not start within 10 s" in all three construct sessions: find whether the persona session lacks something the gates give (binary path, env, assets, snapshot dir) or construct really fails. Harness: fix round.sh or the session setup. Product: do not fix; write the root cause with a repro command in the report and stop that item.
2. C2 compares windows to the start snapshot, so a window opened during the run reads as "new s1" and fails: compare against the windows open when the task starts (check.py begin/end), with a test.
3. T5 probe: round.sh or check.py starts chora-probe with the socket name, not a path (WAYLAND_DISPLAY), and fails loud when the probe dies (no vacuous pass), with a test.
4. Run the construct round's three personas again (round.sh, same caps) and put a short "## r1b" in reports/p2-r1-construct.md: per task (a) contract, (b) breaches, (c) reached, and whether I7 on every output is gone with xp running. Feedback files feedback/2026-11-20-p2r1b-<persona>.md. No ledger edits (T1 triage does that).
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,cube,pull,out`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: red commits before fixes, gate cells pass, r1b run done, reports on claude/session-log, branch pushed.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/p2-harness and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages; no host power call (stand-ins only).
