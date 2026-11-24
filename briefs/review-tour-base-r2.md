# review tour-base r2 (F7, rc 11, 1b): seeded review of claude/tour-base r2 before its merge

Model: sonnet, effort high (harness logic plus rewritten intended.toml rows; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/tour-base head 1e62811f, base 645afaf5 (5 files, +769 -494; most in demo/tour/intended.toml). One round. Stop at 120 k; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green). No tour takes; check by git log, diffs, tests and design docs.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q`, `python3 -I tests/test_tour_diff.py`. A failure: send_message the PM "review-tour-base-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/tour-base-r2.md (done criteria), reports/tour-base.md "## Review r1" (rtb-1..8) and "## r2"; design/tour-v9.md (I6, section 6); `git diff 645afaf5..1e62811f`.
Checks, each with its command and files:
1) Red first: 457b7ee4 before 192aebbe, 1bf04706 before 8649f08d and 1e62811f; each red test fails at its red sha (worktree at the sha, pytest line pasted) and passes at head.
2) rtb-1 (was MAJOR): a row with no proven cause is now verdict `open`, never `change`; `open` counts against F7 in tour-diff and `--check` (tests/tour-diff.py, demo/tour/tour.py). Check that no mc row cites mc-59 as its cause with kind `change`; sample 15 rows whose cause is 675ebd55 or 6fa86c16 (`git show --stat`, element family of the act).
3) rtb-2..rtb-7: each fixed as the r2 report says (co-63 not co-61; candidate colour kept for unmeasured; I6 run part measured; `check` rows name one check and match only it; at most three causes per row; xp-43 only on its named acts). One command per item.
4) Counts: the report's 19 red of `--check` each have a findings id; tour-diff 1 same, 60 change, 38 open, 0 regression, 11 none is what `tests/tour-diff.py` reports for the committed pair if the lane committed the take rows (else say not checkable).
5) A new row or verdict path that could let a later unrelated diff pass as `change` or `same` (owner 11-17: no partial credit).
Known, not findings: rr9-1 (a row is no proof that every diff of its kind comes from its sha); causes by family without controlled takes.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtb2-N, severity, file:line, text, fix in one sentence). Append to reports/tour-base.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
