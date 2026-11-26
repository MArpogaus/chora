# review mcm3 r1 (rc 1b, mc look; rmmr2-1 MINOR, rmmr2-2, rmmr2-3 NIT): seeded review of claude/mc-minor-r3 before its merge

Model: opus, effort high, because it is a look review that also changes chora-assistant (the pill host every look uses), so a wrong rule moves the pill in xp, tiling and construct too (procedures: opus high for look reviews)
Red on head: none (a review; the lane's reds 014a1d67, a991472b, fd315abe precede or pin their fixes)
Lane: cloud, read-only on code: claude/mc-minor-r3 at d8eb907cfdcec95d8f5c620e985c4351ed5585ab, review 59dd6894a..d8eb907cf (assistant.rs, chat/host.rs, mc methods.rs, region.rs, tests/gate-mc-sofa.sh: 5 files, +181 -9). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-mcm3-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/mc-minor-r3.md (done criteria and the PM decision on rmmr2-1), reports/mc-minor.md "## Review r2 (mc-minor-r2)" and the last "## r3", media/2026-11-26/mc-minor-r3/ on claude/session-log.
Checks, each with its command and files (no merge gate; `cargo test -p chora-mc -p chora-assistant`, `cargo clippy -p chora-mc -p chora-assistant --all-targets`):
1) Red first: 014a1d67 red on its parent before 44c4f549; fd315abe red before 33518d2c; a991472b (gate cell clock-chat) fails on 1dd7edc9 with the named line. Run the unit reds; the gate cell only if cheap (CELLS filter of tests/gate-mc-sofa.sh).
2) rmmr2-1 (assistant.rs, host.rs): the pill moved clear of the chat stands under the header text, never over the date or clock, at 1280x720 and 1024x768, pill alone and with badge. The gate cell runs at 1280x720 only: is 1024x768 covered by the unit test for real (not a stub that cannot fail)?
3) Other looks: does the new rule change where the pill stands in xp, tiling, construct (no header there, or another header)? Read the code path; name any look where it moves (POLS).
4) rmmr2-3: chat_drawer calls learn_rooms on close; no sleep; a width changed while open counts at close.
5) d8eb907c "holds not sleeps": no new sleep lines (`tools/sleep-ratchet`), the holds wait on a state, not a time.
6) The lane's first gate run had acts FAIL once (tiling I14, chora-tiling.launch disconnected, not reproduced): is it caused by this branch (it touches the assistant host every look loads)? Read acts.log naming if on the branch, else say foreign.
Known, not findings: rmmr1-3, rmmr1-4, rpm2-6, rpx1-10, rpx1-16, rpx1-17, rv9j-2, mc-16, mc-50, mc-59, sh-13, sh-16, sh-37..sh-45, TF-1.
A failed Done-when line of briefs/mc-minor-r3.md is INCOMPLETE, not MERGE-OK. A fix with no red test commit before it is MAJOR.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rmmr3-N, severity, file:line, text, fix in one sentence). Append to reports/mc-minor.md under "## Review r3 (mc-minor-r3)".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
