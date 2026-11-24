# review tour-base r5 (F7, rc 11): seeded review of claude/tour-base-r5 before its merge

Model: sonnet, effort high (harness verdict logic plus intended.toml rows; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/tour-base-r5 head 61d0ef7a, base d0801b7e (5 files, +1237 -761; intended.toml most of it). One round. Stop at 120 k; cost at most 0.5 %; at most 8 findings. No merge gate run (lane gate: 55 PASS, acts 1 blocking xp I6 'Starting Notepad' 1.04:1, xp acts rerun 80 pass). No tour takes; check by git log, diffs, tests and design docs.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `git fetch origin claude/session-log`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q`, `python3 -I tests/test_tour_diff.py`. A failure: send_message the PM "review-tour-base-r5 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/tour-base-r5.md (done criteria), reports/tour-base.md "## r5" and "## PM review r4" (rtb4-1); reports/tour-v9-judge.md "## r1" (rv9j-4, rv9j-5, its table of rows); design/tour-v9.md; `git diff d0801b7e..61d0ef7a`.
Checks, each with its command and files:
1) Red first: 51a44e9f, 1deb1bc9, a12bc57e, 60e6687d (tests only) before the fixes 6d01db40, dd356a8f, 671b426e, 61d0ef7a; each red test fails at the last red sha and passes at head (paste lines).
2) rtb4-1: base absent counts as not measured, not regression (one command).
3) rv9j-5: a row with `nodes` absorbs only those nodes; a node of the same kind outside stays `open`; a row without `nodes` matches as before. KISS: one optional field, no new verdict.
4) Rows: each new or corrected row cites the judge's act and capture; the rows the brief named (29281054, fc9675ac, 11e34cf8, 94579f91, d9b2c4f1 corrected; 62708d35, d78fc4bb, 86343e3f, 6d336bdf, f1397d98 added) match the judge's table; no row for xp-47, rv9j-1, rv9j-3. Sample 10 other changed rows: a cause naming "open", "proposed", "cause not found" or an open findings id has open = true.
5) Counts: before 1 same/14 change/82 open/2 regression/11 none, after 1/27/71/0/11. Explain the two regressions that left (x6b, c4 to open): honest, or hidden by a broad row? A path that lets a later unrelated diff pass as `change` or `same` is MAJOR (owner 11-17: no partial credit).
6) rv9j-4: handover.py starts the bar and wallpaper as tiling.py:143 does; the test checks the command lines.
7) The acts blocking line: confirm the diff touches no file the acts cell runs (demo/tour and tests/tour* only), so it is foreign (xp-38 family).
Known, not findings: rv9j-1, rv9j-3, xp-47 (fixed elsewhere), the 71 open (F7 rest).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtb5-N, severity, file:line, text, fix in one sentence). Append to reports/tour-base.md on claude/session-log under "## Review r5".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
