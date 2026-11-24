# review tour-base r3 (F7, rc 11, 1b): seeded review of claude/tour-base r3 before its merge

Model: sonnet, effort high (harness verdict logic plus intended.toml rows; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/tour-base head cc71e5d9, base 883214b5 (4 files, +196 -116). One round. Stop at 120 k; cost at most 0.5 %; at most 8 findings. No merge gate run (lane gate: 56 PASS, 1 foreign FAIL acts xp-38). No tour takes; check by git log, diffs, tests and design docs.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `git fetch origin claude/session-log`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q`, `python3 -I tests/test_tour_diff.py`. A failure: send_message the PM "review-tour-base-r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/tour-base-r3.md (done criteria), reports/tour-base.md "## Review r2" (rtb2-1..4) and "## r3"; design/tour-v9.md; `git diff 883214b5..cc71e5d9`.
Checks, each with its command and files:
1) Red first: 286524e9 (tests only) before cc71e5d9; each of (a)..(d) fails at 286524e9 (pytest or test_tour_diff line pasted) and passes at head.
2) rtb2-1: every row whose cause says "cause not found", "(open)", "(proposed)" or cites an open findings id (findings/*.md on claude/session-log) has open = true; count them yourself and compare with the report (156/369 rows, 84/103 acts).
3) rtb2-2: a base "not measured" check read red by the candidate is `regression` unless a `check` row names that check; rtb2-3: the three-cause count includes sha and finding tokens, no grandfather set; rtb2-4: a kind with one found and one open row gives `open`. One command per item.
4) KISS: no new verdict, no new field (brief 2). Verdict counts: the lane did not produce tour-diff counts per verdict (no take pair); if `tests/tour-diff.py` can run on any committed pair or fixture, paste its counts, else say not checkable (a missing count is INCOMPLETE only if the brief's Done-when cannot be met without a take).
5) A path that could let a later unrelated diff pass as `change` or `same` (owner 11-17: no partial credit).
6) The acts FAIL (xp I6 'Starting Notepad' notify-card-body 1.04:1, xp-38): confirm the diff touches no file the acts cell runs (demo/tour and tests/tour* only), so it is foreign.
Known, not findings: rr9-1; causes by family without controlled takes; the 19 red of --check (open F7 rest); xp-38 recurrence (PM tracks).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtb3-N, severity, file:line, text, fix in one sentence). Append to reports/tour-base.md on claude/session-log under "## Review r3".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
