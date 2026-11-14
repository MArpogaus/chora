# flakes r3 (rc 3, rc 9)

Lane: Fable, cloud, branch claude/flakes (at 55c527d4; train 5 merges it into the session branch;
merge the session branch claude/autonomous-dev-session-3w-i33pg7 in when it moves). One round.
Stop at 250 k context and hand over. Read dev-rules.md and conventions.md (with the skills it names)
first. Setup as briefs/flakes-r2.md. Read reports/flakes-r2.md and reports/review-flakes-r2.md.
Items, red first, in this order:
1. R1 MAJOR: a refresh during a node grab sends pointer.moved with the handling time; keep the last
   input time in State for refreshes (or send no move when the position did not change). Red: a
   test where a transaction refresh lands between the last move and the release.
2. R3: time 0 means unknown, the cube falls back to its clock. R2: document the u32 ms wrap in
   docs/api.md.
3. F15: a lost first key after a keyboard hand-over (backlog F15, reports/review-mc-r3.md F4): a
   dedicated cell types one character after focus, reads it back, logs wl_keyboard enter and key
   events; find the cause (core, foot or shell); then drop the leading space in gate-mc-paths.sh.
4. The xp cascade race (xp.sh step 3 "pixel_is 408 57": windows take cascade places in commit order
   but stack in creation order) and gate-construct-layout light-2560x1440@2 (Alt+Escape during the
   fly-in did not step back; a key in use mode did not reach the terminal).
5. If context remains: F7, A5, A17 (backlog), as in flakes-r1.md item 3.
Done when: tests/merge-gate.sh passes three times in a row on the branch under load (say the load);
list the logs. Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 25 lines) and as
reports/flakes-r3.md on claude/session-log.
Rules as briefs/flakes-r2.md; push after every commit.
