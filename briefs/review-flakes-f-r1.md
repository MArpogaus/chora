# review flakes-f r1 (F5): seeded review of claude/flakes-f before its merge

Model: opus, effort high, because the diff changes the core crash words path (crates/chora/src/words.rs:
alarm reads /proc syscall, write_all on EINTR) and the construct pull placement; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/flakes-f head 847f0964, base dc4bc46d (9 files, +322 -35). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, --jobs 3, 41 steps).
Read briefs/flakes-f-r1.md (done criteria) and reports/flakes-f.md on claude/session-log; `git diff dc4bc46d..847f0964`.
Checks, each with its command and files:
1) F21 pull (plugins/chora-construct/src/pull.rs, panes.rs, input.rs): the pane is placed and faced for
   Construct::target(), not the mid fly-in pose; no other caller relies on the old pose. Red f106713d, f1db38ad
   before fix 10c5a50b, 847f0964 (`git log --oneline dc4bc46d..847f0964`); `cargo test -p chora-construct`.
2) F17 words (crates/chora/src/words.rs): a writer asleep in write(2) is not called stuck; a truly stuck
   process still is; write_all loops on EINTR and on short writes, never spins or loses bytes; /proc read
   failure falls back safely; Linux-only code is gated. Red 16dbdd4a before cc413838; `cargo test -p chora words`.
3) F29 tiling-audit (tests/gate-tiling-audit.sh): the wait for the work area is bounded (<= 20 s), fails with
   its own FAIL line, and does not hide a real bar-over-window bug. Red d9100536 before 5955b3a9.
4) gate-lib (tests/gate-lib.sh, gate-lib-selftest.sh): the stale env file fix cannot leak state between steps;
   `bash tests/gate-lib-selftest.sh`.
5) tests/under-load.sh: a test tool only, no sleep-based pass, no system package.
Known, not findings: cube.open 300 ms bound 1/5 under load; F26 doors, F28 chat-keys not reached (next round).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/flakes-f.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
