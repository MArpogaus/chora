# minors-2 r1 (F4, F5): two review MINORs, rl-1 and rd-9b

Model: sonnet, effort high (routine fixes with red tests)
Lane: cloud, branch claude/minors-2 at 9db45f8a (minor-sweep dfa8ddd8 + lint-reason 1f99a98f). One round, stop at 250 k. Cost about 0.6 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "minors-2 r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: backlog.md rows rl-1, rd-9b; reports/lint-reason.md "## r1 PM review"; reports/minor-sweep.md "## r1 PM review".
1. rd-9b: move the Trigger::Move filter into `tell_ungrabbed` (one rule in one place), so connection close (ipc.rs ipc_close) and the grab revoke (input.rs, about line 364) do not tell a move grab either; drop the call-site check in ipc/grabs.rs. Red test commit first: a move grab that ends with its connection is not told.
2. rl-1: remove the two `#[expect(clippy::too_many_lines)]` added for rustfmt wraps (chora-draw theme.rs draw_file, chora-mc layout.rs panel): move the reasoned casts into small helpers so both functions are under the limit again. No behaviour change; existing tests stay green. No new allow or expect anywhere.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy` green. Poll it in the same turn; never end a turn to wait.
Done when: both items in, red first for rd-9b; gate green; reports/minors-2.md "## r1" on claude/session-log naming each id closed and its test; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/minors-2 and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
