# plugins high r1 (rc 6): the three HIGH crashes of the plugin review

Lane: sonnet, cloud, new branch claude/plugins-high from the session branch
claude/autonomous-dev-session-3w-i33pg7 (de55f346). One round. Stop at 250 k context and hand over.
Read dev-rules.md and conventions.md (with the skills it names) first. Setup: clone
MArpogaus/chora to /home/user/chora, fetch all branches, worktree /home/user/wt/log on
claude/session-log, `bash /home/user/wt/log/setup.sh`. Findings: reports/review-plugins-r1.md.
Items, each red first (a unit test that panics or hangs without the fix), then the smallest fix:
1. XP-B-01: plugins/chora-xp/src/startmenu.rs:494 rest() indexes cascades[level] for a pending
   Press::Row after close_from() drained it; Escape or Left within 400 ms of a hover panics xp.
   close_from() clears the pending press of the closed levels.
2. MC-01: plugins/chora-mc/src/main.rs:1788 ty() expects the style label, which is not in NEEDED
   (src/config.rs:17-25); a styles map without label panics mc. Add label to NEEDED so
   Config::check rejects such a file with a message and mc keeps its defaults (user files survive).
3. NO-01: plugins/chora-notify/src/card.rs:217 an unbounded body (bytes, lines) blocks the loop and
   can exhaust memory. Cap the body before layout (lines and bytes that fit the card; an ellipsis).
Another lane (doors) changes chora-xp, chora-mc and the shared parts; keep your diff to these
three spots and their tests.
Done when: tests/merge-gate.sh passes. Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most
10 lines): commits, red and green.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; stage by name; no force push; push only claude/plugins-high; never
check out another branch in an existing worktree.
