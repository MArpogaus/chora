# tour-diff r2 (F7, RC 11): fix review r1 (FIX: rd-1, rd-2 MAJOR; rd-3 to rd-8)

Model: sonnet, effort medium (bounded fixes in two Python tools and their tests)
Lane: cloud, claude/tour-diff at fa796bb9. Cost at most 0.5 %; stop at 150 k context.
Why: reports/tour-diff.md "## Review r1" (1294ecf7) verdict FIX: a missing capture passes (rd-1) and the gate passes with no video check when ffmpeg is absent (rd-2).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "tour-diff r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Red first: one commit with failing tests for rd-1 (BASE capture absent in CAND is a regression; empty or missing CAND folder exits 1), rd-2 (the gate-acts hook fails when ffmpeg is absent; test with PATH emptied), rd-3, rd-5, rd-6 and the rd-7 2-frame flash on testsrc2; plus the uncovered cases: CLI exit code, size mismatch, `baseline: none`.
2) Green: fix rd-1 to rd-7 as the review says (rd-4: list the kinds of the h2 Send To row; rd-7: comment each threshold with the v8 data it came from). rd-8: restate the red counts in a "## r2" note. KISS: no new tool, no new dependency.
3) Rerun on v8 as in r1 (t3 regression, 21 and 7 freezes) and say whether the numbers changed and why.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,gate-lib,build,test,acts`. Poll in the same turn; never end a turn to wait.
Report: append "## r2" to reports/tour-diff.md on claude/session-log (red and green shas, each rd-N with its test name, gate cells); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/tour-diff and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
