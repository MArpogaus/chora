# xp-minor r4 (rc 1 known issues; xp has no open BLOCK or MAJOR, rules.md 6): rxm3-1 MINOR, rxm3-2 NIT

Model: haiku, effort medium (a test-only fix in the assistant tests; testing work)
Lane: cloud, branch claude/xp-minor-r4 at cb5fcb7f (xp-minor r3, MERGE-OK, waits for train140). Areas: plugins/chora-assistant/src/main.rs tests only. Stop at 250 k. Cost at most 0.3 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "xp-minor-r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read reports/xp-minor.md "## Review r3" lines rxm3-1, rxm3-2 on claude/session-log.
1) rxm3-1: the rxm2_1 test asserts on the greeted file inside its own temp dir (HOME or XDG override), so it fails when the greeting is not written there; before == after must not pass vacuously when a real assistant-greeted file exists. Show it red: temporarily break the write, paste the failing line, restore. One test commit.
2) rxm3-2 NIT: only if a one-line revert is safe, restore the unrelated rewrites in Agent::new (methods list, sync_row/apply) at main.rs:399, :453; else leave and say why.
Out of scope: product behaviour, other files.
Run: `cargo test -p chora-assistant` and `cargo clippy -p chora-assistant --all-targets -- -D warnings`; paste the last lines. No full gate (test-only change; the train runs it).
Done when: item 1 commit with the pasted red line; tests and clippy green.
Report: reports/xp-minor.md "## r4 (lane xp-minor-r4)" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/xp-minor-r4 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash.
