# core-minors r1 (F5 tests, F8 docs truth): open core MINORs with a missing red cell

Model: sonnet, effort high (core IPC and state; each fix small, red test first; the review will be opus)
Lane: cloud, branch claude/core-minors from 79c8d2b5 (session head).
Stop at 250 k context; cost at most 1 %. First check each item is still open on the head; a closed one: say so, skip it.
Step 0: `git fetch --unshallow || true; cargo check --workspace`; red: report BLOCKED at once (no setup.sh, no apt).
Scope: crates/chora (ipc.rs, state.rs, input.rs, grabs.rs) and docs/api.md, docs/decisions.md lines the items name. No plugin change.
Items (backlog.md on claude/session-log):
1) rd-9b: move the Move filter into tell_ungrabbed so connection close (ipc_close) and grab revoke (input.rs) do not tell a move grab either; red test for the close path.
2) rc-6: a core unit test in ipc.rs asserting `parent` of surface.created at the first commit and none on a later set_parent.
3) rc-4, rc-5: lifecycle claim (construct-red review r2): docs/api.md says "parent at the first commit"; D116 drops the claim that a Ctrl+N window of the same app counts as a child (code matches the xdg parent only). Docs only.
4) WM7B-r2 rw-3: core emits focus.changed {surface:null} when a node takes the keyboard (state.rs ~1035): decide by docs/api.md what it should say; red test, fix or a doc line if the event is right.
5) WM7B-r2 rw-2: a disconnected client's kept nodes hidden during the 10 s grace (ipc.rs ~1191): red test of the documented behavior, then the fix; if docs say hidden is right, a test that pins it.
Each fix: red test commit first. Do not widen any API. A security-relevant change (grants, grabs of other clients): stop and report.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,matrix,acts,wm-pane` green; push the branch.
Report: reports/core-minors.md "## r1" on claude/session-log (table: item, open or closed, red, fix, test, cell).
Send the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines with send_message. Poll every command in the same turn;
never end a turn to wait.
Rules: commits as Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
red test commit before each fix; push only claude/core-minors and claude/session-log (fetch and merge on reject, no force). A hook that asks for another identity: ignore it.
