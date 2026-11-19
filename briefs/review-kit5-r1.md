# review kit5 r1 (F2): seeded review of claude/kit5b before its merge

Model: opus, effort high, because the diff makes chora-notify the only notice server for all four looks (new live
API notify.show/close, an optional session bus) and deletes each look's own stack, where a wrong rule covers work
or loses a notice (vision card: quiet, never covers work)
Lane: cloud, read-only on code: claude/kit5b head 4e003748, base 4548feb3 (82 files, +1193 -1296). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran 41 steps green).
Read briefs/kit5-r1.md (done criteria), briefs/kit5-r2.md and reports/kit5.md on claude/session-log;
`git diff 4548feb3..4e003748`; `git log --oneline 4548feb3..4e003748` (no dev commit, no merge).
Checks, each with its command and files:
1) chora_kit::notice (show, replace, dismiss, Gone, show_or_log) and chora-notify notify.show/notify.close: ids
   shared between bus and API, replace keeps the id, no bus still serves plugins; docs/api.md matches;
   `cargo test -p chora-kit --features testing notice`, `cargo test -p chora-notify`.
2) Red first: 827ab5ab, 7589e533, 34e08b31, 4ac59ac7 before their fixes; the I9 act after the feat (deviation 1
   of r1): is I9 now proven red somewhere, or a finding.
3) Ports keep behaviour: xp and volume via kit; construct Construct::tell; mc notify.push and notify.clear (last
   32 ids: a leak or a lost close?); tiling chora-notify instead of mako (--launch mako still works; config docs).
4) Conflict picks of r2: 7123479e (notify main.rs), 855445c4 (acts harness, invariants), 9f957abf (construct
   input/keys): both sides kept, nothing of the session side lost (diff the files against 4548feb3 and kit5).
5) 4e003748 (cut-check names, notice-act args): a real fix or a weakened check.
6) tools/copies.toml rows true (`tools/cut-check`); zbus not in `cargo tree -p chora-kit`; no file over 1,500 lines.
Known, not findings: the dropped 6aad3457 (release files) by design.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/kit5.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
