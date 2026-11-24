# rc1-pass: the independent pass (design/rc.md row 1)

## r1 xp, mc

Lane rc1-pass-xp-mc, opus high, cloud, head d40142e4 (claude/rc1-pass-xp-mc). Run and read only, no product change.
Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`, `run.py --help`
(it has no help: it ran the chapters; not a failure).

Runs, each from a fresh compositor:
- tour1: `TOUR_OUT=<scratch>/tour1 python3 demo/tour/run.py xp mc handover`: xp 31 acts (30 green, 1 known),
  mc 22 green, handover 11 green, 0 red.
- tour2: `run.py xp` again: 30 green, 1 known, 0 red (second take for rpx1-2).
- persona (`CHORA_TEST_FLAGS=--test-injected-press-is-real tests/persona/session.sh FLAVOR WxH`, then
  `tests/persona/act.py`): pxp (xp 1280x800), stray1 (xp), pxp2 (xp), pmc (mc dark 1280x720),
  straymc1 (mc), supa-xp and supa-mc (Super+A). The host calls go to the stand-ins (`calls.log`); no host power call.

Captures: `media/rc1-pass/` on claude/session-log (54 PNG, each under 600 KB). The code branch refuses
media (hooks `no-session-files` and `check-added-large-files`; no --no-verify), so they sit beside this
report, as record-v9 did. A path below without folder is in `media/rc1-pass/`.

### Verdicts

- **xp: does not confirm: rpx1-1, rpx1-2** (two MAJOR, each reproduced twice from a fresh compositor).
- **mc: does not confirm: rpx1-8** (one MAJOR, reproduced twice from a fresh compositor).

### xp: shared RC bar and main paths

| Item | Result | Capture |
|---|---|---|
| Start a program (mouse) | pass: Start > Notepad opens the editor (stand-in), Run... box | xp-23, xp-08, xp-19 |
| Start a program (keys) | pass: Super alone opens Start, Super+Space the Run box | xp-07 |
| Key list (mouse) | pass: Start > Help and Support > Keyboard shortcuts | xp-27 |
| Key list (keys) | pass: Super+F1 opens Help on Keyboard shortcuts (1280x800 shows all rows; at 1280x720 the pane scrolls) | xp-12, xp-27 |
| Helper (mouse) | pass with rpx1-2: paperclip and tray clip open the chat; answer shows | xp-02, xp-03, xp-25 |
| Helper (keys) | pass: Super+A opens the chat with the field ready (fresh session) | both-super-a-chat, xp-12 |
| Question card | pass with rpx1-3 (Yes clicked, card goes, done line) | xp-05, xp-05b, xp-28 |
| No stray keys into clients | **rpx1-1 MAJOR** | xp-08, xp-09, xp-10, xp-11 |
| Lock | pass: Start > Lock runs `swaylock -f` (stand-in window "Screen locked") | xp-16 |
| Power | pass: Start > Turn Off Computer > Turn Off runs `systemctl poweroff` (stand-in); Cancel turns nothing off | xp-18 |
| Volume | pass: tray speaker popup ticks 50 %, 75 % runs `wpctl set-volume 75%`; tooltip "Volume 75%" (rpx1-6) | xp-14, xp-21 |
| Screenshot | pass: Start > Screenshot and Super+Print, balloon "Screenshot saved" with the path | xp-15, xp-22 |
| Clock | pass: tray clock (11 px: xp-34); double click opens Date and Time (single click nothing: xp-10) | xp-17 |
| Honest failure states | **rpx1-2 MAJOR**; pass: Notepad with no editor says so (rpx1-7), Run says Nothing found | xp-01..xp-04, xp-06, xp-20, xp-19 |
| Handover from and to xp | pass: xp > tiling by Send To, construct > xp by Super+drag; title bar, task button, all lines kept | hand-01 |
| Erika (74, mouse): letter, helper, sound 75 %, question Yes | pass with rpx1-2, rpx1-3, rpx1-4 | xp-02..xp-05, xp-23, xp-28 |
| Murat (52, mouse): volume popup, screenshot, Run, Help, Turn Off Cancel, minimize and task button | pass | xp-14, xp-15, xp-19, xp-27, xp-18, xp-24 |
| Style (xp-style.md) | rpx1-3, rpx1-4, rpx1-5, rpx1-6 | xp-05b, xp-26, xp-13, xp-21 |

### mc: shared RC bar and main paths

| Item | Result | Capture |
|---|---|---|
| Start a program (mouse) | pass with rpx1-10: Programs in the column opens the launcher, Foot starts as page 7 | mc-07, mc-19 |
| Start a program (keys, remote) | pass with rpx1-10: Super+Space; unknown name says Nothing found | mc-13 |
| Key list (mouse) | pass: six key rows always in the column | mc-01, mc-10 |
| Key list (keys) | pass with rpx1-9: Super+F1 shows all keys, Esc shuts it | mc-11, mc-12 |
| Helper (mouse) | pass: "Chat with a helper ..." opens the chat, a typed question gets the answer | mc-08, mc-21 |
| Helper (keys) | pass: Super+A opens the chat in the column, the field takes the text | both-super-a-chat |
| No stray keys into clients | **rpx1-8 MAJOR** | mc-19, mc-20 |
| Lock | pass: Lock door runs `swaylock -f` (stand-in page) | mc-17 |
| Power | pass: Power door opens the menu, Shut down runs `systemctl poweroff` (stand-in) | mc-16 |
| Volume | pass with rpx1-15: remote Vol+ 50 to 55 %, click mutes and unmutes ("Muted") | mc-09, mc-14, mc-15 |
| Screenshot | pass: door runs grim and `notify-send Screenshot saved <path>` (the persona mc has no notify server, so no card in mc-18; chora-notify cards show in the tour: mc-02) | mc-18 |
| Clock | pass with rpx1-13: header "TUESDAY 24 NOVEMBER 13:50" | mc-10, mc-04b |
| Honest failure states | pass: Nothing found, Muted, No helper; rpx1-14 (empty Helpers page) | mc-13, mc-14, mc-06 |
| Handover from and to mc | pass: tiling > mc by Super+Alt+3 (page 7 NOTEPAD, all lines), mc > construct by Super+Alt+4 | hand-01 |
| Cook (light): voice next, timer, answer, mouse Next, remote Left/Right/Home, Super+F1, Super+n, alarm | pass with rpx1-11, rpx1-12, rpx1-13, rpx1-17 | mc-01..mc-05, mc-22 |
| Sofa (dark, remote): Right twice, Home, Programs, chat, Vol+ | pass with rpx1-10, rpx1-14 | mc-06..mc-09 |
| Nurse at a distance (dark, 1280x720) | pass with rpx1-9, rpx1-10: page and column at 14 px or more, launcher not | mc-10, mc-11, mc-07 |
| Style (mc-style.md) | rpx1-9, rpx1-11, rpx1-12, rpx1-13, rpx1-16 | as listed |

### Findings

| id | look | severity | text |
|---|---|---|---|
| rpx1-1 | xp | MAJOR | Super chords that xp binds to nothing type into the focused window and open Start on release |
| rpx1-2 | xp | MAJOR | "no helper is set up" greeting stays and is put into the chat of a working helper |
| rpx1-3 | xp | MINOR | question card not in Tahoma (xp-14 seen again) and Yes/No above the key line |
| rpx1-4 | xp | MINOR | helper balloon and notices 11 px |
| rpx1-5 | xp | COSMETIC | cube key caps in a bold italic fallback face |
| rpx1-6 | xp | COSMETIC | volume tooltip under the paperclip, "%" on its border; notice over the tooltip |
| rpx1-7 | xp | COSMETIC | no-editor notice names a config file as the next step |
| rpx1-8 | mc | MAJOR | Super chords that mc binds to nothing type into a window page |
| rpx1-9 | mc | MINOR | Super+F1 list: rule strikes "Mute" out, doors fall off at 1280x720 |
| rpx1-10 | mc | MINOR | launcher and power menu are unthemed fuzzel: cream box on the dark page, 3.2:1 |
| rpx1-11 | mc | MINOR | timer card covers the last line of the page |
| rpx1-12 | mc | MINOR | last page line "parsley." cut by the footer band |
| rpx1-13 | mc | MINOR | with the column hidden the pill covers the clock |
| rpx1-14 | mc | MINOR | Helpers page without helpers is empty, no words |
| rpx1-15 | mc | MINOR | no mouse way to change the volume level |
| rpx1-16 | mc | COSMETIC | state as a bordered pill (a box); "Super+a" vs "Super+A" |
| rpx1-17 | mc | COSMETIC | critical alarm card has no urgency mark |

**rpx1-1 (xp, MAJOR; bar: no stray keys into clients).** With a window focused, Super+x types `x` into
it and the Super release opens Start; the next key goes to the menu (in pxp a later letter opened Turn
Off Computer). Every second unbound chord leaks: while Start holds the keyboard the next chord types
nothing. shared.md: "an unbound Super key never types into a window"; D118 names `Super+q types no q
in a terminal`. Cause (read): `plugins/chora-xp/src/wm.rs:857` calls `grab_swallowed` only for
`CubeKeys::Flat | Hosted` (xp as a face of the construct); the plain xp session never swallows. Not
sh-24 (that one was a pointer-gated grab). Repro: fresh `session.sh xp 1280x800`; `act.py key Super_L`,
`click 62 606` (Notepad), `type start:`, `key Super+x`, `type "|"`, `key Super+k`, `type "|"`,
`key Super+m`, `key Super+q`. R1 pxp: "abcxkmdef" from abc, Super+x, q, k, 1, m, F1, Super+Alt+9, b, def
(xp-08), then Turn Off opened by a typed letter (xp-09). R2 stray1: "start:x|m|:end" and Start open
after Super+x (xp-10, xp-11).

**rpx1-2 (xp, MAJOR; bar: honest failure states).** When the voice agent tells its first state later
than 1.5 s after chora-assistant starts, the paperclip greets "Hi, I am your helper, but no helper is
set up yet, so you cannot ask anything. Click me to see what to do. This balloon closes by itself."
(xp-01), and every open of the chat puts that sentence into the transcript as the newest line: under
"Atlas — Idle" (xp-02), after Atlas's answer (xp-03, xp-28), or under "You" (xp-04, the card cuts the
start of the sentence). Erika reads that she cannot ask while the helper answers. Cause (read):
`plugins/chora-assistant/src/voice.rs:80-107` picks the greeting once at `GREET_AFTER` and never again
when a helper connects; `chat_toggled` (voice.rs:316-330) gives the last balloon sentence to the chat on
every open, with no age limit. With the agent up before 1.5 s (persona) the greeting is right and no
line enters the chat (xp-25). A local voice model that loads for seconds is the normal case. Related:
xp-30 (welcome balloon). Repro: `run.py xp` (the chapter emits the first `voice.state` 6 s after the
start); acts x1, x5, x6. R1 tour1 (xp-01..xp-04), R2 tour2 (xp-06).

**rpx1-3 (xp, MINOR).** The question card's title, prompt, keys and notes are in a wide sans (DejaVu
form), the chat around it in Tahoma (xp-05b); xp-14 is marked fixed at d40142e4 for "card text not
Tahoma", seen again on d40142e4. The order puts Yes and No above the key line and "A spoken 'yes' does
not count."; shared.md puts them last. No time left: sh-12. Repro: tour act x6 (xp-05).

**rpx1-4 (xp, MINOR).** The paperclip's greeting balloon has 11 px text (cap height 8 px, line 13 px,
xp-26); xp-style 1: the helper's balloon stays 14 px. The notices ("Cannot open Notepad", "Screenshot
saved") are 11 px too (xp-20, xp-22). Related xp-7 (fixed, text px unmeasured).

**rpx1-5 (xp, COSMETIC).** Help > Keyboard shortcuts: the caps Ctrl+Alt+←, Ctrl+Alt+→,
Ctrl+Alt+Shift+← and → are set in a bold italic face with a space and a small arrow (xp-13); every
other cap is Tahoma regular; xp-style: Tahoma, no italic.

**rpx1-6 (xp, COSMETIC).** The volume tooltip "Volume 75%" is drawn under the paperclip and its "%"
touches the right border (xp-21); the screenshot balloon then lies over the tooltip that is still up
(xp-22). Repro: tour acts x7, x8.

**rpx1-7 (xp, COSMETIC).** "Cannot open Notepad / No text editor installed (for example mousepad). To
name one yourself: editor in chora-xp.yaml" (xp-20): the next step is a file, not words for Erika (P4).

**rpx1-8 (mc, MAJOR; bar: no stray keys into clients).** On the page of a window, Super+x, Super+k,
Super+m and Super+q type their letters into foot: "start:x|k|m|q|:end" (mc-19). chora-mc calls no
`grab_swallowed` (grep: tiling, construct and xp faces only; D118). Repro: fresh `session.sh mc
1280x720`; `click 918 658` (Programs), `type Foot`, `key Return`, `type start:`, then `key Super+x`,
`type "|"` for x k m q, `type :end`. R1 pmc (mc-19), R2 straymc1 (mc-20).

**rpx1-9 (mc, MINOR).** Super+F1 at 1280x720: the hairline of the list head runs along the state line
and strikes "Mute" (and "speaking") through, so Mute reads as off (mc-11 dark, mc-12 light, tour act
m6); the door lines fall off the output: "Programs Lock Power" cut at y 713, "Screenshot Volume" gone.
mc-style 16: the list fits 1024x768. mc-7 (fixed) seen again in part.

**rpx1-10 (mc, MINOR).** The launcher (Programs, Super+Space) and the Power menu are bare fuzzel: a
cream box with a monospace face over the dark page, text (101,123,131) on (224,218,201) 3.2:1, rows
about 13 px, "Nothing found for "nosuchpro…" cut (mc-07, mc-13, mc-16). chora-mc runs `fuzzel` with no
theme (`plugins/chora-mc/src/main.rs:460`, `chora_kit::launcher::run`). For the sofa and the nurse at
a distance it is the start-a-program path. Words: sh-16.

**rpx1-11 (mc, MINOR).** The timer card (title and one line) covers the third line of the page's Timer
panel, "spaghetti is in." (mc-02, tour act m3 at 1280x720); `plugins/chora-mc/notify/README.md` says
such a card fits the two lines above the footer.

**rpx1-12 (mc, MINOR).** Step 2: the sauce at 1280x720: the last line "parsley." is cut at its
descenders by the footer band (mc-03, mc-22); mc-style 12: whole lines only.

**rpx1-13 (mc, MINOR).** After Super+n hides the column, the state pill lies over the last digit of the
clock "18:51" (mc-04, mc-04b).

**rpx1-14 (mc, MINOR).** Sofa, Right twice: page "3 HELPERS" shows only its title, no word that no
helper is there (mc-06); notes have "Nothing new." (P4).

**rpx1-15 (mc, MINOR).** A click on "Volume 50%" toggles mute only and scrolling over it does nothing
(calls.log: `set-mute toggle` twice, no `set-volume`); the level changes only by remote, keys or voice
(mc-14, mc-15). Vision card: mouse a full second path.

**rpx1-16 (mc, COSMETIC).** The state is a bordered pill ("Listening", "No helper", mc-10, mc-04b): a box
(mc-style 11). The first page says "press Super+a", the key list "Super+A" (mc-10). Lower-case state and
helper words ("listening", "idle", "working", "atlas") beside "Idle", "Working": sh-13, seen again.

**rpx1-17 (mc, COSMETIC).** The critical alarm "Spaghetti is ready / Drain it now" has the look of
any notice, no red and no word (mc-05); the act wants a red note; red is the urgent colour.

### Known ids seen again (not new)

xp-10 (single click on the clock), xp-14 (rpx1-3), xp-20 (green dot), xp-21 (Help close box only),
xp-28 ("Answering, step 1 of 1" under Done), xp-34 (clock 11 px), sh-12 (no time left), sh-13
(rpx1-16), sh-16 (launcher words), mc-7 (rpx1-9), mc-36 ("No helper" pill in the mc tour: no helper in
the chapter; the persona mc shows atlas idle and "Listening").

### Notes for the PM

- Tour row ca-4 (act 31, Super held, drag) is green in both xp takes: "known act 31 is green: drop known".
- `run.py --help` runs every chapter: an unknown flag is ignored.
- Two MAJOR in xp and one in mc are new at this pass; the tour and the gates did not see them (no act
  presses an unbound Super chord over a window; the first `voice.state` of a chapter is what makes rpx1-2).
- Context used about 160 k; no `chora --headless` left.
