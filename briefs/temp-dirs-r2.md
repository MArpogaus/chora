# temp-dirs r2 (F2, F5): one home for TempDir, the kc-fix conflict

Model: sonnet, effort medium (move one module, resolve one conflict)
Lane: cloud, branch claude/temp-dirs at 5b0aa407 (your r1). One round, stop at 250 k. Cost about 0.4 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "temp-dirs r2 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/temp-dirs.md "## PM review r1" (tdr-1..3).
Goal: TempDir lives once; the branch merges cleanly with claude/kc-fix.
1. `git fetch origin claude/kc-fix claude/autonomous-dev-session-3w-i33pg7` and merge both (kc-fix f666a025 conflicts on on_fake_core; keep both changes).
2. tdr-1: move TempDir and its test into chora-scene (a `testing` module; behind a feature the dev-dependencies enable, if clippy or docs ask for it). The kit re-exports it as chora_kit::testing::TempDir; choractl and the core use the chora-scene one. Delete the three copies (crates/chora-scene, choractl, chora testing/temp_dir.rs become the one chora-scene module or none).
3. tdr-3: the copies.toml row allows only the chora-scene file, max 0 elsewhere. Do not change what the tests check.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy` green. Poll it in the same turn; never end a turn to wait.
Done when: one TempDir in the tree (`grep -rn "pub struct TempDir" crates plugins services` gives one line); kc-fix merged; gate green; reports/temp-dirs.md "## r2" on claude/session-log with the head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Rules: push claude/temp-dirs and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
