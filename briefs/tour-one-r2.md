# tour-one r2 (F7, RC 11): fix review r1 (FIX: rt-1, rt-2 MAJOR; rt-3 to rt-7)

Model: sonnet, effort high (tour harness semantics across five chapters; bounded by the review's fixes)
Lane: cloud, claude/tour-one at 8b996cee. Cost at most 1 %; stop at 200 k context.
Why: reports/tour-one.md "## Review r1" (83770400) verdict FIX: 87 of 90 acts show green with no expect (rt-1); the F-1 notify fix is in xp only, mc loses its cards (rt-2).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "tour-one r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Red first: one commit with failing tests in tests/test_tour.py for rt-1 (a row without `expect` is `inv`, not `green`, and counted apart in the run.py tally), rt-4 (`known=I4` keeps an I5 red RED; a known act that turns green prints "drop known"), rt-5 (missing pane rect raises), rt-6 (contract, cells, capture columns in the timeline).
2) Green: fix rt-1 to rt-7 as the review says. rt-2: the notify timing in one place (a launch.py helper used by every chapter), and RECORD checks the card frame in xp and mc; do not change chora-notify (product fix goes to backlog). rt-3: wait for `busy` empty before the read, else mark the act known-flaky with its id. rt-7: delete ip.py, move probe.py to tools/, rename row files to chapter names. KISS: no new dependency.
3) `demo/tour/run.py --check` twice on all five chapters; give green, inv, known, red per chapter for both runs (they must match).
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,gate-lib,build,test,acts` plus any cell that imports demo/tour. Poll in the same turn; never end a turn to wait.
Report: append "## r2" to reports/tour-one.md on claude/session-log (red and green shas, each rt-N with its test, both --check tallies, gate cells); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/tour-one and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
