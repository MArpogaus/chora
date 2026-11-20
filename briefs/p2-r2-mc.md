# p2-r2-mc (RC 2 persona round 4, P2 item 1): persona round 2 on mc after mc-p2-fix and mc-remote (MERGE-OK, train 85)

Model: sonnet, effort medium (driver of a scripted round; personas are haiku subagents, model haiku)
Lane: cloud, branch claude/p2-r2-mc at dac835dd (mc-remote, contains mc-p2-fix 17d9a798; both MERGE-OK, train 85). Run only: no product or test code changes. One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "p2-r2-mc BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-plan.md sections 1, 2; reports/p2-r1-mc.md "## r1" (its deviation and harness notes); reports/mc-p2-fix.md, reports/mc-remote.md; findings/mc.md rows from P2 r1 and their fix shas; feedback/README.md; tests/persona/README.md on the branch.
Items:
1. R2-mc: the same three mc personas as r1 (cook, sofa, nurse; the nurse runs T7 xp -> mc this time; if check.py start fails NotQuiet again, record it as an issue with the write path and run the nurse on plain mc), each a haiku Agent driving tests/persona/act.py only, under the section 2 limits (45 actions, 20 looks, no retries past 3, no coaching), via tests/persona/round.sh (binary snapshot once, 2 slots, PERSONA_CHECK=1). Seed the helper transcript and one agenda item first (r1 left M2 untestable). The sofa persona also uses the remote keys XF86Back and XF86HomePage (mc-46). After each persona: `check.py end TASK...` for its tasks.
2. Fix check: for each mc finding fixed since r1 (its row names `fixed <sha>` and an `act:` line), say whether the persona reached it and whether it holds (pass, fail, not reached), with the capture.
3. Write feedback/2026-11-20-p2r2-<persona>.md per persona in the README form; each breach and red contract an `issue` with its capture. Run `tools/triage.py` over the three; put its output in the report. No ledger edits (triage lane does that).
Poll each run in the same turn; never end a turn to wait.
Done when: three feedback files; reports/p2-r2-mc.md "## r2" with per task (a) contract, (b) breaches, (c) reached; the fix-check table; triage output; sha of the round's binaries; a stop-rule line per p2-plan.md (does mc meet round 4?).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; no host power call (stand-ins only).
