# review wm5 r1 (F3): seeded core review of claude/wm5 before its merge

Model: opus, effort high, because (core: the scene draw order is the escape hatch's ground (D89); a wrong order
hides the lock or a modal in every look); the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/wm5 head d14f5342, base 4548feb3 (21 files, +758 -46, 9 commits). One round.
Stop at 120 k context; cost at most 0.8 %; at most 8 findings. No merge gate run (the lane ran it green, --jobs 3).
Read briefs/wm5-r1.md (done criteria), reports/wm5.md on claude/session-log, docs/decisions.md D83, D89;
`git diff 4548feb3..d14f5342`.
Checks, each with its command and files:
1) Red first: 3cae1523 (and 780ab99c for tiling) before the fixes; run the order cells at 3cae1523 if cheap: they
   fail on an assertion, not a build error.
2) Escape hatch: the core's lock surface always draws above every client group; a client cannot set order `lock`
   or reach above it by any prop path (set, create, reparent, reload). Name the refusing lines. `cargo test -p chora`.
3) Order: background < normal < held < overlay < lock, total and stable; inside an order the look's order holds;
   an unknown order value is refused, not defaulted silently. scene.boxes names it.
4) `chora_kit::wm::stack`: only the two rules (modal above its parent, held on top), generic, typed;
   `cargo test -p chora-kit --features testing`.
5) Ports: mc root in background (D83 reorder deleted, nothing left that reorders roots); tiling held = dragged;
   xp and construct unchanged. `cargo test -p chora-mc -p chora-tiling -p chora-xp -p chora-construct`.
6) I12 in tests/acts/invariants.py: the held rule reads keys the looks write (see wm3 rw-1: `for` vs `by`);
   no known-red row added; `python3 -m pytest tests/test_acts.py -q`.
7) Docs: D89 extended, D83 retired, api and glossary say the same as the code; tools/cut-check ok.
Known, not findings: `pull` flake F21; "no look moves a dragged window into held across looks" (proposal 1).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/wm5.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
