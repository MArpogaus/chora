# mc-minor r2 (mc look; PM review FIX: rmm-1 MAJOR lint ratchet raised)

Model: sonnet, effort low (a function split with no change of behaviour)
Lane: cloud, claude/mc-minor at fb9827a0. Cost at most 0.3 %; stop at 150 k context. One area: chora-mc. Not a pixel round (no look change).
Read reports/mc-minor.md "## PM review r1b" on claude/session-log.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "mc-minor BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) rmm-1: drop both new `#[allow(clippy::too_many_lines)]`: split `Mc::panel` (plugins/chora-mc/src/layout.rs) into helpers for title, lines and empty parts, and the long test in plugins/chora-mc/src/minor.rs into helpers; tools/lint-ratchet.list back to `allow 360` (or lower). No change of behaviour: `cargo test -p chora-mc` stays green with the same tests.
2) Merge origin/claude/autonomous-dev-session-3w-i33pg7 (no conflicts expected; `chore:` subject).
3) Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,acts,doors,mc-paths,mc-sofa,mc-column,mc-handover` green. A red cell you did not cause: name it with its log line. Poll in the same turn; never end a turn to wait.
Done when: no new allow, ratchet at most 360, gate green.
Report: reports/mc-minor.md "## r2" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/mc-minor and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
