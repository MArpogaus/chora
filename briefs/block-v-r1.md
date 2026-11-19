# block-v r1 (RC 1, P2): verify the two open BLOCKs on today's head; fix what still holds

Model: sonnet, effort high (verify then red-first fix; plan given)
Lane: cloud, branch claude/block-v from the train 74 merge a9e7084d. One round, stop at 250 k. Cost about 1.2 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "block-v r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: findings/construct.md rows co-18 and co-16, findings/mc.md row mc-32, findings/README.md, reports/p2-plan.md section 0 (acts, invariants, doors). Code: tests/acts/ (harness.py, looks/*.py), tests/gate-construct-*.sh, plugins/chora-construct, plugins/chora-mc.
Do not touch the cells of the running lanes flake-h (matrix, layout, doors, construct-pull), flake-k (construct hud), flake-l (xp slow-manager), pw (tests/persona), kit-sec (chora-kit private dirs, ipc listen).
Both rows date from tour v4; the tree has changed a lot since. For each of co-18 and mc-32:
1. Verify on the head with a scripted act (tests/acts, real input path through input.inject, no hand looking): co-18: a drag on the stepped-back cube turns it, no window leaves face 1, chips keep key caps (check co-16 too: the cube stays on screen after the drag). mc-32: in dark mc, remote Right and Home turn the page; tell product from injection first (the injected-press flag of p2-plan risk 4).
2. Holds: the act is the red test commit (or a known-red.toml row in the same commit), then the fix in the plugin, the act green. Gone: the act stays as a guard (green), the ledger row becomes `closed: <act name>, <sha>` with one line of evidence.
3. Edit only those rows of findings/*.md on claude/session-log.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,acts,mc-paths,construct-pull` (names from `--list`; drop a name that does not exist and say so). Before the final gate merge origin/claude/autonomous-dev-session-3w-i33pg7. Poll it in the same turn; never end a turn to wait.
Done when: co-18, co-16 and mc-32 each closed with an act, or fixed with red then green commits; gate green; reports/block-v.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/block-v and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
