# kit-sec r1 (F5, security hardening): the six KIT1 security MINORs

Model: sonnet, effort high (security hardening with a fixed list)
Lane: cloud, branch claude/kit-sec from the train 74 merge a9e7084d. One round, stop at 250 k. Cost about 1 %.
Step 0: `command -v cargo sccache python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "kit-sec r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: reports/kit1.md "## Security review" (MINOR 1-6), backlog.md rows "KIT1 security MINORs" and "Refactor (kit1 review, ranked high)". Code: crates/chora-kit/src (check_private, private_dir, runtime_file), crates/chora/src/ipc.rs listen, services/voice/src/coder.rs:73, shader and icon cache dirs.
Do not touch the cells of the running lanes flake-h, flake-k (hud), flake-l (xp slow-manager), nits-3 (notify card), pw (tests/persona).
Each item: red test commit first (rules.md 8), then the fix.
1. XDG_RUNTIME_DIR itself is checked before use in listen and runtime_file: owned by the euid, no group or other write bits (refuse with a logged reason that names the path and the mode; fail safe).
2. The private-dir mode check accepts any mode with no 077 bits (0700, 02700, 0500 by umask) and refuses group or other bits; or fchmod right after create. Say which in the report.
3. Shader and cache dirs go through private_dir (one helper, fd-based if it stays small: the backlog Refactor row; `paths::runtime_subdir(name)` if it removes copies).
4. voice coder.rs:73: no relative "~/.local/share" fallback; use chora_scene::paths data_home or HOME, else refuse with a message.
5. CHANGELOG lines (Unreleased, conventional wording) for each behaviour change a user can see.
6. Tests: env wrappers, listen refusal on a foreign or world-writable runtime dir, icons in a foreign dir, the refusal is logged.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,doors,xp` green. Before the final gate merge origin/claude/autonomous-dev-session-3w-i33pg7. Poll it in the same turn; never end a turn to wait.
Done when: each of 1-6 names its red commit, its fix commit and the test that proves it; gate green; reports/kit-sec.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/kit-sec and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
