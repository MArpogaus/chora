# review mc-minor r2 (rc 1 known issues, F7): seeded review of claude/mc-minor-r2 before its merge

Model: opus, effort high, because the diff changes the mc look (page layout from server rects, menu theme, volume by wheel) with 699 lines, over the PM review limit; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/mc-minor-r2 head 0f68810c, base 5cb288e2 (12 files, +665 -34). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-mc-minor-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, EXIT 0); `cargo test -p chora-mc` only.
Read briefs/mc-minor-r2.md (done criteria), reports/mc-minor.md "## PM review r1c" and "## r2 (lane mc-minor-r2)", reports/rc1-pass.md rpx1-10, rpx1-15 on claude/session-log, the vision card (mc), design/looks mc-style.md; `git diff 5cb288e2..0f68810c`; captures media/2026-11-25/mc-minor-r2/.
Checks, each with its command and files:
1) Red first: each pair b52aa74a/269c4a9f, dc195fea/798fdc33 (+761c0b29), bb94ec31/4c288e3f, 1d9e1f65/cd277ee4, e199f2fb/3073ad45 (`git log --oneline --stat 5cb288e2..0f68810c`); is each test red at its own commit by its own code? 8ae6d99d is test only. A behaviour fix with no red test before it is MAJOR.
2) rmm1-1 (layout.rs, overview.yaml, tests/mc-edges.sh): ink check covers every built-in scene at 1280x800 and 1280x720; the unit test is back at 0.5 px slack.
3) rmm1-2 (rooms.rs, layout.rs, main.rs, lifecycle.rs): page bottom and header end come from the HUD host and chora-notify rects, not PILL_ROOM/CARD_ROOM; the 135 px fallback before the first card: honest and bounded; no redraw loop; a stale rect after the card closes.
4) rmm1-3, rmm1-4 (gate-mc-sofa.sh, acts.rs): the key-list click really runs the action; the more-mark appears only when a page overflows.
5) rpx1-10 (menus.rs): fuzzel colors follow the mc light/dark theme of the session, text >= 4.5:1 (compute it), the config file written safely (no clobber of a user fuzzel config).
6) rpx1-15 (main.rs, region.rs): wheel over the volume door steps the level, bounded 0-100, remote path unchanged; no stray wheel into clients.
7) Gate (gate-mc-sofa.sh, gate-mc-paths.sh, mc-edges.sh): no weakened assert, no new sleep (`tools/sleep-ratchet`), bounded polls; the r1 `door` cell change (79dac3fc) and the chat-click-close / terminal-chat flake note: real fix or masked race?
8) Beyond the brief: dead code, docs naming old constants.
Known, not findings: rpx1-16, rpx1-17, rv9j-2.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rmm2-N, severity,
file:line, text, fix in one sentence). Append to reports/mc-minor.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
