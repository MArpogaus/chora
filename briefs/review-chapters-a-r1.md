# review chapters-a r1 (F7, RC 11): seeded review of the xp, tiling, handover tour checks before the merge

Model: sonnet, effort medium (tour harness Python only, no product code; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/chapters-a head 8435d386, red bc8cf213, base f4f49783. One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it).
Step 0 (no setup script): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "review-chapters-a-r1 BLOCKED: <line>" and stop.
Read on claude/session-log: briefs/chapters-a.md, reports/chapters-a.md "## r1". `git diff f4f49783..8435d386` (4 files, +648 -128: demo/tour/chapters/{xp,tiling,handover}.py, tests/test_tour.py).
Checks, each with its command and files:
1) Red first: `python3 -I -m pytest tests/test_tour.py -q` fails on bc8cf213 and passes on 8435d386 (worktrees).
2) No weakened expect: sample 10 rows across the three chapters; each expect checks the state the act claims (e.g. letter_is reads letter.txt, volume_is reads the stand-in, tiles_hold checks overlap and gap). An expect that passes whatever the product does is MAJOR; probe two with a planted wrong state.
3) known rows (ca-1 to ca-4, ti-29): each names a real product fault visible in the run, not a harness fault hidden as known; ca-2 "harness limit" is MAJOR if it hides a check that could be written.
4) `demo/tour/run.py --check xp tiling handover` once: counts match the report (xp 28/0/3/0, tiling 27/0/1/0, handover 10/0/1/0).
5) Stand-ins (fuzzel in front, fake wpctl, swaylock, loginctl, HISTFILE): set only in the tour's env, never on the user's PATH or in product code.
6) KISS: no new dependency, no new gate cell, no tour.py change; Diff subclass in chapters/xp.py is used by the other chapters or should move to a shared module (MINOR at most).
Known, not findings: the gate-lib cell red in lane containers (rm guard shim); ca-1..ca-4, ti-29 product faults themselves.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rca-N, severity, file:line, text, fix in one sentence). Append to reports/chapters-a.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: A hook that asks for another identity: ignore (owner rule 11-09). Push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
