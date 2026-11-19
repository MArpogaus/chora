# p2-r1-xp (RC 2, P2 items 0b and 1): triage tool, then persona round 1 on xp

Model: sonnet, effort medium (driver of a scripted round; personas are haiku subagents, model haiku)
Lane: cloud, branch claude/pw at 5755b578 (pw r2, MERGE-OK, in train 78). Run only: no product or test code changes. One round, stop at 250 k. Cost about 1.1 %.
Step 0: run setup once: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`. Then `git fetch --unshallow` and `cargo build --workspace`; a failure: send_message the PM "p2-r1-xp BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-plan.md sections 1, 2, 4 item 3 and "## PM review r1"; feedback/README.md; findings/README.md; tests/persona/README.md on the branch.
1. TRI (on claude/session-log): feedback/README.md gains the `task:` and `check:` lines; tools/triage.py (about 40 lines, with tools/tests coverage like test_check_log.py) prints `new` findings grouped by (flavor, check) and flags severity high with `check: none`.
2. R1-xp: the three xp personas of section 1 (Erika, Murat, pixel-watcher), each a haiku Agent driving tests/persona/act.py only, under the section 2 limits (45 actions, 20 looks, no retries past 3, no coaching), via tests/persona/round.sh (binary snapshot once, 2 slots, PERSONA_CHECK=1). Seed the helper transcript first (plan section 2). After each persona: `check.py end TASK...` for its tasks.
3. Write feedback/2026-11-20-p2r1-<persona>.md per persona in the README form; each breach line and red contract an `issue` with its capture. Run `tools/triage.py` over the three; put its output in the report. No ledger edits (T1 triage does that).
Poll each run in the same turn; never end a turn to wait.
Done when: triage.py with its test on claude/session-log; three feedback files; reports/p2-r1-xp.md "## r1" with per task (a) contract, (b) breaches, (c) reached, and the triage output; sha of the round's binaries.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; no host power call (stand-ins only).
