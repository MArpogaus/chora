# review wm1c r1 (F2, F3): seeded review of claude/wm1c before its merge

Model: sonnet, effort high (port review on fixed types; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/wm1c head 5e72e4f9, base 0ffc53a8 (wm1b head; 18 files, +557 -74; the lane
merged the session branch, so diff with `git diff 0ffc53a8...5e72e4f9 -- crates/ tools/ docs/`). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it).
Read briefs/wm1c-r1.md (done criteria) and reports/wm1c.md on claude/session-log.
Checks, each with its command and files:
1) Construct lifecycle only through kit::wm::book: no claim filter or `Event::SurfaceCreated` arm left in
   crates/chora-construct (grep, name lines); give out refused keeps window and pane.
2) Red first: 62a6747d before 1365525c (`git log --oneline`); the 3 red cells named in the report.
3) Behaviour unchanged except the named points: detach and pull-in, overlay takes no free window, restart
   adoption leaves xp's windows; `cargo test -p chora-construct`, `cargo test -p chora-kit --features testing`.
4) `Metrics::keep_estimates` is cfg(test) only and changes no production path.
5) tools/copies.toml counts true (`tools/cut-check`); docs/api.md `construct.window` matches the code.
Known, not findings: construct-hud flake "no claim after the restart of voice" (backlog, flakes lane);
narrow readings listed in the report under "Where I read the brief narrowly" (judge only if wrong).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/wm1c.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
