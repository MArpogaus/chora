# review tour-base r1 (F7, rc 11, 1b): seeded review of claude/tour-base before its merge

Model: sonnet, effort high (harness logic plus 164 data rows; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/tour-base head 9616d7d6, base 8c411416 (4 files, +855 -5; 820 of them demo/tour/intended.toml rows). One round. Stop at 120 k; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green). No tour takes (the lane's captures are lost); check rows by git log, diffs and design docs.
Read on claude/session-log: briefs/tour-base-r1.md (done criteria), reports/tour-base.md "## r1", design/tour-v9.md (I6, section 6); `git diff 8c411416..9616d7d6`.
Checks, each with its command and files:
1) Red first: a5eeec7c before e7b256d0 (`git log --oneline 8c411416..9616d7d6`); `python3 -m pytest tests/test_tour.py -q` and `python3 -I tests/test_tour_diff.py` green at head; the red case fails at a5eeec7c.
2) Not measured (tests/tour-diff.py verdict, demo/tour/tour.py dump): "not measured" only where a side has no `scene.boxes`; it can never hide a check that both sides measured; a candidate red I6 still counts in `run.py --check`. Write one extra case in a scratch copy if needed; do not commit it.
3) Rows (demo/tour/intended.toml): sample at least 20 of the 164 new rows across xp, tiling, mc, construct, handover; for each, is the cause sha in 733967aa..8c411416 and does it touch the element family of the act (`git show --stat`)? Count rows whose cause is an open finding rather than an intended commit, and judge whether tour-diff "change" then hides a regression from the F7 metric (owner 11-17: no partial credit). Flag act rows with many causes or kind "*" that would let a future unrelated diff pass.
4) The 23 red acts of `--check`: each has a findings id (findings/*.md, including the new mc-59, xp-43, xp-44, xp-45, co-63, ti-30 reopened).
5) The lane's question: should I2, I5, I13 (also box reads) be marked "not measured" on a box-less baseline? Answer with the files and lines that read boxes.
Known, not findings: rr9-1 (a row is no proof that every diff of its kind comes from its sha); causes by family without controlled takes (report "### Limits").
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtb-N, severity, file:line, text, fix in one sentence). Append to reports/tour-base.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
