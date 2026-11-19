# review wm3 r1 (F3): seeded review of claude/wm3 before its merge

Model: sonnet, effort high (a kit type plus four ports on it); the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/wm3 head 0e0bd700, base 4548feb3 (24 files, +595 -111). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green).
Read briefs/wm3-r1.md (done criteria) and reports/wm3.md on claude/session-log; `git diff 4548feb3..0e0bd700`.
Checks, each with its command and files:
1) Red first: ee61246e holds the cells and comes before 1c3c4d99. The brief's I4 act was green on the base (deviation in
   the report): are the two xp unit cells the real sd-h2 bug (a give, a pull-out drag leaving the Start menu open)?
   Run them at ee61246e if cheap.
2) `chora_kit::wm::open` (menu, popup, dialog, sheet, launcher; modal; Esc order; shut at a drag or a give) small and
   generic; it feeds `kit::wm::state` open; `cargo test -p chora-kit --features testing`.
3) Each look lists its open things through it, no look keeps its own list: xp, tiling, mc, construct.
   `cargo test -p chora-xp -p chora-tiling -p chora-mc -p chora-construct`.
4) Known-red rows removed (net2-a1, net2-a3 x3): each removal backed by a cell now green, not a weakened match.
5) Visible behaviour unchanged except the named bugs (Esc closes the top open thing first; a drag or give shuts menus).
6) tools/copies.toml and size-ratchet.list true (`tools/cut-check`); docs/api.md names `open` for every look.
Known, not findings: `pull` flake F21 (194 px for 200; flakes-f lane owns it).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/wm3.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
