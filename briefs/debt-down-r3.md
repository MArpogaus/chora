# debt-down r3 (F4): close review r2 of claude/debt-down (rd-7 MAJOR)

Model: sonnet, effort high (the fix is named by the review; one core path and a cell)
Lane: cloud, branch claude/debt-down at c8fe032b (r2). One round, stop at 250 k. Cost about 1 %.
Step 0: `command -v cargo python3 jq`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "debt-down BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/debt-down.md "## r2" and "## Review r2" on claude/session-log.
1. rd-7 MAJOR (crates/chora/src/ipc.rs:402, :1165): a grab that ends with its connection (holder exit, crash) or a revoked grab-any
   sends input.ungrabbed for each dropped non-move grab (ipc_close, drop_free_grabs), as an explicit release does. Red test commit first
   (a holder connection closes while it holds Alt+Escape: a subscriber gets input.ungrabbed {binding}; and xp then grabs Alt+Escape), then the fix.
   Drop the "sends nothing" clause in docs/api.md and the D-ungrab record; keep D-ungrab unnumbered (the PM numbers it at merge).
2. rd-8 NIT: none to do (noted); do not rewrite history.
Done when: rd-7 red then green; `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` clean;
`tests/merge-gate.sh --jobs 3 --base 85764974` once (start it in the background and poll it in the same turn; a known flake: rerun that step with --only and name it).
Write reports/debt-down.md "## r3" on claude/session-log. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines). Poll every command in the same turn; never end a turn to wait.
Rules: push claude/debt-down and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the two branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
