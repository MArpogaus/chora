# review mv4 r1 (F4): seeded review of claude/mv4 before its merge

Model: sonnet, effort high (moves plus two small fixes; owner 06:56 routine)
Lane: cloud, read-only on code: claude/mv4 head 7e339880, base a17e7858 (10 files, +1981 -1833). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it in pieces).
Read briefs/mv4-r1.md (done criteria) and reports/mv4.md on claude/session-log; `git diff a17e7858..7e339880`.
Checks, each with its command and files:
1) Moves only in 91714434 (plugins/chora-mc/src/region*) and 17186311 (voice chat*): compare moved bodies
   (`git diff --color-moved=dimmed-zebra -M a17e7858 91714434`, same for 17186311); no logic change, no
   visibility wider than pub(crate) without need; `cargo test -p chora-mc`, voice crate tests.
2) Line counts as reported (region.rs 1,284, chat.rs 1,049 production lines); size-ratchet list lowered
   honestly in da60b6cf (`tools/cut-check`).
3) Red first: 675be38d before 6c6f776e (SB2), 890d952b before 7e339880 (UX-D1); each red cell fails on its
   parent (run one only if cheap). Fixes minimal and right (dropped spoken note kept as agent words; list text
   cleared from the field on hide).
4) construct-hud flake named in reports/mv4.md: not caused by this diff (no construct file touched).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/mv4.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
