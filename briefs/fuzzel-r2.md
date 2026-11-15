# fuzzel r2 (rc 1): ti-1 BLOCK, a click in the tiling Apps list runs the first row

Model: sonnet, effort high (cause unknown; a red cell and a bisection between compositor and fuzzel)
Lane: cloud, new branch claude/fuzzel-click from the session branch
claude/autonomous-dev-session-3w-i33pg7 (1014dddf or newer). One round. Stop at 250 k context.
Read dev-rules.md and conventions.md (with the skills it names) first, then reports/fuzzel-r1.md,
findings/tiling.md ti-1. setup.sh on claude/session-log builds fuzzel 1.11.1 under $HOME/.local.

Facts from r1: pointer enter, motion (on the Zebra row) and button reach fuzzel 1.11.1; it runs
the first row. The PM read crates/chora/src/input.rs: move_pointer and press_button call
pointer.frame (lines ~643, ~727), so "no frame" is unlikely; verify, do not assume.

Items, in order:
1. Red cell: tests/gate-tiling-apps-click.sh (from the scratch cell of r1): open Apps, pointer to
   a row that is not the first, click, expect that row's app_id. Commit red.
2. Cause: a full WAYLAND_DEBUG=1 of fuzzel (frame events, timing between motion and button,
   the surface scale and the position fuzzel gets against its row layout). Read fuzzel 1.11.1's
   pointer code (wl_pointer listeners, how hover selects a row). Try: a wait of one frame between
   move and click; scale 1 against 2. Name the cause with the line that shows it.
3. Fix in chora if the cause is chora's (KISS; no wait in the cell to hide it). If the cause is
   fuzzel's, hand back with the evidence and a proposal (for example key selection on hover by a
   chora-kit launcher); no workaround in this round.
Done when: the cell passes 3/3 and tests/merge-gate.sh passes once, or the cause is proven as
fuzzel's with evidence. Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines) and
as reports/fuzzel-r2.md on claude/session-log, with the earliest gate per item.
Rules as briefs/flakes-r2.md, with push only claude/fuzzel-click and that report; never end a
turn to wait for a long background job. Lanes do not install system packages beyond setup.sh.
