# p2-r4-co (RC 2 persona round 4 on construct): after co-pane (t94b), co-hud-load and give-raise (t95b) and persona-begin (t96)

Model: sonnet, effort medium (driver of a scripted round; personas are haiku subagents, model haiku)
Lane: cloud, branch claude/p2-r4-co at SHA_BASE (session head after train 96). Run only: no product or test code changes. One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; then `bash tests/persona/round.sh --help` and one smoke `round.sh begin smoke` + `check.py end smoke` on construct (the begin of r2 must work: t95 was red there). A failure: send_message the PM "p2-r4-co BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-plan.md sections 1, 2 (construct rows); reports/p2-r3-co.md (r3 and its stop rule); reports/co-pane.md, reports/persona-begin.md; findings/construct.md rows co-25 and up; feedback/README.md; tests/persona/README.md on the branch.
Items:
1. R4-co: the same three construct personas as r3 (explorer mouse first; Max ricer, keys and mouse, with `round.sh probe`; demo watcher), the p2-plan.md tasks, each a haiku Agent driving tests/persona/act.py only, under the section 2 limits (45 actions, 20 looks, no retries past 3, no coaching), via tests/persona/round.sh (binary snapshot once, construct 1280x800, 2 slots, PERSONA_CHECK=1). Seed the helper transcript first. `round.sh begin TASK` before each goal and `check.py end TASK` after it (persona-begin r2).
2. Fix check against r3: C2 (window in the xp face after a pull; co-pane), Max's I3 on a pane dragged out of the picture, the pull hint on a held drag (co-33), co-35 (Tab list after the flight). Each: pass, fail or not reached, with the capture.
3. feedback/2026-11-21-p2r4co-<persona>.md per persona in the README form; each breach and red contract an `issue` with its capture. Run `tools/triage.py` over the three; its output into the report. No ledger edits.
Poll each run in the same turn; never end a turn to wait.
Done when: three feedback files; reports/p2-r4-co.md "## r4" with per task (a) contract, (b) breaches, (c) reached; the fix-check table; triage output; sha of the round's binaries; a stop-rule line per p2-plan.md (does construct meet round 4: no new check-backed BLOCK or MAJOR?).
Known, not findings: the hint flashes on a plain title click (co-fix pr2-1, backlog); I7 at the first click before the faces draw (r2, harness).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; no host power call (stand-ins only).
