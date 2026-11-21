# deps r1 (rc 7): the minor Dependabot bumps on the session head 1c599c74

Model: sonnet, effort low (version bumps with a fixed list; build and test)
Lane: cloud, claude/deps at 1c599c74. Cost at most 0.9 %; stop at 150 k context. One area: Cargo.toml/Cargo.lock, .pre-commit-config.yaml, .github/workflows (versions only).
Why: plans/week-3.md row 17 (claude/session-log): rc 7 wants dependencies current. Dependabot PRs #108, #107, #94, #88, #86, #2, #1 on MArpogaus/chora are minor bumps; zbus 6 (#96) and serde-saphyr 2 (#87) are major and stay out.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "deps BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Read each PR (GitHub MCP pull_request_read, read only; never comment, merge or close a PR) and list package, from, to.
2) Apply each bump as its own commit on claude/deps (`cargo update -p <crate> --precise <ver>` or the manifest line; the action or hook rev for workflows and pre-commit). A bump that breaks build, clippy or a test: drop it and name the error line in the report; no code change beyond what the bump needs (at most 20 lines per bump, else drop it).
3) Checks: `cargo deny check`, `cargo machete`, `pre-commit run --all-files`, `cargo test --workspace`.
4) Gate: the full gate `tests/merge-gate.sh --jobs 1` (no --only) green on the branch merged with origin/claude/autonomous-dev-session-3w-i33pg7. A red cell you did not cause: name it with its log line; do not rerun to green. Start runs in the background and poll in the same turn; never end a turn to wait.
5) Known issues: one line in docs/known-issues.md (or the file the repo uses) naming zbus 6 (#96) and serde-saphyr 2 (#87) as deferred major bumps.
Done when: each minor bump applied or dropped with a reason, deny and machete clean, full gate green.
Report: reports/deps.md "## r1" on claude/session-log (table PR, package, from, to, sha or dropped reason; check results; gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/deps and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
