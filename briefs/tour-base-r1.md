# tour-base r1 (F7, rc 11, 1b): the tour baseline measures I6 and every red act is named

Model: sonnet, effort high (harness logic over three files; red-first test design)
Lane: cloud, branch claude/tour-base at 8c411416 (session head after t121, xp-readable merged). One area: demo/tour and tests/test_tour.py; no product code. Stop at 250 k. Cost at most 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q`, `python3 demo/tour/run.py --help`. A failure: send_message the PM "tour-base BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/audit-f.md section 6 row 3 (TOUR-BASE) and the F7 part of section 5; reports/tour-fix.md (rw-6, the 43 flipped acts, t26, t28); design/tour-v9.md (I6); findings/ rows named there.
1) Red commit first: tests/test_tour.py gets a case where a baseline snapshot without boxes is diffed by demo/tour/tour-diff.py; today it reports "I6 flip" (fails the case); commit "test(tour): a baseline without boxes is not an I6 flip"; paste the failing line.
2) Fix commit: tour-diff.py reports "I6 not measured" for such a baseline; the case passes; paste it.
3) demo/tour/intended.toml: the mc rows after a controlled take of fa7faabd (or a chapter step that lands both builds on one page), and the tiling t8, t9, t10 causes; each row names its cause. Name the unnamed kinds of the 43 flipped acts, each with a findings id or a known-red row; no new known-red row without a findings id.
4) `python3 demo/tour/run.py --check`: 0 acts with neither row nor finding (now 24); paste the summary line. New faults found: a findings row proposal in the report (id free, look, severity, capture), no ledger edit.
5) Full gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --only), paste the last 20 lines; `tests/merge-gate.sh --red-first-check` on 8c411416..HEAD passes.
Never end a turn to wait; poll any long command in the same turn.
Done when: red and fix shas with their pasted lines; the --check summary with 0 unexplained acts; the table of the 43 flipped acts with kind and id; the full gate green.
Report: reports/tour-base.md "## r1" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines with the head sha.
Rules: push claude/tour-base and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
