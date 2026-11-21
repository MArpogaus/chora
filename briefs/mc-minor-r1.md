# mc-minor r1 (mc look, MINOR; no open BLOCK or MAJOR on mc): mc-50, mc-41, mc-14, mc-13

Model: sonnet, effort medium (routine fixes in one look's widgets and text; no core change)
Lane: cloud, claude/mc-minor at 598a34ff (session head). Cost at most 0.8 %; stop at 250 k context. One area: the chora-mc plugin. Pixel round 1 of 2 this week for mc (rules.md 6).
Rows (findings/mc.md on claude/session-log), in this order:
- mc-50: a yellow square appears after the first Return and stays (P2 r2 cook, nurse). Find the node, fix its cause.
- mc-41: the hit rect of the wrapped key row "Start a program or run a command" covers only its first line (row wraps at 224 px); the hit covers every line.
- mc-14: chat body 18 px/27 against the spec 19/28 (design/looks for mc); "Super+ <- ->" key text without the stray space.
- mc-13: page 2 scene copy: no "collector" jargon, one font family per role, no red heads over nothing, divider centred.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "mc-minor BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Red first, own commit: one assert per row (acts cell in tests/acts/looks/mc.py, unit test or gate-mc pixel assert), each red 3 of 3 on 598a34ff; a row you cannot make red: say so, do not fake it.
2) Smallest fixes in chora-mc; green 5 of 5.
3) Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,mc,acts,doors` green (gate-lib left out). A red cell you did not cause: name it with its log line; do not rerun to green. Poll in the same turn; never end a turn to wait.
Captures: after PNGs per row into reports/evidence/mc-minor-r1/ on claude/session-log, named with the build sha.
Done when: each row closed with its cell and sha in findings/mc.md, or left open with the reason; gate green.
Report: reports/mc-minor.md "## r1" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/mc-minor and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
