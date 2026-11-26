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

## r1 tiling

Lane rc1-pass-tiling, opus high, cloud, head d40142e4 (claude/rc1-pass-tiling). Run and read only, no product change.
Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`, `run.py --help`
(it has no help: it ran the chapters, as in r1 xp, mc; not a failure).

Runs, each from a fresh compositor:
- tour1: `TOUR_OUT=<scratch>/tour1 python3 demo/tour/run.py tiling`: 28 acts, 28 green, 0 red (t9 green, ti-29 holds).
- tour2: `run.py handover`: 11 acts, 11 green, 0 red.
- persona (`CHORA_TEST_FLAGS=--test-injected-press-is-real tests/persona/session.sh tiling WxH`, then
  `tests/persona/act.py`): p1 (1280x800), p2 (1280x720, with `choractl run -- chora-notify`, so a notice could
  show), p3 (1280x720, with chora-notify). Host calls go to the stand-ins (`calls.log`); no host power call.

Captures: `media/rc1-pass/tiling-*` on claude/session-log (35 PNG and `tiling-36-logs.txt`, 4.7 MB in all, each
under 600 KB). claude/rc1-pass-tiling refuses `media/` (hook `no-session-files`; no --no-verify), so they sit
beside this report as in r1 xp, mc. A name below without folder is `media/rc1-pass/tiling-<name>`.

### Verdict

- **tiling: does not confirm: rpt1-1, rpt1-2** (two MAJOR, each reproduced twice from a fresh compositor).

### tiling: shared RC bar and main paths

| Item | Result | Capture |
|---|---|---|
| Start a program (mouse) | pass with rpt1-5, rpt1-6: Apps, then a click on Foot starts a terminal that takes the focus | 09, 10 |
| Start a program (keys) | pass: Super+Return (themed foot); Apps by Super+r/Super+Space (tour t8) | 02, 32 |
| Key list (mouse) | pass with rpt1-3: Keys in the bar opens it, with a Mouse line | 07 |
| Key list (keys) | pass with rpt1-3: Super+F1 and Super+/ (tour t6), Escape gives the keys back to foot | 04, 06, 31 |
| Helper (mouse) | **rpt1-1 MAJOR**: the bar word opens the chat, but the keys stay in the window | 11, 12, 13 |
| Helper (keys) | pass: Super+a opens the chat beside the tiles, the text reaches the helper | 14, 15 |
| No stray keys into clients | pass: 20 unbound chords (Super+x q z d g b e w y i p c v F5 period, Super+Alt+9, Super+Shift+x, Super+Ctrl+x, Super+Delete, Super+BackSpace) between `|` in foot: `start:||||...|:end`, no letter; tour t2 probe heard no key | 03 |
| Lock | pass with rpt1-10: power icon > Lock runs `swaylock -C <palette> -f` (stand-in window "Screen locked"); Super+Shift+Escape runs `loginctl lock-session` | 17, 18, 16, 36-logs |
| Power | pass: power icon > Shut down runs `systemctl poweroff` (stand-in); Escape leaves the menu | 17, 19 |
| Volume | pass: wheel on "Volume 50%" gives 55 % (`wpctl set-volume 5%+`), a click shows "Muted" and a second unmutes | 20, 21 |
| Screenshot | pass with rpt1-4: Super+Print and power > Screenshot, notice "Screenshot saved" with the path (the date in the name is the tour's fixed clock stand-in, launch.py:75) | 22, 23 |
| Clock | pass: "Tue 24 Nov 14:22" centred in its island | 01 |
| Honest failure states | **rpt1-2 MAJOR**; rpt1-7, rpt1-8; pass: Apps "frobnicate" says Nothing found (ti-49) | 25, 26, 28, 29, 30, 32 |
| Handover to and from tiling | pass: xp > tiling by Send To, typed line on tiling, tiling > mc by Super+Alt+3, all lines kept (the chapter starts tiling without bar and wallpaper on purpose: its checker exempts them) | 33, 34, 35 |
| Power user (keys): shell, layouts, master, tags, float, Super+drag, key list, chat, Escape | pass with rpt1-3, rpt1-9 (tour 28 green; gaps 12 outer and 12 inner from the border, as docs/configuration.md:248) | 02, 04, 14, 15, 31 |
| Ricer: Super+s, live edit, new key | pass: Super+s opens the yaml in vi in themed foot; `gap_inner: 40` then Super+Ctrl+r applies at once; a new `MOD+x: {run: ...}` shows in the key list | 24, 27, 08 |
| Ricer: a broken line | **rpt1-2 MAJOR**: silent; rpt1-8 (missing program, silent); not stranded: the old settings and every key keep working | 25, 26, 28, 30 |
| Beginner, mouse only, no key known | pass for Apps, Keys, power, volume; **rpt1-1** for the helper; rpt1-5, rpt1-6 on the way | 09, 10, 07, 17, 20, 11 |
| Style (tiling-v2) | rpt1-3, rpt1-4, rpt1-5, rpt1-9; bar, tags, chip, gaps, radius and focus glow as designed | 01, 02 |

### Findings

| id | severity | text |
|---|---|---|
| rpt1-1 | MAJOR | a click on the bar's helper word opens the chat without the keyboard: the typed message goes into the focused window, Return runs it in a shell |
| rpt1-2 | MAJOR | Super+Ctrl+r with a broken line in chora-tiling.yaml fails silently: no notice, no word; the reason only on stderr |
| rpt1-3 | MINOR | key list at 1280x720: the last rows run past the panel, its bottom border crosses the text |
| rpt1-4 | MINOR | notices in tiling are chora-notify's neutral light card (fill 250, border 140, system sans) on the Ink desktop |
| rpt1-5 | MINOR | Apps > Foot starts an unthemed foot: about 10 px text 2 px from the border, grey ground; Super+Return starts the themed one |
| rpt1-6 | MINOR | Apps preselects "Dark look": Apps then Enter changes the theme; setting and give rows sit among the programs |
| rpt1-7 | MINOR | Super+Alt+2..4 to a look that is not running: the window stays, nothing says why (the Apps row of the same act does say) |
| rpt1-8 | MINOR | a binding `{run: PROGRAM}` whose program is missing does nothing and says nothing, not even on stderr |
| rpt1-9 | COSMETIC | while the chat field has the keys, the tile keeps the focus glow ("exactly one thing glows: the window that has the keys") |
| rpt1-10 | MINOR | Super+Shift+Escape runs `loginctl lock-session`, which locks only while swayidle (an optdepend) listens; without it the key does nothing and says nothing |

**rpt1-1 (MAJOR; bar: the helper by mouse; P11, tiling.md "Super+a or a click on the segment opens the panel ... the
field has the keyboard").** With foot focused, a click on "atlas · idle" in the bar opens the chat panel beside the
tiles, but the field has no caret; typing goes into foot and Return runs it: `hello there` gave `bash: hello: command
not found` (11), `echo LEAKED-R2` printed LEAKED-R2 (12). A beginner who clicks the helper, types a question and presses
Enter runs it as a command in a shell. Workaround: click into the field (13) or use Super+a (14, same session, works).
Cause (read): the module's `on-click` is `choractl run -- chora-chat` (flavors/tiling/waybar/config.jsonc:118), a new
connection; `activation.token` is stamped with the act only for a connection that received it
(crates/chora/src/ipc.rs:996-1011), so the chat gets a token without the act and may not take the keyboard. The doors
cell (tests/doors/tiling.sh:128-140) checks only that the chat opens. Repro: fresh `session.sh tiling 1280x720`;
`key Super+Return`, wait 3 s, `click 932 24`, wait 2 s, `type "echo LEAKED"`, `key Return`, `look`. R1 p1 (11),
R2 p2 (12).

**rpt1-2 (MAJOR; bar: honest failure states; main path of the ricer, tiling.md "see errors at once").** Insert the
line `  gap_inner 20` (no colon) under `radius:` in `$XDG_CONFIG_HOME/chora/chora-tiling.yaml`, press Super+Ctrl+r:
nothing changes on screen and nothing says why (25); with chora-notify running, `notify state.get` lists `shown: []`
(26). A wrong action name (`MOD+Shift+c: closee`) is the same (28). The reason goes to stderr only ("simple key
expected ':' at line 24, column 3", "unknown variant, expected one of ...", 36-logs). Not stranded: the old settings
and keys stay in force (28: gap 40 of the edit before still on, Super+Return works). Cause (read):
plugins/chora-tiling/src/main.rs:1097-1100 (`eprintln!` only); at start the same file gives the defaults with a line on
stderr (crates/chora-kit/src/settings.rs:134-135; read, not run). Repro: fresh session, Super+s once (writes the file)
or `chora-tiling --print-config > FILE`, break a line, Super+Ctrl+r, `look`. R1 p1 (25, 28), R2 p2 (26).

**rpt1-3 (MINOR).** At 1280x720 the key list has rows past its panel: "Super+Shift+S  Screenshot of an area" sits on the
bottom border, the keycap reaches 4 px below it (04, zoom 05, tour t6; p2 06); with one more binding the right column
grows and "All keys" sits on the border (p3 08). At 1280x800 it fits (07). review-checklist 2: no text touches an edge.
Likely cause: the group "Other desktops" (ti-48, in d40142e4) adds two rows; related ti-9 (verify), rtd-5.

**rpt1-4 (MINOR).** The screenshot notice is a white card (250,250,250) with a grey border and dark sans text over the
Ink desktop (22, 23); review-checklist 8: no light panel on dark; tiling-v2 build step 10 wants chora-notify in the
flavor. flavors/tiling/chora-tiling-notify.service:7 runs `/usr/bin/chora-notify` with no `--assets`, so the neutral
folder (plugins/chora-notify/assets/values.yaml) draws it. It is also a third face beside Inter and JetBrains Mono
(ti-18).

**rpt1-5 (MINOR).** Apps > Foot (fresh p1, 09 then 10): the new foot has foot's own defaults: line pitch 13 px (about
10 px text, under 12), a grey (35,35,35) ground and text 2 px from the border; the foot of Super+Return beside it has
the flavor's 14 px face, padding and Ink ground. The `.desktop` entry runs `foot` without the flavor's foot.ini; only
`chora-tiling --launch foot` passes it. The mouse door gives the beginner the worse terminal.

**rpt1-6 (MINOR).** Apps lists settings and acts among programs, sorted: the first and preselected row is "Dark look"
(09; calls.log `fuzzel-first-row Dark look`), then Edit settings, Foot, Foot Client, Foot Server, ImageMagick ...;
"Light look" and "Move window to the construct desktop" follow. Apps then Enter changes the theme, not a program
(POLS; ti-2 fixed the same with Screenshot). Raw rows: ti-25.

**rpt1-7 (MINOR).** In a tiling-only session (the default) Super+Alt+2 leaves the window where it is (not lost) and
shows nothing (29; notify `shown: []`); stderr only: `give to xp: command 0: "xp" holds no claim`
(plugins/chora-tiling/src/main.rs:567-573). The key list heads the key with "Other desktops: xp, media center,
construct" (08). flavors/tiling/give.sh, the Apps row, sends "Window not moved" with the reason.

**rpt1-8 (MINOR).** `MOD+x: {run: firefoxx}` loads (the key list shows "Super+X  Run firefoxx", 08); Super+x then
does nothing, with no notice and no line in the log (30, 36-logs). A ricer's typo in a program name cannot be found.

**rpt1-9 (COSMETIC).** After Super+a the chat field has the keys, and the left foot still carries the focused
border and glow (14); tiling-v2 philosophy: light means the window that has the keys.

**rpt1-10 (MINOR; read plus the call).** Super+Shift+Escape runs `loginctl lock-session` (16, 36-logs). It locks only
when swayidle runs (flavors/tiling/chora-tiling-swayidle.service, `ConditionPathExists=/usr/bin/swayidle`), and
swayidle is an optdepend (dist/arch/PKGBUILD); without it the key the key list calls "Lock the screen" does nothing and
says nothing. The power menu's Lock runs swaylock directly (18). The doors cell `lock-key` accepts the loginctl call as
a pass (tests/doors/tiling.sh:149-152). Not run against a real logind (no host call).

### Known ids seen again (not new)

sh-13 (bar "atlas · idle" lower case in the persona, "Atlas · idle" in the tour; a "Listening" pill and dot float at
the bottom right over the tile while the bar says idle, 03, 10; tiling.md "No dot at the bottom right"), ti-9 ("Window
that asks"), ti-25 (floated window half off the output after Super+drag, 22; raw launcher rows Foot Client, Foot
Server, ImageMagick, 09), ti-28 (Super+q: nothing and no sign, 03), ti-49 (Nothing found only after Return, 32),
ti-50 (no helper row in Apps; in flight). Look fixed on d40142e4: ti-14 (the key list has a Mouse line, 07), ti-8 (a
new window takes the focus, 10), ti-13 (bar says "Connected", the volume word works, 20).

### Notes for the PM

- rpt1-1 and rpt1-2 are new at this pass; no gate or act types after a click on the helper word, or reloads a broken
  file.
- ti-31's text says the inner gap of 10 holds; code, docs/configuration.md:248 and gate-tiling-gaps.sh say 12, and 12
  is what shows (02); tiling-v2 still says 10. A ledger line to rewrite.
- Tour act t2 ("Super+d and two more Super chords") presses Super+m, which is bound (maximize), so the probe fills the
  output in still 02-02d-stray.
- A `chora-notify` started with `choractl run` in a persona session outlived `session.sh --stop` by about a minute.
- Context used about 175 k; no `chora --headless` left.

## r1 construct

Lane rc1-pass-construct, opus high, cloud, head 280b9dcf (claude/rc1-pass-construct: co-pull r2, co-70 and co-71 fixed). Run and
read only, no product change. Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`,
`run.py --help` (it has no help: it ran the chapters, as in r1 xp, mc and r1 tiling; not a failure).

Runs, each from a fresh compositor:
- tour1: `TOUR_OUT=<scratch>/tour1 python3 demo/tour/run.py construct`: 18 acts, every `expect` held; all 18 rows red on
  invariants only: co-44 (chips 9 apart), co-45 (11 px heads, act 11) and xp-45 ('chart' title 4.31:1; its fix f1397d98 is
  not an ancestor of this head).
- tour2: `run.py handover`: 11 acts, 11 green ("known act 2 is green: drop known", ca-3).
- persona (`CHORA_TEST_FLAGS=--test-injected-press-is-real tests/persona/session.sh construct WxH`, then
  `tests/persona/act.py`): p1 and p2 (1280x800), p3 (1280x800 under `dbus-run-session`; chora-notify started by hand for the
  notice), p4 (1280x720, chora-notify started by hand, plugin only). Host calls go to the stand-ins (`calls.log`); no host
  power call. The persona construct starts no chora-notify (the flavor does: `chora-construct-notify.service`), so without
  the hand start a notice is only logged.

Captures: `media/rc1-pass/construct-*` on claude/session-log (40 PNG, 7.6 MB, each under 600 KB). claude/rc1-pass-construct
refuses `media/` (hook `no-session-files`; no --no-verify), so they sit beside this report as in r1 xp, mc and r1 tiling.
A number below is `media/rc1-pass/construct-NN-*.png`.

### Verdict

- **construct: does not confirm: rpc1-1, rpc1-2** (one BLOCK reproduced three times, one MAJOR twice, each from a fresh
  compositor).

### construct: shared RC bar, vision card and main paths

| Item | Result | Capture |
|---|---|---|
| Start a program (mouse) | pass with rpc1-6: the start card row Terminal flies into a new terminal; the Programs door opens the launcher | 11, 12, 37 |
| Start a program (keys) | pass: Super+t (a terminal flown in, typed text reaches it), Super+Space (fuzzel) | 39, 37 |
| Key list (mouse) | pass stood back and in a face (hint All keys, door Keys); **rpc1-2 MAJOR** in a pane | 27, 28, 16, 18, 19 |
| Key list (keys) | pass with rpc1-3, rpc1-4: Super+F1 in a pane, F1 stood back, Escape shuts it (co-71 holds for Escape; the Close row shows at 800 and walk 720) | 14, 07, 40 |
| Helper (mouse) | pass: a click on the pill and on the door Helper opens the chat with the field ready ("by mouse" typed) | 23 |
| Helper (keys) | pass: Super+a, a typed question, Atlas answers "Hello" | 22 |
| No stray keys into clients | pass: 22 unbound Super chords between markers in a pane (`SABCDEFGHIJKLM...WZ`) and on a face (`SABC...UVWZ`), no letter, Start shut | 13, 29 |
| Lock | pass: door and Super+Shift+Escape run `swaylock -f` (the stand-in window shows as a pane: co-57) | 24 |
| Power | pass with rpc1-6: door and Super+Shift+e open the menu; Shut down runs `systemctl poweroff` (stand-in) | 25 |
| Volume | pass with rpc1-7: a click mutes ("Volume Muted"), the media key unmutes and gives 55 % | 26 |
| Screenshot | pass with rpc1-9: door and Super+Print run grim and `notify-send Screenshot saved <path>`; with chora-notify the card shows | 36 |
| Clock | pass: strip clock 17:51 to 18:05 real time, the xp face tray says the same minute | 11, 28 |
| Honest failure states | pass with rpc1-5, rpc1-10: "Media center did not take the window: it is not running."; failed face card with the reason; a broken `chora-construct.yaml` is refused on reload with file, line and reason, the old settings stay (choractl: "while parsing a flow sequence, expected ',' or ']' at line 14") | 39, 38 |
| No lost window in a handover | pass: mc > construct by Super+Alt+4, construct > xp by Super+drag, every line kept (tour2 11/11); a face window pulled by Super+Alt+4 is a pane; Escape on the stood-back cube keeps the face's window as a pane. **rpc1-1 BLOCK**: Super+x in a face closes a pane out of view | 09, 10, 31, 15, 33, 34 |
| Step out and in | pass: Alt+Escape, Return, a click on a face; the bar names the way out in a face | 27, 28, 30, 08 |
| Pull windows across faces | pass: a title drag over the edge (second output) makes a pane, a plain drag puts it back on its face; stood back the bar names "Super Alt 4 Pull out" for the face window and the key pulls it (co-70 fixed) | 02, 03, 05, 30, 31 |
| Glass, calm, minimal HUD | pass with rpc1-8, rpc1-9, co-44, co-45: the cube at rest and mid-turn, faces meet, lit edge on hover, one accent | 04, 06, 27 |
| Explorer (mouse first) | **rpc1-2**: start card, a click on a face, the pill, the doors work; in a pane after 3 s nothing leads out | 11, 12, 28, 16, 18 |
| Demo watcher | pass with co-44, co-45: tour1 every `expect` held; the hints name each act | 01..08 |
| Ricer (keys, config) | pass with **rpc1-1**, rpc1-3: the file, `settings.reload`, `cube_manager` applies on the next Super+c | 38, 14 |
| Beginner in the stood-back cube | pass with rpc1-5: hints name Turn, Step in, Fly in, All keys; a click on a face steps in; Escape closes the cube (in the key list) | 27, 28, 07, 35 |

### Findings

| id | severity | text |
|---|---|---|
| rpc1-1 | BLOCK | Super+x in a face closes a pane that waits beside the cube, out of view, and never the focused face window |
| rpc1-2 | MAJOR | in a pane flown into, after the 3 s of the hints nothing on screen leads to the key list or out; the bottom-edge dwell never shows them |
| rpc1-3 | MINOR | Super+F1 in a pane flies out first: the sheet says WALKING, after Esc the next typed letters walk |
| rpc1-4 | MINOR | key sheet at 1280x720 with the cube rows runs past the bottom: footer and Close off screen, top 8 px from the edge |
| rpc1-5 | MINOR | after Escape closes the cube on an empty floor, its face label and its chips (words gone) stay under the start card |
| rpc1-6 | MINOR | Programs and Power open unthemed fuzzel: cream box, mono, square border on the glass look |
| rpc1-7 | MINOR | the Volume door has no mouse way to set the level: the wheel does nothing |
| rpc1-8 | MINOR | in a pane the helper pill is a grey slab, state word 2.4:1 |
| rpc1-9 | MINOR | notices are the neutral white card at the bottom right, body about 11 px, over the hint bar and the key sheet |
| rpc1-10 | MINOR | a cube whose every face failed says "The other faces still work." |

**rpc1-1 (BLOCK; bar: no lost window; findings/README "a window is lost").** Open a terminal pane, open the cube (Super+c): the
pane stands beside the cube. Step into a face (Return or a click) and press Super+x, the "Close window" of the key list (14):
the face window, if any, stays, and the terminal pane, out of view, closes; its foot process ends (no foot left in the
session), stood back the cube stands alone and the window list says "0 WINDOWS" (33). With a face window focused ("echo
faceA"), Super+x leaves it and closes the pane "echo paneB" (34). Cause (read): bound chords are grabbed on a cube (co-26),
Super+x runs `close_ahead` (plugins/chora-construct/src/adopt.rs:309), "the pane nearest the middle of the view"
(docs/configuration.md:952); in a face that is the pane parked at home in front of the cube. foot closes at once, so the
person's shell and its work are gone. Repro: fresh `session.sh construct 1280x800`; `act.py click 480 363` (Terminal row),
`type "echo first-terminal"`, `key Super+c`, `key Return`, `key Super+x`, `key Alt+Escape`, `key Tab`. R0 p2 (seen while
probing stray keys: the first Terminal pane was gone, 27 vs 32), R1 p3 (33), R2 p3 (34: start card, Super+t, Super+c,
Return, Start > Terminal on the face, Super+x).

**rpc1-2 (MAJOR; bar: the key list by mouse; vision card "step out and in"; explorer, mouse first).** In a pane flown into
from the walk (the start card row Terminal, Super+t), the hint bar "Alt Esc Step back · Super F1 All keys" shows 3 s (12),
then nothing remains on screen but the dimmed pill: no hint, no chip Keys, no strip (doors hidden), so no mouse way to the
key list, to step back, Programs, Lock, Power, Volume or Screenshot. The designed second door, the bottom-edge dwell
(construct-v2 polish 7; hints.rs:36 `BOTTOM`, overlay.rs:1059), never shows the hints: the pointer at y 770, 790 and 799
for 1.2 s each, and a click where "All keys" stood reaches the terminal (16, 18, 19). The same gap was co-42 for a face
(fixed there by a lasting chip); the pane has no such chip. Cause (likely, read): over a pane window the pointer belongs to
the window and the construct gets no motion (as co-42 found: InputMoved only during a held grab), so `hints.low` never
turns true. Workaround: Alt+Escape or Super+F1 by key. Repro: fresh `session.sh construct 1280x800`; `click 480 363`, wait
6 s, `move 640 770`, `move 640 790`, `move 640 799`, `look`; `click 760 724`, `look`. R1 p1 (16), R2 p2 (18, 19).

**rpc1-3 (MINOR; POLS, P1 "Escape backs out one step").** In a pane, Super+F1 flies out before it shows the sheet
(plugins/chora-construct/src/keys.rs:33): the sheet says "CONSTRUCT · WALKING" and lists the walk keys (14); Escape shuts it
and leaves the camera in the walk, so the next typed "def" walks (d moves right) instead of reaching the terminal (21).
help_takes_keyboard notes the window to give the keys back, but the camera is out of the pane already. Repro: p1 (14, and
"w" walked after Esc), p2 (20, 21).

**rpc1-4 (MINOR; construct.md checklist 2 and 7).** At 1280x720 with a cube stood back, the key sheet starts 8 px from the
top and runs past the bottom: "Mute" at y 696, the footer "Only the keys that work now are listed. · Esc Close" off screen
(07, tour act 11). In the walk at 1280x720 it just fits, 11 px from the top and the bottom, not 32 (40). Like rpt1-3 in tiling.

**rpc1-5 (MINOR; honest states; construct-v2 section 2 "No hint bar while it shows").** Open the cube on an empty floor
(Super+c), Escape: the cube goes, the start card comes, and under it the face label "XP 1 · face 1 of 4 ●○○○" and the cube
hint bar stay, its words gone ("Ctrl Alt ←   Return   Ctrl Alt →  Super F1 All keys"), still 7 s later; the start card
covers half of the label (35: p3 007 after the list and two Escapes, p3 016 after Super+x, Super+c, Escape).

**rpc1-6 (MINOR; construct.md Don't "a plain box instead of glass").** The door Programs and Super+Space open fuzzel with its
own defaults: a cream box with a dark hairline, monospace rows, raw entries (Foot Client, Foot Server, ImageMagick (color
depth=...)); the door Power opens the same box (25, 37). Same root as rpx1-10 (mc).

**rpc1-7 (MINOR).** A click on "Volume 50%" toggles mute ("Volume Muted"), the wheel over it does nothing (no `wpctl` call),
so a mouse user cannot set the level; the media keys work (26). Same as rpx1-15 (mc); tiling's wheel works.

**rpc1-8 (MINOR; P7, contrast 4.5:1).** In a pane the pill is drawn at reduced opacity over the dark terminal: a grey slab
(117,117,119) with the state word (59,61,66), 2.4:1; the ring reads no better (12, 17). The hint bar switches to the dark
glass over content for this reason (hints.rs `over_content`), the pill does not; construct-v2: "no grey slab".

**rpc1-9 (MINOR; construct.md "Notices are one line of text on glass, bottom centre"; shared 14 px).** The construct's notices
(the refusal of a give, "Screenshot saved") come as chora-notify's neutral card: white, square-ish, at the bottom right,
title about 13 px bold, body about 11 px (13 px line pitch, 36); it lies over the right end of the hint bar (36) and over
the corner of the key sheet (40). Same root as rpt1-4 (tiling).

**rpc1-10 (MINOR; honest states).** With `cube_manager: chora-nope` every face runs the missing manager: the four dots are
hollow and the label says "Nope 1 · did not start", yet the card says "The other faces still work." (38). The card holds the
reason ("chora-nope stopped after 0 s: sh: 1: exec: chora-nope: not found") and the bar "Return Try again"; the card spills
left of the face (co-7 again).

### Known ids seen again (not new)

co-44 (chips 9 apart: every stood-back act of tour1), co-45 (11 px heads: tour act 11, 07), co-74 (stepped into a face the
bar names no Pull out, 29, 28), co-7 (failed card not fitted to the face, 38), co-11 (the launcher closes silently on an
unknown name, no empty-result text), co-57 (the lock stand-in draws a pane, 24), co-17 (the pulled terminal is "foot" in the
list, "Foot" as a label, 32), sh-13 ("IDLE" in the chat head, "LISTENING" on the pill, 22), xp-45 (tour1 I6, fix not on this
head). co-70 is fixed (30, 31). co-39 (sheet over the chips) did not show (no I5 in tour1).

### Notes for the PM

- rpc1-1 and rpc1-2 are new at this pass; no gate or act presses Super+x in a face with a pane beside the cube, or waits
  out the 3 s in a plain pane before it looks for a way out.
- The letters that repeated after Alt+digit in a terminal (Alt+9 then "y" gave nine y) are bash readline's digit argument,
  not a lost key; the same shows in tiling. Not a finding.
- The tour stubs `date` (launch.py `CLOCK_EPOCH`, 18:51) for the construct strip, but the xp face clock reads the real time
  (5:46 PM in 01, 18:51 in 04): a demo watcher sees two clocks disagree in the recording only. Tour harness NIT.
- The persona construct session starts no chora-notify and has no session bus, so its notices (refusals, Screenshot saved)
  are only logged; a persona never sees them. A harness gap (session.sh line 249).
- Context used about 200 k; no `chora --headless` left.

## r2 xp

Lane rc1-pass-xp-r2, opus high, cloud, head 3be27e32 (claude/rc1-pass-xp-r2, t152 green). Run and read only, no product,
test or design change. Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`,
`run.py --help` (still no help: it starts the chapters, as r1 noted).

Runs, each from a fresh compositor:
- tour1: `TOUR_OUT=<scratch>/tour1 python3 demo/tour/run.py xp handover`: xp 31 acts (30 green, 1 known ca-4, again
  "known act 31 is green: drop known"), handover 11 acts (9 green, 2 red: acts 7, 8, see Notes).
- tour2: `run.py --check xp handover`: the same counts, the same two red rows.
- p1: `CHORA_TEST_FLAGS=--test-injected-press-is-real tests/persona/session.sh xp 1280x800`, then `tests/persona/act.py`.
- p2: the same at 1280x720 (second reproductions). p3: xp 1920x1080. h1: `session.sh xp+tiling 1280x720` (give keys).
Host calls went to the stand-ins (`calls.log`); no host power call. Fallback art (no import), as the tour runs.

Captures: `media/2026-11-26/rc1-pass-xp-r2/` (51 PNG, each under 600 KB). The brief puts them on the code branch;
its installed pre-commit hook `no-session-files` refuses `media/` and --no-verify is out, so they sit beside this
report on claude/session-log, as r1 did. `t-` = tour1 stills, `p1-`, `p2-`, `p3-`, `h1-` = persona sessions.

### Verdict

**xp: confirms rc 1.** No BLOCK or MAJOR found. New: 3 MINOR (rpx2-1, rpx2-3, rpx2-4), 2 COSMETIC (rpx2-2, rpx2-5). The r1 MAJORs are gone: rpx1-1 / xp-46 (p1-02: "start:||||||||:end", nothing typed, Start shut) and rpx1-2 /
xp-47 (t-01: the greeting says "Click me to ask", no "no helper" line in any chat: t-03, t-05, p1-07).

### xp: shared RC bar and main paths (r1 rows rerun, plus the chat)

| Item | Result | Capture |
|---|---|---|
| Start a program (mouse) | pass: Start > Notepad opens the editor (stand-in); Start > Run... | p1-01, p1-02, t-08 |
| Start a program (keys) | pass: Super alone opens Start; Super+Space Run box; Enter on no match says Nothing found for "nosuchprog" (before Enter the list is empty, fuzzel) | p1-03, p1-04, p1-05 |
| Key list (mouse) | pass: Start > Help and Support > Keyboard shortcuts | t-09 |
| Key list (keys) | pass: Super+F1 on Keyboard shortcuts, all rows at 1280x800; at 1280x720 the pane scrolls to Vol-, Mute (ca-1); Esc shuts Help | p1-06, p2-09-help720, p2-10c |
| Helper (mouse) | pass: paperclip, tray clip and Start > Chat with a helper each open the chat; the field takes the keys | p1-20, t-03 |
| Helper (keys) | pass: Super+A opens the chat, field ready; Esc shuts it | p1-07 |
| Chat: send, Working, Stop, answer | pass: 250 ms after Return "atlas — Working" with Stop live (xp-busy); answer lands, Stop greys; Stop sends `agent.cancel` and shows Idle (the stand-in answers anyway: fake-voice ignores cancel, not chora) | p1-07, p1-08 |
| Question card | pass with rpx1-3 (seen again): Yes clicked, card goes, Done line | t-04, t-05 |
| Paperclip and tail | pass: tail tip on the head (xp-tail), the blink seen (p1-27: eyes shut) | t-01, p1-07 |
| No stray keys into clients | pass: Super+x, k, m, q, j, 7, F2, Tab over Notepad type nothing and open nothing | p1-02 |
| Lock | pass: Start > Lock and Super+Shift+Escape each run `swaylock -f` (stand-in window "Screen locked") | p1-16 |
| Power | pass: Start > Turn Off Computer, Cancel runs nothing; Super+Shift+E opens it, Turn Off runs `systemctl poweroff` (stand-in); Log Off runs `loginctl terminate-user`; rpx2-2, rpx2-4 | p1-17, p1-19, p1-26, t-10 |
| Volume | pass with rpx2-1: tray click opens the popup with 50 % ticked, 75 % runs `wpctl set-volume 75%`; keys Vol+ and Mute run, red no-sign shows; tooltip under the paperclip (rpx1-6 seen again) | p1-10, p1-11, p1-12, p1-13 |
| Screenshot | pass: Start > Screenshot and Super+Print each run grim; balloons "Screenshot saved" with the path, queued one after the other | p1-14, p1-15, t-07 |
| Clock | pass: double click opens Date and Time Properties, now with a taskbar button (xp-10 in part) | p1-15 |
| Honest failure states | pass with rpx2-3: Notepad with no editor says so (rpx1-7 seen again), Super+E "No file manager", Run Nothing found | t-02, p1-25, p2-06c |
| Handover from and to xp | pass: tour Send To tiling and Super+drag construct > xp keep every line (acts 5, 9, 10 green twice); Super+Alt+1 xp > tiling and Super+Alt+2 back keep the window and its lines; the keyboard stays with the giver (D119, by design); the window comes back fitted to the work area | t-12, t-13, h1-01, h1-02, h1-03 |
| Erika (74, mouse): letter, helper, sound 75 %, question Yes | pass with rpx1-3, rpx1-4, rpx2-5 | t-01..t-06 |
| Murat (52, mouse): volume popup, screenshot, Run, Help, Turn Off Cancel, minimize and task button, Super+D twice | pass | p1-10, p1-14, t-08, t-09, p1-17, p1-23 |
| Keyboard user: Ctrl+Esc, arrows, Return, type, Alt+F4, keys back to the first window | pass | p1-21, p1-22, p1-24 |
| 1920x1080 | pass: Start, All Programs flyout over the right column (as XP), chat | p3-01, p3-02 |
| Style (xp-style.md) | rpx2-2, rpx2-4, rpx2-5; seen again rpx1-3..rpx1-7 | as listed |

### Findings

| id | look | severity | text |
|---|---|---|---|
| rpx2-1 | xp | MINOR | the speaker tooltip stays up with the old level after Vol+ or Mute ("Volume 75%" while muted at 80 %) |
| rpx2-2 | xp | COSMETIC | Turn Off Computer: Lock, Log Off, Restart captions on a dark plate, Stand By and Turn Off without; those two 4 px apart |
| rpx2-3 | xp | MINOR | Start > My Documents with no file manager: "Cannot open thunar / The program is not installed.": a program name, no next step; Super+E says "No file manager" with the next step |
| rpx2-4 | xp | MINOR | the Log Off dialog is titled "Log Off Windows": the trademarked name that xp-style forbids |
| rpx2-5 | xp | COSMETIC | while the chat field has the keys, the window behind keeps its active title bar (two focus cues) |

**rpx2-1 (xp, MINOR; honest states).** With the pointer resting on the tray speaker, the tooltip "Volume 75%" stays
when Vol+ (`wpctl set-volume 5%+`) and Mute (`set-mute toggle`) change the state; the red no-sign and the "Volume /
Muted" balloon are right, the tooltip under them is not. Only leaving the speaker and coming back shows "Muted"
(p1-13). Repro: fresh `session.sh xp`; `act.py move 1195 785` (speaker; 705 at 720 high), wait 1.2 s,
`key XF86AudioRaiseVolume`, `key XF86AudioMute`, look. R1 p1 (p1-12: "Volume 75%" while muted), R2 p2 (p2-01c:
"Volume 50%" while muted at 55 %). Also the balloon lies over the tooltip and the tooltip under the paperclip (rpx1-6).

**rpx2-2 (xp, COSMETIC).** Start > Turn Off Computer: the captions Lock, Log Off and Restart sit on a dark blue plate,
Stand By and Turn Off on the bare gradient, and "Stand By" ends 4 px before "Turn Off" starts (p1-18 zoom 3x; t-10 at
1280x720, p1-17 at 1280x800). Log Off's own dialog shows the plate under Log Off only (p2-07c). One look for all
five, and a gap of 8 or more.

**rpx2-3 (xp, MINOR; honest states, P4).** Start > My Documents (right column) with no file manager installed shows
"Cannot open thunar / The program is not installed." (p1-25 right, p2-04c): "thunar" appears nowhere on screen and
there is no next step (shared.md: a failure names the thing and the next step). The key door to the same thing,
Super+E, says "No file manager / No file manager is installed. Ask whoever looks after this computer to install one."
(p1-25 left, p2-06c). Repro: fresh `session.sh xp 1280x720`; `click 48 705`, `click 262 432`. R1 p1, R2 p2.

**rpx2-4 (xp, MINOR; xp-style intro, vision card: no "Windows" name).** Start > Log Off opens a dialog titled "Log Off
Windows" (`plugins/chora-xp/src/session.rs:199`). xp-style: "never in trademarked art: ... no "Windows" name". R1 p1
(p1-26), R2 p2 (p2-07c). Repro: Start, Log Off.

**rpx2-5 (xp, COSMETIC).** With Notepad focused, a click on the paperclip gives the chat field the keys (the editor's
cursor turns hollow) but the Notepad title bar stays active blue (0,83,210) (t-03, t-04, t-05); the person sees two
places that look like they take the typing. XP greys the title of a window that lost the keys.

### Known ids seen again (not new)

rpx1-3 / xp-48 and xp-14 (card in a DejaVu face, Yes/No above the key line: t-04), rpx1-4 / xp-48 (balloon and notices
11 px: t-01, t-02, p1-14), rpx1-5 (bold italic arrow caps: t-09, p1-06), rpx1-6 (tooltip under the paperclip, balloon
over it: t-07, p1-11), rpx1-7 (no-editor notice names chora-xp.yaml: t-02; Super+E too: p1-25), xp-10 (single click on
the clock: not retested), xp-20 (green dot for Chat with a helper: p1-01), xp-21 (Help close box only: t-09), xp-34
(tray clock 11 px), sh-12 (no time left, no answer line after Yes: t-04, t-05), sh-13 (helper name "atlas" lower case
in the persona, "Atlas" in the tour: p1-07), ca-1 (Help pane scrolls at 1280x720: p2-09), ca-4 (act 31 green both
takes). The yellow square at the pointer in persona captures is the injection mark of the core (D23,
`crates/chora/src/inject.rs:8`), not a cursor bug (p1-09).

### Notes for the PM

- Handover chapter, not xp: acts 7 and 8 red in both takes on 3be27e32 ("expected the focus with mc, holders
  ['tiling']"; then "s1 is owned by mc, expected construct"). The window is not lost (t-11: page 7 NOTEPAD with both
  lines; t-12: back on xp with every line). Cause (read): D119 (tiling-focus r2, r3; tests/gate-handover-matrix.sh:10)
  keeps the keyboard with the giver after a give key, so the tour's Expect(focus="mc") and its next give key no longer
  hold. r1 had them green on d40142e4; tiling-focus-r3 merged after tour-rows-r3. Either the rows follow D119 (click
  the page before Super+Alt+4) or D119 is revisited for the nurse; a decision, not a fix lane, and outside this pass.
- The tour still `01-02-start-menu` shows the no-editor balloon, not the Start menu (the still is taken at the act's
  end); the menu is in p1-01.
- `run.py --help` still runs every chapter.
- Context used about 175 k; `pgrep -fa 'chora --headless'` empty at the end.
