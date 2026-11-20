# mc-fix-r4 (RC 2 persona round 4; rule 6, mc MAJOR): review r3 rv-1 page-turn while waiting, rv-2, rv-3

Model: sonnet, effort high (three named findings with file:line and fix from the seeded review; a seeded review follows)
Lane: cloud, branch claude/mc-fix at b0997b71 (merge origin/claude/autonomous-dev-session-3w-i33pg7 first when it moved past d81c8293). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure: send_message the PM "mc-fix r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/mc-fix.md "## r3" and "## Review r3" (rv-1 to rv-5, each with file:line and fix).
Items (severity order; each a red test commit first, then the fix):
1. rv-1 MAJOR, plugins/chora-mc/src/show.rs:149: Right to the terminal page (starts foot), Left, Right before the window maps: `show` clears plain_start and, while `waiting`, does not set it again, so the window that comes takes the keys (mc-42). Red: a unit test in chora-mc (FakeCore) and the gate cell mc-paths terminal-turn extended with this path (Right, Left, Right before map; then Left, Right, Escape act as pages). Fix: set plain_start also while waiting; take it only on a name match.
2. rv-2 MINOR, main.rs:649: refresh queries `*` on every state.get; query the own node (id:<node>) with the focused filter. Red: a test that counts the nodes the query asks for.
3. rv-3 MINOR, tests/acts/invariants.py:389: `hides` counts any full-output image as a cover; count only an opaque image of another look's own wallpaper or the shown look (as the review says). Red: a unit test on a recorded state with a translucent mc picture over xp that must still breach.
4. rv-4, rv-5 NIT only if cheap: the FakeCore records Focused (keep case test); refresh logs a failed query.
Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,gate-lib,ci-check,build,test,clippy,keys,mc-paths,mc-sofa,mc-handover,acts`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: red commits before fixes; cells pass; reports/mc-fix.md "## r4" on claude/session-log (per item: red sha, fix sha, test, cell); findings/mc.md row mc-42 `fixed <sha>`.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/mc-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
