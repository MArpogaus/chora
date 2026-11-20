# mc-p2-fix-r1 (RC 2 and shared bar; P2 plan row 3 FIX mc): mc-42 BLOCK, mc-43, mc-44 MAJOR, mc-45 verify

Model: sonnet, effort high (named findings with captures and the cell to build; a seeded review follows)
Lane: cloud, branch claude/mc-p2-fix from 79c8d2b5 (session branch, train 81). One round, stop at 250 k. Cost about 2 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "mc-p2-fix BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/mc.md rows mc-42 to mc-45 (and mc-6, mc-38 named there); reports/p2-triage.md "## T1" mc part; feedback/2026-11-20-p2r1-cook.md, -sofa.md, -nurse.md and their captures. On the branch: plugins/chora-mc (pages, column, keys sheet, launcher), tests/gate-doors.sh, tests/mc-paths cell, tests/persona/check.py contracts T2, T4, T6.
Items (severity order; each a red gate test commit first, then the fix):
1. mc-42 BLOCK: on a window page (Foot), Left or Escape from the remote returns to the page before and the column doors stay reachable; Super+F1 works there. Cell to build: `mc-paths terminal-back`. Keys that the client must get in normal typing stay the client's; say in the report which keys mc takes on a window page and why (POLS, remote path), and add the rule to docs (mc section).
2. mc-43 MAJOR: Super+F1 on a window page draws the key sheet above the window. Cell: gate-doors keys-key on a window page.
3. mc-44 MAJOR: an unknown program name plus Return in the mc launcher draws a notice or "Nothing found", as xp-2 and ti-2 do. Cell: `mc launcher-none`.
4. mc-45 verify: read nurse shot 011 first. If the focus holder after Escape from the chat is nobody, red test then fix (focus back to the page); if it is the chat, close as not-a-bug with the capture.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,chat-keys,doors,mc-paths,mc-sofa,mc-handover,matrix,layout`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: red commits before fixes, gate cells pass, before and after captures in reports/mc-p2-fix/, reports/mc-p2-fix.md "## r1" on claude/session-log (per item: red sha, fix sha, test, gate cell, capture); ledger rows mc-42 to mc-45 state `fixed <sha>` or the verify result.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/mc-p2-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
