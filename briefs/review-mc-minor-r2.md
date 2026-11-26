# review mc-minor r2 (rc 1b, mc look; rmmr1-1 MAJOR, rmmr1-2 MINOR): seeded review of claude/mc-minor-r2 before its merge

Model: opus, effort high, because it is a look review that decides the mc RC bar (procedures: opus high for look reviews)
Red on head: none (a review; the lane's reds ac9fab66a, 00c85214a precede their fixes)
Lane: cloud, read-only on code: claude/mc-minor-r2 at 59dd6894a1ba497add0dcdb10bd665b81b53eefb, review 1dd7edc9a..59dd6894a (plugins/chora-mc layout.rs, region.rs, rooms.rs: 3 files, +129 -24). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-mc-minor-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/mc-minor-r2.md (done criteria), reports/mc-minor.md "## Review r1 (mc-minor-r1)" (rmmr1-1..4) and "## r2", media/2026-11-26/mc-minor-r2/ on claude/session-log.
Checks, each with its command and files (no merge gate; `cargo test -p chora-mc`, `cargo clippy -p chora-mc --all-targets`):
1) Red first: ac9fab66a fails on its parent with the review's numbers (932 closed, 960 open), c25784b82 makes it pass; 00c85214a red before 59dd6894a likewise. Run each.
2) rmmr1-1: read_pill and header_end (rooms.rs, region.rs): with the chat open the pill edge read before it opened holds, a badge may only move it left; header end and date stay put on Super+a with the column shown and hidden, pill alone and with badge, 1280x720 and 1024x768. Does the test cover each of the 8 cases, and does a pill that changes width while the chat is open (room change) leave a stale edge (POLS)?
3) rmmr1-2: measure returns Option; a shown item with no rect keeps the places for that update. Can a probe that never gets a rect freeze the header for good (a hidden-then-shown item, a removed item)? Is `props.visible not Some(false)` the right test for shown?
4) The old test the lane replaced (the_header_stays_where_it_is_while_the_pill_moves_into_the_chat): was its rule wrong, or is a case lost?
5) Captures in media/2026-11-26/mc-minor-r2/ show the date not moving on Super+a, column hidden, pill and badge (name the file).
Known, not findings: rmmr1-3, rmmr1-4, rpm2-6, rpx1-10, rpx1-16, rpx1-17, rv9j-2, mc-16, mc-50, mc-59, sh-13, sh-16, sh-37..sh-45, TF-1.
A failed Done-when line of briefs/mc-minor-r2.md is INCOMPLETE, not MERGE-OK. A fix with no red test commit before it is MAJOR.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rmmr2-N, severity, file:line, text, fix in one sentence). Append to reports/mc-minor.md under "## Review r2 (mc-minor-r2)".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
