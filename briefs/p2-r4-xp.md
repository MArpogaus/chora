# p2-r4-xp (RC 2 persona round 4, P2 item 1): persona round 4 on xp after xp-36 (train 87e)

Model: sonnet, effort medium (driver of a scripted round; personas are haiku subagents, model haiku)
Lane: cloud, branch claude/p2-r4-xp at fee9c695 (session branch, train 88b; xp-36 fix 3290f1d4 merged in t87e). Run only: no product or test code changes. One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "p2-r4-xp BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-plan.md sections 1, 2; reports/p2-r3-xp.md "## r3"; reports/xp-36*.md or the xp-36 brief; findings/xp.md rows from P2 r1 to r3 (xp-2x, xp-3x) and their fix shas; feedback/README.md; tests/persona/README.md on the branch.
Items:
1. R4-xp: the same three xp personas as r1 to r3 (Erika, Murat, pixel-watcher), each a haiku Agent driving tests/persona/act.py only, under the section 2 limits (45 actions, 20 looks, no retries past 3, no coaching), via tests/persona/round.sh (binary snapshot once, 2 slots, PERSONA_CHECK=1). Seed the helper transcript first. After each persona: `check.py end TASK...` for its tasks.
2. Fix check: for each xp finding fixed since r2 (xp-36, xp-35, xp-24, and xp-26, xp-27, xp-33 that r3 did not reach) (its row names `fixed <sha>` and an `act:` line), say whether the persona reached it and whether it holds (pass, fail, not reached), with the capture.
3. Write feedback/2026-11-20-p2r4-<persona>.md per persona in the README form; each breach and red contract an `issue` with its capture. Run `tools/triage.py` over the three; put its output in the report. No ledger edits (triage lane does that).
Poll each run in the same turn; never end a turn to wait.
Done when: three feedback files; reports/p2-r4-xp.md "## r4" with per task (a) contract, (b) breaches, (c) reached; the fix-check table; triage output; sha of the round's binaries; a stop-rule line per p2-plan.md (does xp meet round 4?).
Known, not findings: lane containers have fuzzel 1.9.2: the Run box (xp-24) and launcher rows do not open there (r3 "Run... opens no Run box"; the gate now says SKIP for it, gate-silent); count such a task as not reached with the fuzzel version, not as a breach. e2e red on gate-tiling-apps-click in lanes (F27/ti-1).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; no host power call (stand-ins only).
