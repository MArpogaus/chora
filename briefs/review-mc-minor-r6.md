# review mc-minor r6 (rc 1b, F7): seeded review of claude/mc-minor-r6 before its merge

Model: opus, effort high, because the diff changes the mc look (panel cut at the page edge, urgent notice words) and procedures name opus high for look reviews; the builder never reviews itself (rules.md 7)
Red on head: none (a review; the lane's reds 12bf1410, 05cd4c35, 91e38d91 precede their fixes)
Lane: cloud, read-only on code: claude/mc-minor-r6 head e7e4bca0, base 3098926e (10 files, +141 -30). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo test -p chora-mc`. A failure: send_message the PM "review-mc-minor-r6 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/mc-minor-r6.md (done criteria), reports/mc-minor.md "## r6", findings/mc.md rows mc-61, mc-63; docs/mc-style.md; `git diff 3098926e..e7e4bca0`; stills media/2026-11-28/mc-minor-r6/.
Checks, each with its command and files (no merge gate run; `cargo test -p chora-mc` and single gate cells only):
1) Red first: each test commit (12bf1410, 05cd4c35, 91e38d91) fails on its own tree with its fix absent and passes at e7e4bca0; a fix without its red: MAJOR.
2) mc-63 (layout.rs `edge_takes`): a panel the edge cuts shows whole or waits; does `scroll_most` and the down mark still reach every line (no line that can never show)? Open the before and after stills, light and dark.
3) rpx1-17 (agent.rs `urgent_title`): every critical path gets "Urgent: " once (no double prefix on a re-push or an mc error), the card text fits at 1280x720, and mc names no other plugin (cut-check).
4) rpx1-16: spelling Super+A one way on page and key list; the pill half stays open in findings (host draws it): is that row honest?
5) Gate: the lane had no single EXIT 0. It names mc-sofa clock-chat, remote-back, rooms-pill-status red under --jobs 2 and green alone as foreign; this lane changed tests/gate-mc-sofa.sh. Run `tests/gate-mc-sofa.sh` 3 times on e7e4bca0 and once on 3098926e; a red that this diff causes or worsens: MAJOR; a pre-existing flake: name it with its log line (the train runs the full gate).
Known, not findings: rpx1-16 pill half, rpm2-6, rpx2-5, co-44.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rmm6-N, severity, file:line, text, fix in one sentence). Append to reports/mc-minor.md under "## Review r6".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait. Before ending: `pgrep -fa 'chora --headless'` empty.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
