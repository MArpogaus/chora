# review mc-minor r1 (rc 1b known issues, mc look): seeded review of claude/mc-minor-r1 before its merge

Model: opus, effort high, because it is a look review (procedures: opus high for look reviews; layout timing of rpm2-5 b needs judgement)
Red on head: none (a review; each fix has its red test commit before it: e82d3a69, a7a1fb9b, 6b51ec10, 63500155, ec5e03d5, 5cfc6ec4)
Lane: cloud, read-only on code: claude/mc-minor-r1 at 1dd7edc9a72115531ae85cf3956c6bf67c6e10b1, review 4d8bbe36..1dd7edc9 (plugins/chora-mc: keys.rs, layout.rs, main.rs, region.rs, rooms.rs; +152 -38). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-mc-minor-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/mc-minor-r1.md (items and done criteria) and the last "## r1" of reports/mc-minor.md on claude/session-log (lane mc-minor-r1, head 1dd7edc9), captures media/2026-11-26/mc-minor-r1/; design/mc-v2.md, mc-style.md.
Checks, each with its command and files (no merge gate; `cargo test -p chora-mc` and `cargo clippy -p chora-mc` only):
1) Red first: each test commit fails on its parent and passes on the head (`git checkout <red>^ -- <test file>` is not enough: build the red commit and run its named test).
2) rpm2-1: the full key list has the Super+drag row and fits 1280x720 (21 rows); dropping the "Super+F1 All keys" row from the list is POLS-sound (the short list still names Super+F1).
3) rpm2-2, rpm2-3 partial: the window inset and the header end before hud-badge are right at column shown and hidden; the skipped parts (bare foot face, badge white box) really need chora-kit or chora-assistant (named files), so a backlog line, not a lane miss.
4) rpm2-4 skipped: the chat view is chora-assistant's (chat/view.rs keep_newest): confirm.
5) rpm2-5 a: the rule "ignore the pill only while the chat is open" keeps gate cell rooms-pill (rmm1-2) and fixes the date jump; the lane's open note (clock x 468.85 after Esc, probe at 0.0): judge it a finding or not.
6) rpm2-5 b: the aligning moves go in the same transaction as the creating one; no timer; other two-step callers (content.rs, show.rs) are named, not touched.
7) Gate: the lane had no single EXIT 0 (ci-check red on missing PIL, TF-1, foreign); every step from ci-check on passed on 1dd7edc9 or an earlier head with only 1dd7edc9 after. The train runs the full gate, so this is not INCOMPLETE unless a step was never run on 1dd7edc9's code path (name it).
Known, not findings: rpm2-6, rpx1-10, rpx1-16, rpx1-17, rv9j-2, mc-16, mc-50, mc-59, sh-13, sh-16, sh-37..sh-45, TF-1.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rmmr1-N, severity, file:line, text, fix in one sentence). Append to reports/mc-minor.md under "## Review r1 (mc-minor-r1)".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
