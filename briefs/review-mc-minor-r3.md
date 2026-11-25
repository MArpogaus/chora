# review mc-minor r3 (rc 1 known issues, F7): seeded review of claude/mc-minor-r3 before its merge

Model: opus, effort high, because the diff changes the mc look (page room from notify cards, wheel steps) with 245 lines, over the PM review limit, and procedures name opus high for look reviews; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/mc-minor-r3 head 42418379, base 0f68810c (6 files, +224 -21). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-mc-minor-r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, EXIT 0); `cargo test -p chora-mc` only.
Read briefs/mc-minor-r3.md (done criteria), reports/mc-minor.md "## Review r2" and "## r3 (lane mc-minor-r3)" on claude/session-log, design/looks mc-style.md; `git diff 0f68810c..42418379`.
Checks, each with its command and files:
1) Red first: 7076ea34 and 904a334c before ff7e9c91, b67ea317 before 9503cc30 (`git log --oneline --stat 0f68810c..42418379`); is each test red at its own commit by its own code (run `cargo test -p chora-mc <name>` at the test commit)? A behaviour fix with no red test before it is MAJOR.
2) rmm2-1, rmm2-2 (rooms.rs): only a foot card takes room; the lane's deviation (room grows for a taller foot card and goes back on close, a fitting card never rebuilds the pages) against the brief: sound and POLS? The tall-card test moved in the fix commit (to [160,500,291,220]): a weakened assert or an honest change?
3) rmm2-3 (main.rs wheel_steps): discrete notch, continuous dy/40, reversal resets; bounded 0-100; no stray wheel to clients.
4) rmm2-4 (tests/mc-edges.sh, 42418379): PM call: accepted as test hardening only (no red; rmm1-1 is pinned by the audit test). Check it weakens nothing and adds no sleep (`tools/sleep-ratchet`).
5) rmm2-5 test and rmm2-6 docs (docs/mc-style.md rules 13, 16) true to the code.
Known, not findings: rpx1-16, rpx1-17, rv9j-2.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rmm3-N, severity,
file:line, text, fix in one sentence). Append to reports/mc-minor.md on claude/session-log under "## Review r3".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
