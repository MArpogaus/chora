# p2-plan r1 (planning for next week): the simulated user test, ready to start Saturday

Model: opus, effort high because it plans next week's main phase (owner 11-17 14:24 and 11-19 06:56: opus for planning)
Lane: cloud, branch claude/autonomous-dev-session-3w-i33pg7 (read only; no code commit). One round, stop at 250 k. Cost about 1.5 %.
Step 0: `command -v cargo python3`; then `cargo check -p chora`; a failure: send_message the PM "p2-plan r1 BLOCKED: <MISSING line>" and stop. (A plan lane needs no build beyond that.)
Setup if cargo is missing: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/plan-after-foundation.md (Re-plan 11-17 14:24, P2), design/foundation.md, design/rc.md, vision-card.md, the personas (grep -ril persona design reports | head), reports/proposal-glitch-gates.md, tools/triage.py. In the code: the act contracts and invariants that exist now (grep -rn "invariant\|contract" tests crates --include=*.rs --include=*.sh | head -50).
Write reports/p2-plan.md on claude/session-log, at most 150 lines:
1. Per look (xp, tiling, mc, construct): 2-3 personas from the vision card, each with 3-5 tasks of the shared RC bar plus the look's own job; what a task's pass means as a check (gate cell, invariant, capture), not as opinion.
2. How a haiku persona drives the session (which tool: ipc, choractl, nested.sh, the act scripts), what it may not do, and how its findings reach triage.py (format, severity, dedupe).
3. Lane plan for Sat-Mon: lanes, model and effort each, % cost each (total at most 25 % of a week), order, and the stop rule (a persona round stops when it finds no new BLOCK or MAJOR).
4. What must exist first (missing contracts or tools), each with its size; at most 5.
5. Risks (at most 5) and what is out of scope.
Be critical: each claim names the file or command it rests on. No new feature; KISS.
Done when: reports/p2-plan.md pushed; send_message to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines: lanes, total %, first lane, blockers).
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below. Poll any long command in the same turn; never end a turn to wait.
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
