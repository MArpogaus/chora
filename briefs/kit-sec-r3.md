# kit-sec r3 (F5, security): pid temp folders and review r2 findings

Model: sonnet, effort medium (mechanical moves to TempDir plus three small fixes with given plans)
Lane: cloud, branch claude/kit-sec at 6de357b9. One round, stop at 250 k. Cost about 0.8 %.
Step 0 (no setup script: auto mode refuses it in lanes; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "kit-sec r3 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Why: train 79 refused kit-sec: cut-check row "temp folder by pid outside the counted helper" (tools/copies.toml, max 0) found 4 sites of this branch.
Read on claude/session-log: reports/kit-sec.md "## Review r2" if present, and backlog.md KIT-SEC-R3 (rw-8 to rw-10).
1. Merge first: `git fetch origin claude/autonomous-dev-session-3w-i33pg7`; it must contain a2f50532 (`git merge-base --is-ancestor a2f50532 FETCH_HEAD`); if not, re-fetch every 10 min, at most 90 min, then BLOCKED. Merge it. Known conflicts: crates/chora-kit/src/lib.rs `scratch()` takes `crate::testing::TempDir::new(&format!("chora-kit-{tag}")).keep()`; tests/merge-gate.sh ci-check keeps the HEAD line (with grab-gap) and adds `tests/tiling-scripts-selftest.sh &&` after screenshot-notice.
2. Move the pid temp folders to chora_kit::testing::TempDir (or chora_scene::testing::TempDir below the kit): crates/chora-draw/src/theme.rs:1223, crates/chora-kit/src/icons.rs:303, crates/chora-kit/src/icons.rs:424, crates/chora/tests/runtime_dir.rs:17 (line numbers at 6de357b9). icons.rs:424: if the test needs a predictable name on purpose (a planted file), build it inside a TempDir, not from the pid. `tests/cut-check.sh` (or the ci-check step) reports 0 left.
3. rw-8 MINOR: the Wayland socket binds below the checked, canonical runtime dir, as the IPC socket does, not the raw XDG_RUNTIME_DIR (crates/chora-scene/src/lib.rs about :113). Red test first: a symlinked runtime dir swapped after the check gets no socket.
4. rw-9 NIT: script /tmp fallbacks use mktemp, not fixed names; the selftest covers it.
5. rw-10 NIT: runtime_dir.rs gets a case for a dir owned by another uid (skip with a message when not root and none exists).
Each fix: a red test commit first, then the fix commit.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,doors,xp,tiling-look`; all PASS.
Poll each run in the same turn; never end a turn to wait.
Done when: each item names red commit, fix commit and test; gate green; reports/kit-sec.md "## r3" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/kit-sec and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
