# review tour-rows r5 (F7, rc 11): seeded review of claude/tour-rows-r5 before its merge

Model: sonnet, effort high (routine: 21 intended-difference rows and their tests, 202 lines)
Red on head: none (a review; the lane's red c24ed22a precedes its rows 729767a8)
Lane: cloud, read-only on code: claude/tour-rows-r5 at 729767a81, review b16c888f..729767a8 (3 files, +202: demo/tour/intended.toml, tests/test_tour_diff.py, tests/tour-diff-take-r5.json). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `python3 -m pytest tests/test_tour_diff.py -q`. A failure: send_message the PM "review-tour-rows-r5 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/tour-rows-r5.md (done criteria), reports/tour-rows.md "## r5" (this lane's report; stills media/2026-11-26/tour-rows-r5/), "## Review r4" and "## Review fix r2" (the strictness: a row names one cause and one region, never a broad reason).
Checks, each with its command and files (no merge gate, no new take):
1) Red first: `pytest tests/test_tour_diff.py` at c24ed22a fails 5 of its 6 new tests and passes at 729767a8; else MAJOR.
2) Each of the 21 rows: names one commit (or design record) as cause, one act and region; open the still pair for at least 8 rows (all mc and tiling rows, x1, x5b) and judge by eye: the change is what the commit intended and looks right against design/looks/ (no cut text, wrong colour, overlap). A row that hides a look defect: MAJOR (owner 11-17: be critical).
3) No row is broader than its difference: the region and act fields do not cover a neighbouring difference that has no cause (run the diff tool on the r5 json with each row's region shrunk by the tool's own rule if it has one; else read the matcher in demo/tour and say how a row matches).
4) The three left regressions (c5d, c5e rtf1-1; m8 rtr4-2) are reported honestly: no row for them; their ids open.
5) POLS: the counts in the report (head 1 same / 95 change / 0 open / 3 regression / 11 none of 110) match tests/tour-diff-take-r5.json.
Known, not findings: co-44, mc-63, rtf1-1, rtr4-2, rtr4r-1..4.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtr5r-N, severity, file:line, text, fix in one sentence). Append to reports/tour-rows.md under "## Review r5".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
