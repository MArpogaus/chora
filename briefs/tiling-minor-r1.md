# tiling-minor r1 (rc 1, 2: tiling has no open BLOCK or MAJOR; close the open MINOR of the tiling-honest and tiling-doors reviews)

Model: sonnet, effort high (routine fixes, one core routing change with a unit test)
Lane: cloud, claude/tiling-minor-r1 at 84f66d43 (session head, t133). One round.
Step 0: `git fetch --unshallow`; `cargo check --workspace`. A failure: send_message the PM "tiling-minor-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 150 k context; cost at most 1.0 %.
Findings (full text: reports/tiling-honest.md "rth-", reports/tiling-doors.md "rtd1-" on claude/session-log):
- rth-1 MINOR input.rs:1037: a layer press outside every claim goes to `first_manager`; give the act to the manager whose claim lies on the layer surface's output (else every live manager); two-manager unit test.
- rth-2 MINOR: docs/api.md:941, docs/architecture.md:441, state.rs:1070 list a layer-surface press as an act.
- rth-3 MINOR input.rs:1025: core unit test for `press_layer` (injected press: no act; locked: no act; outside claims: right manager).
- rth-4 MINOR reload.rs:62: without chora-notify a failed reload shows the reason in tiling's own chip (`show_chip`).
- rth-5 MINOR main.rs:1137: on `Ok` of `reload_settings` dismiss `reload_notice`; a gate step checks it.
- rth-7 NIT, rth-8 NIT, rtd1-2 MINOR, rtd1-3 MINOR, rtd1-4 MINOR, rtd1-5 COSMETIC: the fixes the reports name.
- rth-6 stays open (backlog).
Steps:
1) Red first: per finding a commit with the failing test (or the gate step) before its fix (rules.md 8). Docs-only rows need no test.
2) No new `sleep` line anywhere: waits only (gate-lib until_ok / ctl wait). tools/sleep-ratchet, tools/size-ratchet, tools/lint-ratchet pass on your head (counts never raised).
3) One full tests/merge-gate.sh (no --only) on your head.
Done when: each finding above has its red test commit and fix (or a docs commit), the three ratchets pass, full gate PASS on one run.
Report: reports/tiling-minor.md "## r1" on claude/session-log (table finding | red sha | fix sha | test, ratchet output, gate run dir). send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines): head sha, ratchets, gate.
Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/tiling-minor-r1 and the report on claude/session-log (fetch and merge on reject, no force); author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
