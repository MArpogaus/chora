# co-majors r1 (RC, vision bar): the three open construct MAJORs

Model: opus, effort high, because co-21 is a 3D geometry fault in the cube (face placement, seams, reflection) that needs careful math; co-19, co-20 are small
Lane: cloud, branch claude/co-majors from 30727c82 (train 77). One round, stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace` (then `cargo build --workspace`); a failure or a missing tool: send_message the PM "co-majors r1 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/construct.md co-19 to co-21, findings/README.md (severity), vision-card.md (construct: glass, calm, minimal HUD; step out and in).
Each of co-19 (F1 in the stepped-back view shows the key overlay), co-20 (with the cube open, Escape closes it and the key overlay lists that key; POLS), co-21 (stepped back, the cube is closed: neighbour faces share an edge at equal size, no face juts or recedes, the floor reflection lines up with the faces):
1. Reproduce on the current head first (a construct gate cell or act; co-21 with a pixel or geometry assert: shared edge corners within 1 px, equal projected edge length within 2 %). Gone already: a green guard, mark it `fixed <sha> (gone before)`.
2. Else a red test first (its own commit), then the fix (its own commit), smallest change, construct plugin only unless the fault is in kit or scene (say so).
3. findings/construct.md: each row `fixed <fix sha> cell:<name>` on claude/session-log, with a capture of the stepped-back cube before and after for co-21.
Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,cube,construct-hud,pull,placed,keys,doors`; all PASS.
Done when: each of the three names its red commit (or guard), fix commit and passing cell; gate green; reports/co-majors.md "## r1" on claude/session-log with captures; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/co-majors and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
