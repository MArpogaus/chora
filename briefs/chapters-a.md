# chapters-a (F7, RC 11; design/tour-v9.md section 8 lane 4): every act row of xp, tiling, handover checked (green or known red), no bare inv

Model: sonnet, effort high (act semantics across chapters; each expect must test what the row says, not pass vacuously)
Lane: cloud, branch claude/chapters-a at f4f49783 (session head after train 95b). Cost at most 1.2 %; stop at 250 k context.
Why: review-tour-one r2 (reports/tour-one.md) tallies xp 2 green/18 inv, tiling 0/21, handover 1/8: rows without `expect` prove nothing. F7 needs each act of design/tour-v9.md sections 1, 2 and 5 checked.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "chapters-a BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read: design/tour-v9.md sections 0, 1, 2 and 5 and 8 (on claude/session-log); reports/tour-one.md (r2, Review r2); demo/tour/README.md; tests/acts/ and tests/persona/check.py (the contracts) on the branch.
Files: demo/tour/chapters/xp.py, chapters/tiling.py, chapters/handover.py and tests/test_tour.py only; a shared helper goes into demo/tour/tour.py only if both chapter lanes need it, kept under 30 lines (the other lane edits the other chapters).
1) Red first: one commit in tests/test_tour.py: a test that fails while any row of xp, tiling, handover has neither `expect` nor `contract` (the tally's inv count for these chapters is 0), and one planted wrong state per chapter that its expect turns red.
2) Green: for each act row of sections 1, 2 and 5 give `expect` (Expect on the snapshot, as the acts in tests/acts/looks do) and the `contract` and `cells` the design row names; reuse existing checks, no new gate cell. A row that is red on the product: `known="<id>"` with a findings id (existing or a new line in the report), never a weakened expect. KISS: no new dependency.
3) `demo/tour/run.py --check xp tiling handover` twice; green, inv, known, red per chapter for both runs (must match; inv 0).
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,gate-lib,build,test,acts`. Poll in the same turn; never end a turn to wait.
Report: append "## r1" to reports/chapters-a.md on claude/session-log (red and green shas, rows per chapter with expect and contract, both --check tallies, known ids, gate cells); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/chapters-a and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
