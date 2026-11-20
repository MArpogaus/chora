# p2-r3-tiling (RC 2 persona round 3): persona round 3 on tiling after the hand-over fix (handover-ti: I3, T7)

Model: sonnet, effort medium (driver of a scripted round; personas are haiku subagents, model haiku)
Lane: cloud, branch claude/p2-r3-tiling at dcb77733 (session branch after train 92c, handover-ti merged). Run only: no product or test code changes. One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "p2-r3-tiling BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-plan.md sections 1, 2 (tiling rows); reports/p2-r2-tiling.md (stop rule: I3, T7 hand-over high, I5 medium); reports/handover-ti.md; findings/tiling.md; feedback/README.md; tests/persona/README.md on the branch.
Items:
1. R3-tiling: the same three tiling personas as r2 (Kemal keyboard first, beginner mouse first, ricer keys and an editor, ricer with T7 tiling -> xp), each a haiku Agent driving tests/persona/act.py only, under the section 2 limits (45 actions, 20 looks, no retries past 3, no coaching), via tests/persona/round.sh (binary snapshot once, 2 slots, PERSONA_CHECK=1). Seed the helper transcript first. After each persona: `check.py end TASK...` for its tasks. No driver action in a persona's log (r2 slip): scripted steps in a separate log.
2. Fix check: I3, T7 hand-over and I5 from r2, plus each tiling finding fixed since r2: pass, fail or not reached, with the capture.
3. Write feedback/2026-11-20-p2r3-<persona>.md per persona in the README form; each breach and red contract an `issue` with its capture. Run `tools/triage.py` over the three; put its output in the report. No ledger edits.
Poll each run in the same turn; never end a turn to wait.
Done when: three feedback files; reports/p2-r3-tiling.md "## r3" with per task (a) contract, (b) breaches, (c) reached; the fix-check table; triage output; sha of the round's binaries; a stop-rule line per p2-plan.md (does tiling meet round 3?).
Known, not findings: ti-1 is fuzzel below 1.11 (name the version); ti-12 and ti-28 (keymap by design) are duplicates; L1 and L2 harness limits of r2.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; no host power call (stand-ins only).
