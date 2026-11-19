# docs-f8d r1 (F8): the doc defects of the audit and the reviews

Model: sonnet, effort high (routine docs fixes)
Lane: cloud, branch claude/docs-f8d from the train 71 head 6aa0bd37. One round, stop at 250 k. Cost about 0.4 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "docs-f8d r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/audit-f.md "## r2" (rank 5, row F8), backlog.md rows rf-1, rf-2, rc-4, rc-5, rc-6, rd-10.
Already done in train 71 (check, do not redo): D116 accepted; D-ungrab is D117 everywhere.
1. docs/decisions.md: D95 status line (rf-1: accepted, amends D77, owner 11-19 05:59); no accepted record still says proposed.
2. plugins/chora-construct/src/lifecycle.rs:34: drop the Ctrl+N child claim to match D116 (rc-4).
3. docs/api.md: "parent at the first commit" for `surface.created` (rc-5); rc-6 only if it is a doc line.
4. crates/chora-kit/src/wm/mod.rs:12: name every wm part that exists (rf-2); CONTRIBUTING.md:17 names every kit part or points to the list.
5. rd-10: a docs note (api.md, input.ungrabbed) that a shared button binding (D55) is told while another WM still holds it.
Doc and comment changes only; no behaviour change, so red-first does not apply. Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test` green (poll it in the same turn; never end a turn to wait).
Done when: `grep -n 'proposed\|Proposed' docs/decisions.md` shows no accepted record, `grep -rn D-ungrab` empty, steps 2-5 in; reports/docs-f8d.md "## r1" on claude/session-log listing each backlog id closed; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/docs-f8d and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
