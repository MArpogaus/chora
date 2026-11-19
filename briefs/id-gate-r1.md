# id-gate r1 (F6, harness): the merge gate checks commit identity

Model: sonnet, effort high (routine shell gate with a selftest)
Lane: cloud, branch claude/id-gate from the train 72 merge 219d5afe. One round, stop at 250 k. Cost about 0.5 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "id-gate r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: backlog.md HARNESS row "(cube-tests r1): a lane stop hook set the committer"; friction.md entry 11-19 20:40.
Why: twice a lane hook re-authored commits as another identity; no gate catches it before a merge.
1. tests/merge-gate.sh: the `red-first` step (or a new step `identity` right after it, listed in the Steps comment and `--list`) fails when any commit in BASE..HEAD, merges included, has an author or committer other than `Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>`; the failure names each sha and the identity it found. The expected identity is one variable at the top, not repeated.
2. A selftest case in tests/gate-lib-selftest.sh (or the selftest that covers red-first): a throwaway repo with one bad committer fails, all good passes. Red first: the selftest commit before the gate change.
3. docs: the gate's step list in CONTRIBUTING.md (or wherever merge-gate steps are listed) names the new check.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,red-first,gate-lib` green (add `identity` if it is its own step). Poll it in the same turn; never end a turn to wait.
Done when: items 1-3 in; gate green; reports/id-gate.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/id-gate and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
