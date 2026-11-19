# review wm1d r1 (F2, F3): seeded review of claude/wm1d before its merge

Model: sonnet, effort high (a port review on fixed kit::wm types, owner 06:56)
Lane: cloud, read-only on code: claude/wm1d head 18231aa1, base 1d1aa5fd (17 files, +555 -149). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, 41/41).
Read briefs/wm1d-r1.md (done criteria) and reports/wm1d.md on claude/session-log; `git diff 1d1aa5fd..18231aa1`.
Checks, each with its command and files:
1) xp lifecycle through kit::wm::book (create, attach, detach, destroy, output removed, give in and out);
   xp's own copy and its state adapter gone (plugins/chora-xp/src/wm.rs, dispatch.rs, windows.rs);
   `cargo test -p chora-xp`.
2) Red first: 8d194c19 holds the port cells and comes before 18231aa1; rw-1 cell inside it fails on the base
   (run one cell at 8d194c19 only if cheap).
3) Restart from the old meta (29e2cd77 format) keeps stacking, frames, desktop, minimized, taskbar order;
   no window opened twice; a window reserved for mc left alone (cells named in reports/wm1d.md).
4) wm1c rw-1 fix (plugins/chora-construct/src/adopt.rs): adopts only what the book claimed; no window lost
   when `free` is false; `cargo test -p chora-construct`. rw-2 in docs/api.md true.
5) tools/copies.toml counts true (`tools/cut-check`); SurfaceCreated outside kit 0; size-ratchet ok.
6) Visible behaviour unchanged; the narrow readings in reports/wm1d.md acceptable or a finding.
Known, not findings: refactor proposals 1-6 of reports/wm1d.md (backlog); xp keeps its own meta key.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/wm1d.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
