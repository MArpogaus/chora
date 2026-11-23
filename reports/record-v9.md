
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

## r2 (record-v9, F7, fair comparison, RC 11)

Heads: claude/record-v9 ec45efbb (merge ecfc213c of origin/claude/autonomous-dev-session-3w-i33pg7 at 40a2ba92 into d7a80ff8; fixes 61f3b19e, rows ec45efbb). Step 0 green (unshallow, cargo check and build, foot ffmpeg waybar swaybg dbus-daemon, PIL). Replay baseline 733967aa (worktree, target/ deleted after), take on ec45efbb, same defaults as r1 (1280x720, stepped scene clock). `pgrep -fa 'chora --headless'` empty at the end.

### Harness fixes (red first, then green)

- **TOUR-CLOCK**. Cause: the product reads the system time (plugins/chora-xp/src/wm/dispatch.rs:298 `chrono::Local::now`), the xp tray node `xp-tray-<OUTPUT>` carries it as its meta text ("1:54 AM"), and `scene_diff` (tests/tour-diff.py, the `ta != tb` line) compared it, so every capture with a tray differed between two runs. The stepped clock of the tour (dual.py `clock.step`) steps the scene clock, not the time of day, so it cannot pin the text. Fix at tour-diff: the meta text of nodes whose id matches `^xp-tray-[A-Z]+-\d+$` is exempt; by node id only (`mc-clock`, `construct-door-clock` carry no meta text). Red: tests/test_tour_diff.py `test_the_tray_clock_of_two_takes_twenty_minutes_apart_is_no_diff` failed first, plus `test_the_clock_exemption_is_by_node_id_only`; green after. Meta diffs: 91 in r1 to 1 clock-free (xp-item 70 + help text 1 left, see below), 0 clock.
- **TOUR-LAUNCH**. Cause: demo/tour/launch.py:115 (`base_env`) built PATH from `os.environ['PATH']` and left `XDG_DATA_DIRS`/`XDG_CONFIG_DIRS` to the calling shell, so the xp Start menu saw other desktop files per run. Fix: `LAUNCHER_PATH`, `LAUNCHER_DATA_DIRS`, `LAUNCHER_CONFIG_DIRS` constants, set by every session. Red: tests/test_tour.py `LauncherEnv.test_the_launcher_environment_does_not_follow_the_shell` (XDG differed) first, green after. **Result: the xp-item-3/4 diff survived** ("UXTerm" to "Foot", "XTerm" to "Foot Client", 35 captures each) with one environment for both runs: it is a real v9 change, commit 29281054 (DB8: the fill leaves out raw entries only). It has an intended row.

### Counts per chapter (tour-diff, rows are captures; r1 to r2)

| chapter | same | change | regression | none |
|---|---|---|---|---|
| xp (31 acts) | 0 to 0 | 0 to 0 | 31 to 31 | 0 |
| tiling (28) | 0 to 0 | 0 to 0 | 28 to 28 | 0 |
| mc light (14) | 0 to 0 | 0 to 0 | 14 to 14 | 0 |
| mc dark (8) | 0 to 0 | 0 to 1 (m7) | 8 to 7 | 0 |
| construct (18) | 0 to 0 | 0 to 0 | 18 to 18 | 0 |
| handover (11) | 0 to 0 | 0 to 0 | 4 to 11 | 7 to 0 |

Total 0 same, 1 change, 109 regression, 0 none (r1: 0, 0, 103, 7). The 7 handover none captures (h2, h3, h3b, h4, h5, h6, h7) are now regression, not a worse take: their baseline has none by design, but I6 is red on the candidate (11 rows with flipped I6, 14 rows I2+I6), and a red check on a none row is a regression by rule. Flipped checks over the take: I6 63 rows, I2 14, expect 1.

Diff kinds, 110 captures: rect 3198 element diffs, added 1491, missing 805, visible 280, meta 71 (70 xp-item launchers, 1 xp help text), pixel 62 captures. Both takes: 5 chapters, 110 acts, 0 broken in the take; take --check-style reds: xp 30 red + 1 known, tiling 23 green 5 red, mc 22 green, construct 18 red, handover 11 red.

### Causes found (intended.toml rows, one per act and kind where every diff of that kind in the act belongs to the element; pixel only where all scene diffs of the act are covered; no "*")

| element | commit |
|---|---|
| xp-item-3/4 meta | 29281054 (DB8 raw entries) |
| xp-pager-N-VIRTUAL visible, rect | 8ca85eab pager on a face turns the hosted cube |
| xp-desktop added | 371f0fa1 (T7) |
| construct-hint(s)-* rect, added, missing | cfc805d4 (co-42), f5bdc61f (co-40, co-41) |
| construct-hud* missing (72) | 6a5cfc1d chora-assistant is the only voice presenter |
| construct-help-*, construct-start-help added | b8872054 one key list (kit::keysheet), 7c44ceb5 |
| mc-rN-right-keys-*, tiling-keys-body-* | b8872054, 6fa86c16 |
| mc-rN-right-notify missing | 675ebd55 chora-notify shows the notices |
| mc-rN-right-chat-*, doors-* | 42585079 (mc-56), a3037cb2 (I6) |
| xp-dlg-help-content-body meta | b8872054, c64752fa |
| tb/tf/tg/ts/tc-sN rect (tiling boxes) | fa3e20a6 tiles clear of the helper (judged from the t10 diff: a 20 px shift; weakest row, the reviewer should check) |

143 [[change]] rows in demo/tour/intended.toml (131 new). Result: 1 act reaches `change` (mc dark m7, only mc elements). The other acts stay regression for two reasons: (a) a **flipped check** (I6 11 px text, contrast, I2 gaps: real reds of the candidate that the replay cannot show, as its checks are empty), which no row covers on purpose; (b) diffs with no cause found. Elements with no cause (stay regression):
- tiling: the rect of the windows (`shell`, `page`, `code`, `notes` by app_id, 71 diffs) and `tiling-keys-panel`; t1 shows pixel blobs in the bar at the top left (12,8,399,32) with no scene diff: the tiling bar clock is a pixel diff of a surface that scene.query does not carry as text, so it is **not masked: TOUR-CLOCK is not closed for the tiling bar** (follow-up: mask the bar rect, or pin the bar clock).
- mc: `mc-s-agenda-panels-today-*` rect (30 per element), `mc-rN-header-head`, `mc-rN-footer-next`, `mc-regions-N`/`mc-s-agenda`/`mc-s-boil`/`mc-s-agents` visible. The agenda today rows probably follow the wall clock or date (not verified).
- construct: `construct-mode-*` rect and text (11 each), `construct-face-label-dot-N` missing, `construct-cube-mirror/face-N` visible.
- xp: `xp-item-N` rect (15), `notify-card*`, `xp-menu`/`xp-menu-frame`, `xp-dlg-help-content-c-scroll`.
- handover: `mc-rN-footer*` missing in the baseline's mc face.

### Video check (tour-video-check per chapter, `--cut 0,2.6` for the 2.5 s slide at the start of every chapter)

xp 27 flags, tiling 15, mc-light 7, mc-dark 5, construct 8, handover 9 (r1: 27, 15, 7, 4, 8, 9). All `freeze` (still over 1.5 s: held moments after the slide, 2.50 s on in each chapter), 0 black, 0 flicker. The slide is no flag now, the count did not fall, because the same held moments were flagged before (the first flag of r1 ran 0.00 to the first held moment). Lines: media/2026-11-23/tour-v9/video-check/*.txt. Media 19 MB: six chapter MP4s, tour-all.mp4, 106 stills as JPEG (quality 82, PNG did not fit 30 MB), timeline.md (one row per act: verdict, unnamed kinds, flipped checks, then the harness rows), tour-diff.json (the full diff lines).

Media is on claude/session-log, not claude/record-v9: the `no-session-files` pre-commit hook refuses media/ (tried, refused, no --no-verify), as in r1.

### Regression rows by act (unnamed diff kinds, flipped checks, first uncovered diff line)

| chapter | act | unnamed | flipped | first diff line |
|---|---|---|---|---|
| construct | c1 | - | I6 | - |
| construct | c2 | pixel rect | I2 I6 | rect construct-focus HEADLESS-1 [585.3310000306703, 106.79326668349354, 692.6124499146727, |
| construct | c2b | pixel rect | I2 I6 | rect construct-mode-step-in HEADLESS-1 [571.0, 652.0, 129.0, 32.0] -> [485.0, 652.0, 129.0 |
| construct | c3 | rect | I2 I6 | rect construct-door-clock HEADLESS-1 [52.0, 36.0, 37.0, 32.0] -> [52.0, 36.0, 40.0, 32.0] |
| construct | c3b | pixel rect | I2 I6 | rect construct-door-clock HEADLESS-1 [52.0, 36.0, 37.0, 32.0] -> [52.0, 36.0, 40.0, 32.0] |
| construct | c3c | pixel rect | I2 I6 | rect construct-door-clock HEADLESS-1 [52.0, 36.0, 37.0, 32.0] -> [52.0, 36.0, 40.0, 32.0] |
| construct | c3d | pixel rect | I2 I6 | rect construct-door-clock HEADLESS-1 [52.0, 36.0, 37.0, 32.0] -> [52.0, 36.0, 40.0, 32.0] |
| construct | c3e | - | I2 I6 | - |
| construct | c3f | - | I2 I6 | - |
| construct | c3g | - | I2 I6 | - |
| construct | c3h | - | I2 I6 | - |
| construct | c4 | missing pixel rect visible | I2 I6 | missing construct-door-clock in the baseline only |
| construct | c4b | pixel rect | I2 I6 | rect construct-mode-step-in HEADLESS-1 [571.0, 652.0, 129.0, 32.0] -> [485.0, 652.0, 129.0 |
| construct | c5 | - | I6 | - |
| construct | c5b | - | I6 | - |
| construct | c5c | rect | I2 I6 | rect construct-door-clock HEADLESS-1 [52.0, 36.0, 37.0, 32.0] -> [52.0, 36.0, 40.0, 32.0] |
| construct | c5d | pixel rect | I2 I6 | rect construct-door-clock HEADLESS-1 [52.0, 36.0, 37.0, 32.0] -> [52.0, 36.0, 40.0, 32.0] |
| construct | c5e | missing pixel rect visible | I6 | rect construct-cube-face-0 HEADLESS-1 [416.3970041360827, 229.3871119422142, 375.487402042 |
| handover | h1 | added missing rect | I6 | rect construct-door-clock HEADLESS-1 [52.0, 36.0, 40.0, 32.0] -> [52.0, 36.0, 37.0, 32.0] |
| handover | h1b | added missing rect | I6 | missing mc-r1-footer in the baseline only |
| handover | h1c | added missing rect | I6 | missing mc-r1-footer in the baseline only |
| handover | h1d | added missing rect | I6 | missing mc-r1-footer in the baseline only |
| handover | h2 | - | I6 | - |
| handover | h3 | - | I6 | - |
| handover | h3b | - | I6 | - |
| handover | h4 | - | I6 | - |
| handover | h5 | - | I6 | - |
| handover | h6 | - | I6 | - |
| handover | h7 | - | I6 | - |
| mc-light | m1 | rect | - | rect mc-s-ingredients-panels-lead-rule HEADLESS-1 [42.0, 168.0, 1.0, 332.0] -> [94.0, 168. |
| mc-dark | m10 | pixel rect visible | - | rect mc-r1-footer-next HEADLESS-1 [750.0, 672.0, 82.0, 28.0] -> [711.0, 672.0, 121.0, 28.0 |
| mc-light | m1b | rect | - | rect mc-s-ingredients-panels-lead-rule HEADLESS-1 [42.0, 168.0, 1.0, 332.0] -> [94.0, 168. |
| mc-light | m1c | pixel rect | - | rect mc-s-boil-panels-lead-rule HEADLESS-1 [42.0, 168.0, 1.0, 332.0] -> [94.0, 168.0, 1.0, |
| mc-light | m2 | added pixel rect visible | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 111.0, 28.0] -> None |
| mc-light | m2b | added pixel rect visible | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> None |
| mc-light | m3 | added pixel rect visible | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 63.0, 28.0] -> None |
| mc-light | m3b | added missing pixel rect visible | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> [340.5, 672.0, 158.0, 28.0 |
| mc-light | m4 | rect visible | - | rect mc-r0-footer-hud HEADLESS-1 None -> [337.5, 672.0, 158.0, 28.0] |
| mc-light | m4b | rect visible | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> None |
| mc-light | m4c | pixel rect visible | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> [340.5, 672.0, 158.0, 28.0 |
| mc-light | m5 | pixel rect visible | - | rect mc-r1-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> [340.5, 672.0, 158.0, 28.0 |
| mc-light | m5b | pixel rect visible | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> [340.5, 672.0, 158.0, 28.0 |
| mc-light | m6 | pixel rect visible | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> None |
| mc-light | m6b | pixel rect visible | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> [340.5, 672.0, 158.0, 28.0 |
| mc-dark | m7b | rect | - | rect mc-r1-header-head HEADLESS-1 [160.0, 28.0, 101.0, 28.0] -> [160.0, 28.0, 109.0, 28.0] |
| mc-dark | m7c | pixel rect visible | - | rect mc-r0-footer-next HEADLESS-1 None -> [711.0, 672.0, 121.0, 28.0] |
| mc-dark | m8 | pixel rect visible | - | rect mc-r0-footer-next HEADLESS-1 None -> [711.0, 672.0, 121.0, 28.0] |
| mc-dark | m8b | pixel rect visible | - | rect mc-r0-footer-next HEADLESS-1 None -> [711.0, 672.0, 121.0, 28.0] |
| mc-dark | m9 | added pixel rect visible | - | rect mc-r0-footer-next HEADLESS-1 [692.0, 672.0, 140.0, 28.0] -> [711.0, 672.0, 121.0, 28. |
| mc-dark | m9b | pixel rect visible | - | rect mc-r0-footer-next HEADLESS-1 [692.0, 672.0, 140.0, 28.0] -> None |
| tiling | t1 | pixel | - | pixel Q fraction 0.0136 blob [12, 8, 399, 32] |
| tiling | t10 | pixel rect | expect | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t10b | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t10c | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t1b | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 54.0, 1252.0, 652.0] -> [14.0, 94.0, 1252.0, 612.0] |
| tiling | t1c | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 54.0, 1252.0, 652.0] -> [14.0, 94.0, 1252.0, 612.0] |
| tiling | t1d | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 54.0, 1252.0, 652.0] -> [14.0, 94.0, 1252.0, 612.0] |
| tiling | t2 | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 54.0, 1252.0, 652.0] -> [14.0, 94.0, 1252.0, 612.0] |
| tiling | t3 | pixel rect | - | rect shell:None HEADLESS-1 [647.0, 54.0, 619.0, 652.0] -> [647.0, 94.0, 619.0, 612.0] |
| tiling | t3b | pixel rect | - | rect shell:None HEADLESS-1 [647.0, 387.0, 619.0, 319.0] -> [647.0, 407.0, 619.0, 299.0] |
| tiling | t3c | pixel rect | - | rect shell:None HEADLESS-1 [647.0, 498.0, 619.0, 208.0] -> [647.0, 511.0, 619.0, 195.0] |
| tiling | t3d | pixel rect | - | rect shell:None HEADLESS-1 [647.0, 498.0, 619.0, 208.0] -> [647.0, 511.0, 619.0, 195.0] |
| tiling | t3e | pixel rect | - | rect shell:None HEADLESS-1 [774.0, 498.0, 492.0, 208.0] -> [774.0, 511.0, 492.0, 195.0] |
| tiling | t3f | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 498.0, 492.0, 208.0] -> [14.0, 511.0, 492.0, 195.0] |
| tiling | t3g | pixel rect | - | rect shell:None HEADLESS-1 [858.0, 454.0, 408.0, 252.0] -> [858.0, 470.0, 408.0, 236.0] |
| tiling | t3h | pixel rect | - | rect shell:None HEADLESS-1 [647.0, 387.0, 619.0, 319.0] -> [647.0, 407.0, 619.0, 299.0] |
| tiling | t4 | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t5 | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t5b | rect | - | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t6 | pixel rect | I6 | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t6b | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t6c | pixel rect | I6 | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t6d | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t7 | added pixel rect | I6 | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t8 | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t8b | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| tiling | t9 | pixel rect | - | rect hud-chat HEADLESS-1 [868.0, 52.0, 400.0, 656.0] -> [868.0, 92.0, 400.0, 616.0] |
| tiling | t9b | pixel rect | - | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0, 407.0, 1252.0, 299.0] |
| xp | x1 | - | I6 | - |
| xp | x10 | - | I6 | - |
| xp | x10b | added | I6 | added xp-dlg-help-content-c-scroll in the candidate only |
| xp | x10c | - | I6 | - |
| xp | x11 | - | I6 | - |
| xp | x11b | - | I6 | - |
| xp | x12 | - | I6 | - |
| xp | x12b | - | I6 | - |
| xp | x12c | rect visible | I6 | rect xp-item-0 HEADLESS-1 [4.0, 419.0, 186.0, 42.0] -> None |
| xp | x2 | pixel rect | I6 | rect notify-card-body HEADLESS-1 [935.0, 628.0, 257.0, 70.0] -> [935.0, 615.0, 257.0, 83.0 |
| xp | x2b | pixel rect | I6 | rect notify-card-body HEADLESS-1 [935.0, 628.0, 257.0, 70.0] -> [935.0, 615.0, 257.0, 83.0 |
| xp | x2c | - | I6 | - |
| xp | x3 | - | I6 | - |
| xp | x4 | - | I6 | - |
| xp | x5 | - | I6 | - |
| xp | x5b | - | I6 | - |
| xp | x5c | - | I6 | - |
| xp | x5d | - | I6 | - |
| xp | x5e | - | I6 | - |
| xp | x5f | - | I6 | - |
| xp | x5g | - | I6 | - |
| xp | x6 | - | I6 | - |
| xp | x6b | - | I6 | - |
| xp | x6c | - | I6 | - |
| xp | x6d | - | I6 | - |
| xp | x6e | - | I6 | - |
| xp | x7 | - | I6 | - |
| xp | x7b | - | I6 | - |
| xp | x8 | - | I6 | - |
| xp | x9 | - | I6 | - |
| xp | x9b | - | I6 | - |

### Bad to good, for the reviewer

Nothing was fixed in the product. The comparison is now fair on launchers (same environment, one real diff named) and the tray clock; the remaining regression is dominated by the red checks of the candidate and by the unattributed element diffs above. Candidate acts with every check green (their regression is diff only, no flipped check): t1 t1b t1c t1d t2 t3 t3b t3c t3d t3e t3f t3g t3h t4 t5 t5b t6b t6d t8 t8b t9 t9b t10b m1 m1b m1c m3 m2 m2b m4 m4b m4c m6 m6b m5 m5b m3b m7 m7b m7c m8 m8b m9 m9b m10. Recommended next: fix I6 (11 px text) and I2 (gaps of 9 against 8 in construct) so a flipped check no longer decides, mask or pin the tiling bar clock, and have the review lane judge the unattributed rows by eye on the stills.

## Review r2 (PM; code 55 lines in 61f3b19e, data 701 lines in ec45efbb; base ecfc213c)

- TOUR-CLOCK: tests/tour-diff.py exempts meta text only for ids matching `^xp-tray-[A-Z]+-\d+$` (by node id, rd-9 holds); tests test_the_tray_clock_... and test_the_clock_exemption_is_by_node_id_only. Met for the xp tray.
- TOUR-LAUNCH: demo/tour/launch.py base_env sets PATH, XDG_DATA_DIRS, XDG_CONFIG_DIRS from constants; test LauncherEnv. Met; xp-item-3/4 traced to 29281054 (intended row).
- Red first: tests sit in the fix commit, not a commit before it. tests/red-first.sh accepts a test "before it or in it"; the red runs are claimed in the report, not provable from history. Nit, no finding (gate rule holds).
- Done-when: replay, take, diff, video check on the merged head; 110 verdict rows; every regression listed with its first diff line; counts r1 vs r2. Met.
- Open for r3 (not findings of this branch): RV9-1 the tiling bar clock is an unmasked pixel diff (t1, rect 12,8,399,32); RV9-2 the fa3e20a6 rows (tiling boxes) rest on one 20 px diff, verify on a second act; RV9-3 no cause for tiling window rects, mc agenda today rows, construct-mode-*, xp-item rects; RV9-4 the flipped checks (I6 63 rows, I2 14) need the element and check line per row, the take keeps none.
- Verdict (computed, 0 MAJOR): MERGE-OK ec45efbb (tour tools and intended rows only). Joins the next train.
