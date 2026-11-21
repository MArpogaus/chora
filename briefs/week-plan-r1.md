# week-plan r1 (planning, rc 1-11, F5, F7): the plan for the week 11-21 04:00 to 11-28 03:59 UTC

Model: opus, effort high because it is the week's planning (owner 11-19 06:56: opus for planning) and must rank refactor slices against the RC and the vision card
Lane: cloud, read-only on code; writes on claude/session-log only. Cost at most 2 %; stop at 250 k context. One area: planning files.
Why: the critic of 11-21 12:50 found no plan for the new week (routines.md "a sprint starts with a plan"), Queue lines worth about 3 % of a 95 % week, a stale RC table (state.md), and rules.md rule 1 naming the old deadline.
Step 0: `git fetch --unshallow`, `cargo check --workspace` (to read the code with a working build); a failure: send_message the PM "week-plan BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: decisions-now.md, rules.md, vision-card.md, design/rc.md, design/foundation.md, state.md, backlog.md (refactor proposals, FLK rows, open findings), reports/week-*.md, reflections/selfcheck-2026-11-21.md. On claude/autonomous-dev-session-3w-i33pg7: debt-*.md, the lint ratchet, files over 1,000 lines.
1) RC table: for each design/rc.md criterion, state met or open on the session head 1c599c74 with the evidence (gate run, report, capture) and its date; a criterion whose evidence predates a rule change (e.g. greens at --jobs 1) is open. What closes each open one, in lane-sized slices.
2) Week plan: day by day to 11-28 03:59 UTC, at 6 cloud lanes and about 13 % of the week per day: F5 and F7 first (the carried goals), then the RC open criteria, then the top refactor slices (ranked by risk removed per %; debt counts never grow). Each slice: area, model and effort (haiku tests, sonnet routine, opus complex), % cost, done criterion, the gate cell that proves it. Sum of slices about 85 % of the week (5 % reserve).
3) Check against vision-card.md: stability and looks over new features; name any slice that is a new feature and drop it.
4) Draft rule 1 of rules.md for the new week (the same size as today's).
Never end a turn to wait; poll anything you start in the same turn.
Done when: plans/week-3.md on claude/session-log (RC table, ranked slices with %, day plan, rule 1 draft), every slice names its criterion.
Report: send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines: open RC criteria, top 6 slices with %, total %.
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; no code changes.
