# review tour-rows r4 (F7, rc 1, 11): seeded review of claude/tour-rows-r4 before its merge

Model: opus, effort high because 75 intended rows decide which v9-vs-v8 differences count as wanted; a row too broad hides a regression (owner 11-17: be very critical about met targets); the builder never reviews itself (rules.md 7)
Red on head: none (a review; the lane's red 2680e71b precedes its fixes)
Lane: cloud, read-only on code: claude/tour-rows-r4 at 972f9415 (67a2dc4d merged with session head 4d8bbe36), review 035df1fc..67a2dc4d (2 files, +501 -4: demo/tour/intended.toml, tests/test_tour_diff.py). One round. Stop at 120 k context (grep and sample intended.toml); cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it: EXIT 0, xp-notepad flake sh-45 green alone).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-tour-rows-r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/tour-rows-r4.md (done criteria), reports/tour-rows.md "## r4" and "## Review r3" if present on claude/session-log; `git diff 035df1fc..67a2dc4d`.
Checks, each with its command and files:
1) Red first: 2680e71b fails `python3 -m pytest tests/test_tour_diff.py -q` before 30f0ac17 and passes at 67a2dc4d; the new test truly rejects a row whose cause is not a product commit in 733967aa..HEAD (try one bad row locally, do not commit).
2) Rows: every new [[change]] row names a commit that exists and touches the node's look (`git cat-file -e`; `git show --stat <sha>`); no row absorbs rtr4-1 (m2 m2b m3), rtr4-3 (construct c2b..c5e) or rtr4-2 (m8); no `*` or nodeless row wider than its judged act. Sample 15 of the 75 rows against their capture jsons in media/2026-11-26/tour-rows-r4/ on claude/session-log: is the difference that commit's effect, or a hidden regression?
3) The 13 new regressions (c2b..c5e from 75d4623e, m8b m9 m9b from 80dbd0b4): are rows for m8b, m9, m9b justified as intended (cite the commit's intent), and are the construct acts correctly left as findings?
4) Counts: 1 same, 86 change, 0 open, 12 regression, 11 none; reproduce with tests/tour-diff.py on the r4 take files if present; a count that does not reproduce is MAJOR.
Known, not findings: rtr4-1..rtr4-5, rv9ka-2, co-44, co-45, sh-37..sh-45.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtr4r-N, severity, file:line, text, fix in one sentence). Append to reports/tour-rows.md under "## Review r4".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
