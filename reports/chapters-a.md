# chapters-a

## r1 (lane 4 of design/tour-v9.md section 8, F7, RC 11; 2026-11-21)

Branch `claude/chapters-a` from f4f49783. Red `bc8cf213` (tests/test_tour.py only), green `8435d386` (head, pushed). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` ok.

**Red first.** `ChaptersARows` in tests/test_tour.py: every `t.act` of xp, tiling, handover has an `expect` and a `contract` or `cells` (no bare `inv` row, read from the AST of the chapter files); no `todo` stub; one planted wrong state per chapter that the chapter's own check turns red (xp: a mixer at 40 % for 75 %; tiling: one window for four; handover: a node owned by xp for mc). On bc8cf213: 5 failures (rows without `expect`, the `todo` row h2, three missing checks). The planted tests of the red commit had two slips of mine in the test itself (the local `snap()` has no window, and shadows `test_acts.snap`); the green commit fixes them and adds three more planted cases (letter text and Run box, tiling overlap and unequal gaps, lock call and lost window). 28 tests ok on 8435d386.

**What the rows are now** (31 + 28 + 11 = 70 acts, 0 `inv`, 0 `todo`). Each `expect` is an `Expect` of the act harness on the snapshot (`Diff`, a small subclass in `chapters/xp.py`, also reads the read before the act, for "moved" and "layout changed"). No new gate cell, no new dependency, no change to tour.py.
- xp: `letter_is` (letter.txt equals the typed text), `volume_is` (the stand-in wpctl holds 75), `tooltip_says`, `notice_is_new` (screenshot), `box_says` (the Run box listed `Nothing found for "nosuchprogram"`: a `fuzzel` in front of the real one keeps the rows), `logged` (the question is in the calls of the fake voice, the Yes in its answers), `helper=` words (idle, working, answered), `shown`/`gone` (chat, card), `desk_is` (cube), `looks_xp.pane_says` and `key_table_whole` (Help). New rows: x8 Print Screen, x9 Run box and Escape, x10 Help and the topic Keyboard shortcuts and Escape, x11 Turn Off Computer and Cancel, a helper-works row.
- tiling: `windows_are`, `tiles_hold` (n tiles, no overlap, one gap), `master_is_focused`, `master_wider`, `layout_changed`, `on_tag`, `floating_on_top`, `moved`, `ran` (the shell's HISTFILE holds `ls`, `cat friday.md`, `python3 main.py`), `file_has` (demo/main.py), `probe_heard_no_key` (T5: a chora-probe window beside the editor, focused, its report has no `key` line of d, x, m), `box_says` and `rows_have` (Apps, power menu), `volume_above`, `chat_beside`. New rows: t2 stray keys, Super+F1 and Escape (t6), Apps and `frobnicate` then Foot (t8), chat (t9), volume scroll (t10), Super+Print notice with chora-notify as a companion.
- handover: `owner_is`, `focus_with`, `windows_only` (one window, none lost, T7), `shown_in`, `no_task_button`, `lock_called` (stand-in `swaylock` and `loginctl` log their calls). The rows are the design's h1 to h7: Notepad and the first line, Send To > Tiling desktop by the title-bar menu (h2 is not a stub any more: xp-give is in), the second line, Super+Alt+3 to mc, Super+Alt+4 to the construct, the pane dragged onto xp, the audit of all lines, Super+Shift+Escape. The five v8 drags and the two give-key rows are gone (the gates keep them).

Rows per chapter (`contract`; `expect` shortened; the `known` column is the finding id):

**xp** (31 rows)

| # | act | contract | expect | known |
|---|-----|----------|--------|-------|
| 1 | Erika sits down at the xp desktop | X1 | `focus="xp", verify=clock_shown` |  |
| 2 | Start, then Notepad | T1 | `open=[], verify=notice_shown` |  |
| 3 | No text editor is installed: Notepad says so | T6 | `open=[], verify=notice_shown` |  |
| 4 | Start, Notepad again, now with a stand-in GUI editor on PATH | T1 | `new_window={"owner": "xp", "focused": True}, open=[]` |  |
| 5 | Types a short letter, one typo corrected, saves with Ctrl+D (sta | - | `verify=letter_is(TEXT)` |  |
| 6 | Drags the letter window aside by its title bar | X2 | `open=[], delta=moved("xp")` |  |
| 7 | Clicks the paperclip on the desktop (node hud-figure): she wants | T3 | `helper="idle", verify=shown("hud-chat-close")` |  |
| 8 | Clicks the field and types her question, sends it with Return | T3 | `verify=logged(f"{claims}.chat", ASK)` |  |
| 9 | The helper works on it | T3 | `helper="working"` |  |
| 10 | She reads the answer | T3 | `helper="answered", verify=shown("hud-chat-close")` |  |
| 11 | Closes the chat with its X button | T3 | `verify=gone("hud-chat-close")` |  |
| 12 | Start, Control Panel, Sound, 75% | T4:volume | `open=[], verify=volume_is(75, wpctl_state)` |  |
| 13 | The pointer rests on the speaker: the tray says 75% | T4:volume | `verify=tooltip_says("Volume 75%")` |  |
| 14 | Murat presses Print Screen (Super+Print) | T4:screenshot | `delta=notice_is_new` |  |
| 15 | Start, Run..., types a name that is no program | T6 | `open=[], verify=box_says(L.dir, NOSUCH)` |  |
| 16 | Escape shuts the Run box | T6 | `open=[]` |  |
| 17 | Start, Help and Support | T2 | `open=[looks.HELP], verify=looks.pane_says("Click Start in the bottom left corner ...` | ca-2 |
| 18 | Clicks the topic Keyboard shortcuts | T2 | `verify=looks.key_table_whole` | ca-1 |
| 19 | Escape shuts Help | T2 | `open=[]` |  |
| 20 | Start, Turn Off Computer | T4:power | `open=["xp-session"]` |  |
| 21 | Murat thinks again and clicks Cancel | T4:power | `open=[]` |  |
| 22 | Opens the paperclip again and asks the helper to keep the letter | T3 | `verify=logged(f"{claims}.chat", ASK2)` |  |
| 23 | The helper asks before it touches a file | T3 | `helper="answered", verify=shown("hud-card-yes")` |  |
| 24 | Erika clicks Yes | T3 | `verify=both(gone("hud-card-yes"), logged(f"{claims}.answers", '"yes": true'))` |  |
| 25 | She reads that it is done | T3 | `helper="answered", verify=shown("hud-chat-close")` |  |
| 26 | Closes the chat again | T3 | `verify=gone("hud-chat-close")` |  |
| 27 | Clicks the paperclip icon in the tray (xp-helper) | T3 | `verify=shown("hud-chat-close")` |  |
| 28 | Closes the chat with its X button | T3 | `verify=gone("hud-chat-close")` |  |
| 29 | Ctrl+Alt+Right: turns the xp desktop cube by key | - | `verify=desk_is(2)` |  |
| 30 | Ctrl+Alt+Left: back to the first desktop | - | `verify=desk_is(1)` |  |
| 31 | Super held, drags on the desktop to turn the cube | - | `verify=desk_is(2)` | ca-4 |

**tiling** (24 rows)

| # | act | contract | expect | known |
|---|-----|----------|--------|-------|
| 1 | The hacker looks at the empty tiling desktop | L2 | `verify=windows_are(0)` |  |
| 2 | Super+Return: a shell | T1 | `new_window={"owner": "tiling", "focused": True}, focus="tiling", verify=tiles_hol...` |  |
| 3 | Types ls and cat friday.md, one typo | T1 | `verify=both(ran(history, "ls"), ran(history, "cat friday.md"))` |  |
| 4 | Starts a small project: mkdir, a Python file in nano, run it | T1 | `verify=both(file_has(f"{NOTES}/demo/main.py", "print('build', n)"), ran(history, ...` |  |
| 5 | Super+d and two more Super chords while a window of a program li | T5 | `verify=probe_heard_no_key(report, (32, 45, 50))` |  |
| 6 | Super+Return ({name}) | T1 | `new_window={"owner": "tiling", "focused": True}, verify=both(windows_are(i), tile...` |  |
| 7 | Super+j moves focus down the stack, Super+Ctrl+Return makes it t | L2 | `focus="tiling", verify=both(master_is_focused, tiles_hold(4)), delta=layout_changed` |  |
| 8 | Super+l widens the master | L2 | `verify=tiles_hold(4), delta=master_wider` |  |
| 9 | Super+Shift+space (next layout) | L1 | `focus="tiling", verify=tiles_hold(4), delta=layout_changed` |  |
| 10 | Super+Shift+2 sends the focused window to tag 2, Super+2 views i | L3 | `focus="tiling", verify=both(on_tag(2, 1), windows_are(3))` |  |
| 11 | Moves onto a window, Super+Ctrl+space floats it | L2 | `verify=floating_on_top` |  |
| 12 | Super+drag moves the floating window | L2 | `verify=floating_on_top, delta=moved("tiling")` |  |
| 13 | Super+F1 opens the key list | T2 | `open=["tiling-keys"]` |  |
| 14 | Escape closes it | T2 | `open=[]` |  |
| 15 | Super+slash opens the key list once more | T2 | `open=["tiling-keys"]` |  |
| 16 | Escape closes it | T2 | `open=[]` |  |
| 17 | Super+Print: a screenshot of the rice | T4:screenshot | `delta=notice_is_new` |  |
| 18 | The beginner clicks Apps in the bar, the visible door to program | T6 | `verify=box_says(L.dir, "frobnicate")` |  |
| 19 | Types Foot in the launcher and starts it | T1 | `new_window={"owner": "tiling", "focused": True}, verify=windows_are(4)` |  |
| 20 | Super+a: the chat of the helper beside the tiles | T3 | `helper="idle", verify=both(shown("hud-chat-close"), chat_beside)` | ti-29 |
| 21 | Escape closes the chat | T3 | `verify=gone("hud-chat-close")` |  |
| 22 | Scrolls on the volume word of the bar | T4:volume | `verify=volume_above(40, wpctl_state)` |  |
| 23 | Clicks the power icon at the right end of the bar | T4:power | `verify=rows_have(L.dir, ["Lock", "Log out", "Restart", "Shut down", "Screenshot"])` |  |
| 24 | Clicks the row Screenshot | T4:screenshot | `delta=notice_is_new` |  |

**handover** (11 rows)

| # | act | contract | expect | known |
|---|-----|----------|--------|-------|
| 1 | Two screens: xp on the left, tiling on the right (empty) | X1 | `verify=windows_only()` |  |
| 2 | Click Start | T1 | `open=["xp-menu"]` | ca-3 |
| 3 | Click Notepad | T1 | `new_window={"owner": "xp", "focused": True}, open=[]` |  |
| 4 | The nurse types the first line in the editor | T1 | `verify=letter_is(LINE1)` |  |
| 5 | Title-bar right click, Send To, Tiling desktop | T7 | `focus="tiling", verify=both(owner_is("s1", "tiling"), no_task_button("s1"), windo...` |  |
| 6 | The nurse types a line in the editor on its new screen | T7 | `verify=both(letter_is(both_lines), owner_is("s1", "tiling"))` |  |
| 7 | Super+Alt+3 gives the window to mc | T7 | `focus="mc", verify=both(owner_is("s1", "mc"), shown_in("mc", "s1"), windows_only(...` |  |
| 8 | Super+Alt+4 gives the window to the construct | C2 | `verify=both(owner_is("s1", "construct"), windows_only("s1"))` |  |
| 9 | Super+drag the pane of the editor from the construct onto the xp | C2 | `verify=both(owner_is("s1", "xp"), windows_only("s1"))` |  |
| 10 | xp has the window with all the lines | T7 | `verify=both(owner_is("s1", "xp"), windows_only("s1"), letter_is(both_lines))` |  |
| 11 | Super+Shift+Escape locks every screen | T4:lock | `verify=lock_called(calls)` |  |

`cells` are on every row as in the design (not repeated here; see the chapter files).

**`demo/tour/run.py --check xp tiling handover`, twice** (8435d386, this container, software rendering; both runs print the same):

| chapter | acts | green | inv | known | red | todo |
|---|---|---|---|---|---|---|
| xp | 31 | 28 | 0 | 3 | 0 | 0 |
| tiling | 28 | 27 | 0 | 1 | 0 | 0 |
| handover | 11 | 10 | 0 | 1 | 0 | 0 |

`tour: check: 0 red, 0 broken` both times. Before (tour-one r2): xp 2 green and 18 inv, tiling 0 and 21, handover 1 and 8 with 1 todo. The first runs with the new `expect`s found real reds that are fixed in the chapter, not hidden: the chat field was clicked at (1070, 672), a pixel of 1280x800, and on the 1280x720 output that is the desktop, so the typed question never reached the helper (xp acts 8 and 22 were `inv`-green for months); the helper stayed `working` after its answer (now `done`, helper word `answered`); `Super+l` did nothing in the fair layout (the master acts moved before the layouts); the handover file letter.txt held the xp chapter's text (truncated at the start); the "Foot" act counted a window hidden on tag 2.

**Known rows** (`known=` names the invariant or `expect`, and the finding id; any other red of the row stays `RED`):
- **ca-1** (new), xp "Clicks the topic Keyboard shortcuts", `expect` and I6 (and I5): at 1280x720 the key list of Help ends below its pane (the row `Mute` at 643 to 657 in a pane of 645 high); the acts hold it at 1280x800 only. Product: the Help window at a 720 high output.
- **ca-2** (new), xp "Start, Help and Support" and the topic act, I5: the dialog Help, a picture, lies over the title of the window under it, and I5 counts a picture as not opaque (tests/acts/invariants.py "Limits"). A harness limit, not a product fault; the fix is in the invariant (a picture that is a dialog hides the text under it), which is lane GUIDE-CHECKS.
- **ca-3** (new id for the known I3 of tour-one), handover "Click Start": construct and xp each say they hold the focus on a four-output session. The first act of the chapter is green now (it was `known` in tour-one r2): "drop known" printed for it, so only the second act keeps it.
- **ca-4** (new id for the known I4 of tour-one), xp "Super held, drags on the desktop": the Start menu opens when Super is released after the drag. The cube turn itself is now judged (`desk_is(2)` holds with a drag of 800 px; 400 px snapped back).
- **ti-29** (existing, p2-r2 and p2-r3 tiling), tiling "Super+a: the chat of the helper beside the tiles", `expect`: the chat covers the tiled windows s5 and s1; design t9 holds that it stands beside them.

**Observations, not findings of this lane**
- T5 holds only the last key of a chord (check.py `verify_probe`); the probe window does get `key 125` (the Super key itself) for each chord. Left as the contract has it; a reviewer may want it a finding.
- The tour's stand-in `wpctl` (launch.py) reads `$3` as the level, so a scroll of the waybar pill (`set-volume -l 1.0 @DEFAULT_AUDIO_SINK@ 5%+`) sets 100 %. The tiling row only asks that the level went up; launch.py is not in this lane's files.
- Handover: `Expect(focus="tiling")` and `focus="mc"` hold on the four-output session; only the first Start click shows I3.
- h5 is the Super+drag of tour v8 (construct pane onto the xp output), not the plain drag of design c5; the gate cell `gate-construct-pull.sh drop` is named on the row.

**Gate** `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,gate-lib,build,test,acts`: pre-commit PASS (1 s), ci-check PASS (93 s), build PASS (30 s), test PASS (204 s), acts PASS (287 s, includes tests/test_tour.py, 28 tests). **gate-lib FAIL (86 s) and the gate stops there**, so ci-check to acts ran in a second call `--only ci-check,build,test,acts` (all PASS). The gate-lib failure is the container, not the diff: `tests/merge-gate-selftest.sh` runs `tests/ci-local.sh` with a PATH of seven symlinked tools, and `command -v rm` here is the rm guard of the harness (a shim under `~/.claude/rm-guard`), which prints `rm: command not found` when its own PATH is gone, so the log directory stays: three checks ("an unknown job leaves no log directory", "--help leaves no log directory", "a run without a failure removes its log directory": got 4, 4, 6, want 0). Reproduced by hand on this checkout without any of the lane's files in play (`PATH=nobin TMPDIR=... tests/ci-local.sh deny machete pre-commit` leaves `ci-local.*`, prints the rm error). I did not take the shim out of the PATH to get it green (a guard of the harness); on a host without it the cell should pass. The diff touches only demo/tour/chapters/{xp,tiling,handover}.py and tests/test_tour.py.

**Not done**
- No recording (`run.py` without `--check`) in this lane: the stills `01-10c-run`, `01-10d-help` and so on are cues for RECORD; the Run box and the power menu are real fuzzel windows (a wrapper keeps their rows), so RECORD sees them on frames. Whether the Escape of the Run box shuts fuzzel on a frame is judged by the next row's `open=[]` only (a layer surface is not in the scene).
- The mc and construct chapters are CHAPTERS-B.

## Review r1 (review-chapters-a-r1; read-only; head 8435d386, red bc8cf213, base f4f49783)

Verdict: **FIX** (one MAJOR open on the brief's own rule for ca-2, one MAJOR on a vacuous expect). Findings: 2 MAJOR, 3 MINOR, 0 BLOCK.

Checks: (1) red first holds: `python3 -I -m pytest tests/test_tour.py -q` is 5 failed/20 passed on bc8cf213 and 28 passed on 8435d386. (2) 10+ expects read across the three chapters; letter_is (letter.txt), volume_is/volume_above (wpctl state, starts 0.40), tiles_hold (count, overlap, equal gaps), owner_is, windows_only, lock_called, probe_heard_no_key (refuses a vacuous unfocused probe) test the state the act claims; the planted-state tests in tests/test_tour.py cover xp volume, tiling count and gaps, handover owner and lock; `notice_shown` does not (rca-1). (3) ca-1, ca-3, ca-4, ti-29 each name a product fault visible in the run; ca-2 is a harness limit (rca-2). (4) `demo/tour/run.py --check xp tiling handover` on 8435d386: xp 31 acts 28/0/3/0, tiling 28 acts 27/0/1/0, handover 11 acts 10/0/1/0, `0 red, 0 broken`: counts match the report. (5) Stand-ins (fuzzel, foot with HISTFILE, mousepad, swaylock, loginctl, wpctl) are written to `{L.dir}/stub`, the PATH of the tour's own session env only; no product file and no user PATH is touched. (6) No new dependency, no new gate cell, tour.py and launch.py untouched.

- rca-1, MAJOR, demo/tour/chapters/xp.py:175 (used by acts 2 and 3): `notice_shown` passes when chora-notify has any notice in `shown` or `waiting`, so act 3 ("a notice names the missing editor mousepad") is green with a stale notice of act 2 or any other text. Fix: compare the notice text with the editor name (and, for act 2, that no notice is new).
- rca-2, MAJOR, demo/tour/chapters/xp.py:316 and :321: ca-2 is declared a harness limit (I5 counts a dialog picture as not opaque) and hides I5 on two acts although the report states the fix (a picture that is a dialog hides the text under it in tests/acts/invariants.py "Limits"). Fix: land that invariant change (GUIDE-CHECKS) or write it here, and drop `ca-2` from the two rows; if left, name the lane and sha that owns it.
- rca-3, MINOR, demo/tour/chapters/tiling.py:87: `volume_above(40)` is true after any scroll, and the stand-in wpctl (launch.py) sets 100 % for a `5%+` step, so "goes up by one step" is not judged; Fix: have the stand-in honour `N%+` and expect 45.
- rca-4, MINOR, demo/tour/chapters/xp.py:117 (`logged`), :166 (`box_says`), :152 (`rows_have`): they search the whole log of the session, not what the act added, so a repeated question or box text from an earlier act would pass a later one; the strings differ today. Fix: read the line count before the act, or use a unique string per act.
- rca-5, MINOR, demo/tour/chapters/xp.py:20-175: `Diff`, `both`, `shown`, `gone`, `logged`, `moved`, `fuzzel_log`, `box_says`, `rows_have`, `volume_is` are imported by tiling.py and handover.py from the xp chapter. Fix: move to demo/tour/chapters/checks.py (or leave and note it).
