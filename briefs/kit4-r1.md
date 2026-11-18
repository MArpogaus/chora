# kit4 r1 (F2): kit::ask the only question model in mc and construct

Model: sonnet, effort high
Lane: cloud, branch claude/kit4 from the session branch (bdf84f41, train 51: kit2 in). One round,
stop at 250 k. Cost about 1.5 %. Run `git fetch --unshallow` first.
Read reports/arch-plan-fable.md 1.4, 2.2, slice row KIT4, and ARC section 2 (reports/architecture-rc.md)
on claude/session-log; design/grants.md 112-120 in the repo; findings sh-26, sh-27 (findings ledger).
1. chora-kit `ask` is the only question model: mc and construct ask through it; their own question
   code goes (tools/copies.toml rows for it at 0). The answer `always` is reserved in the type (no look
   offers it yet; grants stay in the core).
2. Security cells red first (sh-26, sh-27): a prompt with bidi or control characters shows sanitized;
   a press inside 400 ms of the card showing is refused; the buttons follow the session language
   (kit words, no literal strings).
3. Visible behaviour otherwise unchanged (door acts, NET2 acts; known-red only shrinks). No change to
   tiling or xp beyond compile fixes. No new feature.
Done when: cells red then green; kit `ask` coverage >= 80 % measured with
`cargo llvm-cov -p chora-kit --features testing` if the tool is there (else name it a blocker, do not
install); merge gate once on the head (poll it in the same turn). An opus security review follows
(PM), not by you. Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/kit4.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
