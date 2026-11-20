# review handover-ti r1 (RC 2, I3/T7): seeded review of claude/handover-ti before its merge

Model: opus, effort high, because the diff adds a core event (focus.node) and moves xp's desktop to a background root, in the focus and hand-over path of two running looks; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/handover-ti head 371f0fa1, base ce2c436b (17 files, +386 -67). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green); `cargo test -p` only.
A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/handover-ti-r1.md (done criteria) and reports/handover-ti.md "## r1" on claude/session-log; `git diff ce2c436b..371f0fa1`.
Checks, each with its command and files:
1) Core event focus.node (crates/chora/src/ipc.rs, state.rs, crates/chora-scene/src/event.rs, docs/api.md): sent once per change, to whom, no leak of another client's node beyond owner and id; escape hatch untouched; `cargo test -p chora`.
2) One focus holder (crates/chora-kit/src/wm/focus.rs, tiling, xp): a look gives up the keys while another client's node holds them and takes them back on a click; no loop where two looks hand the keys back and forth; `cargo test -p chora-kit --features testing --test wm_focus`, `cargo test -p chora-tiling -p chora-xp`.
3) xp background root (plugins/chora-xp/src/wm/startup.rs, state.rs, choices.rs; D89): xp alone looks the same (wallpaper, icons, Show Desktop, icon clicks and drags, context menu on the desktop); `tests/gate-xp-faces.sh` or the xp unit cells if cheap.
4) Red first: cb953cfe holds the cells and comes before f33684bf and 371f0fa1; tests/gate-handover-shared.sh fails on ce2c436b for the named reason, and the matrix row of merge-gate.sh runs it.
5) Known open, not findings: I5 key list over a notice; overlapping tiling and xp windows stack by root order, not by focus (say only whether this breaks POLS badly enough to be MAJOR for the RC shared bar).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity, file:line, text, fix in one sentence). Append to reports/handover-ti.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
