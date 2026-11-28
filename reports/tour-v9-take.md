# tour-v9-take

## r1 (F7, RC 1b, 11)

Head claude/tour-v9-take d40142e4 (session head). Step 0 green: unshallow, cargo check and build, foot ffmpeg waybar swaybg dbus-daemon, PIL. No code, tool or intended.toml change.

### 1 `run.py --check` (all chapters)
`tour: check: 14 red, 0 broken`. xp has no red act; tiling none; mc none; handover 11 of 11 green. The 14 red are all in construct (acts 4..12, 14..18 minus 13: 4,5,6,7,8,9,10,11,12,14,15,16,17,18; 4 green): every one carries `I2: gaps: HEADLESS-1: construct-mode-step-in and construct-mode-turn-left/-right are 9 apart, no whole number of 8` (rv9-2, 1e7cc905); act 11 also `I6` on the CONSTRUCT · CUBE key overlay. Not fixed. (The xp/tiling/mc lines of the check output were cut by my own `tail`; the full take below shows the same: xp 31 acts 30 green 1 known (act 31: "known act 31 is green: drop known", I4 ca-4), tiling 28 green, mc 22 green, construct 4 green 14 red, handover 11 green.)

### 2 Full take
`TOUR_OUT=... demo/tour/run.py` all five chapters (xp, tiling, mc light and dark, construct, handover): `tour: record: 14 red, 0 broken`, same 14 red construct acts. Joined MP4 `tour-all.mp4`: 6.9 MB (limit 19 MB, no re-encode, no crf), 326 s. 106 stills.

### 3 v8 comparison (tests/tour-diff.py, baseline replay 733967aa built in a worktree, `TOUR_BASELINE=1`, `--intended demo/tour/intended.toml`)
Baseline: xp, tiling, mc taken; construct stops at `construct.cube.list` (no exported method) -> 4 captures `baseline: none`; handover BROKE at handover.py:190 (`mid("id:xp-ctx-1-0")` is None on the old build, same as r4; `baseline_none` covers the rest, 7 rows `none` plus 4 open).

| chapter | acts | same | change | open | regression | none |
|---|---|---|---|---|---|---|
| xp | 31 | 0 | 7 | 23 | 1 | 0 |
| tiling | 28 | 1 | 3 | 24 | 0 | 0 |
| mc | 22 | 0 | 3 | 18 | 1 | 0 |
| construct | 18 | 0 | 0 | 13 | 1 | 4 |
| handover | 11 | 0 | 0 | 4 | 0 | 7 |
| total | 110 | 1 | 13 | 82 | 3 | 11 |

r4 (record-v9, report): 1 same, 21 change, 88 regression, 0 open (xp 0/0/31, tiling 1/20/7, mc 0/1/21, construct 0/0/18, handover 0/0/11). Now: 1 same, 13 change, 82 open, 3 regression, 11 none. The rows are not comparable one to one: since r4 the head carries cc71e5d9 "no unproven row passes as change (rtb2-1..4)": a diff with a row of intended.toml whose cause is not proven now reads `open` (exit 1 on F7), where r4 read it `change` or `regression`; and handover is counted `none` where the baseline has no capture. So the 85 open+regression acts are what r4 called 88 regression and 21 change, less the rows now judged `none` and 3 acts that went to `change`. I did not re-judge any row. tour-diff exits 1 (82 open, 3 regression).

Regression acts (3), each with its check line:
- c4 (construct, Q): flipped I6 (cand red I2, I6); scene: `construct-door-clock`, `-door-helper`, `-door-keys`, `-door-lock` (and their text nodes) missing in the baseline only. Check lines: I2 construct-mode-step-in/turn-left 9 apart (rv9-2); I6 as in r4 table (construct help/key list sizes).
- m1c (mc, Q): unnamed `pixel` plus scene: `mc-r0-right-notify` and `-notify-title` missing in the baseline only, `mc-r1-right-chat-door*` rects 84 px higher (194 -> 110), `mc-r1-right-chat-key` 58 -> 60 wide. No flipped check. Cause not found by me (r4: mc page divergence fa7faabd and the notify card, 675ebd55).
- x6b (xp, Q): unnamed `missing`; `hud-card` 312x188 -> 312x208 and moved (946,292 -> 858,298), meta gains the line "Typing does not answer."; no flipped check. Cause not found by me (the hud card text is new at the head).
The 82 open acts (cause row present, not proven) are in `tour-diff-r1.txt` (one line each: act, capture, verdict, cause or unnamed kinds, unmeasured checks).

### 4 Contact sheet
`tour-v9-sheet.png`, 1920 x 1890, all 106 stills, 8 columns of 240 x 135 with the still name on each. I made it with PIL, not `demo/tour/sheet.sh`: that script lays out 2 columns, 106 stills would be 19 000 px high.

### 5 Where the files are (deviation, say plainly)
The brief asks for `media/tour-v9.mp4` and `media/tour-v9-sheet.png` on claude/tour-v9-take. The commit was refused by the repository's hooks and I did not bypass them: `no-session-files` (media/ not allowed in the repository) and `check-added-large-files` (500 KB limit, mp4 6740 KB, sheet 780 KB). Nothing was pushed to claude/tour-v9-take (it stays at d40142e4). The files are on claude/session-log, which holds media/ already: `media/2026-11-24/tour-v9/` (tour-v9.mp4, tour-v9-sheet.png, tour-diff-r1.txt and .json, timeline-table.md). The PM can publish from there.

`pgrep -fa 'chora --headless'` empty at the end.

## PM note r1

- Run-only round, nothing to merge. Done when met except the media place: the brief (PM) named a place the hooks refuse (media/ and 500 KB on product branches); the lane was right not to bypass. Brief fault, not the lane's; the next take brief names media/<date>/ on claude/session-log. Published on the status page as status/video/tour-v9.mp4. Next: tour-v9-judge r1.

## r2 (F7, RC 1b, 11)

Head claude/tour-v9-take-r2 e39448ad (session head, t139 green, tour-base r8 in). Step 0 green: unshallow, cargo check and build, foot ffmpeg waybar swaybg dbus-daemon, PIL. No code, tool or intended.toml change. The baseline 733967aa was built in a worktree and replayed with `TOUR_BASELINE=1` as in r1.

### 1 `run.py --check` (all chapters)
`tour: check: 14 red, 0 broken`. xp, tiling, mc, handover: no red act. construct: 14 red (acts 4..12, 14..18; act 13 is green with 1..3), same set as r1: every one `I2: gaps: HEADLESS-1: construct-mode-step-in and construct-mode-turn-left/-right are 9 apart, no whole number of 8` (rv9-2); act 11 also ten `I6` (the CONSTRUCT · CUBE key overlay: heads 11 px, the guide wants 14 px). Also `known act 2 is green: drop known` (xp, I3 ca-3) and in the take xp act 31 is the one `known`. Not fixed.

### 2 Full take
All five chapters: `tour: record: 14 red, 0 broken`, the same 14 construct acts. xp 31 acts 30 green 1 known; tiling 28 green; mc 22 green; construct 4 green 14 red; handover 11 green. Joined MP4 `tour-all.mp4` 6.3 MB (limit 19 MB: no re-encode, no crf). 106 stills.

### 3 v8 comparison (tests/tour-diff.py, baseline 733967aa replay, `--intended demo/tour/intended.toml`)
Baseline take: xp, tiling, mc taken (81 red rows on the old build, expected); construct stops at `construct.cube.list` (no exported method), 4 captures `baseline: none`; handover BROKE at the same place as r1 (the old build has no `id:xp-ctx-1-0`); the captures it reached count below.

| chapter | acts | same | change | open | regression | none |
|---|---|---|---|---|---|---|
| xp | 31 | 0 | 0 | 31 | 0 | 0 |
| tiling | 28 | 1 | 0 | 27 | 0 | 0 |
| mc | 22 | 0 | 0 | 16 | 6 | 0 |
| construct | 18 | 0 | 0 | 14 | 0 | 4 |
| handover | 11 | 0 | 0 | 0 | 4 | 7 |
| total | 110 | 1 | 0 | 88 | 10 | 11 |

r1 (take on d40142e4): 1 same, 13 change, 82 open, 3 regression, 11 none (xp 0/7/23/1, tiling 1/3/24/0, mc 0/3/18/1, construct 0/0/13/1 +4 none, handover 0/0/4/0 +7 none). Now: 1 same, 0 change, 88 open, 10 regression, 11 none. tour-base r5..r8 changed the verdict logic (every scene row lists its nodes, every pixel row its boxes; a row that does not name the diff absorbs nothing), so the 13 `change` of r1 are all `open` or `regression` now: the rows no longer absorb what they did not list. xp 0 regression (r1: x6b), construct 0 regression (r1: c4). I did not re-judge or edit any row. tour-diff exits 1.

Regression acts (10), each with its check line. No check flipped in any of them (the old build measures none of I2 I5 I6 I9 I12 I13: "not measured"); each is an unnamed scene or pixel diff:
- h1, h1b (handover, Q): unnamed `rect`; `construct-hud*` (5 nodes) missing in the baseline only; `construct-start-help-0` 54 -> 29 wide and moved (694 -> 752), `construct-start-help-1` missing in the baseline only; pixel blobs at the bottom-right pager (1052..1100, 694..716), 0.5 % and 0.7 % of the pixels.
- h1c, h1d (handover, Q): same scene rows; pixel 15.2 % and 15.4 % (blob 22,22 95x532 and 100x532: the left column).
- m1, m1c (mc, Q): unnamed `rect`; `mc-r0-right-chat-door*`/`-key`/`-state`/`-title` 84 px higher (194 -> 110), `-chat-key` 58 -> 60 wide; m1c also `mc-r0-right-notify`, `-notify-title` missing in the baseline only. Pixel 0.0.
- m7b (mc, Q): `added`; the notify nodes missing in the baseline only, `mc-r1-header-head` 101 -> 223 wide, chat rows 84 px higher. Pixel 0.0.
- m7c, m8, m8b (mc, Q): `added`; `mc-r0-footer-*`, `mc-r0-header-*`, `-right-chat-*` have no rect on the baseline (`None -> ...`), the baseline page is another one at those acts. Pixel 0.2 %, 1.9 %, 0.1 %; blobs at (201,98) and (204,98) 1x33, (888,378) 336x1, (505,305) 18x15.
Cause not found by me for any (r1 named: the mc page divergence fa7faabd and the notify card 675ebd55; the same rows are the likeliest, not proven). The 88 open acts (cause row present, not proven) are in `tour-diff-r2.txt`, one line each.

### 4 Contact sheet
`tour-v9-sheet.png`, 1920 x 1890, all 106 stills, 8 columns of 240 x 135 with the still name on each, made with PIL. `demo/tour/sheet.sh` lays out 2 columns: 106 stills would be 19 000 px high.

### 5 Where the files are (deviation, say plainly)
The brief asks for `media/tour-v9.mp4` and `media/tour-v9-sheet.png` on claude/tour-v9-take-r2. The commit was refused by the hooks again, as in r1: `no-session-files` (no media in the repository) and `check-added-large-files`. I did not bypass them and pushed nothing to claude/tour-v9-take-r2 (it stays at e39448ad). The files are on claude/session-log in `media/2026-11-25/tour-v9-r2/`: `tour-v9.mp4` (6.3 MB), `tour-v9-sheet.png`, `tour-diff-r2.txt` and `.json`, `timeline-table.md`.

`pgrep -fa 'chora --headless'` empty at the end.

## r3 (F7, RC 1b, 11)

Head claude/tour-v9-take-r3 3098926e (session head, t162 green; tour-rows r5 and tour-fix r1-r2 in). Step 0 green: unshallow, cargo check and build, foot ffmpeg waybar swaybg dbus-daemon, PIL. No code, tool or intended.toml change. Baseline 733967aa built in a worktree, replayed with `TOUR_BASELINE=1` (the head's harness, `TOUR_REPO` on the worktree), each chapter its own TOUR_OUT.

### 1 `run.py --check` (all chapters)
`tour: check: 12 red, 0 broken`. xp 31 acts 30 green 1 known; tiling 28 green; mc 22 green; handover 11 green; construct 18 acts 6 green 12 red (acts 4, 7..12, 14..18), every one `I2: gaps: HEADLESS-1: construct-mode-step-in and construct-mode-turn-left/-right are 9 apart, no whole number of 8` (co-44, open, COSMETIC). r2 had 14 red: act 11's ten `I6` (key overlay chips, 11 px) and acts 5 and 6 are gone. Also `known ... is green: drop known` for xp 31 (I4 ca-4), tiling 24 (ti-29), mc 4, 5, 6 (I5 notice over the column), handover 2 (I3 ca-3); `AUDIT [('s1','xp')]`. Not fixed.

### 2 Full take
All five chapters: `tour: record: 12 red, 0 broken`, the same 12 construct acts; xp 30 green 1 known, tiling 28, mc 22, construct 6 green 12 red, handover 11. Joined MP4 (the five `tour-all.mp4` joined with ffmpeg concat, stream copy) **7.0 MB**, 327 s; limit 19 MB: no re-encode, no crf. 106 stills.

### 3 v8 comparison (tests/tour-diff.py, `--intended demo/tour/intended.toml`)
v8 take: xp 31 red (old build, expected), tiling 28 red, mc 22 red, construct stops after c3 with 1 `baseline: none` note, handover BROKE (Send To TypeError on the old build); same as r2.

| chapter | acts | same | change | open | regression | none |
|---|---|---|---|---|---|---|
| xp | 31 | 0 | 31 | 0 | 0 | 0 |
| tiling | 28 | 1 | 27 | 0 | 0 | 0 |
| mc | 22 | 0 | 19 | 0 | 3 | 0 |
| construct | 18 | 0 | 12 | 0 | 2 | 4 |
| handover | 11 | 0 | 4 | 0 | 0 | 7 |
| **r3 total** | 110 | 1 | 93 | 0 | 5 | 11 |
| r2 total | 110 | 1 | 0 | 88 | 10 | 11 |

r2 by chapter: xp 0/0/31/0, tiling 1/0/27/0, mc 0/0/16/6, construct 0/0/14/0 (+4 none), handover 0/0/0/4 (+7 none). Now 88 open -> 0 and 93 change: the rows of tour-rows r5 and tour-fix absorb what they name. tour-diff exits 1 (5 regression).

Regression acts (5), each with its check line. No check flipped in any (I2 I5 I6 I9 I12 I13 "not measured" on the old build):
- **c5d** (construct, Q): unnamed `visible`, `rect`, `missing`, `added`, `pixel`. v9 steps into face 0 (face-0 rect 375x261 -> full frame 1280x720; cube edge, face-1, mirrors, floats gone), v8 stays stood back. Still a regression: **yes** (rtf1-1, no row possible, comparability of the replay, not a look defect).
- **c5e** (construct, Q): unnamed `pixel` (face-0 full frame, face-1/mirrors/floats gone as c5d). Still a regression: **yes** (rtf1-1).
- **m8** (mc, Q): unnamed `pixel`; the launcher without icons on a dark ground, blob at about [463, 198, 354, 324]; scene rows (footer, header, chat rects `None -> ...`) are named, the pixels are not. Still a regression: **yes** (rtr4-2, MINOR, open).
- **m6, m6b** (mc, Q): unnamed `missing`, **new since r2** (r2: open). Baseline-only nodes: `mc-r0-right-notify-0`, `-notify-0-time`, `-notify-title` (a notice card in v8 at that moment, none in v9), `mc-r1-right-doors-*` (lock, power, programs, screenshot, volume, each with `-hit` and `-mark`), `mc-r1-right-keys-12-key-hit/-mark`, `-2-key-hit/-mark`, `-keys-title-rule`. Rows for m6 exist (675ebd55 notices out of the column, 798fdc33, 6fa86c16, fa7faabd) but none lists these nodes, so they do not absorb them. Pixel clean (no unnamed pixel diff). Cause likeliest the page divergence fa7faabd plus 675ebd55 (not proven); m6b shows the same missing set plus footer/header rect moves that rows name.

c5d, c5e, m8 are filed regressions (rtf1-1, rtr4-2); m6 and m6b are new and want rows or a look from the next rows lane. The 93 `change` and 1 `same` are per act in `tour-diff-r3.txt`.

### 4 Contact sheet
`tour-v9-sheet.png`, 1920 x 1890, all 106 stills, 8 columns of 240 x 135 with the still name on each (PIL, as r2; `demo/tour/sheet.sh` lays out 2 columns, 106 stills would be 19 000 px high).

### 5 Where the files are (deviation, say plainly)
The brief asks for `media/tour-v9.mp4` and `media/tour-v9-sheet.png` on claude/tour-v9-take-r3. The commit was refused by the hooks again, as in r1 and r2: `no-session-files` (no media in the repository) and `check-added-large-files`. I did not bypass them; nothing was pushed to claude/tour-v9-take-r3 (stays at 3098926e). The files are on claude/session-log in `media/2026-11-28/tour-v9-r3/`: `tour-v9.mp4` (7.0 MB), `tour-v9-sheet.png`, `tour-diff-r3.txt` and `.json`, `timeline-table.md`.

`pgrep -fa 'chora --headless'` empty at the end.
