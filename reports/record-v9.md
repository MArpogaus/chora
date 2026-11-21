
## r1 (BLOCKED after step 1)

Head 892d035c. Step 0 green (unshallow, cargo check and build, foot ffmpeg waybar swaybg dbus-daemon, PIL).

Step 1, `python3 demo/tour/run.py --check` (22m45s): **90 red, 0 broken**.
- xp 31 acts: 1 known, 30 RED
- tiling 28: 5 green, 23 RED
- mc 22: 14 green, 8 RED
- construct 18: 18 RED
- handover 11: 11 RED
- Reds by invariant: I6 1793 lines, I2 90 (one per red act), I10 2.
- I6: 11 px text (xp icons, pager, tray clock) against the 14 px rule; contrast under 4.5:1 (xp pager 2.57 to 3.77, tray 3.21); mc right-door labels 1.00:1 (28,28,28 on 28,28,28, mc acts 6 and 7).
- I2: "construct-mode-step-in and construct-mode-turn-left are 9 apart, no whole number of 8" (construct acts 4 to 18, every handover act via its construct screen).
- I10: mc act 2 shows 'AGENTS' (mc-s-agents-chapter-title, mc-r1-header-head), a word of the builders.
- Full log: /tmp/claude-0/check-892d035c.log (container only, not kept).
These reds look like the new section-7 checks (guide-checks lane) finding real, unfixed breaches, not harness faults. Not expected as 90/90; the PM should decide the known-red rows or the fix round before a take.

Step 2, baseline replay: **failed**, two blockers, no media committed.
1. The v9 harness cannot run on 733967aa: `scene.boxes: no method` (every act "no snapshot") and `choractl call notify state.get` is no exported method, so chapter xp BROKE at `S.dismiss_notices()` (demo/tour/tour.py:101). Needs a harness fallback for the old build (a change to demo/tour, outside this lane's rule).
2. Even on the candidate, `demo/tour/tour.py` writes only stills and rows; it writes no `<act>.<capture>.json` (nodes, masks) and no `<act>.<capture>.png` named per capture, which `tests/tour-diff.py` reads. tour-diff has no input from run.py (gap between lanes 1 and 2).
Steps 3 to 5 not run (a take without a diffable baseline gives no verdicts). Counts same/change/regression/none: n/a. Video check: not run. Worktree 733967aa removed.

Needed before r2: (a) tour.py to dump per-capture png + json (scene.query nodes, scene.boxes masks) and tolerate a missing `scene.boxes`/`state.get`; (b) a decision on the 90 reds.
