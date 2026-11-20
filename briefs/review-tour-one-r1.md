# review tour-one r1 (F7, RC 11): seeded review of claude/tour-one before its merge

Model: sonnet, effort high (Python tour tooling, no core change; seeded review)
Lane: cloud, read-only on code: claude/tour-one head 8b996cee, base 9dc384b3 (67 files, +1284 -4467). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran pre-commit, ci-check, gate-lib, build, test, acts green); `python3 -I tests/test_tour.py`, `demo/tour/run.py --check` and `tools/cut-check` only.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo build --workspace`; a failure: send_message the PM "review-tour-one r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/tour-one-r1.md (done criteria), design/tour-v9.md sections 0 and 8 row 1, reports/tour-one.md "## r1" on claude/session-log; `git diff 9dc384b3..8b996cee`.
Checks, each with its command and files:
1) One set, no copies: demo/tour holds only {tour,human,launch,slides,assemble,run}.py and chapters/{xp,tiling,mc,construct,handover}.py; no ch*, v4-v8_*, tour4-8, assemble4-8 left and nothing else imports them (grep the tree, tests, docs, gate cells); tools/copies.toml row and `tools/cut-check` red on a planted `v9_` copy (cut-check-selftest).
2) v8 fixes kept: PATH without writers, pane-centre aim, keep-bus, probe bus (diff each against its v8 copy).
3) Act rows: expect, contract, cells, capture on each row; a failing expect shows red in the timeline (tests/test_tour.py); `known=` marks only real open findings (I3, I4, I5) with their id, and `todo` only acts that need xp-give; no green that hides a red (read three rows per chapter against their expect).
4) `run.py --check` on all five chapters: reproduce the lane's counts (xp 20/1, tiling 21, mc 14/4, construct 18, handover 9/2/1 todo).
5) Red first: the lane says test-first held only for the cut-check row, not tests/test_tour.py; check whether the red-timeline test fails on its parent for the named reason; rate the gap (the brief asked it).
6) F-1 (chora-notify expires notices on the wall clock while record steps the scene clock): is the diagnosis in the report right (read chora-notify and the record path); a fix under 20 lines, if made, is correct.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rt-N, severity, file:line, text, fix in one sentence). Append to reports/tour-one.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
