# handover-ti r1 (RC 2, shared bar "no lost window in any handover"): tiling -> xp hand-over, one focus holder

Model: opus, effort high, because the fix is in the core focus and hand-over path across two running looks (complex, cross-look state)
Lane: cloud, branch claude/handover-ti at ce2c436b (session head). Stop at 250 k. Cost about 2 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "handover-ti BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-r2-tiling.md "## r2", feedback/2026-11-20-p2r2-ricer.md (the two high rows and I5), media/2026-11-20/persona-tiling-r2-ricer/ (breaches.jsonl, shots 013-017, evidence 001-004).
Findings (session tiling+xp 1280x800, terminal s1 open in tiling, then Super+Alt+1, then Apps "look"):
1. I3: "tiling and xp each say they hold the focus"; "the core focuses xp-icons of xp, tiling says it holds the focus". One holder only, always.
2. T7: the screen shows the xp desktop, xp lists no window, tiling still lists s1 shown (rect 14,54,619,702). Find what Super+Alt+1 does (give s1 to look 1, or show look 1). If it gives: the window must arrive in xp (taskbar button, visible, focused). If it shows a look: tiling must not report s1 as shown or focused while it is not on screen. Either way, a window is never on no visible screen.
3. I5 (medium) only if 1-2 are done under 150 k: the key list must not overlap a notice.
Each item: a red test commit first (a gate cell script under tests/ added to the matrix row of merge-gate.sh, or a unit test where the logic lives), then the fix. Reuse tests/gate-handover-matrix.sh and gate-give-holders.sh helpers; no new copies (F2).
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,matrix,virtual,wm-pane,mc-handover,layout` plus persona-selftest if tests/persona changed. Poll the gate in the same turn; never end a turn to wait.
Done when: each item red then green with shas; gate PASS lines; reports/handover-ti.md "## r1" with per item sha, test, cell, capture.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/handover-ti and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
