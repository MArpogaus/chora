# review wm2 r1 (F3, F2): seeded review of claude/wm2 before its merge

Model: opus, effort high, because the diff changes the core input path (key place, escape hatch) where a
wrong rule sends stray keys; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/wm2 head 751dfbcb, base dc4bc46d (23 files, +658 -199). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green).
Read briefs/wm2-r1.md (done criteria) and reports/wm2.md on claude/session-log; `git diff dc4bc46d..751dfbcb`.
Checks, each with its command and files:
1) Core key place (crates/chora/src/input/place.rs, commit 767249d7): the escape hatch stays above every
   plugin; a key a look binds reaches the look; no key reaches a client the pointer only hovers. Read the
   rule against docs/api.md; `cargo test -p chora input::`.
2) Red first: 93e50bfd holds the cells and comes before the fixes (`git log --oneline dc4bc46d..751dfbcb`);
   run the three cells at 93e50bfd only if cheap.
3) kit::wm::focus (Focus, Holder, Policy) generic and small; tiling ShownFocus gone; mc focus through kit;
   `cargo test -p chora-kit --features testing`, `cargo test -p chora-mc -p chora-tiling`.
4) Visible behaviour unchanged except the named bugs (hover no longer moves the keys, click does: D55).
5) tools/copies.toml and size-ratchet.list counts true (`tools/cut-check`); region.rs not grown.
Known, not findings: I3 for xp open (needs core proposal 1); no mc book on this base (deviation from 2.2).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/wm2.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
