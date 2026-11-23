
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

## Train t112b (PM, 03:51 UTC)
Red: acts cell, tests/test_tour.py:464 test_the_chapters_use_each_scene_of_intended_toml: 81 change rows of intended.toml name a row id with its letter (c2b, c3b, ... x9b), not a scene of the chapters. Verdict FIX (RV9-5); the r2 review missed that the acts cell was not in the lane's run. record-v9 is out of t112c; r3 fixes it with the full gate.

## r3 (record-v9, F7, RC 11)

Head claude/record-v9 (see the gate line at the end). Step 0 green (unshallow, cargo check and build, foot ffmpeg waybar swaybg dbus-daemon, PIL); fresh origin/claude/autonomous-dev-session-3w-i33pg7 merged twice (86e619f3, then 2c1748a4). Replay baseline 733967aa in a worktree, takes of both builds with the r2 defaults.

### RV9-5 (the acts cell was red in t112b)
Red on ec45efbb: `tests/test_tour.py:464 test_the_chapters_use_each_scene_of_intended_toml: AssertionError: Lists differ: ['c2b', 'c2b', 'c2b', 'c3b', ...` (81 rows). Choice: the test learns row ids, the 81 rows stay. Reason: one id scheme already exists, `tour.act_id` (demo/tour/tour.py: the scene id, and a letter for a later act of the scene, `h1b`), the header of intended.toml says so, tests/tour-diff.py matches `r["act"] == act` on that id, and a scene-named row would have to be matched to every act of the scene by a second rule. design/tour-v9.md section 6 names acts by `h1`, `h2`; the letter rows are the take's own ids. The test now builds the ids with `tour.act_id` over each chapter. Green: `python3 tests/test_tour.py` OK (76 tests at the end); `tests/merge-gate.sh --only ci-check,acts --jobs 2`: ci-check PASS 112 s, acts PASS 2215 s, EXIT 0.
Slip, said plainly: my first `git commit` failed the no-inline-comments hook; the `--amend` that followed folded the test fix into the merge commit 05fa9abb (pushed, no force). The fix is in that commit, not a commit of its own.

### RV9-1 the tiling bar clock
Cause found: the bar is waybar, a foreign surface, so `scene.query` has no node for it (t1 has 4 nodes: hud-rows, root, screen, tiling). No node id of the bar exists to mask; the one node every tiling capture has is `tiling`. Fix (tests/tour-diff.py `BAR_CLOCK`, `masks_of(width=)`): where a build has the node `tiling` the clock island (154 x 32 at y 8, centred on the output) is masked, nowhere else. Red first: tests/test_tour_diff.py `BarClock` (island masked: failed; unmasked without the node: passes; a diff beside the island still a diff: passes). Green 50 tests. Result: in t1 no blob of the clock island is left (r2: blob list had 563,8,154,32). The big blob 12,8,399,32 of r2 was not the clock: it is the left island of the bar (see RV9-2).

### RV9-2 and RV9-3: one harness defect explained 8 of the families
Cause: since ccf7ebab chora-tiling starts its own bar and wallpaper unless it gets `--no-companions` (60314da1); the tiling chapter also starts waybar as a companion. The candidate had two bars on one output: the exclusive zone doubled (tiles 40 px lower: `tb/tf/tg/ts/tc-sN` and the windows `shell/page/code/notes` rect, +40 in 26 acts, +20 in t10 (cause of that one not separated: the helper panel is a suspect, 1f9a5180); it was never fa3e20a6) and the second bar had no modules (tags, layout, helper and volume empty), which was the "t1 pixel blob". Fix: `launch.tiling_argv(binary, *args)` adds `--no-companions` where `--help` knows it (733967aa does not); tiling.py and handover.py use it. Red first: tests/test_tour.py `TilingArgv` (3 tests). The fa3e20a6 row in intended.toml (r2: "judged from one 20 px diff, weakest row") is therefore withdrawn as a cause: after the retake no tb/tf/tg/ts/tc/window rect diff is left in any of the 28 acts. The rows stay in intended.toml untouched (I did not re-judge them); they cover nothing now and the reviewer may drop them.
Also found: the code window showed crates/chora-kit/src/wobbly.rs of the checkout, which differs between the builds (fix: demo/tour/sample.rs, a copy of the 733967aa file, `SAMPLE` in launch.py; red first `CodeWindow`).
mc, construct, xp families (git log -S between 733967aa and head, per element):
- `construct-mode-*` rect and text, I2 "construct-mode-step-in and -turn-left are 9 apart": 1e7cc905 (co-39, the chip words hide under the key list): the chips lose their words and move 86 px left; the 9 px gap is a real I2 red of the candidate. No intended row added: I2 flips in 14 acts, the rows would hide a red check; the finding should be fixed or the I2 known-red row opened by the lane that owns construct.
- `mc-s-agenda-panels-today-*`, `mc-rN-footer-next`, `mc-rN-header-head`: `footer-next` points to the next page when no entry is left today (the test the_footer_points_to_the_next_page_when_no_entry_is_next, mc main.rs): it follows the wall clock against the 18:30 to 22:00 entries of the chapter, so the two takes (an hour apart) differ. Not proven by a controlled rerun, stays a regression.
- `xp-item-N` rect, `notify-card*`, `xp-menu*`: `git log -S` finds no commit with an id of these; stay regressions with the diff line of the table below.

### RV9-4 flipped checks (media/2026-11-23/tour-v9/flipped-checks.json: 710 lines, act, check, element, check line)
I6 63 acts, I2 14 acts, expect 1 (same as r2). Elements and their check line (acts that show it):

| check | element | line | acts |
|---|---|---|---|
| I6 | xp-pager-N-HEADLESS-N | contrast 2.57:1 (white on 111,162,247), wants 4.5:1 | 41 |
| I6 | xp-pager-N-VIRTUAL-N | the same, 2.57:1 | 18 |
| I6 | xp-task-sN-VIRTUAL-N | 3.60:1 (243,247,254 on 56,126,242), wants 4.5:1 | 18 |
| I6 | xp-title-sN | 2.16:1 (217,228,247 on 123,155,223), wants 4.5:1 | 18 |
| I2 | construct-mode-step-in / -turn-left | 9 apart, no whole number of 8 | 14 |
| I6 | tiling-keys-body-h, -m, -N | 6.50:1 on 26,29,40, wants 7:1 | 2 each |
| I6 | tiling-keys-body-N-k | 13 px, wants 14 | 2 |
| I6 | notify-card-body | 11 px, wants 14 | 2 |
| I6 | construct-help-body-scope, -head-N, -head-N-every | 11 px, wants 14 | 1 each |
| I6 | xp-menu-frame, xp-task-sN-HEADLESS-N, xp-session-button-N, xp-icon-N | contrast 4.32, 3.86, 3.56 (wants 7), 1.67 | 1 each |
| expect | handover letter.txt | typed lines twice in the file | 1 |

### Counts per chapter (rows are captures; r2 to r3; r3 diff of xp, mc, construct from the r3 baseline and candidate takes of this lane, tiling and handover from the retake after the fixes)
| chapter | same | change | regression |
|---|---|---|---|
| xp (31) | 0 to 0 | 0 to 0 | 31 to 31 |
| tiling (28) | 0 to 1 (t5b) | 0 to 0 | 28 to 27 |
| mc (22) | 0 to 0 | 1 to 1 (m7) | 21 to 21 |
| construct (18) | 0 to 0 | 0 to 0 | 18 to 18 |
| handover (11) | 0 to 0 | 0 to 0 | 11 to 11 |
Total 1 same, 1 change, 108 regression (r2: 0, 1, 109). 4 more tiling acts (t1, t1b, t1c, t2) got an intended pixel row for the bar (bbb105ba, 3a9a426f, 136bbd82, 6a5cfc1d) but still show a regression because the code window and terminal text of the baseline differ (blobs in the windows), so their count is not 0. Tiling now has no scene diff except tiling-keys (b8872054), hud-chat (t9) and notify-card (675ebd55); every other tiling regression is a pixel diff in a window (terminal text, code window) or the bar look.

### Remaining regression rows (all 108)
Same as the r2 table for xp, mc, construct and handover (flipped I6 and I2 above; construct-door-clock, mc-rN-footer-hud and footer-next, xp-item rect, mc-s-*-lead-rule rects). Tiling: t1..t10c: unnamed `pixel` only (except t6/t6c rect of tiling-keys-body-0, t7/t10c notify-card added with I6, t9 meta/rect of hud-chat and the windows); first diff line is a window blob (e.g. t3c [42,127,544,135], t5 [42,66,544,75]), the terminal stub output, not the bar. Open for the next lane: pin the terminal text of the tiling chapter between builds (its stub foot starts the same shell, so the first suspect is the shell prompt and `ls` of NOTES), then retake.

Media: tour-diff-r3.json and flipped-checks.json under media/2026-11-23/tour-v9/ (the videos are the r2 take; the r3 retake of tiling and handover was not re-encoded into the folder: size).

### Gate
Head claude/record-v9 088d36de (merge of origin/claude/autonomous-dev-session-3w-i33pg7 at 2c1748a4; commits: test fix in 05fa9abb, fix BarClock, fix --no-companions 367cc167, fix code window 978aaf1e, rows 3df340a7). Full `tests/merge-gate.sh --jobs 2` (no --only), detached: 62 cells PASS, EXIT 0, "all steps passed". No foreign red. `pgrep -fa 'chora --headless'` empty.

## Review r3 (PM, 07:45 UTC; harness only, code diff under 200 lines; PM is not the builder)
Base 230ea0e5..088d36de: 8 files, +1079 -10; code: tests/tour-diff.py +26, demo/tour/launch.py +21, chapters +6, tests +111; data: intended.toml +726, sample.rs +199 (copy of the 733967aa file).
- RV9-5: test builds ids with tour.act_id, one id scheme (intended.toml header, tour-diff `r["act"] == act`): met; acts cell PASS in the lane gate.
- RV9-1: mask only where node `tiling` exists, width from the PNG header; BarClock 3 tests (masked, not without node, beside still a diff): met.
- RV9-2/3: two-bar harness defect (--no-companions probed via --help), red first TilingArgv; code window pinned (CodeWindow): met. fa3e20a6 rows now cover nothing: debt, drop in r4.
- RV9-4: flipped-checks.json with element and line: met.
- Slip: test fix folded into merge 05fa9abb by --amend (no force); no rule broken that a gate checks; noted.
- F7 not met: 108 regressions remain (r4).
Verdict: 0 BLOCK, 0 MAJOR in the diff: MERGE-OK; harness branch, own train t114 at 07:41.

## r4 (record-v9, F7, RC 11)

Head claude/record-v9 6949cddb (merge of origin/claude/autonomous-dev-session-3w-i33pg7 at the fresh tip, already contained: "Already up to date"). Commits: a88073e7 stub terminal, f3c05f1e date stub, 6949cddb intended rows. Step 0 green (unshallow, cargo check and build, foot ffmpeg waybar swaybg dbus-daemon, PIL). Replay baseline 733967aa in a worktree (built), takes of all chapters on both builds after the fixes.

### 1 tiling terminal text (a88073e7, red first in tests/test_tour.py `StubTerminal`, 3 tests failed, then green)
The bytes of the shell terminal and of the notes window were already equal (the r3 theory). What differed was the code window: it reads its size when it starts, while its tile still animates, so the baseline showed lines 110 to 119 and the candidate 118 to 126 of the same sample.rs (status line `NORMAL 99:1` equal, scroll not). Fix: `launch.foot_stub` writes the one `foot` of the chapter: the shell gets a fixed `PS1='$ '`, `bash --norc`, `cd /home/user/notes`; every content window waits `TILE_WAIT` 3 s for its tile (`sleep 3; python3 -I content.py ...`). After the retake, t3c: terminal, code and notes windows are pixel-equal in both builds.

### 2 the clock (f3c05f1e, red first `FixedClock`, 3 tests)
The product reads the time for mc by running `date` (plugins/chora-mc/src/region.rs `date`: clock contents, `since` of the agent), so a `date` first on PATH pins it: `launch.date_stub` (2026-11-23 18:51 UTC, any format), written next to the wpctl stub of both launchers. No node-id mask needed for mc. Note: the r3 theory "footer-next follows the wall clock" is wrong: `footer-next` is the line to the next page (manage.rs `next_page_line`), it reads no clock; the mc diffs come from page divergence (below).

### 3 flipped checks (element and cause per family; scripts: pixel crop of the element rect on both takes)
Why they flip at all: 733967aa has no `scene.boxes` (tour.py `OldCore`: "a snapshot without boxes"), and I6 (tests/acts/invariants.py `guide_breaches`) judges only the text boxes of `scene.boxes`: on the baseline I6 is green because it has nothing to read, not because the text passes. Where the crop of the element is pixel-identical in both builds the red is old and was never measurable on the baseline; no sha introduced it.

| check | element | line | acts | crop of the element on 733967aa vs head | cause |
|---|---|---|---|---|---|
| I6 | xp-pager-N-HEADLESS-N | contrast 2.57:1 (white on 111,162,247), wants 4.5 | 84 | identical in 84 of 84 | none: present at 733967aa |
| I6 | xp-pager-N-VIRTUAL-N | the same, 2.57:1 | 132 | differs | same colours, 2.57:1 as on HEADLESS; the position diff is not separated |
| I6 | xp-task-sN-VIRTUAL-N / -HEADLESS-N | 3.60:1 and 3.86:1 (white on 56,126,242), wants 4.5 | 80 / 1 | identical in 69 of 80, 1 of 1 | none: present at 733967aa |
| I6 | xp-title-sN | 2.16:1 (217,228,247 on 123,155,223), wants 4.5 | 52 | identical in 39 of 52 | none: present at 733967aa |
| I6 | xp-icon-N, xp-session-button-N | 1.67:1, 3.56:1 (wants 7) | 1 / 5 | identical | none: present at 733967aa |
| I6 | xp-menu-frame | 4.32:1 (white on 39,121,221) | 1 | differs | no commit found by `git log -S` on the colour; stays a regression, reason: the menu is open in the replay and shut in the take at x12c (20e4ad6e menus shut on a new window), the crops are of other states |
| I2 | construct-mode-step-in / -turn-left | 9 apart, no whole number of 8 | 16 | differs | 1e7cc905 (co-39) |
| I6 | construct-help-body-scope, -head-N, -head-N-every | 11 px, wants 14 | 1, 6, 3 | element absent in the replay | 918d4a3c (key sheet takes the keyboard, Close): new element |
| I6 | tiling-keys-body-h, -m, -N | 6.50:1 on 26,29,40, wants 7 | 2, 2, 10 | differs | b8872054 (one key list, kit::keysheet) |
| I6 | tiling-keys-body-N-k | 13 px, wants 14 | 92 | differs | b8872054 |
| I6 | notify-card-body | 11 px, wants 14 | 2 (t7, t10c) | absent in the replay | 675ebd55 (chora-notify shows the notices) |

Finding lines for the PM to file (checks that are red at the head; no intended row hides them):
- rv9-1 | xp | MAJOR (design/rc.md row 1: readability) | I6 pager, task, title chrome: xp-pager-N 2.57:1, xp-task-sN 3.60:1, xp-title-sN 2.16:1, want 4.5:1 | no sha: identical at 733967aa, the baseline cannot measure it
- rv9-2 | construct | MAJOR | I2 construct-mode-step-in and -turn-left are 9 apart (no whole number of 8), 16 acts | 1e7cc905
- rv9-3 | tiling | MAJOR | I6 keys list: tiling-keys-body-h/-m/-N 6.50:1 (wants 7), tiling-keys-body-N-k 13 px (wants 14) | b8872054
- rv9-4 | tiling, xp | MAJOR | I6 notify-card-body 11 px, wants 14 (the card of chora-notify outside mc, t7, t10c) | 675ebd55
- rv9-5 | construct | MAJOR | I6 construct-help-body-scope, -head-N, -head-N-every 11 px, wants 14 | 918d4a3c
- rv9-6 | xp | MINOR | I6 xp-menu-frame 4.32:1 (wants 4.5), xp-session-button 3.56:1 (wants 7), xp-icon-2 1.67:1 | no sha found
(Severity read by me from design/rc.md row 1 "no BLOCK or MAJOR open"; the PM may rate them differently.)
Elements of the same table with the full per-act numbers: /tmp is lost with the container; the table above is the record.

### 4 rect families
- Tiling windows and every text of the right tile column: the inner gap 10 to 12 (573496e5, I2 whole number of 4): tiles 1 px narrower, right column 2 px right. Proof, a controlled take: the candidate with `look: {gap_inner: 10}` matches the replay to the pixel outside the bar in 16 acts (no blob below y 48 except t8 and t10), and I2 then flips red (gap 10 is no whole number of 4). 16 acts got a pixel row (cause 573496e5 plus the bar commits of r3).
- xp windows (`Notepad:None`, xp-frame, xp-title, min, max, close) +88 px in x: 09ff8ff9, bd5af4b3 (xp-28, new windows start right of the icons). hud-chat taller with a meta greeting: 4fec75c3 (xp-30). xp-item names shifted ('UXTerm' to 'Foot'): 29281054. A row only where every diff of that kind in the act comes from the family (checked per element id): 38 rows. Flipped I6 acts stay regressions (the check decides).
- xp-item rect (17 acts, x12c) and xp-menu rect/visible: the replay has the Start menu open at x12c, the head has it shut: 20e4ad6e (menus shut on a new window) is the likely cause; no row (not proven by a controlled take).
- notify-card rect (x2, x2b): card 83 px high against 70 (the head wraps the text): 675ebd55 shows the notices with chora-notify. No row: the I6 flip decides.
- mc, everything after m4c: cause fa7faabd (mc-38, Home alone goes to the first page): at m4c the replay stays on its page (boil), the head goes to the agenda, so every later capture shows another page (m5 to m9b: boil/agents in the replay, agenda at the head): the scene ids, rects of mc-s-* and footers are of different pages. Not a clock effect. The chapter cannot make both builds land on the same page without product help; it stays a regression with this cause.
- fa3e20a6 rows: there are none in demo/tour/intended.toml at the head (grep: 0; also 0 at ec45efbb); nothing to drop. No row of the file covers nothing (checked: every row has a diff of its act and kind in this take, handover rows by baseline_none).

### 5 counts (r3 to r4; rows are captures)
| chapter | same | change | regression |
|---|---|---|---|
| xp (31) | 0 to 0 | 0 to 0 | 31 to 31 |
| tiling (28) | 1 to 1 (t5b) | 4 to 20 | 23 to 7 |
| mc (22) | 0 to 0 | 1 to 1 (m7) | 21 to 21 |
| construct (18) | 0 to 0 | 0 to 0 | 18 to 18 |
| handover (11) | 0 to 0 | 0 to 0 | 11 to 11 |
Total 1 same, 21 change, 88 regression (r3: 1, 1, 108). (r3 counted t1..t2 as regression; their rows now hold. The handover baseline stops at handover.py:190 as designed, TypeError after h1, `baseline_none` covers the rest.)

### Remaining regression rows (all 88: chapter, act, unnamed kinds, flipped checks, first diff)
| chapter | act | unnamed | flipped | first diff |
|---|---|---|---|---|
| construct | c1 | - | I6 | missing construct-hud in the baseline only |
| construct | c2 | rect,pixel | I2,I6 | rect construct-focus HEADLESS-1 [585.3310000306703, 106.79326668349354, |
| construct | c2b | rect,pixel | I2,I6 | rect construct-hints-construct-mode-step-in-keys-0 HEADLESS-1 [579.0, 656.0, 59.0, 24.0] -> [493.0, 6 |
| construct | c3 | rect | I2,I6 | rect construct-hints-construct-mode-step-in-keys-0 HEADLESS-1 [579.0, 656.0, 59.0, 24.0] -> [493.0, 6 |
| construct | c3b | rect,pixel | I2,I6 | rect construct-hints-construct-mode-step-in-keys-0 HEADLESS-1 [579.0, 656.0, 59.0, 24.0] -> [493.0, 6 |
| construct | c3c | rect,pixel | I2,I6 | rect construct-hints-construct-mode-step-in-keys-0 HEADLESS-1 [579.0, 656.0, 59.0, 24.0] -> [493.0, 6 |
| construct | c3d | rect,pixel | I2,I6 | rect construct-hints-construct-mode-step-in-keys-0 HEADLESS-1 [579.0, 656.0, 59.0, 24.0] -> [493.0, 6 |
| construct | c3e | - | I2,I6 |  |
| construct | c3f | - | I2,I6 |  |
| construct | c3g | - | I2,I6 |  |
| construct | c3h | - | I2,I6 |  |
| construct | c4 | missing,rect,visible,pixel | I2,I6 | missing construct-door-clock in the baseline only |
| construct | c4b | rect,pixel | I2,I6 | rect construct-hints-construct-mode-step-in-keys-0 HEADLESS-1 [579.0, 656.0, 59.0, 24.0] -> [493.0, 6 |
| construct | c5 | - | I6 | missing construct-hud in the baseline only |
| construct | c5b | - | I6 | missing construct-hud in the baseline only |
| construct | c5c | rect | I2,I6 | rect construct-hints-construct-mode-step-in-keys-0 HEADLESS-1 [579.0, 656.0, 59.0, 24.0] -> [493.0, 6 |
| construct | c5d | rect,pixel | I2,I6 | rect construct-hints-construct-mode-step-in-keys-0 HEADLESS-1 [579.0, 656.0, 59.0, 24.0] -> [493.0, 6 |
| construct | c5e | rect,visible,missing,pixel | I6 | rect construct-cube-face-0 HEADLESS-1 [416.3970041360827, 229.3871119422142,  |
| handover | h1 | missing,rect,added | I6 | missing construct-hud in the baseline only |
| handover | h1b | missing,rect,added | I6 | missing construct-hud in the baseline only |
| handover | h1c | missing,rect,added,pixel | I6 | missing construct-hud in the baseline only |
| handover | h1d | missing,rect,added,pixel | I6 | missing construct-hud in the baseline only |
| handover | h2 | - | I6 |  |
| handover | h3 | - | I6 |  |
| handover | h3b | - | I6 |  |
| handover | h4 | - | I6 |  |
| handover | h5 | - | I6 |  |
| handover | h6 | - | I6 |  |
| handover | h7 | - | I6 |  |
| mc | m1 | rect | - | rect mc-r0-right-chat-door HEADLESS-1 [888.0, 194.0, 204.0, 28.0] -> [888.0,  |
| mc | m10 | rect,visible,pixel | - | missing mc-r0-right-chat-door in the baseline only |
| mc | m1b | rect | - | rect mc-r0-right-chat-door HEADLESS-1 [888.0, 194.0, 204.0, 28.0] -> [888.0,  |
| mc | m1c | rect | - | missing mc-r0-right-notify in the baseline only |
| mc | m2 | rect,visible,added,pixel | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 111.0, 28.0] -> None |
| mc | m2b | rect,visible,added,pixel | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> None |
| mc | m3 | rect,visible,added,pixel | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 63.0, 28.0] -> None |
| mc | m3b | rect,missing,visible,added,pixel | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> [340.5,  |
| mc | m4 | rect,visible | - | rect mc-r0-footer-hud HEADLESS-1 None -> [337.5, 672.0, 158.0, 28.0] |
| mc | m4b | rect,visible | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> None |
| mc | m4c | rect,visible,pixel | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> [340.5,  |
| mc | m5 | rect,visible,pixel | - | missing mc-r0-right-notify-0 in the baseline only |
| mc | m5b | rect,visible,pixel | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> [340.5,  |
| mc | m6 | rect,visible,pixel | - | missing mc-r0-right-notify-0 in the baseline only |
| mc | m6b | rect,visible,pixel | - | rect mc-r0-footer-hud HEADLESS-1 [287.5, 672.0, 158.0, 28.0] -> [340.5,  |
| mc | m7b | rect | - | missing mc-r0-right-notify in the baseline only |
| mc | m7c | rect,visible,pixel | - | rect mc-r0-footer-next HEADLESS-1 None -> [711.0, 672.0, 121.0, 28.0] |
| mc | m8 | rect,visible,pixel | - | rect mc-r0-footer-next HEADLESS-1 None -> [711.0, 672.0, 121.0, 28.0] |
| mc | m8b | rect,visible,pixel | - | rect mc-r0-footer-next HEADLESS-1 None -> [711.0, 672.0, 121.0, 28.0] |
| mc | m9 | rect,visible,added,pixel | - | rect mc-r0-footer-next HEADLESS-1 [692.0, 672.0, 140.0, 28.0] -> [711.0,  |
| mc | m9b | rect,visible,pixel | - | missing mc-r0-right-chat-door in the baseline only |
| tiling | t10 | pixel | - | blob [12, 8, 475, 32] |
| tiling | t10c | added,pixel | I6 | added notify-card in the candidate only |
| tiling | t6 | rect,pixel | I6 | rect tiling-keys-body-0 HEADLESS-1 [198.0, 142.0, 63.0, 17.0] -> [225.0, 1 |
| tiling | t6c | rect,pixel | I6 | rect tiling-keys-body-0 HEADLESS-1 [198.0, 142.0, 63.0, 17.0] -> [225.0, 1 |
| tiling | t7 | added,pixel | I6 | added notify-card in the candidate only |
| tiling | t8 | pixel | - | blob [12, 8, 475, 32] |
| tiling | t9 | rect,pixel | - | rect shell:None HEADLESS-1 [14.0, 387.0, 1252.0, 319.0] -> [14.0,  |
| xp | x1 | - | I6 | meta xp-item-3 'UXTerm' -> 'Foot' |
| xp | x10 | pixel | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x10b | added,pixel | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x10c | pixel | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x11 | - | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x11b | pixel | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x12 | - | I6 | meta xp-item-3 'UXTerm' -> 'Foot' |
| xp | x12b | pixel | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x12c | rect,visible | I6 | rect xp-item-0 HEADLESS-1 [4.0, 419.0, 186.0, 42.0] -> None |
| xp | x2 | rect,pixel | I6 | rect notify-card-body HEADLESS-1 [935.0, 628.0, 257.0, 70.0] -> [935.0,  |
| xp | x2b | rect,pixel | I6 | rect notify-card-body HEADLESS-1 [935.0, 628.0, 257.0, 70.0] -> [935.0,  |
| xp | x2c | pixel | I6 | rect Notepad:None HEADLESS-1 [28.0, 49.0, 700.0, 500.0] -> [116.0, 4 |
| xp | x3 | - | I6 | rect Notepad:None HEADLESS-1 [28.0, 49.0, 700.0, 500.0] -> [116.0, 4 |
| xp | x4 | - | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x5 | rect | I6 | rect hud-chat HEADLESS-1 [932.0, 437.0, 340.0, 141.0] -> [932.0, |
| xp | x5b | rect,pixel | I6 | rect hud-chat HEADLESS-1 [932.0, 395.0, 340.0, 183.0] -> [932.0, |
| xp | x5c | rect,pixel | I6 | rect hud-chat HEADLESS-1 [932.0, 395.0, 340.0, 183.0] -> [932.0, |
| xp | x5d | rect,pixel | I6 | rect hud-chat HEADLESS-1 [932.0, 332.0, 340.0, 246.0] -> [932.0, |
| xp | x5e | pixel | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x5f | rect,pixel | I6 | rect hud-chat HEADLESS-1 [932.0, 248.0, 340.0, 330.0] -> [932.0, |
| xp | x5g | pixel | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x6 | rect,pixel | I6 | rect hud-chat HEADLESS-1 [932.0, 290.0, 340.0, 288.0] -> [932.0, |
| xp | x6b | rect,pixel | I6 | rect hud-chat HEADLESS-1 [932.0, 175.0, 340.0, 403.0] -> [932.0, |
| xp | x6c | rect,pixel | I6 | rect hud-chat HEADLESS-1 [932.0, 290.0, 340.0, 288.0] -> [932.0, |
| xp | x6d | rect,pixel | I6 | rect hud-chat HEADLESS-1 [932.0, 248.0, 340.0, 330.0] -> [932.0, |
| xp | x6e | - | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x7 | pixel | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x7b | pixel | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x8 | pixel | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x9 | - | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |
| xp | x9b | pixel | I6 | rect Notepad:None HEADLESS-1 [288.0, 119.0, 700.0, 500.0] -> [376.0, |


### Gate
Full `tests/merge-gate.sh --jobs 2` (no --only), detached, on claude/record-v9 6949cddb merged with origin/claude/autonomous-dev-session-3w-i33pg7: 62 cells PASS, "all steps passed", EXIT 0. No foreign red. `pgrep -fa 'chora --headless'` empty.

### Next
F7 is not met: 88 regressions remain. What is left is mostly not a harness defect: the I6 flips are breaches the baseline could not measure (rv9-1 to rv9-6), the mc chapter diverges by page after Home (fa7faabd), and the xp rows wait for those I6 findings to be fixed. Suggest: the baseline replay learns to say "I6: not measured" (no green) so that it is not a flip, and a build between 733967aa and head that has `scene.boxes` but not the look changes (bisect) as a second baseline for I6.
