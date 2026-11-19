# kit7 r1 (F2): chora-assistant the only helper presenter

Model: opus, effort high (because a design pass over four looks' helper doors and one state machine; I8 holds in all)
Lane: cloud, branch claude/kit7 from 4548feb3 (session head). First `git fetch --unshallow`; when the session
branch moves (trains 61, 62), `git merge origin/claude/autonomous-dev-session-3w-i33pg7`. One round, stop at 250 k. Cost about 5 %.
Read reports/arch-plan-fable.md row KIT7, its row 5 and C5 (and rows 9, 11), the I8 line; reports/kit5.md and
reports/kit6.md for the kit pattern, on claude/session-log.
1. Design first (in the report, as a draft decision record, at most 30 lines): one helper state machine
   (idle, listening, working, answered, no helper) in chora-kit or chora-assistant, one presenter in
   chora-assistant; each look gives only its costume (place, frame, words). Smallest design that holds (KISS).
2. Port the xp and tiling helper doors onto it; mc and construct keep working through it (no second presenter
   left: grep names each old one gone).
3. Cells red first, own commit: one state machine unit test (every state and the honest `no helper` state;
   never `working` after the answer event); I8 in every look (tests/acts): one helper surface per output.
4. No new look or feature; a visible change of a look is a finding, not a fix (F7 baseline tour v8).
Done when: red cells fail at the red commit, green after; `cargo test -p chora-kit --features testing`,
`-p chora-xp`, `-p chora-tiling`, `-p chora-mc`, `-p chora-construct`, the assistant crate green;
`python3 -m pytest tests/test_acts.py -q` green; `cargo build --workspace --locked` then
`tests/merge-gate.sh --only acts,xp,hud,chat-keys,construct-hud,mc-handover,keys` green (poll in the same turn).
No full gate (the train runs it). Push claude/kit7.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines: head sha,
red sha, design in one line, each cell) and write reports/kit7.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
