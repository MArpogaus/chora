# review kit5 r2 (F2): seeded check of the r3 fixes on claude/kit5b before its merge

Model: sonnet, effort high (fix verification of 8 known findings; the design was reviewed by opus in r1)
Lane: cloud, read-only on code: claude/kit5b head 7b01dd5a; r1 head 4e003748; merge of the session branch d7cd08b9
(daf83391, 7 conflicts). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run.
Step 0, first 3 minutes: `cargo check -p chora` and `command -v foot fuzzel waybar`; if either fails, send_message
the PM at once "review kit5 r2 BLOCKED: no setup" and stop.
Read reports/kit5.md "## Review r1" and "## r3" on claude/session-log; `git log --oneline 4e003748..7b01dd5a`;
the fix diff without the merged session side: `git diff d7cd08b9 7b01dd5a` plus each fix commit named in "## r3".
Checks, each with its command and files:
1) rw-1..rw-8 each closed: the red commit fails before its fix and passes after (check out the red sha, run the named
   test; `cargo test -p chora-notify`, `cargo test -p chora-kit --features testing --test notice`, `cargo test -p chora-mc`).
   rw-1: a second chora-notify (bus taken) keeps serving notify.show and exits 0 only as the r3 report says: is that right?
   rw-3: the card steps aside of the chat rect (gate-chat-keys box cell): read the cell, not just its name.
   rw-4: the acts now assert shown/closed (tests/acts/looks/common.py, mc.py); not vacuous.
2) The merge d7cd08b9: each of the 7 conflict picks keeps both sides; nothing of daf83391 lost
   (`git diff daf83391 d7cd08b9 --stat`, read the conflict files the report names).
3) No new MINOR work beyond the findings; `python3 tools/cut-check` ok; no file over 1,500 lines; clippy
   `cargo clippy -p chora-notify -p chora-kit -p chora-mc --all-targets` clean.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/kit5.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
