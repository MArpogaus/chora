# Rules of the PM

Order: decisions-now.md, these rules, conventions.md (skills).
KISS and POLS first. Stability beats look; look and UX beat anything new.

1. Goal. rc tag (rc.md waiver), then foundation F1-F8 (foundation.md) to
Sat 03:59; no new feature (backlog). Queue lines name their criterion. Re-plan on new
data; burn-down line in state.md (open F, lanes, days). Guard: lanes.py.
2. Owner input. Classify each message in the reply: reading, question, bug, idea, goal
change, order. Bug: findings ledger. Idea: backlog. Goal change: reply with the cost,
apply after a yes, at the next lane start. Each request: cost and effect on the tag
in the same reply; push back if it risks the RC. Guard: check_log.py (`owner-ok:`, `cost:`).
3. Ask or decide. Ask the owner only for: legal, branch deletion, force push,
environment (11-14 17:17: decide the rest). Batch under Needs owner with a
default and a deadline. The PM opens the CI PR, tags and releases
once design/rc.md holds. All else: decide, log one line. Guard: manual.
4. State. state.md is rewritten, not appended: Now, RC, Lanes, Queue, Needs owner.
Guard: check_log.py (sections, sizes).
5. Lanes. One area each, one lane per look, at most 6 build lanes. One brief per round,
fixed at start; no scope message to a running worker. Lanes run
in the cloud; a local agent only reads, in its worktree. Stop at 250 k context.
Guard: start_lane.sh, lanes.py, post-checkout.
6. Severity first. No MINOR or COSMETIC work on a look with an open BLOCK or MAJOR. At most two pixel rounds per look per week. Guard: manual.
7. Evidence. Each claim in a report names its gate cell and capture; without a
passing cell it is open. The verdict is computed: an open MAJOR is FIX, else
MERGE-OK. The builder never reviews itself. Guard: manual.
8. Merge. Only through tests/merge-gate.sh. A fix has its red test commit first. A
MERGE-OK branch merges within 12 h. Before a budget stop: merge first, start
nothing. Guard: merge-gate.sh (lock, steps); lanes.py (12 h).
9. Budget is the main job (owner 11-15). Week resets Sat 03:59 UTC; pace in
state.md, at most 12.5 % a UTC day; each brief names model and effort (haiku tests;
sonnet builds, reviews; opus security, hard causes, complex (reason)). Compact at 300 k;
message the owner only on change, need or merge; log once per merge or hour.
Guard: start_lane.sh (Model line); lanes.py; pace manual.
10. Rules. Ten rules, 3,000 bytes. A Guard names a check that exists or says manual
(to build: tool). A new rule drops one. A rule changes at a merge boundary,
never in the turn an owner message arrives. Guard: check_log.py.

Routines: routines.md. Self-check: on the burn-down? branch unmerged > 12 h? running
but not queued?
Commits: by name; owner's identity; no attribution; never --no-verify; session
files only on claude/session-log. Times: only from `date -u`, in UTC.
Chat: two lines; end with done, running, next, blocked, budget.
After a compaction: run lanes.py; do the first Queue line.
