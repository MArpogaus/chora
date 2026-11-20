# review co-pane r3 (RC 2, construct I3): seeded review of claude/co-pane before its merge

Model: opus, effort high (because r3 changes core input.rs: a key grab press now ends other clients' lone-modifier taps; core escape hatch and safety)
Lane: cloud, read-only on code: claude/co-pane head 18417f06, base 628bb930 (9 files, +181 -38); r2 reviewed f10842d8..628bb930. One round.
Stop at 120 k context; cost at most 0.6 %; at most 8 findings. No merge gate run (the lane ran 16 cells green); `cargo test -p chora`, `cargo test -p chora-construct`, `bash tests/persona-selftest.sh` only.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-co-pane r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/co-pane-r3.md (done criteria) and reports/co-pane.md "## Review r2" and "## r3" on claude/session-log; `git diff 628bb930..18417f06`.
Checks, each with its command and files:
1) rw-1 MAJOR fix (crates/chora/src/input.rs, camera.rs): in cube use mode xp keeps Super: Super+e, Super+r, Super+d reach the client as a chord; xp's lone Super tap still opens Start; construct no longer grabs the Super tap in use mode, and its own construct chords leave no lone tap behind. Say whether the tests pin the client-side key stream or only internal state; the lane says injected keys never choose a manager, so the swallow is not gated: judge if a gate step could reach it.
2) Core change (input.rs): a press taken by a key grab ends lone-modifier taps other clients hold, without their release. Check: no client is left with a stuck modifier (release still delivered, or the tap only cancelled); the escape hatch (VT switch, the core quit chord) is untouched and no plugin can override it; the one helper shared with buttons has no behaviour change for buttons; unit tests cover grab, no grab, two clients.
3) rw-2 (docs/cube-modes.md, docs/configuration.md, docs/api.md): the F5 drag sentence is in the same words as plans/construct-requirements.md F5; api.md describes the new tap rule truthfully.
4) rw-3 (overlay.rs:310 rewrap): no behaviour change.
5) tests/gate-construct-pull.sh new step (04a05d31): red on r2 code for the named reason, green on 18417f06; not flaky by design (no sleeps without a wait condition).
6) Red first: cf1d028a and 04a05d31 before 68a97ef0, 876f028b before c7679e69; each red fails on its parent for the named reason; build at least cf1d028a's parent.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rx-N, severity, file:line, text, fix in one sentence). Append to reports/co-pane.md on claude/session-log under "## Review r3".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
