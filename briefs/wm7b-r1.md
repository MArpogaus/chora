# wm7b r1 (F3): the xp cube hosted on construct (owner 11-19 10:52: "Hosted")

Model: opus, effort high, because the cube hand-over spans two looks, the focus owner (I3) and crash recovery (I1)
Lane: cloud, branch claude/wm7b from claude/wm7a f4e4f4d9 (WM7a r3, MERGE-OK, in train 62). One round, stop at 250 k.
Cost about 4 %. Run `git fetch --unshallow` first. Read reports/wm7a.md (sections 3, 4, review r1, r2) and
reports/arch-plan-fable.md row WM7b on claude/session-log. KISS, POLS: no new feature, no visible change outside the cube.
Do, each fix with its red test commit first (rules.md 8):
1. (a) construct alone holds the mouse-turn binding on faces; xp as face manager drops its own turn binding while hosted.
   xp's turn code goes or moves to kit: wm/desktops.rs `turn_to` uses `chora_kit::cube::quarters_to` (no copy left).
2. (b) wm7a-a1: one focus owner when the cube opens (I3); wm7a-a2: the pager on a face calls `cube.turn {to}`.
   Both leave tests/acts/known-red.toml. Fix rw-5 with it (step-back act starts from `cube.turn {to: 0}`).
3. (c) wm7a-m2 (I1): construct killed, no restart: within 3 s the windows go back to a flat xp, none ownerless;
   restarted: the cube reopens and the windows go back to the faces' xp, not construct panes. wm7a-m1: the
   restarted start card does not say "Nothing is open yet" while it holds a window.
4. wm7a-m3 (rw-8): `construct.cube` sends a last event on close or manager end (turning false).
   wm7a-m4: `MANAGER` reads the `cube_manager` setting (cube.rs:1152, 1251), or the setting goes; KISS.
5. Flat fallback: without construct xp switches desks flat; its key sheet says so. Act for it.
Done when: acts in the hosted form AND without construct (arch-plan row: "the WM7a acts in the chosen form and
without construct") pass; known-red has no wm7a id; `cargo test -p chora-construct -p chora-xp -p chora-kit
--features chora-kit/testing`; kit coverage stays >= 80 %; size ratchet holds (construct cube.rs: split start
into cube/start.rs if it grows); `tests/merge-gate.sh --only build,acts,cube,pull,construct-hud,xp,xp-faces,keys`
green (poll in the same turn). No full gate (the train runs it). docs/api.md and a decision record line for hosted.
Push claude/wm7b. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 8 lines:
head sha, each item with its cell) and write reports/wm7b.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
