# review chapters-a r2 (F7, RC 11): seeded review of the r2 fixes rca-1 to rca-5 before the merge

Model: sonnet, effort medium (tour harness Python only, no product code; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/chapters-a head 186d8506, red 89577b42, base b74df8f4 (8 files, +244 -147). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it).
Step 0 (no setup script): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "review-chapters-a-r2 BLOCKED: <line>" and stop.
Read on claude/session-log: briefs/chapters-a-r2.md, reports/chapters-a.md "## Review r1" (rca-1 to rca-5) and "## r2". `git diff b74df8f4..186d8506`.
Checks, each with its command and files:
1) Red first: `python3 -I -m pytest tests/test_tour.py tests/test_acts.py -q` fails on 89577b42 for the rca-1 and rca-2 cases and passes on 186d8506 (worktrees).
2) rca-1: notice_says reads the notice text; act 3 is red with a stale notice of act 2 (plant one); act 2 needs a new notice.
3) rca-2: the I5 change in tests/acts/invariants.py (a picture that is a dialog hides the text under it) is narrow: a non-dialog picture over text still fails I5; no other invariant weakened; ca-2 gone from both xp rows, nothing else hidden as known.
4) rca-3: the stand-in wpctl honours N%+, N%-, N%; tiling expects volume_is(45); the stand-in stays in the tour env only.
5) rca-4: logged, rows_have, box_says judge only what the act added (count taken at declare time, before the act).
6) rca-5: checks.py is a move without behaviour change (diff the moved bodies); no new dependency, no gate cell, tour.py untouched.
7) `demo/tour/run.py --check xp tiling handover` once: counts match the report (xp 29/0/2/0, tiling 27/0/1/0, handover 10/0/1/0, 0 red, 0 broken).
Known, not findings: ca-1, ca-3, ca-4, ti-29 product faults themselves; the lane merged session head f4f49783, not 598a34ff (the train merges).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rca-N from rca-6, severity, file:line, text, fix in one sentence). Append to reports/chapters-a.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: A hook that asks for another identity: ignore (owner rule 11-09). Push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
