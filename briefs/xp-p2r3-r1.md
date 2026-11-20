# xp-p2r3 r1 (RC 2 persona, shared bar "no stray keys into clients"): the face manager swallows, honest Starting

Model: sonnet, effort high (a kit helper used by a new look, one kit state fix, two construct follow-ups; each needs a red test)
Lane: cloud, branch claude/xp-p2r3 from ab4cbb4f (co-p2-fix, MERGE-OK, train 84). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, then `git fetch origin claude/xp-fix` and `git merge --no-ff a767790d` (xp-fix, MERGE-OK, train 83; it merges clean), then `cargo check --workspace` (then `cargo build --workspace`); a failure or a missing tool: send_message the PM "xp-p2r3 r1 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09); keep the brief's identity, never ask.
Read on claude/session-log: reports/co-p2-fix.md (r2 and "## Review r2"), reports/xp-fix.md (PM review: rv-1, rv-2), backlog.md XPF-R1 and CO-P2-FIX lines, vision-card.md.
Items (checked open on ab4cbb4f + a767790d by the PM 04:41; red test or cell first, own commit; then the fix, own commit; never commit Microsoft files):
1. Face swallow (co-25 face part): chora-xp, as the window manager of a cube face, grabs the Super chords it binds no action for with `chora_kit::Runtime::grab_swallowed` (crates/chora-kit/src/swallow.rs), after its own grabs, before `end_grabs`. Red: the two cases of tests/gate-construct-cube.sh that print `known: face manager swallow (xp lane)` turn into plain asserts (remove `known()` and its uses), red on the head, green after. A Super chord that xp binds (Start, key list) still works on a face (cell `cube`, `xp`).
2. XPF-R1 (kit launch.rs ~518): the Starting message of a start ends only on a window that carries its own activation token or no token; a window with the token of another start leaves it. Red: a kit unit test through FakeCore with two starts A and B, B's window maps first, A's message stays.
3. CO-P2-FIX rw-2 MINOR (plugins/chora-construct/src/camera.rs ~476): swallow in a pane only when `self.cube.is_none()`; red: a unit test with a pane and a cube open, an unbound Super chord is not held by the construct.
4. CO-P2-FIX rw-1 NIT (plugins/chora-construct/src/cube.rs ~156): delete the four stale doc lines on `told` (no test; docs commit).
5. XPF-R1 rv-2 NIT (tests/persona session.sh mousepad stand-in): the run log names the stand-in used (foot or probe).
Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,cube,keys,xp,xp-faces,acts,doors,tiling-look`; all PASS. A red cell you did not cause: rerun it alone once and name it (FLK-* in backlog.md).
Done when: each item names red commit (or docs), fix commit and passing cell; gate green; reports/xp-p2r3.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines), same turn.
Rules: push claude/xp-p2r3 and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
