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
