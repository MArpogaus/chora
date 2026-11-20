# p2-r3-co (RC 2 persona round 3, P2 item 1): persona round 3 on construct after co-majors r4, co-fix r1 (co-25 stray Super chords) and co-fix r2 (co-33 one-screen pull hint)

Model: sonnet, effort medium (driver of a scripted round; personas are haiku subagents, model haiku)
Lane: cloud, branch claude/p2-r3-co at bbaad12b (session branch ce2c436b plus co-majors r4 e0c3a266 and co-fix r1, r2). Run only: no product or test code changes. One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "p2-r3-co BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-plan.md sections 1, 2 (construct rows: explorer, Max, demo watcher); reports/p2-r2-construct.md; reports/co-fix.md (r1, r2, PM reviews); findings/construct.md rows from P2 r1 on (co-25 and up) and their fix shas; feedback/README.md; tests/persona/README.md on the branch.
Items:
1. R3-co: the same three construct personas as r2 (explorer mouse first; Max ricer, keys and mouse, with `round.sh probe`; demo watcher), with the p2-plan.md tasks, each a haiku Agent driving tests/persona/act.py only, under the section 2 limits (45 actions, 20 looks, no retries past 3, no coaching), via tests/persona/round.sh (binary snapshot once, construct 1280x800, 2 slots, PERSONA_CHECK=1). Seed the helper transcript first. After each persona: `check.py end TASK...` for its tasks.
2. Fix check: for each construct finding fixed since r2 (co-25, co-33, co-16, co-18 and any newer row), say whether the persona reached it and whether it holds (pass, fail, not reached), with the capture. For co-33: did a persona that tried a title drag see the hint and then pull by the helper (`cube.pull`)?
3. Write feedback/2026-11-20-p2r3co-<persona>.md per persona in the README form; each breach and red contract an `issue` with its capture. Run `tools/triage.py` over the three; put its output in the report. No ledger edits (triage lane does that).
Poll each run in the same turn; never end a turn to wait.
Done when: three feedback files; reports/p2-r3-co.md "## r3" with per task (a) contract, (b) breaches, (c) reached; the fix-check table; triage output; sha of the round's binaries; a stop-rule line per p2-plan.md (does construct meet round 3?).
Known, not findings: the hint flashes on a plain title click (co-fix pr2-1, backlog); I7 at the first click before the faces draw (r2, harness).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; no host power call (stand-ins only).
