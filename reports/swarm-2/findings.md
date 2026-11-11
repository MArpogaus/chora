# Swarm 2 findings (build 06044cf)

Raw: 107 tester rows (106 findings, 1 ok) in 9 reports, plus about 15 in swarm 1. Deduped: 31 sections: 20 verified, 9 rejected (8 env or harness, 1 by design), 2 duplicates. Detail: feedback/2026-11-11-swarm2.md. "Hit" counts swarm-2 testers, swarm-1 in brackets. Shots are under media/2026-11-12/swarm-2/. "read" = I opened the PNG and the tester read it right.

| # | Title | Look | Sev | Hit | Class | Evidence |
|---|-------|------|-----|-----|-------|----------|
| 1 | No visible way to open the helper chat | all | high | 3 (+3) | product | c-chat/008, 010 (read) |
| 2 | Key list and help only in construct (F1) | tiling, xp, mc | high | 4 (+2) | product | x-keys/036 (read) |
| 3 | mc first screen: no hint, no key reacts | mc | high | 1 (+1) | product (harness hid Super) | m-tv/001 (read) |
| 4 | Cube: 10 s black, no message when manager cannot start | construct | high | 1 (+1) | env cause, product gap | c-cube-run1-env/033 (read) |
| 5 | Construct: walk/help keys dead or typed into terminal after a window opens | construct | high | 2 | product | c-novice/027-030, c-cube2/036 (not re-read) |
| 6 | Unbound Super chords type into the focused terminal | tiling | med | 2 (+1) | product | t-power/019, t-chat/011 (read; chars unreliable) |
| 7 | Launcher, terminal, close, Super alone differ per look | all | med | 6 | product (POLS) | c-novice/008, x-keys/028 |
| 8 | Second construct pane is a grey slab, no label | construct | med | 2 | product | c-novice/020 (read) |
| 9 | Cube faces B, C, D bare wallpaper | construct | med | 1 | product, cause unclear | c-cube2/011 (read) |
| 10 | give-it-to toast has no reason or no feedback | construct | med | 2 | product (env makes it likely) | c-novice/020 (read) |
| 11 | X11 program: only "No window opened in 30 seconds" | xp, construct | med | 2 (+1) | product | x-office/009 (read) |
| 12 | Cube turn glitches (yellow square, strips, late labels) | construct | low | 1 (+1) | product | c-cube2/010, 014 |
| 13 | Closed window leaves foot pill | construct | low | 1 | product | c-cube2/034, 044 |
| 14 | xp menu keyboard focus hidden or wrong start | xp | low | 1 | product | x-keys/004, 018 |
| 15 | xp taskbar button stays pressed | xp | low | 2 | product | x-keys/026, x-office/013 |
| 16 | xp menu leftovers (Sounds, Display, lists) | xp | low | 2 | product | x-office/026-028 |
| 17 | Tiny light-grey text | all | low | 5 (+1) | product (800 px look exaggerates) | m-tv/001, c-chat/008 |
| 18 | Launcher rows: cut label, no icons, no empty text | construct | low | 2 | product | c-novice/008, c-chat/010 |
| 19 | Empty construct floor has no landmark | construct | low | 1 (+1) | product | c-cube-run1-env/012 |
| 20 | Tiling bar inert, workspace 1 vanishes | tiling | low | 2 | product | t-power/009, t-chat/007 |
| - | Thunar not installed | xp | - | 2 | env | x-office/018 |
| - | LibreOffice does not open | xp, construct | - | 2 (+1) | env | x-office/009 |
| - | Locale warning in terminals | all | - | 5 | env | x-office/010 |
| - | Super/Meta key names rejected | all | - | 4 | env (harness) | c-chat actions.jsonl |
| - | Sound unavailable, no Alt+Tab preview, Vim blank, plain drag | various | - | 4 | env | see feedback file |

Product findings kept at the foot: Super+Left/Right and Super+l on tiling are the awesome keymap (rejected, by design).

## Main-path blockers for the RC
- Helper chat is unreachable in every look for a newcomer (1). The F1 sheet in construct names the answer keys but not how to start the chat; tiling and xp document Super+a, but it showed nothing in two tiling runs (cause unknown, check chora-chat output next time).
- Only construct has a key list (2). Tiling, xp and mc users have no way to learn a key, and mc shows an empty agenda (3).
- The construct keyboard model breaks after the first window opens (5), and the cube can be a black screen (4) or empty (9). Two testers never saw a second window or another face work.
- Second windows render as grey slabs (8); a program that cannot start only says "30 seconds" (11).

## Consistency across looks (POLS)
- Help: F1 in construct; nothing in tiling, xp, mc (Super+F12 is a test overlay).
- Assistant: Super+a is chat in tiling and xp, the agenda scene in mc, nothing in construct.
- Launcher: Super+space in construct and mc; Super+r in tiling, where Super+space changes the layout and looks dead; Ctrl+Escape or Start in xp.
- Terminal: Super+t, Super+Return, Super+e. Close: Super+x, Super+Shift+c, Alt+F4, none in mc.
- Super alone: nothing in all four; xp users expect Start.
- Turning: Super+Left/Right is workspace (tiling) or scene (mc); Ctrl+Alt+Left/Right turns the xp cube; Q/E in construct.
- Same everywhere and good: Super+Alt+1..4 give-to keys, but the toast has no reason (10).
- Unbound Super letters type into the terminal in tiling (6) while construct says its Super keys start nothing by mistake.

## Env fixes for the next swarm
- Install thunar; for LibreOffice install xwayland-satellite and make session.sh start chora-xp-xwayland (or install the Wayland VCL plugin). Only vclplug_gen exists here.
- Set LANG=C.UTF-8 in swarm.sh to remove the locale warning (5 testers).
- Alias Super and Meta to Super_L in act.py and list valid key names in the error; add a hold-Super drag so Super+drag can be tested.
- Keep /usr/share/chora/flavors linked (done); log chora-chat and companion stderr into PERSONA_KEEP so a dead Super+a can be explained.
- Give a hold time for Alt+Tab, or accept that the xp switcher cannot be seen.
- Use 1280 px looks at least for text findings; 800 px made text look tiny.
- Brief the mc tester with the Super keys (or the key list), else m-tv repeats.

## Swarm 3 (build 06044cf)
4 reports (t, x, m everyday; c handover), 12 findings plus 5 dropped extras = 17 sections: 6 verified, 3 duplicates, 8 rejected (6 env or harness, 2 by design or unproven). Detail: feedback/2026-11-11-swarm3-*.md; shots under media/2026-11-12/swarm-3/ (all read).

New verified, ranked (lock/power and Print are two sections):
1. Tiling has no lock or power item and no screenshot key: bar offers only an inert Connected pill; Super+Shift+Escape locks but nothing says so; Print is unbound (t-everyday/013, 005).
2. Construct hint bar omits Super+T: an empty floor shows Walk, Fly in, Windows, F1, Cube but no way to start a terminal (c-handover/001; the F1 sheet has it).
3. Construct: D after Super+T types into the fullscreen terminal (c-handover/011); same family as finding 5 above.
4. mc: Super+slash types "/" into the terminal (m-everyday/007); same family as finding 6 above.
5. xp: Control Panel submenu opens only from its arrow, not the label (x-everyday/007, 008).

Only duplicates: mc no key list (finding 2), second construct pane missing after Super+T (finding 5), cube face with a floating foot pill (finding 13).

Rejected: sound (3 reports; no audio device, label honest), xp left column with one program, xp terminal behind Start menu (harness timing), mc colour errors (foot 1.16.2, mc.yaml needs 1.26), mc notifications empty, mc Super+Return (drawer by design), tiling paste (no selection in shot).
Env fix: install foot 1.26 or later for mc, and keep the harness Super gap in mind (Super alone cannot be pressed).

## Swarm 4 (build 06044cf, mouse-only testers)
4 reports (c4 construct, x4 xp settings, t4 tiling, m4 mc), 20 findings: 0 new verified, 13 duplicates, 7 rejected (3 env or harness, 2 tester error, 1 by design, 1 no defect). Detail: feedback/2026-11-11-swarm4-*.md; shots under media/2026-11-12/swarm-4/ (key shots read).

New verified: none. The mouse-only angle only re-found known gaps:
- construct, tiling, mc give a mouse user nothing to click: empty floor, no root menu, bar without launcher (duplicates of the empty-desktop and mc first-screen findings, ranks 1 and 3 above).
- tiling bar "icon" at top-left is the workspace tag label (flavors/tiling/waybar/config.jsonc:11), inert by design of waybar; dup of the inert bar finding.

Product bug or not, from source (build 06044cf):
- Turn Off Computer "no dialog": tester clicked outside the menu (x 270 vs buttons at x 90 and 127). Dialog exists: plugins/chora-xp/src/startmenu.rs:972, wm.rs:2679. Env/tester, no bug.
- xp Terminal and Control Panel "do not open": second Start click toggled the menu shut, last click hit the desktop (actions.jsonl). Terminal entry: startmenu.rs:834. Tester error plus known arrow-only cascade.
- mc clock frozen at 17:33: session lasted 25 s, minute resolution (plugins/chora-mc/src/main.rs:55, 1278). Env, no bug.
- top-left tiling icon: tag label with no on-click (config.jsonc:11), by design.
- Harness: yellow square is the pointer marker, not product. Thunar, sound: env as before.
Harness note: mouse-only runs of 25 s give no evidence on window drag, resize or close; brief mouse testers with Super+T first.

new data: no
