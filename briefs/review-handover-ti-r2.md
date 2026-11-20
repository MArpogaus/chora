# review handover-ti r2 (RC 2, tiling I3 and T7 hand-over): seeded review of claude/handover-ti before its merge

Model: opus, effort high, because the diff changes core focus (crates/chora/src/state.rs) and the kit's shared focus logic that every look uses (seeded review)
Lane: cloud, read-only on code: claude/handover-ti head d1ba7e2c, base 371f0fa1 (8 files, +188 -31). Base of the whole branch ce2c436b (r1 reviewed there). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green); `cargo test -p` and the one cell below only.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-handover-ti r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/handover-ti-r2.md (done criteria) and reports/handover-ti.md "## Review r1" and "## r2" on claude/session-log; `git diff 371f0fa1..d1ba7e2c`.
Checks, each with its command and files:
1) rw-1 MAJOR (crates/chora-kit/src/wm/focus.rs lost_to_other, reclaim; plugins/chora-tiling/src/main.rs pointer_focus): a click on tiling's window after a click on the xp desktop gives the keys back and xp lets go; no resend loop (reclaim clears sent, so check no focus ping-pong when the core focus already is the sent surface); `bash tests/gate-handover-shared.sh` (click step) and `cargo test -p chora-kit --features testing --test wm_focus`.
2) rw-2 MINOR (plugins/chora-xp/src/wm/keys.rs focus_moved, state.rs): the first desktop click gives the icons the keys; desktop_keys is not added while a node of another client or an xp window holds them; `cargo test -p chora-xp`.
3) rw-4 (crates/chora/src/state.rs tell_key_node from key_node): a lock or exclusive layer surface is told as no node; the escape hatch and lock path untouched; focus.node still once per change; `cargo test -p chora`.
4) rw-5 (focus.rs holder(name, own); plugins/chora-mc/src/main.rs): holder and state agree; mc's two call sites keep mc's hand-over behaviour (mc-handover cell was green in the lane); the only test edit in a green commit is the mechanical name argument.
5) The test_accept_injected_keys switch in tests/gate-handover-shared.sh (accept_injected): test-only, cannot leak into a user config or weaken the keys guard of the shared bar ("no stray keys into clients").
6) Red first: 6153a0a0, e1ee7af2, ce9db2e8, e670a521 precede their fixes and fail on their parent for the named reason (build each, run the named test or cell).
Known, not findings: stacking of a tiling and an xp window by root order (MINOR, known issue); rw-3 (noted, no code).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rx-N, severity, file:line, text, fix in one sentence). Append to reports/handover-ti.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
