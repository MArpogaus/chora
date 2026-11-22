
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

## r1 (record-v9, F7)

Candidate 6b01c976 (claude/record-v9; intended.toml row d7a80ff8). Baseline: 733967aa worktree, replay with `TOUR_BASELINE=1 TOUR_REPO=<worktree>` (without TOUR_BASELINE every act reads RED "no scene.boxes"; first replay discarded). Its target/ deleted.

Commands: `python3 demo/tour/run.py --check`; baseline `TOUR_BASELINE=1 TOUR_REPO=/tmp/claude-0/base TOUR_OUT=/tmp/claude-0/v9-base python3 demo/tour/run.py`; take `TOUR_OUT=/tmp/claude-0/v9-cand python3 demo/tour/run.py`; `python3 tests/tour-diff.py v9-base/captures v9-cand/captures --intended demo/tour/intended.toml --json` (folders are the `captures` subfolders); `tests/tour-video-check.py` per chapter MP4.

**Media could not be committed on claude/record-v9**: the pre-commit hook `no-session-files` (media/ forbidden in the repo) and `check-added-large-files` (500 KB) refuse it; no --no-verify. Media (23 MB) is committed on claude/session-log at media/2026-11-22/tour-v9/ instead. intended.toml row is on record-v9.

### --check on 6b01c976: 110 acts, 44 green, 66 RED (chapter lines)

- xp: 31 acts, 0 green, 1 known, 30 red (I6 on every act: pager buttons contrast 2.57/3.77/3.63 vs 4.5 (xp-pager-N-HEADLESS-1); act 20 contrast 4.06; act 23 Recycle Bin label 1.67; act 2 also none else)
- tiling: 28 acts, 24 green, 4 red (act 17, 19: key list I6 13 px runs and contrast 6.5 vs 7; act 21, 28: notify-card-body 11 px vs 14). The take run showed 23 green/5 red (one more red; flake or timing, not named)
- mc: 22 acts, 20 green, 2 red (act 2: I10 "AGENTS" word of the builders in mc-s-agents-chapter-title, mc-r1-header-head); known act 4/5/6 "is green: drop known"
- construct: 18 acts, 0 green, 18 red; handover: 11 acts, 0 green, 11 red (see timeline.md check column)
- tiling act 24 "known is green: drop known" (ti-29)

### Replay and take

- Replay on 733967aa: xp 31, tiling 28, mc 22 acts run, all RED by missing state.get/focus (expected, old build); construct stops at `construct.cube.list` (no such method): `baseline-none.json` names c3e.H50 c3f.Q c3g.H50 c3h.Q; handover BROKE at handover.py:190 (`id:xp-ctx-1-0` Send To row absent, xp-give 22fead94): intended.toml `baseline_none` row for h2.Q h3.Q h3b.Q h4.Q h5.H50 h6.Q h7.Q.
- Take: 5 chapters, 110 acts, 0 broken, 6 chapter MP4s + tour-all.mp4 (video 1280x720; recorded at 9-11 fps measured per segment, not 30).

### Diff verdicts per chapter (tour-diff, intended.toml has only the h2 rows)

| chapter | same | change | regression | none |
|---|---|---|---|---|
| xp | 0 | 0 | 31 | 0 |
| tiling | 0 | 0 | 28 | 0 |
| mc light | 0 | 0 | 14 | 0 |
| mc dark | 0 | 0 | 8 | 0 |
| construct | 0 | 0 | 18 | 0 |
| handover | 0 | 0 | 4 | 7 |

Total 0 same, 0 change, 103 regression, 7 none. Diff kinds over the 110 rows: rect 70, added 59, pixel 59, meta 49, missing 40, visible 32. Flipped checks: I6 on 57 acts, I2+I6 on 14 of them, I10 on 1, expect on 1.

Findings about the comparison itself (not product):
1. **Clock text is not pinned**: 91 meta diffs are clock nodes (wall clock "2:22 PM" vs "2:40 PM"); the replay and take ran 1.5 h apart. tour-diff should ignore clock meta or the take should set the stepped clock; unnamed clock diffs make every act a regression.
2. Launcher names differ by environment (xp-item-3 "UXTerm" -> "Foot", xp-item-4 "XTerm" -> "Foot Client"): in every xp act; cause unknown (PATH of the two runs or a real v9 change); not named in intended.toml.
3. Large systematic scene diffs by element: construct-hint rects 189, construct-hud nodes missing 72 (baseline only), tiling bar nodes tb/tf/tg/ts/tc rect ~72 each, mc-r0/r1 chat and doors rect 45-58, xp-pager-N-VIRTUAL visible+rect 56 each, new construct help rows (added). These look like the F2-F4 look changes; no cause row exists, so they stay regressions until the review lane names commits. No intended row was invented.

Regression rows by act: `timeline.md` (verdict, diff kinds, flipped checks per act, in media/2026-11-22/tour-v9/); full lines in /tmp/claude-0/diff.json (not kept; rerun the diff command).

### Video check (each chapter MP4)

xp 27 flags, tiling 15, mc-light 7, mc-dark 4, construct 8, handover 9: all `freeze` (still over 1.5 s outside a cut, no --cut given: slides at 0.00-2.50 and held moments); 0 black, 0 flicker. Lines in video-check/*.txt.

### Bad to good for the reviewer

Replay acts are RED for lack of state.get, so no per-act bad-to-good exists from checks. Candidate green acts: m7, m7c, m8, m8b, m9b, m10, m1, m1b, m1c, m3, m2, m2b, m4, m4b, m4c, m6, m6b, m5, m5b, m3b, t1, t1b, t1c, t1d, t2, t3, t3b, t3c, t3d, t3e, t3f, t3g, t3h, t4, t5, t5b, t6b, t6d, t8, t8b, t9, t9b, t10b. Tiling act 24 and mc acts 4-6 knowns are green: drop `known`.

Not done: no regression fixed, no chapter or product code changed; no intended.toml change rows (causes unnamed).
