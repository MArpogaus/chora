# review tour-base r7 (F7, rc 11: rtb6-1 MAJOR, rtb6-4, rtb6-5): seeded review of claude/tour-base-r7 before its merge

Model: sonnet, effort high (harness verdict logic plus intended.toml rows; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/tour-base-r7 head 7c2cdc95, base 9311e155 (3 files, +223 -99). One round. Stop at 120 k; cost at most 0.5 %; at most 8 findings. No merge gate run (the train runs it; the lane's gate passed only in pieces, not a finding by itself). No tour takes.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `git fetch origin claude/session-log`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q`, `python3 -I tests/test_tour_diff.py`. A failure: send_message the PM "review-tour-base-r7 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/tour-base-r7.md (done criteria), reports/tour-base.md "## r7" and "## Review r6" (rtb6-1..5); design/tour-v9.md; `git diff 9311e155..7c2cdc95`.
Checks, each with its command and files:
1) Red first: 88aa6e17, 9608747f (tests only) before 0ce3e09c, 7c2cdc95; each red test fails at its red sha and passes at head (paste lines). A fix with no red test before it is MAJOR.
2) rtb6-1 (tests/tour-diff.py): strict is the default for every row kind; the table-driven test covers every kind tour-diff knows (read its source, list the kinds); any kind that still absorbs an unnamed node is MAJOR (owner 11-17: no partial credit).
3) The 42 bare rows: each now names nodes or boxes, or is open; sample 12; a row naming a node its act's capture does not show is MINOR.
4) rtb6-4, rtb6-5: rows cite their own act's capture; m1 names rv9j-2.
5) Counts: before 1 same/4 change/91 open/3 regression/11 none, after 1/2/93/3/11. m4, m4b change to open honest? The 3 regressions x2, x2b, x8 (notify card added, paperclip blink): take noise or a real regression? Say which with the capture names; no row may hide them.
Known, not findings: rv9j-1..5 (MINOR), the 93 open (F7 rest, v9 take after merge).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtb7-N, severity, file:line, text, fix in one sentence). Append to reports/tour-base.md on claude/session-log under "## Review r7".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
