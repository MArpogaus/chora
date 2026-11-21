# p2-r5-co (RC 2 persona round 5 on construct): after co-keys r3 (t101) closed co-41

Model: sonnet, effort medium (driver of a scripted round; personas are haiku subagents, model haiku)
Lane: cloud, branch claude/p2-r5-co at 892d035c (session head after train 101: co-keys r3, guide-checks, ti-29, mc-minor, core-rv). Run only: no product or test code changes. One round, stop at 250 k. Cost at most 0.8 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; then `bash tests/persona/round.sh --help` and one smoke `round.sh begin smoke` + `check.py end smoke` on construct. A failure: send_message the PM "p2-r5-co BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-plan.md sections 1, 2 (construct rows); reports/p2-r4-co.md (r4, its T2 fail and stop rule); reports/co-keys.md (r3); findings/construct.md rows co-25 and up; feedback/README.md; tests/persona/README.md on the branch.
Items:
1. R5-co: the same three construct personas as r4 (explorer mouse first; Max ricer, keys and mouse, with `round.sh probe`; demo watcher), the p2-plan.md tasks T1-T7 and C1-C3, each a haiku Agent driving tests/persona/act.py only, under the section 2 limits (45 actions, 20 looks, no retries past 3, no coaching), via tests/persona/round.sh (binary snapshot once, construct 1280x800, 2 slots, PERSONA_CHECK=1). Seed the helper transcript first. `round.sh begin TASK` before each goal and `check.py end TASK` after it.
2. Fix check against r4: T2 (co-41: the key list by mouse and by F1, co-keys r3), and every r4 breach and red contract. Each: pass, fail or not reached, with the capture.
3. feedback/2026-11-21-p2r5co-<persona>.md per persona in the README form; each breach and red contract an `issue` with its capture. Run `tools/triage.py` over the three; its output into the report. No ledger edits.
Poll each run in the same turn; never end a turn to wait.
Done when: three feedback files; reports/p2-r5-co.md "## r5" with per task (a) contract, (b) breaches, (c) reached; the fix-check table; triage output; sha of the round's binaries; a stop-rule line per p2-plan.md (does construct meet round 5: T1-T7, C1-C3 reached, no new check-backed BLOCK or MAJOR?).
Known, not findings: the hint flashes on a plain title click (co-fix pr2-1, backlog); the co-42 face hint (lane co-hint running); construct rx-1 swallowed Super chords after Super+F1 (lane keys-route running); I7 at the first click before the faces draw (r2, harness).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; no host power call (stand-ins only).
