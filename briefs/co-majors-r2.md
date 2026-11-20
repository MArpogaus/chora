# co-majors r2 (RC, vision bar): review r1 findings rv-1 to rv-4

Model: sonnet, effort high (geometry fixes with plans given by the review; tests in an existing harness)
Lane: cloud, branch claude/co-majors at db4d703b. One round, stop at 250 k. Cost about 0.8 %.
Step 0 (no setup script: auto mode refuses it in lanes; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "co-majors r2 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/co-majors.md "## Review r1" (rv-1 to rv-4), vision-card.md (construct: calm, no lost window).
Each item: a red test commit first, then the fix commit.
1. rv-3 first (the gate that missed rv-1): tests/gate-construct-pull.sh compares the mirror y range with the face, tightens the length rule so it fires for long edges, checks pane against pane, and puts a second pane on one side. It must fail on db4d703b (that is the red commit for rv-1 too).
2. rv-1 MAJOR: aside.rs make_way keeps a running offset per side, advanced by pane width plus GAP; no two placed panes overlap.
3. rv-2: unit tests in the construct harness for make_way (placed, in front, overlap, side), step_asides (exact home, cleared on close and step in) and the removal at panes.rs:230; gate 2a steps in again and compares the home rect.
4. rv-4 NIT: clamp the aside place to the leash radius() and keep the pane on screen (shrink the offset).
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,construct`; all PASS.
Poll each run in the same turn; never end a turn to wait.
Done when: each item names red commit, fix commit and test; gate green; reports/co-majors.md "## r2" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/co-majors and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
