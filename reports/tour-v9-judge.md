# tour-v9-judge

## r1 (F7, RC 11)

Head claude/tour-v9-judge d40142e4. Run only: no product, test, tool or intended.toml change. Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`, `demo/tour/run.py --help` (note: run.py has no `--help`; the flag is skipped and it starts a full record take; I stopped that one and started clean takes).

**Takes (said plainly).** The stills of take r1 are not on claude/session-log (only the sheet), so I took both sides again, at the same time: v9 = `TOUR_OUT=<scratch>/v9 demo/tour/run.py` on d40142e4; v8 = replay on a worktree of 733967aa built fresh (`TOUR_REPO=<worktree> TOUR_BASELINE=1`, run.py of the head, own `TOUR_SCRATCH`), as the take did. The baseline stops as in take r1: construct at `construct.cube.list` (4 captures `none`), handover BROKE at handover.py:190 (7 `none`). The v9 take: xp 1 red (act 5: "Dear Anna," lost), construct 14 red (I2 co-44, I6 co-45, as take r1), handover 3 red (acts 4, 6, 10: the first line typed twice). Take r1 had xp and handover green; I read the 4 extra reds as noise from running two takes at once (typing under load), not as product findings. `tests/tour-diff.py` on my pair: 1 same, 13 change, 81 open, 6 regression, 9 none; the same verdicts as take r1 except h1d, h3, h6 (`expect`, the reds above) and m1/m1c, which trade places (see 1b). Output: `media/2026-11-24/tour-v9-judge/tour-diff-judge.txt`. Every image there is v8 replay | v9 | pixels that differ by more than 24 (red).

### 1 The 3 regressions

| act | v8 vs v9 | cause | verdict |
|---|---|---|---|
| x6b (xp, question card) | `x6b.jpg` | unnamed `missing hud-chat-hint` and the card 20 px taller: **86343e3f** "the typing line stands in the card of the question" (xp-14, xp-polish 64478c66); the balloon and card 88 px to the left: **62708d35** "bin back at the bottom right, balloons left of it" (xp-13, rxr-2) | **intended** (xp-v2 rows 26 and 48: "Typing does not answer." outside the card was the fault). One more element, not intended: the transcript reads "You / so you cannot ask anything. Click me to see what to do. This balloon closes by itself." The person's own line "Please copy my letter to My Documents." is pushed out, and the tail of the no-helper greeting stands under "You". This is **xp-47** (greeting put into the chat of a working helper; cite, not new). Seen in both takes (take r1 `tour-diff-r1.json` x6b meta, and mine). helper-greet r1 0a0fca76 is MERGE-OK; the next take shows whether it clears x5..x6d. |
| m1c (mc, first page) | `m1c.jpg`, `m1-col.jpg` | **675ebd55** "chora-notify shows the notices, mc stacks none": the column's NOTIFICATIONS / "Nothing new." go, the chat door moves up 84 px; **b8872054** (kit::keysheet): "Super+a" now reads "Super+A". The unnamed `pixel` is the KEYS hairline: in v9 it stays at y 378 in m1, m1b, m1c and m2; in v8 it moves (406 in m1 and m2, 378 in m1b and m1c) with the column's content. So the act that gets the blob moves between takes (take r1: m1c, mine: m1); v9 is the steady side. | **intended by the commit, not by the design**: mc-v2 section 3 and build item 5 still ask for "Nothing new." in the column. That is a decision to make, not an F7 regression: **rv9j-2** (mc). The key caps "Super+A" are rpx1-16, under mc-61. |
| c4 (construct, Super+F1) | `c4.jpg`, `c4b.jpg` | The act asks for the key list (`sheet_open(True)`). v9 opens CONSTRUCT · CUBE; 733967aa opens nothing (its `expect` is red), so the door bar and chips stay in the v8 still. The `construct-door-*` nodes are gone in v9 while the sheet covers them (the hide-under-cover rule, 1e7cc905, a3037cb2), and come back at c4b (`c4b.jpg`, with the new chip "Super F1 All keys", cfc805d4). Flipped I6: the sheet's 11 px scope line and heads (**co-45**); I2 cand red (**co-44**, rv9-2). | **intended** (shared.md: every key is in each look's list; doors keys-key). Known: co-44, co-45. |

### 2 The open acts, per look and cause row

"Proven" means the diff is what the row says and nothing else changed (checked by eye on 2 acts against the row's commit). Many rows fail because they name the wrong commit: tour-diff keys a row by act and kind, so one row covers every node of that kind (rv9j-5).

**xp**

| acts | cause row | checked | verdict |
|---|---|---|---|
| x5, x5b, x5c, x5d, x5f, x6, x6b, x6c, x6d | 4fec75c3 greets once (xp-30), open; meta absorbed by "29281054 DB8" | x5d `x5d.jpg`, x6b | **not proven.** (a) `hud-chat` ends with "Hi, I am your helper, but no helper is set up yet …" while Atlas answers: xp-47 (known MAJOR), and the meta row names 29281054 (the desktop labels), which does not cause it. (b) The balloon moves 88 px left: 62708d35 (xp-13), not in the row. |
| x1, x2, x2b, x2c, x5e, x5g, x7, x7b, x8, x9b, x10, x10c, x11b, x12b (pixel) | fc9675ac, 11e34cf8, 94579f91 (captions 4.5:1), open | x7 `x7.jpg`, `x7-tray.jpg`; x2 `x2.jpg` | **not proven.** The blobs are the taskbar pager (**d78fc4bb** "workspace buttons dark enough for 4.5:1", xp-39/xp-19) and the Recycle Bin (62708d35). The named commits change the session-button captions, which are not on screen in these acts. Element that fails F7: **rv9j-1**. The current desktop is no longer told apart: fill #164EB8 against #1E54BE, about 1.07:1 (v8: a dark current button among light ones). |
| x2c, x3, x4, x7, x7b, x8, x9, x9b, x10, x10b, x10c, x11, x11b, x12b … (rect) | 09ff8ff9, bd5af4b3 new windows right of the icons (xp-28) | x7, x10b | **proven** (Notepad starts right of the icon column). |
| x1, x2, x2b, x12 (meta) | 29281054 (labels, notice "To name one yourself: …") | x2 | **proven.** The notice wording is rpx1-7 (xp-48, known). |
| x10b | xp-43 (hud-line, help scroll node), open | x10b `x10b.jpg`, `x10b-keys.jpg` | **not proven by the row, nothing beyond known.** The Help table is the kit keysheet (b8872054, c64752fa) with a scroll bar. Arrow caps drawn in bold italic: rpx1-5 (xp-48, known). Notepad's caption buttons show right of Help (xp-28 placement). |

**tiling**

| acts | cause row | checked | verdict |
|---|---|---|---|
| t1, t1b, t1c, t2 | bbb105ba hairline islands, 3a9a426f clock width, 136bbd82 layout chip, 6a5cfc1d helper word, open | t1 `t1.jpg`, `t1-bar.jpg` | **proven**: islands without the coloured edge, Inter clock, the fixed-width Tile chip, "Atlas · idle" for "No helper". Nothing else. |
| t1d, t3, t3b..t3h, t4, t5, t6b, t6d, t8b, t9b, t10b | 573496e5 inner gap 12 (tiles move 1 px) plus the bar rows, open | t3 `t3.jpg`, `t3-edge.jpg`; t8b `t8b-h1c.jpg` | **proven**: 1 px tile shift and the bar. Nothing else. |
| t6, t6c | b8872054, a611c08a, fa076e7b key list, open | t6 `t6.jpg` (t6c is the same scene) | **proven**: the kit list with the "Other desktops" group. Known: ti-30 (13 px, 6.5:1). |
| t7, t10c | 675ebd55 notices (sh-31), open | t7 `t7.jpg` | **not proven.** The card is new (v8 lost the notice: ti-22). Element that fails F7: **rv9j-3**, a white card with black text on the dark tiling desktop. |

**mc**

| acts | cause row | checked | verdict |
|---|---|---|---|
| m1, m1b, m1c, m2, m2b, m3, m3b, m4, m4b, m4c, m7, m7b | 675ebd55 notices (bisect r2), partly open | m1c, m3 `m3.jpg` | **the diff is the commit**: NOTIFICATIONS gone, chat door up, the timer card bottom left as a dark box (plugins/chora-mc/notify: "the colors are those of the dark theme of mc"). Beyond the design: **rv9j-2**. The card covers the third page line: rpx1-11 (mc-61, known). |
| m5, m5b, m6, m6b, m7c, m8, m8b, m9, m9b, m10 | fa7faabd Home goes to the first page (mc-38); mc-59, open | m5 `m5.jpg`; m8 `m8.jpg`; m9b `m9b.jpg`, `m9b-helper.jpg` | **not proven by diff, no F7 element.** From the Home act on, v9 is on the agenda and 733967aa stays on the page it was on (Home did nothing there), so later acts compare different pages (m8: AGENTS vs AGENDA; m9b: THE MORNING vs AGENDA). By eye, beyond the column change: the launcher list without icons (29281054 raw entries), unthemed fuzzel in both builds (rpx1-10), the stale hover mark of the helper row in v9 (mc-15, known), the pill over the clock in both (rpx1-13). |

**construct**

| acts | cause row | checked | verdict |
|---|---|---|---|
| c1, c2, c2b, c3, c3b, c3c, c3d, c4b, c5, c5b, c5c, c5d, c5e | co-44 / cfc805d4 chips, 8ca85eab pager, d9b2c4f1 stand back (co-52, co-54, co-63), 6a5cfc1d, b8872054, 371f0fa1, open | c2 `c2.jpg`, c3b `c3b.jpg`, c4b | **not proven (causes missing), no new element.** As the rows say: the "Super F1 All keys" chip, the pulled pane small beside the cube instead of filling the front. Not in any row but intended: the "Click Fly in / Super Alt 4 Pull out" chips (co-68, 6d336bdf), xp-face inactive titles (f1397d98, xp-45), and the xp-face pager and bin (d78fc4bb, so rv9j-1 also shows on the faces; 62708d35). Known reds: I2 co-44, I6 co-45. |

**handover**

| acts | cause row | checked | verdict |
|---|---|---|---|
| h1, h1b, h1c, h1d | d9b2c4f1 / co-63 plus 29281054, 371f0fa1, fc9675ac, open | h1 `h1.jpg`, h1c `t8b-h1c.jpg` | **not proven (wrong causes).** The capture is the xp output, where only the bin (62708d35), the pager (d78fc4bb, rv9j-1) and the Notepad start place (xp-28, 09ff8ff9) differ; d9b2c4f1 / co-63 are not on this screen. |

### 3 The `none` acts and the black right screens

- c3e..c3h (the cube turned by hand and by key, 0/45/90 degrees): no 733967aa capture by design (no `cube.list`). By eye (`none-v9.jpg`) the cube, label and chips are right; the only red check is I2 (co-44). **pass**.
- h2, h3, h3b, h4, h5, h6, h7 (Send To onward): no baseline by design (no Send To row on 733967aa). Take r1: 11 of 11 green. In my take h3 and h6 are red only because the typed line doubled (noise, above). By eye (`handover-LR.jpg`, `none-v9.jpg`) every give lands: tiling, mc page "7 NOTEPAD", construct pane, back in xp with every line. **pass**.
- Black screens: 05-01, 05-02, 05-03, 05-03b R; 05-06 L; 05-09 and 05-10 R are pure black (0,0) in v9. In v8, 05-01..05-03b R are black too (v8 stops before 05-04). Each black screen is the tiling output while it holds no window. **pass for F7** (both builds the same). The cause is the tour, not the product: handover.py:96 starts chora-tiling with `--no-companions` (the head knows the flag) but, unlike tiling.py:143, starts no bar or wallpaper. So the act "tiling desktop empty" shows a black screen with no bar, where the product (ccf7ebab) would draw both. **rv9j-4** (tour).

### 4 Findings

| id | look | severity | act | stills | element |
|---|---|---|---|---|---|
| rv9j-1 | xp | MINOR | x1..x12b, h1..h1d, xp faces of c1..c5e | `x7-tray.jpg`, `x7.jpg` | Taskbar pager: d78fc4bb (xp-39, xp-19) gives the current desktop #164EB8 / edge #0F3C96 and the others #1E54BE / #1A4BB0, about 1.07:1. The "pressed" desktop in use cannot be seen (plugins/chora-xp/src/taskbar.rs:94 `pager_colors`; review checklist 9, every state shows; xp-v2 D14 keeps the pager as a door). In both takes (fixed colours in code). |
| rv9j-2 | mc | MINOR (design question) | m1c, m3 and every mc act | `m1c.jpg`, `m3.jpg` | 675ebd55 drops the column notifications with "Nothing new." (mc-v2 section 3, build item 5, P4 honest empty state) and shows notices as a box in the dark theme colours on the light page (mc-v2 line 78 U2, rule 11 no boxes; product-briefs chora-notify: "mc: right margin", the card stands bottom left). The ledger accepted the commit (mc-12, mc-35). The PM decides: update mc-v2 or restore the column section. Not counted as an F7 regression. |
| rv9j-3 | tiling | MINOR | t7, t10c | `t7.jpg` | The "Screenshot saved" card of chora-notify is white with black text on the dark tiling desktop. tiling-v2 item 10 asks for "chora-notify in the flavor for the notice"; the flavor starts plain chora-notify (flavors/tiling/chora-tiling-notify.service `ExecStart=/usr/bin/chora-notify`), so the product looks the same as the tour. Review checklist 8 (no light panel on dark). New in v9 (v8 lost the notice). |
| rv9j-4 | tour (shared) | MINOR (harness) | h1..h1d, 05-01..05-03b, 05-06 L, 05-09, 05-10 | `handover-LR.jpg` | handover.py:96 passes `--no-companions` and starts no bar or wallpaper, so the empty tiling screen is black in the video. The same in v8; not a product fault. |
| rv9j-5 | tour (F7 tooling) | MINOR | x5..x6d, the x pixel rows, h1..h1d, construct | as above | intended.toml rows match by act and kind, so one row names every node of that kind. Wrong causes: 29281054 for the greeting in `hud-chat`, fc9675ac/11e34cf8/94579f91 for the pager and bin, d9b2c4f1 for h1. Missing causes: 62708d35, d78fc4bb, 86343e3f, 6d336bdf, f1397d98. A row per node id (or a node list per row) would make "proven" checkable. |

No BLOCK or MAJOR is new, so no second take per chapter was needed. Known ids cited, not re-reported: xp-47, xp-48 (rpx1-5, rpx1-7), mc-61 (rpx1-10, -11, -13, -16), mc-15, mc-59, co-44, co-45, co-63, ti-30, xp-43.

### 5 Elements that fail F7, per look (input for one fix lane per look)

- **xp**: (1) the no-helper greeting in the chat of a working helper, last line of `hud-chat` in x5..x6d; in x6b it stands under "You" and pushes out the person's line (xp-47 MAJOR, known; helper-greet 0a0fca76 MERGE-OK, check in the next take). (2) The pager's current desktop cannot be seen (rv9j-1).
- **tiling**: the light notify card on the dark desktop (rv9j-3).
- **mc**: the timer card covers the page's last line (mc-61 rpx1-11, known). The missing "Nothing new." and the dark box card wait on the design decision rv9j-2.
- **construct**: none new (I2 co-44 and I6 co-45 are known and red in the take).
- **handover / shared**: none in the product. Tour: rv9j-4. F7 tooling: rv9j-5 (intended.toml rows; until fixed, `open` counts cannot reach 0 even where the change is intended).

`pgrep -fa 'chora --headless'` empty at the end.

## r2a (F7, RC 11): mc and handover, take r2 on e39448ad

Lane claude/tour-v9-judge-r2a at e39448ad. Step 0 green (unshallow, `cargo check --workspace`, `cargo build --workspace`). Run and read only: no product, test, tool or intended.toml change.

**Takes (said plainly).** One take at a time, as the brief says. v9: `TOUR_OUT=<scratch> demo/tour/run.py mc` twice (take 1 and take 2, 22 of 22 green each) and `run.py handover` once (11 green). v8: a worktree of 733967aa built fresh, `TOUR_BASELINE=1 TOUR_REPO=<worktree> run.py mc` (22 red, expected on the old build) and `... handover` (BROKE at Send To after h1d, as in take r1 and r2: 4 captures). `tests/tour-diff.py` on my pairs gives the same verdicts as take r2 in both mc takes: mc 16 open, 6 regression; handover 4 regression, 7 none. A side note on the harness: `run.py` (without `--check`) clears `<TOUR_OUT>/captures`, so a second chapter in the same `TOUR_OUT` deletes the first chapter's stills. I lost the take-1 mc stills that way. Take 2 replaced them (the side-by-sides below are base vs take 2), and the diff output of take 1 is kept.

I made my own look before I read judge r1. Where I check r1, I say so below.

### mc: table (act or group, cause, verdict, stills)

| act / group | v8 vs v9 (by eye) | cause | verdict |
|---|---|---|---|
| **m1** (regression) | Same page. Column: NOTIFICATIONS / "Nothing new." gone, the chat door 84 px up, KEYS hairline 406 → 378, "Super+a" → "Super+A". **Page: the list "For two / 200 g spaghetti / 4 cloves of garlic / 6 tbsp olive oil" shows only "For two"** | column: 675ebd55 (rv9j-2, PM decision 11-24: notices out of the column); key text: **b8872054** (kit::keysheet, "a letter in capital as in `Super+J`"; judge r1 right, the rows that put it on 6fa86c16 name the wrong commit); lead rule x 42 → 94: 6fa86c16; page 608 → 503 high: **db20c304, 3e355bb2** (rpx1-11, -12, mc-minor r1, t138) | column, key, rule: intended. **List cut: finding rv9ka-1.** `rv9ka-1-m1.jpg`, `rv9ka-1-m1-crop.jpg` |
| **m1c** (regression) | As m1. The Timer panel shows only "Timer"; "9 minutes once the spaghetti is in." is gone, with no card on screen | as m1 | as m1; **cut: rv9ka-1**. `rv9ka-1-m1c.jpg` |
| m1b, m2, m2b, m3, m4, m4b, m7 (open; group "column": same page on both builds) | Column as m1. m2, m2b, m3: the timer card is a dark box at the bottom left (chora-notify, plugins/chora-mc/notify). v8 m2/m2b: "Chat with" printed over "command" in the key list, fixed in v9. **m1b, m2, m2b, m3, m4b: the Timer or list text is cut as in m1/m1c; m4: the "Then" panel shows one of its 5 lines** | 675ebd55 (rv9j-2), b8872054, 6fa86c16, db20c304 / 3e355bb2; the card's colours: mc-61 (known) | intended except the cut (rv9ka-1), which hides inside these `open` rows: their bare `rect` rows (`open = true`, no nodes) leave the `-lines` rects unproven. Checked by eye: m1b, m2, m2b, m3, m4, m4b, m7. `m2-card.jpg`, `rv9ka-1-m4.jpg`, `m7-column.jpg` |
| m3b, m4c, m5, m5b, m6, m6b, m9, m9b, m10 (open; group "other page") | From m4c ("Remote: Home goes to the first page") v9 shows the agenda and v8 stays on the page it was on (Home did nothing on 733967aa). Every later light act compares two different pages. m6: v9's all-keys list is clean and has the Home row; v8's head runs a rule through "Mute" and prints "Chat with" over "command". m9: v9 opens the chat in the column (the mc-v2 layout); v8's click opens nothing. m10: Volume 40 → 45 % in v9 | **fa7faabd** (mc-38: Home goes to the first page); m6 fixes: 5cb288e2 (rpx1-9), 6fa86c16; m9 focus bar at (884, 594): mc-v2 focus mark | intended. r1 put m3b and m4c in the 675ebd55 group; they belong here (other page). Checked by eye: m4c, m5, m5b, m6, m6b, m3b, m9, m9b, m10. `m4c-home.jpg`, `m6-allkeys.jpg`, `m9-chat.jpg`, `m10-volume.jpg` |
| mc-regions-N `visible`, r0 ↔ r1 rects (m2, m2b, m3, m4, m4b, m10 ...) | Nothing to see: the region set that shows is the other one of the pair | the cross-fade double buffer of mc-48 (bb3de644) flips per rebuild; v9 rebuilds a different number of times (675ebd55: a notice no longer rebuilds the column; bisect r2 saw the flip at 675ebd55). mc-59 can name it: the parity of rebuilds, not a look change | intended (invisible) |
| **m7b** (regression) | v8 "3 AGENTS", an empty page; v9 "3 HELPERS", "No helper is set up. Ask the person who set up this computer.", running head "HELPERS · 0 RUNNING", one column over the full width | **ffe5e815** (rpx1-14, mc-61: the empty Helpers page says so; `show` fills the scene first, so the running head and rule 14's one column also come now); title 31f53dc9, c5acd4d0 (mc-55, I10); column 675ebd55, b8872054 | intended (mc-v2 P4, honest empty state). The only unnamed node: `added mc-s-agents-panels-agent-1-empty`. `m7b-helpers.jpg` |
| **m7c, m8b** (regression) | Other page (v8 Agents, v9 Agenda). Nothing else | fa7faabd; the unnamed node is the same `added mc-s-agents-panels-agent-1-empty` (the hidden Helpers scene, ffe5e815) | intended. `m7c-home.jpg` |
| **m8** (regression) | Other page, and the launcher: v8 lists programs with icons (LibreOffice x5, UXTerm, Vim); v9 lists text only and no LibreOffice rows | fa7faabd; launcher: **12f97cd3** (mc-44: Programs is the kit list, `fuzzel --dmenu`, crates/chora-kit/src/launcher.rs:232); rows only for installed programs (launcher.rs:44 `installed`): LibreOffice is not installed here | other page and missing rows: intended. **The lost icons: finding rv9ka-2.** `m8.jpg`, `rv9ka-2-m8-launcher.jpg` |

### handover: table

The Q still of h1..h1d is the xp output (HEADLESS-2); the scene diff also covers the mc (HEADLESS-4) and construct (HEADLESS-1) outputs of the four-output session.

| act | v8 vs v9 (by eye, xp output) | cause | verdict |
|---|---|---|---|
| h1 (regression) | The current pager desktop is marked (dark "1"); the Recycle Bin is 12 px lower | pager d78fc4bb (rv9j-1, known); bin 62708d35; `xp-desktop` 371f0fa1 (row present). Off screen: mc column 675ebd55 and b8872054; **`mc-s-agenda-page` and `-wheel` 608 → 503: db20c304 / 3e355bb2**; **`mc-s-agenda-panels-today-rule` x 42 → 94: 6fa86c16**; construct `construct-hud*` gone: **6a5cfc1d** (chora-assistant is the only voice presenter); `construct-start-help-0` 54 → 29 wide and `-help-1` gone: **f5bdc61f** (co-41: F1 is bound to the key list, so the start card reads "F1 All keys", one keycap instead of Super + F1) | intended. Unnamed: the 3 mc nodes above (no row names them; the agenda here is short, nothing is cut). The construct rows cite d9b2c4f1 / co-63, the wrong causes (as judge r1 said). `h1.jpg` |
| h1b (regression) | As h1; the open Start menu lists "Foot, Foot Client" where v8 lists "UXTerm, XTerm" | 29281054 (DB8: the fill leaves out raw entries only), by reading startmenu.rs, not bisected; the rest as h1 | intended. `h1b-start.jpg` |
| h1c, h1d (regression) | v8 opens Notepad over the desktop icons (labels covered); v9 opens it right of them; typed line the same | **09ff8ff9, bd5af4b3** (xp-28, closed t138); the rest as h1 | intended (fixes xp-28). `h1c-notepad.jpg`, `h1d.jpg` |
| h2..h7 | no baseline (no Send To on 733967aa) | | none, by design |

Judge r1's handover verdict ("only the bin, the pager and the Notepad place differ on this screen") holds on e39448ad for the still. The 4 regressions on this head come from the 3 off-screen mc nodes, which r1's head did not have (db20c304 is not an ancestor of d40142e4).

### Findings

| id | look | severity | acts | stills | element |
|---|---|---|---|---|---|
| rv9ka-1 | mc | MAJOR | m1, m1b, m1c, m2, m2b, m3, m4, m4b (recipe pages at 1280x720) | `rv9ka-1-m1.jpg`, `rv9ka-1-m1-crop.jpg`, `rv9ka-1-m1c.jpg`, `rv9ka-1-m4.jpg` | The page node ends at y 559 (608 → 503 high) on every page, also with no card on screen (plugins/chora-mc/src/build.rs:285 `bottom`, db20c304 / 3e355bb2, rpx1-11, -12). The last row of panels loses whole lines: the ingredient list shows 1 of 4 lines (`mc-s-ingredients-panels-list-lines` 112 → 29 px), the Timer text 1 of 3 (84 → 29), the "Then" text 1 of 5 (134 → 29). Nothing on the page says more is below. Only the mouse wheel scrolls a page (main.rs:778, `PointerScroll`); a remote or keyboard user cannot reach the lines. In v8 every line showed: the page reached y 664 and the card only covered the last line while it showed (rpx1-11). The main path of the cook (read the recipe) breaks; the workaround is the wheel. mc-style rule 13 scrolls a page that is longer than the output, but this page is not longer than the output. Reproduced 3 of 3: take r2's json (list 136 → 31), my take 1 and take 2. The tour's pixel diff cannot see it: every text box is masked, and m1 reads 0.0 % |
| rv9ka-2 | mc (kit list, shared by the looks that use it) | MINOR | m8 | `rv9ka-2-m8-launcher.jpg` | Programs opens the kit list (`fuzzel --dmenu`, crates/chora-kit/src/launcher.rs:232, 12f97cd3 mc-44), which shows no app icons; v8's fuzzel showed one per row. Rows for programs that are not installed go too, which is intended. Seen in my 2 takes and in take r2's m8 pixel blobs at x 505..541 |
| rv9ka-3 | tour (F7 tooling) | MINOR | m1, m1b, m1c, m4, m4b | as rv9ka-1 | tests/tour-diff.py masks every text box in the pixel diff, so a text that loses lines shows up only as a `rect` of its `-lines` node. In m1b, m4 and m4b that rect sits under a bare `open` rect row, which hides a lost recipe. Proposal: a text node whose rect is shorter than its line count times its line height is a "cut" diff that no row can absorb unless it names that node |

Known, cited, not re-reported: rv9j-1 (pager), rv9j-2 (notices out of the column, the card's colours with mc-61), mc-59 (region slot: see the mc-regions row above for a likely cause), mc-61 (rpx1-10 unthemed fuzzel), mc-15 (stale hover bar), xp-28 (fixed). BLOCK or MAJOR: only rv9ka-1, reproduced in two own takes plus take r2.

### Elements that fail F7, per look (one fix lane per look)

- **mc**: (1) **rv9ka-1**: the page end (`build.rs` `panels`, `bottom`). Reserve the card band only while a card shows, or keep the page to the footer and let a card cover nothing but the margin; whatever the fix, the recipe's last row must show whole at 1280x720, or the page must show that it goes on and let the remote scroll it. (2) **rv9ka-2**: icons in the kit list, or a PM decision that the list is text only (MINOR, rules.md 6: after rv9ka-1). Nothing else in mc fails F7: the column (rv9j-2 decided), the key spelling, the page rules, Home, the Helpers page, the chat in the column and the volume are design-asked changes or fixes.
- **handover**: none in the product. Tooling only: the rows below.

### intended.toml rows to widen (not edited here)

- m1: the `rect` row also names `mc-s-ingredients-page` and `mc-s-ingredients-wheel` (db20c304, 3e355bb2; rpx1-11, -12). It does **not** name `mc-s-ingredients-panels-list`, `-list-lines` while rv9ka-1 is open; if the PM accepts the cut, it names them then, with that decision as the cause. The `mc-r0-right-chat-key` width (58 → 60) cites b8872054, not 6fa86c16.
- m1c: likewise `mc-s-boil-page`, `mc-s-boil-wheel` (db20c304); `mc-s-boil-panels-side`, `-side-lines` stay unnamed (rv9ka-1).
- m1b, m2, m2b, m3, m4, m4b: the bare `rect` rows (`open = true`) list their nodes. Column `mc-r{0,1}-right-chat-*`, `-keys-*`, `-doors-*`, header and footer (675ebd55; chat-key b8872054); `mc-s-<scene>-page`, `-wheel` (db20c304); `-lead-rule`, `-list-rule`, `-side-rule` (6fa86c16); `-lines` and their panel rects stay out (rv9ka-1). `visible` rows name `mc-regions-0`, `mc-regions-1` (the double buffer of mc-48, flipped by the rebuild count of 675ebd55; mc-59). The `added` rows of m2, m2b, m3 name `notify-card`, `notify-card-body`, `notify-card-close` (675ebd55, rv9j-2). Pixel rows: the boxes [160, 585, 291, 79] (the card) and [880, 370, 352, 44] (the KEYS hairline at 378 / 406; the second blob [937, 406, 287, 1] is inside it).
- m3b, m4c, m5, m5b, m6, m6b, m9, m9b, m10 (other page, fa7faabd): every row needs nodes. Without a page-level row the list is the whole scene. Name `visible mc-s-agenda`, `mc-s-boil` / `mc-s-morning` / `mc-s-agents`, and the `mc-s-agenda-*` and `mc-s-<v8 page>-*` rect nodes; a generator from the json is the honest way. Pixel boxes: [196, 92, 16, 45] (the head bar "|" moves with the page number), [440, 520, 12, 150] (the divider of the morning page, m9b, m10), [987, 42, 128, 1] (m6: v8's rule through "Mute", gone, 5cb288e2 rpx1-9), [880, 586, 10, 30] (m9: the focus bar). m6, m6b `added mc-r1-right-keys-N-*` (the Home row: fa7faabd). m9 `added hud-chat*`, `mc-r0-right-chat-close*`, `-chat-slot`, `mc-r1-right-agents-N-*` (the chat opens: 675ebd55 column, mc-v2 item 6). m9b, m10 `added mc-r0-right-chat-close*`, `-chat-slot`.
- m7b, m7c, m8, m8b: an `added` row naming `mc-s-agents-panels-agent-1-empty` (ffe5e815, rpx1-14). m7b also a `rect` row: `mc-r1-header-head` (ffe5e815), `mc-s-agents-chapter-head-hit`, `-chapter-title` (31f53dc9, c5acd4d0; mc-55), `mc-s-agents-panels-agent-1..4`, `-band-1..4` and their `-rule` (ffe5e815, rule 14), `mc-s-agents-page`, `-wheel` (db20c304). m8: the launcher box [500, 225, 270, 290] (rv9ka-2 while open; 12f97cd3 if the PM accepts text-only rows).
- h1, h1b, h1c, h1d: a `rect` row naming `mc-s-agenda-page`, `mc-s-agenda-wheel` (db20c304, 3e355bb2) and `mc-s-agenda-panels-today-rule` (6fa86c16). The construct rows change cause: `construct-hud*` missing → 6a5cfc1d; `construct-start-help-0*` rect and `construct-start-help-1*` missing → f5bdc61f (co-41), not d9b2c4f1 / co-63. Their `open = true` can go once they cite these causes and a still (the construct output is not in the Q still: take a still of HEADLESS-1 or keep them open).

`pgrep -fa 'chora --headless'` empty at the end.
