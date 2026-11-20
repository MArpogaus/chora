# tour-plan r1 (F7, RC 11): plan of tour v9 with automatic checks, before the build next week

Model: opus, effort high, because this is planning (owner 11-19 06:56): the scenes, checks and the v8 comparison fix the cost of F7 next week
Lane: cloud, read-only on code (branch claude/autonomous-dev-session-3w-i33pg7 at 9dc384b3); writes only design/tour-v9.md and reports/tour-plan.md on claude/session-log. One round, stop at 150 k; cost at most 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "tour-plan r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: vision-card.md, design/foundation.md (F7), design/rc.md (11), design/tour-v4.md, decisions-now.md lines 11-17 14:24Z and 11-17 10:59Z, reports/review-tour-v7.md, backlog.md DB11 and xp-T7, the persona reports of the last round per look (reports/p2-*.md or the newest persona report per look). On the code branch: the tour scripts (find them: `git grep -l -i tour -- tests tools`), the gate cells and tests/persona.
Write design/tour-v9.md (at most 250 lines):
1. One scenario per look for its own users (vision card), plus one hand-over chapter that moves one window through all four looks (xp-give, Super+Alt+N); each act names the shared RC bar door it shows.
2. Per act its automatic check: the gate cell or scene query that must hold (no new cell without saying which existing one it extends), and the frame where the capture is taken.
3. The v8 comparison (F7): how each chapter's captures are compared to tour v8 (media of build 733967aa): pixel diff with masks for clocks and text, or a scene diff; the threshold; what counts as a change versus a regression.
4. Visual validation against design/looks/* guidelines: which checks are automatic, which a fresh-eyes review does (rules 7).
5. Build plan: lanes, model, effort, % cost each, order; reuse of the v4-v8 scripts (DB11: one set, not copies, F2).
Done when: design/tour-v9.md and reports/tour-plan.md "## r1" (open questions, risks, the cost total) pushed on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines). Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages; no change to code branches.
