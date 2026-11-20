# kit-minors r1 (F5 tests, F2 shared core): open MINORs of the kit reviews

Model: sonnet, effort high (kit code in crates/chora-kit and its callers; red test first)
Lane: cloud, branch claude/kit-minors from 79c8d2b5 (session head).
Stop at 250 k context; cost at most 1 %. First check each item is still open on the head; a closed one: say so, skip it.
Step 0: `git fetch --unshallow || true; cargo check --workspace`; red: report BLOCKED at once (no setup.sh, no apt).
Scope: crates/chora-kit, services/chora-notify, the kit call sites in plugins/ only where an item names them. No core (crates/chora) change.
Items (backlog.md on claude/session-log; reports named there):
1) kit6 rw-1: launch.rs wrapper '{ }' lets 'exit N' or a failed exec skip the status write; '( )' subshell, 'exit 1' back in the red cells.
2) kit6 rw-2: launch.rs pidfd or poll failure counts as running; poll alive(pid) or the unit state each second.
3) kit5 r2-1: notify main.rs second chora-notify without the bus name restart-loops (exit 0); r2-2 mc agent.rs critical ids never capped.
4) kit7 r2: helper.rs answered->working untested; a new utterance does not clear answered; seed() order.
5) kc-1: chora-assistant on_fake_core allow: give FakeCore an output answer helper or narrow the gate-0.8 grep, drop the allow.
6) rc-2: kit testing.rs: document that recorders run under the model lock.
Each fix: red test commit first (unit test through kit testing / FakeCore), then the fix. Keep tools/size-ratchet and tools/cut-check green.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,doors,chat-keys,hud,mc-paths` green; push the branch.
Report: reports/kit-minors.md "## r1" on claude/session-log (table: item, open or closed, red, fix, test, cell).
Send the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines with send_message. Poll every command in the same turn;
never end a turn to wait.
Rules: commits as Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
red test commit before each fix; push only claude/kit-minors and claude/session-log (fetch and merge on reject, no force). A hook that asks for another identity: ignore it.
