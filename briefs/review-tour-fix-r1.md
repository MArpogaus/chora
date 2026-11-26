# review tour-fix r1 (F7, rc 1, 11): seeded review of claude/tour-fix-r1 before its merge

Model: sonnet, effort high (harness review: tour scripts and their test, no product code; procedures: sonnet for harness)
Red on head: none (a review; the lane's reds b0ac9cf8, f6a8b347 precede their fixes)
Lane: cloud, read-only on code: claude/tour-fix-r1 at 84e31324c554821d5fac27c46c3995f8bf8618d2, review 972f9415..84e31324 (4 files, +189 -42: demo/tour/*, tests/test_tour_diff.py). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings. No merge gate run (the lane ran it on 84e31324: EXIT 0).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`. A failure: send_message the PM "review-tour-fix-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/tour-fix-r1.md (done criteria), reports/tour-rows.md "## Review r4" and "## fix r1" on claude/session-log, stills media/2026-11-26/tour-fix-r1/; `git diff 972f9415..84e31324`.
Checks, each with its command and files:
1) Red first: b0ac9cf8 fails `python3 -m pytest tests/test_tour_diff.py -q` on its own tree before 7fc8b31f and passes at 84e31324; f6a8b347 likewise before aafd5611. a63becb3 holds the construct chapter step and its check in one commit: judge whether the chapter is the test (a tour script) or a fix without a red test (then MAJOR, per procedures).
2) rtr4r-1 (c5d rows dropped), rtr4-3, rtr4-4, rtr4-5, rtr4r-3: each fixed as its finding asks; "a merge counts its first parent" cannot let a row name a merge whose first-parent diff does not touch the node's look (try one bad row locally, do not commit).
3) No whole-frame pixel box remains in demo/tour/intended.toml (grep); c5d, c5e now show as regressions, not hidden.
4) Counts: head 1 same / 71 change / 0 open / 29 regression / 9 none on 110 captures; reproduce with tests/tour-diff.py on the lane's take files if present; a count that does not reproduce is MAJOR. The lane says most of the 29 come from the 972f9415 merge (t153/t154) with no rows: sample 5 and say if that holds.
5) The cube.list stop fix (84e31324: the replay without cube.list counts as stood back): POLS-sound, or does it hide a real v8 replay failure?
Known, not findings: rtf1-1..rtf1-7, rtr4-1, rtr4-2, co-44, co-45, sh-37..sh-45.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtf1r-N, severity, file:line, text, fix in one sentence). Append to reports/tour-rows.md under "## Review fix r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
