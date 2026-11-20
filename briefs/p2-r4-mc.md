# p2-r4-mc (RC 2 persona round 4, P2 item 1): persona round 4 on mc after mc-fix r5 (mc-42 reopened, rv-r4-1)

Model: sonnet, effort medium (driver of a scripted round; personas are haiku subagents, model haiku)
Lane: cloud, branch claude/p2-r4-mc at 9dd23de6 (session branch fee9c695, train 88b, plus mc-fix bb7cbeb8 merged). Run only: no product or test code changes. One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "p2-r4-mc BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-plan.md sections 1, 2; reports/p2-r3-mc.md "## r3", reports/mc-fix.md (r3 to r5, PM review r5); findings/mc.md rows from P2 r1 to r3 (mc-42 and up) and their fix shas; feedback/README.md; tests/persona/README.md on the branch.
Items:
1. R4-mc: the same three mc personas as r1 to r3 (cook, sofa, nurse), each a haiku Agent driving tests/persona/act.py only, under the section 2 limits (45 actions, 20 looks, no retries past 3, no coaching), via tests/persona/round.sh (binary snapshot once, 2 slots, PERSONA_CHECK=1). Seed the helper transcript first. After each persona: `check.py end TASK...` for its tasks.
2. Fix check: for each mc finding fixed since r1 (mc-42 reopened in r3 first, then mc-43, mc-44, mc-46, mc-48, mc-49 and any newer row) (its row names `fixed <sha>` and an `act:` line), say whether the persona reached it and whether it holds (pass, fail, not reached), with the capture.
3. Write feedback/2026-11-20-p2r4-<persona>.md per persona in the README form; each breach and red contract an `issue` with its capture. Run `tools/triage.py` over the three; put its output in the report. No ledger edits (triage lane does that).
Poll each run in the same turn; never end a turn to wait.
Done when: three feedback files; reports/p2-r4-mc.md "## r4" with per task (a) contract, (b) breaches, (c) reached; the fix-check table; triage output; sha of the round's binaries; a stop-rule line per p2-plan.md (does mc meet round 4?).
Known, not findings: launcher rows read with fuzzel 1.9.2 in lanes (the PM runs 1.11.1); a launcher-only breach: name the fuzzel version.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; no host power call (stand-ins only).
