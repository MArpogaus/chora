# mc-answer r2 (RC): bound the notify card size (review r1 rv-1 MAJOR)

Model: sonnet, effort medium (one bounded fix with a red test; routine)
Lane: cloud, branch claude/mc-answer from a271df2b (r1 head). One round, stop at 250 k. Cost about 0.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure or a missing tool: send_message the PM "mc-answer r2 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/mc-answer.md "## Review r1" (rv-1).
1. rv-1 red first, own commit: unit tests in plugins/chora-notify (card.rs) for `size: 0`, `size: u32::MAX` and `size: 100000`: no panic, the card stays within the output, a message names the bad value.
2. Fix, own commit: refuse or clamp card.size to one range (same as the HUD text range, e.g. 14..48) with one log line naming the value and the range; checked or saturating math at card.rs:95 and :377; the docs of the setting name the range. Smallest change; no other behaviour changes.
3. reports/mc-answer.md "## r2" on claude/session-log: red sha, fix sha, cell, gate line.
Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,mc-sofa,mc-column`; all PASS.
Done when: rv-1 has red commit, fix commit and passing tests; the gate line above all PASS on the head; report on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/mc-answer and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
