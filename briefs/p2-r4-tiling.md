# p2-r4-tiling (RC 2 persona round 4, plans/week-3.md row 15): tiling round 4 with the fixed harness (P2-HARNESS-L, train 103)

Model: sonnet, effort medium (driver of a scripted round; personas are haiku subagents, model haiku)
Lane: cloud, branch claude/p2-r4-tiling at 1c9e64f7 (session branch after train 103: P2-HARNESS-L, H-GUARDS, TI-XDG merged). Run only: no product or test code changes. One round, stop at 250 k. Cost at most 0.6 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "p2-r4-tiling BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-plan.md sections 1, 2 (tiling rows); reports/p2-r3-tiling.md (harness limits L1-L3); reports/p2-harness-l.md (what changed in the harness); findings/tiling.md; feedback/README.md; tests/persona/README.md on the branch.
Items:
1. R4-tiling: the same three tiling personas as r3 (Kemal keyboard first, beginner mouse first, ricer keys and an editor with T7 tiling -> xp), each a haiku Agent driving tests/persona/act.py only, under the section 2 limits (45 actions, 20 looks, no retries past 3, no coaching), via tests/persona/round.sh (binary snapshot once, 2 slots, PERSONA_CHECK=1). Seed the helper transcript first. After each persona: `check.py end TASK...` for its tasks. Persona logs hold persona actions only; scripted steps in a separate log; no Bash outside act.py by a persona (r3 slip).
2. Harness check: L1, L2, L3 and T7 now compare against the right baseline (p2-harness-l); each: pass, fail or not reached, with the capture. A harness red that remains is named as such, never counted as a pass.
3. Fix check: every tiling finding fixed since r3 (findings/tiling.md): pass, fail or not reached, with the capture.
4. Write feedback/2026-11-21-p2r4-<persona>.md per persona in the README form; each breach and red contract an `issue` with its capture. Run `tools/triage.py` over the three; put its output in the report. No ledger edits.
Poll each run in the same turn; never end a turn to wait.
Done when: three feedback files; reports/p2-r4-tiling.md "## r4" with per task (a) contract, (b) breaches, (c) reached; the harness-check and fix-check tables; triage output; sha of the round's binaries; a stop-rule line per p2-plan.md (does tiling meet round 4: L1-L3 pass, 0 breaches?).
Known, not findings: ti-1 is fuzzel below 1.11 (name the version); ti-12 and ti-28 (keymap by design) are duplicates; T3 "Hello" twice is the fake agent's fixed answer (env).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; no host power call (stand-ins only).
