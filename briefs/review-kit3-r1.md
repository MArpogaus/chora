# review kit3 r1 (F2): seeded review of claude/kit3 before its merge

Model: sonnet, effort high (port review: shared key list in kit, four looks use it; no core change)
Lane: cloud, read-only on code: claude/kit3 head 5b788c72. Own work: `git diff afeebaf9 b8872054` (21 files, +848 -355)
and `git diff 885ff3d3 5b788c72` (7 files); the rest of the branch is history already on the session head. One round.
Step 0: `cargo check -p chora-kit`; it fails for a missing tool: send BLOCKED (no setup) to the PM and stop.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran all 41 steps green).
Read briefs/kit3-r1.md (done criteria) and reports/kit3.md on claude/session-log.
Checks, each with its command and files:
1) Red first: a7916c4b holds the cells and comes before b8872054 (`git log --oneline afeebaf9..5b788c72`).
2) crates/chora-kit/src/keysheet.rs builds rows from `kit::actions` bindings: one wording per System action (EN, DE),
   one chord spelling; a look-only action is named by its look; no new feature, key or System variant.
   `cargo test -p chora-kit --features chora-kit/testing keysheet`.
3) mc keys.rs `words()` and `volume_order` gone; no look keeps its own row table (grep the four plugins);
   tools/copies.toml rows at 0 true (`tools/cut-check`); cut-check REPORT lines not grown.
4) Visible changes only the named ones (capital letters, Esc, Return); a renamed key or lost row in any look is MAJOR.
   `cargo test -p chora-mc -p chora-tiling -p chora-xp -p chora-construct`.
5) tests/acts/known-red.toml: the tiling I5 row went because it passes; mc I5 (net3-m1) stays red; docs/api.md names the rows.
Known, not findings: mc I5 layout (net3-m1, backlog); conflicts with kit5b and kit7 (the PM merges those).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rk-N, severity,
file:line, text, fix in one sentence). Append to reports/kit3.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
