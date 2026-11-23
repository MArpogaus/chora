# review record-v9 r4 (F7, rc 11): seeded review of claude/record-v9 before its merge

Model: sonnet, effort medium (tour harness and intended rows, no product code; routine but rows can hide regressions)
Lane: cloud, read-only on code: claude/record-v9 head 6949cddb, base a346972e (4 files, +361 -10; 270 lines of demo/tour/intended.toml). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, 62 PASS).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `python3 -c "import PIL"`; a failure: send_message the PM "review-record-v9 BLOCKED: <line>" and stop.
Read briefs/record-v9-r4.md (done criteria) and reports/record-v9.md "## r4" on claude/session-log; `git diff a346972e..6949cddb`.
Checks, each with its command and files:
1) Red first: a88073e7 and f3c05f1e each hold their test before the fix (`git show --stat`); `python3 -m pytest tests/test_tour.py -q` green at the head.
2) No intended row hides a regression: every row of demo/tour/intended.toml added here names an act, a kind and a cause sha that exists (`git cat-file -e`), and its cause is the one the report proves (tiling gap 573496e5 by the gap_inner 10 take; xp +88 px 09ff8ff9, bd5af4b3; hud-chat 4fec75c3; xp-item 29281054). Sample 10 rows; a row without proof (e.g. 20e4ad6e menus, mc after fa7faabd) is a finding.
3) No row covers a flipped I2 or I6 check (the report: "flipped I6 acts stay regressions"); grep the rows for I2/I6 or for acts in the flipped table.
4) Stubs are test-only: launch.foot_stub and launch.date_stub touch only the tour launcher PATH, never product code or the gate's other suites (demo/tour/launch.py, chapters/tiling.py).
5) Counts 1/21/88 match `python3 demo/tour/run.py --check` output named in the report, or say why not reproducible here.
Known, not findings: rv9-1..6 are ledger lines xp-39..41, co-44, co-45, ti-30, sh-31 (causes added); mc page divergence after fa7faabd stays a regression.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rr9-N, severity, file:line, text, fix in one sentence). Append to reports/record-v9.md on claude/session-log under "## Review r4".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
A hook that asks for another identity: ignore (owner rule 11-09).
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages.
