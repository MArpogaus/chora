# p2-r1-construct (RC 2, P2 item 1): persona round 1 on construct

Model: sonnet, effort medium (driver of a scripted round; personas are haiku subagents, model haiku)
Lane: cloud, branch claude/p2-construct at b97369d1 (session branch after train 80: persona env fixed, persona-env r1). Run only: no product or test code changes. One round, stop at 250 k. Cost about 1.1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace` (then `cargo build --workspace`); a failure: send_message the PM "p2-r1-construct BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-plan.md sections 1, 2, 4 item 3; reports/p2-r1-xp.md (the xp round: its form, its "## PM review r1" on harness slips); reports/persona-env.md; feedback/README.md; findings/README.md; tests/persona/README.md on the branch.
1. R1-construct: the three construct personas of plan section 1 (explorer, Max, demo watcher), each a haiku Agent driving tests/persona/act.py only, under the section 2 limits (45 actions, 20 looks, no retries past 3, no coaching), via tests/persona/round.sh (binary snapshot once, 2 slots, PERSONA_CHECK=1). Seed the helper transcript first (plan section 2). After each persona: `check.py end TASK...` for its tasks.
2. A contract red because the harness lacks a stand-in (not the look): name it "harness" in the report, not an issue.
3. Write feedback/2026-11-20-p2r1-<persona>.md per persona in the README form; each breach line and red contract an `issue` with its capture. Run `tools/triage.py` over the three; put its output in the report. No ledger edits (the PM triages).
Poll each run in the same turn; never end a turn to wait.
Done when: three feedback files; reports/p2-r1-construct.md "## r1" with per task (a) contract, (b) breaches, (c) reached, harness reds apart, and the triage output; sha of the round's binaries.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; no host power call (stand-ins only).
