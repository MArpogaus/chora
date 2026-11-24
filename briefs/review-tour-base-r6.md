# review tour-base r6 (F7, rc 11: rtb5-1 MAJOR, rtb5-2, rtb5-3 MINOR): seeded review of claude/tour-base-r6 before its merge

Model: sonnet, effort high (harness verdict logic plus intended.toml rows; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/tour-base-r6 head 9311e155, base 61d0ef7a (5 files, +634 -85; intended.toml most of it). One round. Stop at 120 k; cost at most 0.5 %; at most 8 findings. No merge gate run (lane gate EXIT 0, 65 PASS). No tour takes; check by git log, diffs, tests and design docs.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `git fetch origin claude/session-log`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q`, `python3 -I tests/test_tour_diff.py`. A failure: send_message the PM "review-tour-base-r6 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/tour-base-r6.md (done criteria), reports/tour-base.md "## r6" and "## Review r5" (rtb5-1..3); findings/shared.md rtb5-1..3; design/tour-v9.md; `git diff 61d0ef7a..9311e155`.
Checks, each with its command and files:
1) Red first: 356120cc, 1d59e949, 5214aa8e (tests only) before the fixes 41140d74, 632ea8bd, 9408d181, 9311e155; each red test fails at the last red sha and passes at head (paste lines). A fix with no red test before it is MAJOR.
2) rtb5-1 (tests/tour-diff.py): added and visible rows match only their `nodes`, pixel rows only their `boxes`; a row without them leaves its act open; compare() passes the blobs. Try a later unrelated diff in a covered act: does it stay open? Any path left that absorbs an unnamed diff is MAJOR (owner 11-17: no partial credit).
3) rtb5-2: pixel rows cite the judge's still only for acts it saw; sample 10 rows.
4) rtb5-3 (demo/tour/chapters/handover.py): waits for swaybg and waybar and raises if absent; the first act checks them; no new fixed sleep (`tools/sleep-ratchet` if present).
5) Counts: before 26 change, 71 open, 1 regression; after 4 change, 94 open, 0 regression. The 23 acts change to open (20 xp x1..x12c by the hud-line node, t8, t9, t10) honest? m1 regression to change: does 9311e155 (KEYS heading hairline) name a real intended change with its design cite, or hide a regression?
6) Rows: each new or changed row names nodes or boxes and cites its own act's capture; KISS: no new field beyond nodes and boxes.
Known, not findings: rv9j-1..5 (MINOR), the 94 open (F7 rest, v9 take after merge).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtb6-N, severity, file:line, text, fix in one sentence). Append to reports/tour-base.md on claude/session-log under "## Review r6".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
