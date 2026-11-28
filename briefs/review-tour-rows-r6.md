# review tour-rows r6 (F7, rc 11): seeded review of claude/tour-rows-r6 before its merge

Model: sonnet, effort high (routine: intended-difference rows, a diff-tool guard and their tests, 199 lines)
Red on head: none (a review; the lane's red 46d73a85e precedes its rows)
Lane: cloud, read-only on code: claude/tour-rows-r6 at 7b09b6230, review 3098926e..7b09b6230 (4 files, +199 -13: demo/tour/intended.toml, tests/test_tour_diff.py, tests/tour-diff-take-r6.json, tests/tour-diff.py). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `python3 -m pytest tests/test_tour_diff.py -q`. A failure: send_message the PM "review-tour-rows-r6 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/tour-rows-r6.md (done criteria), reports/tour-rows.md "## r6" (stills media/2026-11-28/tour-rows-r6/), "## Review r5", "## Review r4" (the strictness: a row names one cause and one region, never a broad reason).
Checks, each with its command and files (no merge gate, no new take):
1) Red first: `pytest tests/test_tour_diff.py` at 46d73a85e fails its 8 new tests and passes at 7b09b6230; else MAJOR.
2) c5d, c5e as baseline-none (rtf1-1): v8 has a file for them; is calling them "none" honest, or does it hide the regression rtf1-1 the r5 review required to stay open? Open both still pairs and judge by eye. A row or status that hides a look defect: MAJOR (owner 11-17: be critical).
3) m6, m6b (keys-2, 4363de11) and m8 (box [463,198,354,324]; cd277ee4, 12f97cd33, df1a2d0f): each names one cause and the smallest region; open the still pairs. The lane says rtr4-2's "no icons" premise is wrong (icons show in v8 and v9): check it on the stills; close rtr4-2 only if so.
4) The new guard in tests/tour-diff.py (a box over half the frame needs `wide`): does it reject a broad row, is it tested both ways, and does any existing row now pass only because it says `wide`?
5) Counts: the report (all 110: 1 same, 96 change, 0 open, 0 regression, 13 none; mc 22 change; construct 12 change, 6 none) match tests/tour-diff-take-r6.json; rtr6-1 NIT is in the report.
Known, not findings: co-44, mc-63, rtr4r-1..4, rtr5r-*.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtr6r-N, severity, file:line, text, fix in one sentence). Append to reports/tour-rows.md under "## Review r6".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
