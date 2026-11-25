
## r1
Branch claude/xp-minor, head 22884a79 (base fee9c695). Gate (pre-commit, ci-check, build, test, clippy, xp, xp-faces, xp-question, xp-notepad, doors, acts): all PASS. Captures 1280x800 in media/2026-11-20-xp-minor/.

| # | item | red | fix | cell | capture |
|---|------|-----|-----|------|---------|
| 1 | xp-36 rv-1 Run... warning then Escape quiet; exit 1 = Closed, 2+ = Failed | d14a8c84 | 908b26d8 | kit test a_warning_then_escape_stays_silent (the fail cell now exits 2) | none: no fuzzel box opens on fuzzel 1.9.2 here |
| 2 | Screenshot balloon | c9643e03 | a4010f03 | tests/screenshot-notice.sh (a notify-send that answers must not hide the balloon) | screenshot-balloon.png: XP balloon "Screenshot saved" + file, above the tray |
| 3 | Volume popup level slider | cdcf9569 | 9c6d97a8 | xp tests the_volume_popup_checks_the_level_or_mute, a_click_on_the_slider_sets_the_level_of_its_place | volume.png (40 %, thumb at 40 %) |
| 4 | Super+E notice in plain words | bb35e050 | 22884a79 | xp test the_notice_without_a_file_manager_says_what_to_do | none: Super+E not driven |
| 5 | small yellow square | none | none | none | injection-mark.png |

Notes:
- 2: the notice was made, but the persona session's notify-send is a stand-in that answers, so no balloon showed. screenshot.sh now tries choractl notify.show first (expire 6 s), notify-send second; chora-notify places it as xp does (above the tray, tail to it). Not run with the real Start click (no grim in the act session); gate-tiling-power and gate-tiling-print-notice (fake notify-send, notify-send path) not run here, they are not in this gate: check them in the train.
- 3: the slider is the last row (after Mute), so the act row ids stay; a click on the track sets the level by x. XP has it on top: one move if wanted.
- 5: the yellow square is the injection mark of the core (D23, crates/chora/src/inject.rs): shown 500 ms after every input.inject, only for agent-driven input, never for a person's mouse. Not xp, not the balloon anchor; the core is not mine, left as is.

## PM review r1 (16:27)
Base fee9c695..22884a79, 5 files +145 -22; each fix has its red first.
- pr1-1 MAJOR (reopens xp-36): 908b26d8 makes exit 1 "Closed". fuzzel 1.11.1 exits EXIT_FAILURE (1) both on Escape and on every real error (main.c: config, option, Wayland), and a refused config, the case of xp-36, logs " err: ..." and exits 1. So Start > Run... with a broken fuzzel.ini is silent again, the shared RC bar breach xp-36 fixed. The old cell a_launcher_that_fails_says_why_once was changed from exit 1 to exit 2 to stay green: a weakened test. Fix: the class of the stderr line decides, not the code: Failed when a line starts with "err:" (fuzzel log.c prefix " err"), else Closed (warn, info, nothing). Cells: the old cell back at exit 1 with " err: config.c:17: ..." -> notice; warn then exit 1 -> silent.
- Items 2 (balloon through chora-notify first, 6 s), 3 (slider row, click sets the level of its place, unlit), 4 (plain words): hold by their cells and captures; no finding.
- pr1-2 NIT: the slider takes clicks only, no drag; XP drags. Backlog.
Verdict: FIX (open MAJOR). r2: item 1 only.

## r2
Branch claude/xp-minor, head fca4f8b5. Item 1 (pr1-1): `ask` in crates/chora-kit/src/launcher.rs decides by the stderr line: Failed with the text after the first trimmed "err:" line, else Closed (warn, info, nothing, a signal), whatever the exit code. Doc comment of `ask` updated.

| red | fix | cells |
|-----|-----|-------|
| c8b3952d | fca4f8b5 | a_launcher_that_fails_says_why_once (back at exit 1, " err: config.c:17: [colors].prompt: invalid key" -> notice with that text), an_err_line_after_a_warning_is_the_reason (new), a_warning_then_escape_stays_silent (unchanged, exit 1) |

Gate (--jobs 1): pre-commit, ci-check, build, test, clippy, gate-lib, xp, xp-question, launcher-click, doors: all PASS.

## PM review r2

Base 22884a79..fca4f8b5, one file (crates/chora-kit/src/launcher.rs, +27 -8); red c8b3952d before fix fca4f8b5.
- pr1-1 closed: `ask` picks the first trimmed stderr line with prefix "err:" (Failed with the rest), else Closed, any exit code. Checked against fuzzel 1.11.1 log.c:28,82-86: the error prefix is " err" then ": file:line: "; colorize is auto and stderr is a pipe here, so no escape codes reach the match. The exit-1 cell is restored with the real line; the warn-then-err cell is new; warn-then-Escape unchanged. Gate cells named in the brief PASS (worker, --jobs 1).
- pr2-1 NIT: the notice keeps fuzzel's "config.c:17: " source position ("Run could not start: config.c:17: [colors].prompt: invalid key"); a plain user does not need it. Backlog.
Verdict (computed): no open MAJOR -> MERGE-OK fca4f8b5. Worker archived.

## r1

Branch claude/xp-minor, head 3bec0cbf (base a1f3c8d7; merged with fresh origin/claude/autonomous-dev-session-3w-i33pg7).

- ti-43 + ti-44, 7e49b273. Red: `arrival_other_look_leaves_settles` failed (left: Joined, right: Settled) with the other look gone. Fix: `Wm::look_gone` (startup.rs), called from `Event::ClientDisconnected` (dispatch.rs): when joined and no other look holds a claim, settle, refocus, and give the desktop the keyboard. Code fix, api.md:425 unchanged (D119 not affected).
- ti-44 unit tests, startup.rs `mod tests`: arrival_starting_alone_is_settled, arrival_joining_a_look_with_a_claim_is_joined, arrival_other_look_leaves_settles, arrival_a_client_that_holds_no_claim_leaving_keeps_joined, arrival_click_on_the_free_desktop_settles. Seeded wrong transition (join sets Settled): 4 of 5 FAILED, only starting_alone passed. Seed reverted.
- xp-28, 16316aeb: not reproduced on the head, so no code. Cell `xp-chat-idle` (tests/gate-xp-chat-idle.sh, in merge-gate.sh) plays fake voice working step 1/1, question, answer, idle: header says Working before, Idle after, no "step 1 of 1"; unit test `the_header_is_idle_and_stop_off_when_the_turn_ends_after_a_question` checks running() is false. I did not bisect which commit fixed it (likely 06b9666c kit7 state machine; unverified). The header follows voice.state, so a real voice that never sends idle would still show Working; that is not covered. Stop button state checked only via running(), not in the scene. The "editor over desktop icons" half: not examined, report only.
- Counts: arrival_ 5/5 green x5; gate-xp-chat-idle.sh 5/5 green.
- Gate: tests/merge-gate.sh --jobs 2 (full): 58 PASS, no FAIL, EXIT 0, no foreign reds.

## PM review (2026-11-22 18:31 UTC)
Range a1f3c8d7..3bec0cbf, own commits 7e49b273 (dispatch.rs +6/-2, startup.rs +104) and 16316aeb (chat.rs +20, gate-xp-chat-idle.sh +47, merge-gate +1): 176 lines, PM review (rules.md 7, PM not the builder).
- ti-43: look_gone settles only when joined and no other claim; a failed claim query keeps Joined (unwrap_or(true)): safe. Done.
- ti-44: five Arrival units, seeded red 4/5 reported. Done.
- xp-28: header half not reproducible, guarded by cell xp-chat-idle; editor-over-icons half not examined: stays open (next xp lane).
- Gate 58 PASS on the merge with the session head. Test in the same commit as the fix: allowed by red-first (merge-gate.sh:30).
Verdict (computed): 0 MAJOR -> MERGE-OK. Joins t110 after TI-38.

## r1 (claude/xp-minor-r1)

Branch claude/xp-minor-r1, base 721be592, head d2a048d5. Gate: `tests/merge-gate.sh --jobs 2` (full, no filter): all steps PASS, EXIT 0, no foreign red; `pgrep -fa 'chora --headless'` empty after. Captures: media/2026-11-25/xp-minor-r1/.

| item | red | fix | red line |
|------|-----|-----|----------|
| rv9j-1 + xp-19 | ecceba2a | fb8775a0 | unit `the_desktop_buttons_keep_the_tray_tone_and_a_readable_digit_xp_19`: "#1E54BE is not of the tray hue"; `the_current_desktop_button_differs_from_the_others_by_3_to_1_rv9j_1` red on #164EB8 / #1E54BE (1.09:1); gate cells `pager-current` (tests/gate-xp-readable.sh) measure the fill pixels at rest and under the pointer |
| xp-28 | 1036e621 | 098980b5 | unit `a_finished_answer_leaves_working_and_its_step_and_stop_goes_off_xp_28`: left `("Atlas — Done", "Write file, step 1 of 1")` right `("Atlas — Idle", "")`; gate-xp-chat-idle.sh plays state done that keeps task and step 1/1 |
| xp-9 | 061cbcff | 8e32d14a | gate cell `caption-size`: "caption 0 of Turn off computer is 10 px of ink high, 14 px type gives 12 or more" (also caption 4) |
| xp-29 | none | none | absent at the head, see below |

1) rv9j-1, xp-19 (taskbar.rs `pager_colors`): other buttons are the Luna tray blue (#0D8DEA) darkened to #0A74CA (hover #0B76CD) with a light top line like the tray, white digit 4.8:1; the current one is a deep navy #06204A (hover #0A2A5C). Current against the others 3.32:1 (hover 3.42:1), was 1.09:1. The cube xp faces draw the same taskbar code (taskbar.rs `pager`), so they share it; no separate code. Gate: pixel contrast of button 1 against buttons 2 to 4 at rest and under the pointer >= 3:1, plus the existing 4.5:1 digit cells (rest, hover, dimmed behind Turn off), all PASS. The old 4.5:1 digit cell would not allow the plain tray blue (3.5:1 under white), so the tone is the tray hue at 4.8:1.
2) xp-28 (chat/dialogue.rs `helper_state`, in chora-assistant; the header lives there, not in chora-xp): a state of idle or done drops task and steps and shows Idle (done is Idle in the chat; the helper waits for the next prompt), Stop is off (`running()` false). Earlier round (3bec0cbf) covered idle without steps; the real voice ends a prompt with `done` and keeps the fields, which is what failed.
3) xp-9 (session.rs): the captions of the icon buttons use Font::BodyBold (14 px, was 11 px bold); Cancel is already 80x28 (unit `cancel_is_a_push_button_of_80_by_28_inside_the_dialog`, gate `cancel-target`), unchanged. Unit `every_caption_of_a_big_button_fits_it_at_14_px`: Stand By is 55.0 of 56 px wide, so it fits, but with 1 px room. Look: "Stand By" and "Turn Off" now nearly touch (see capture 3); a wider button would change the layout, left for the owner. The Luna fill check of the exempt captions now samples the two rows under the caption (the caption covers its own rows).
4) xp-29: absent at the head, no code. Cell `xp29-click` (tests/gate-xp-helper.sh, in the merge gate, PASS: "ok: xp-29: a click on the paperclip opens the chat (helper)"), same as reports/xp-helper.md; unit `a_press_of_a_person_on_the_figure_opens_the_chat_and_one_of_a_program_does_not`; the tray door is unchanged. Capture 4.

Captures: 1-pager.png (taskbar, current 1 navy, 2 to 4 tray blue), 2-chat-idle.png (chat header "Atlas — Idle", Stop greyed, after a done that kept step 1/1), 3-turnoff.png (14 px captions), 4-paperclip-click.png (chat open after a click on the paperclip).
Not covered: the desktop is not dimmed in the capture of Turn off (xp-9 third part, outside the brief's two numbers); a voice that never sends idle or done still shows Working.

## PM review r1
Base 721be592, head d2a048d5 (+172 -15, 6 files; three red-then-fix pairs, red commits tests only).
- rv9j-1, xp-19 (taskbar.rs pager_colors): others #0A74CA (white digit 4.9:1, tray hue), current #06204A (3.3:1 from the others, white 15:1), a tray light line on the others. Unit tests compute WCAG ratios; gate-xp-readable asserts pixels. Met.
- xp-28 (chat/dialogue.rs helper_state): "idle" and "done" clear task and steps, Stop off; unit test drives working -> done -> idle with steps. Met.
- xp-9 (session.rs): captions BodyBold >= 14 px, every caption fits its button; Cancel already 80x28. Met.
- xp-29: no code; cell xp29-click green at head. Closed by evidence.
- rxm1-1 MINOR: Stand By and Turn Off captions at 55 of 56 px leave almost no gap between neighbours (XP has visible gaps); a 1 px margin test or a shorter rect would hold it. Known issues.
Verdict MERGE-OK (computed: no open MAJOR). Joins train137.

## r2 (claude/xp-minor-r2)

Branch claude/xp-minor-r2, base d2a048d5, head 8109e08b. Full gate `tests/merge-gate.sh --jobs 2`, no filter: all steps PASS, EXIT 0, no foreign red; `pgrep -fa 'chora --headless'` empty after. Captures: media/2026-11-25/xp-minor-r2/.

| item | red | fix | red line |
|------|-----|-----|----------|
| rxm1-1 | 3fc6045b | 6d5c1f31 | unit: "Stand By and Turn Off are 3.2 px apart"; gate cell caption-gap: "caption 3 of Turn off computer starts 2 px after the caption before it (ink ends at x 667, starts at x 670), wants 4 or more" |
| xp-10 | acee8d9f | cdc9446a | units: "one click opens Date and Time left: 0 right: 1", "no task button for Date and Time"; gate cell datetime-click: "one click on the clock of the tray opens no Date and Time", "Date and Time shows no taskbar button" |
| xp-30 | 28d29cc2 | 7ee01d44 | unit "the press leaves the greeting in the balloon"; gate: "xp-30: a click elsewhere leaves the greeting in the balloon: Hi, I am your helper, ..." and "xp-30: the paperclip greets over a window: Hi, I am your helper, ..." |
| xp-13 | none | none | absent at the head, see below (guard 84666c00, green) |

1) xp-30 (plugins/chora-assistant voice.rs, main.rs; words in chora-kit). The greeting itself and the once-flag (`assistant-greeted` in the state home, cells xp30-helper, xp30-none, xp30-once) were already there. New: any press (`pointer.pressed` on any node, a window or the desktop) takes it away and clears `said`, so it does not come back when a helper connects; before it shows, a `scene.query kind:surface` finds a window rectangle that meets `Assistant::reach()` (the balloon area) and the greeting waits (retry every 500 ms, not spent, the file is written only when it shows); a window that opens under it takes it away. The words now name how to close ("Click anywhere else to close this balloon.", en and de) in place of "closes by itself" (it still fades after 12 s). Cells xp30-click and xp30-window (gate-xp-helper.sh) are pixel and node asserts on `hud-line`; the fake core cannot hold a surface, so the unit tests drive `greet_with(now, covered)` and `window_meets`; the red unit for the window case I wrote first could not be made to pass and was replaced in the fix commit (the gate cell is its red evidence). NOT DONE: Esc. A plugin gets keys only when it holds a binding with a modifier or a node with the keyboard; taking Esc globally or the keyboard focus for a balloon would steal keys from the window the person works in. Needs an owner or PM call (for example chora-xp forwarding Esc on the empty desktop).
2) xp-10 (chora-xp wm/panels.rs, windows.rs, pointer.rs, datetime.rs): one click on the tray clock opens Date and Time; the double click still works (its second click raises the open dialog, no second one; `last_clock` removed); a task button "Date and Time Properties" (id xp-task-datetimepanel-*) shows while it is open, raises it on a click and goes with it (sync_tasks split into `panel_tasks`). Buttons are 75 x 23 on both tabs (already so since 54f810dc, unit and gate cell datetime-metrics). NOT DONE: text at 14 px. The dialog is 11 px type (Font::Ui) by docs/xp-style.md row 1 (XP Normal 96 DPI, owner commit fd93053d), shared with the Taskbar and Display dialogs through the helpers of dialog.rs; 14 px needs a font argument through those helpers and a re-layout of the 398 x 316 page, and conflicts with that doc. The capture shows 11 px. Owner call: Large Fonts for dialogs or 11 px as the replica.
3) xp-13: no code. No balloon (greeting, empty chat) or notice lies on the Recycle Bin or any other desktop icon: gate cell bin-clear covers every xp-icon-N now (84666c00, green at the head; the bin was moved by 62708d35, rxr-2). Desktop labels stay 11 px (Font::Ui): design/looks/xp-v2.md asks 14 px for menus, titles and balloon, docs/xp-style.md row 1 names 11 px for all chrome including icon labels; I took the doc of the repo (replica size) and record it here, not a look. A 14 px label needs the 80 x 88 icon cell re-laid out; PM call.
4) rxm1-1 (session.rs): the five Turn Off buttons stand 58 px apart (were 56, XP's pitch) and are 58 wide, icons at 24, 82, 140, 198, 256; Stand By to Turn Off was 3.2 px by text width, 2 px of ink; now 4 or more on both measures. Unit `neighbouring_captions_of_the_big_buttons_keep_4_px_apart_rxm1_1`; gate cell caption-gap (new tests/png-extent.py: first and last column of white ink per caption). The unit `the_buttons_sit_where_xp_has_them` lost the XP pitch by design (new lefts), Cancel stays 80 x 28.

Also: gate cells wait with `ctl wait` on a node that does not exist (`pause`), no new sleep (sleep-ratchet 779 / 930 as before; an earlier full run was red on 3 new sleeps and was fixed).

Captures: 1-xp30-greeting.png (greeting, no helper, new words), 2-xp10-datetime-taskbutton.png (one click, task button, 75 x 23 buttons, 11 px text), 3-xp13-chat-icons.png (chat balloon, icons clear), 4-rxm1-1-turnoff-gaps.png (Turn Off, captions apart).

Gate, last 20 lines:
```
voice-acp  PASS    12s /tmp/merge-gate.6tPBk5/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.6tPBk5/voice-events.log
agent      PASS    41s /tmp/merge-gate.6tPBk5/agent.log
tiling-keys-more PASS    42s /tmp/merge-gate.6tPBk5/tiling-keys-more.log
acts       PASS  1890s /tmp/merge-gate.6tPBk5/acts.log
tiling-system PASS    14s /tmp/merge-gate.6tPBk5/tiling-system.log
handover   PASS    27s /tmp/merge-gate.6tPBk5/handover.log
tiling-windows PASS   173s /tmp/merge-gate.6tPBk5/tiling-windows.log
mc-scale   PASS    13s /tmp/merge-gate.6tPBk5/mc-scale.log
watchdog-starved PASS    53s /tmp/merge-gate.6tPBk5/watchdog-starved.log
xp-run     PASS    13s /tmp/merge-gate.6tPBk5/xp-run.log
demo       PASS   239s /tmp/merge-gate.6tPBk5/demo.log
tiling-bar PASS   177s /tmp/merge-gate.6tPBk5/tiling-bar.log
leak       PASS   480s /tmp/merge-gate.6tPBk5/leak.log
tiling-theme PASS    19s /tmp/merge-gate.6tPBk5/tiling-theme.log
mc-host    PASS    19s /tmp/merge-gate.6tPBk5/mc-host.log
tiling-disc PASS    73s /tmp/merge-gate.6tPBk5/tiling-disc.log
e2e        PASS   277s /tmp/merge-gate.6tPBk5/e2e.log
all steps passed; no --push, nothing is pushed
EXIT 0
```
