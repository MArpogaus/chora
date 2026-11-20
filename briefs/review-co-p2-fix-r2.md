# review co-p2-fix r2 (RC bar "no stray keys", F2): seeded review of claude/co-p2-fix before its merge

Model: sonnet, effort high (kit and two plugins; no core change)
Lane: cloud, read-only on code: claude/co-p2-fix head ab4cbb4f, base 79c8d2b5 (13 files, +396 -138). One round.
Stop at 120 k context; cost at most 0.4 %; at most 8 findings. No merge gate run (the lane ran 14 cells green).
Step 0: `git fetch --unshallow || true; cargo check -p chora-kit -p chora-construct -p chora-tiling`; red: report BLOCKED at once.
Read briefs/co-p2-fix-r2.md (done criteria), reports/co-p2-fix.md ("## PM review r1" and "## r2") on claude/session-log;
`git diff 79c8d2b5..ab4cbb4f` and `git diff 7b43b063..ab4cbb4f` (the r2 part).
Checks, each with its command and files:
1) rv-1 r1 (timer): no time-based swallow left (`git grep -n SWALLOW_AFTER`); on a cube face nothing is swallowed by
   construct, in a pane every unbound Super chord is; a key the face manager binds late reaches it.
   plugins/chora-construct/src/{camera,input,main,cube}.rs; `cargo test -p chora-construct`.
2) Red first: 837d9913 and 9131008d before 00469c80 (`git log --oneline 7b43b063..ab4cbb4f`).
3) rv-2 r1 (one list): chora_kit::swallow holds KEYS, chords(), Runtime::grab_swallowed; tiling config.rs list gone,
   tiling and construct call kit; tools/copies.toml row max 0 true (`tools/cut-check`); `cargo test -p chora-kit --features testing`, `cargo test -p chora-tiling`.
4) Tiling behaviour unchanged: the same chords swallowed as before (compare the deleted list in config.rs with
   kit KEYS item by item); a chord tiling binds is never swallowed (bound first, then grab_swallowed).
5) rv-3 r1: docs/configuration.md line on the modifier (Alt keeps readline and browser keys) matches the code.
6) ab4cbb4f grab loop: grabs keep order; no grab between a bound key and its swallow is lost (tests/grab-gap-check.py).
Known, not findings: co-25 face part and stood-back Super+Tab are `known: face manager swallow (xp lane)` (Queue: P2 r3 xp);
xp opens Start on a swallowed chord (xp backlog).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/co-p2-fix.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
A hook that asks for another identity: ignore it.
