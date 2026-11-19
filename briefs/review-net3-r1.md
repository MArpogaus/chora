# review net3 r1 (F7, F5): seeded review of claude/net3 before its merge

Model: opus, effort high, because the diff adds a core IPC read call (scene.boxes) whose owner filter is a
security boundary (a client must see only its own nodes), plus invariants that gate every merge
Lane: cloud, read-only on code: claude/net3 head bc54896a, base dc4bc46d (21 files, +1628 -47). One round.
Stop at 120 k context; cost at most 0.6 %; at most 8 findings. No merge gate run (the lane ran it green).
Read briefs/net3-r1.md (done criteria) and reports/net3.md on claude/session-log; `git diff dc4bc46d..bc54896a`.
Checks, each with its command and files:
1) scene.boxes is read-only and leaks nothing: a client socket gets only its own nodes, the PM/plugin socket
   rules match scene.query (crates/chora/src/ipc/boxes.rs, chora-scene boxes.rs, docs/api.md "Boxes");
   `cargo test -p chora-scene`, `cargo test -p chora ipc::`.
2) Text runs: `Fonts::run` shares placement with `draw` (one code path, no re-measure); font check cells
   (`cargo test -p chora-draw`); xp and chora-notify only report (`cargo test -p chora-xp -p chora-notify`).
3) Invariants I2, I5, I6, I9, I10, I12, I13 in tests/acts: each has a planted-breach cell that fails; the
   I7 work_rect fix is right (scene vs output pixels). Read the selftest; run its python cells if cheap.
4) Known-red rows net3-t1, net3-m1 each name a real finding; I13 not red is argued in the report: judge it.
5) Visible behaviour unchanged; no look change beyond reporting; no new dependency (`tools/cut-check`).
Known, not findings: red-first for draw and notify inside the feat commits (process, already noted);
refactor proposals 1-6 of the report.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/net3.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
