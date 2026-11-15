# Review: tiling-audit r1 (claude/tiling-audit, cf4f4f5d..b018c506)

Reviewer: PM (not the builder), 2026-11-15 02:27 UTC. Diff 2 files, +71 -10.

- Red first: cf4f4f5d adds `a_reshape_during_the_focus_fade_ends_it_at_the_mark`
  (last change of tg-s1/tf-s1 must carry opacity 0.0); b018c506 makes it pass.
- Fix: `dress::fade` takes `restyled`; a restyle (frame, radius, border, mark) sends the
  mark's opacity again, animated only when the mark changed. This ends a fade that the
  border and shadow updates froze (the scene ends an animation on any update).
- Side effect: a restyle of an unmarked tile resends opacity 0 at once; harmless.
- Evidence: gate-tiling-audit 44 of 44 cells (ink, paper, 1x, 2x); full merge gate passed.

Earliest gate (routines): a unit test on `changes()` sequences (focus move, then reshape)
would have caught it; the audit at 2x found it. Harness item: a property test over
placement sequences that asserts every mark node ends at its mark's opacity (backlog R6).

Verdict: MERGE-OK (no BLOCK, no MAJOR open). Train 11.
