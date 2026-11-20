# xp findings (train 4, xp a836cbd6) | id | severity | source:line | state | text | assets benchmark A1-A11 deferred (X9 later, fallback art accepted)
xp-1 | MAJOR | ux-deep-xp:35 | fixed d349a6da (doors) check `xp log-off` of gate-doors: Start > Log Off > dialog > Log Off runs loginctl terminate | Log Off click closes the menu and nothing happens; no dialog, session stays, both windows lose focus and stay inactive
xp-2 | MAJOR | ux-deep-xp:36; pixel-xp-r4:21 | fixed 99f40faf (xtc) gate-xp-run: no match shows Nothing found, Escape closes | Run (Start, Super+Space): bare box; no match leaves an empty box and Enter closes it silently. Needs a "Nothing found" word (dialog look: X8 later)
xp-3 | MAJOR | ux-deep-xp:37 | fixed d349a6da (doors) gate-doors f1-free x4: F1 is no binding in any look, reaches the client | F1 leaked into the terminal as ^[OP once (intermittent): a bound key must never leak; table lists only Super+F1 (1059495c may fix the table)
xp-4 | MAJOR | fb-erika-r1:1; fb-erika6:37 | fixed 99f40faf (xtc) gate-xp-notepad: Notepad first program, a click starts the editor of the settings | Start menu has no plain text or letter program (Notepad, Write); map to the configured editor, list only installed programs
xp-5 | MINOR | swarm-6:28; fb-erika6:7 | verify | Start lists LibreOffice though not installed: "Cannot open" after 30 s; list installed programs only
xp-6 | MINOR | fb-erika-r1:23 | verify | No sign that a program is starting (slow start looks like a dead click); a busy cursor or taskbar button (X4 panel is later)
xp-7 | MINOR | ux-deep-xp:39 | verify | Notify balloon sat on the paperclip head, text 12 px under the 14 px floor (165a1105 moved it left of the clip)
xp-8 | MINOR | ux-deep-xp:48 | verify | Caption buttons 22 px, 4 apart; spec 24 px, 2 apart, 4 from the edge (8e1cf99d)
xp-9 | MINOR | ux-deep-xp:49; pixel-xp-r4:57 | open | Turn Off dialog: Cancel about 56x16 (under 24 px target), icon captions 13 px, desktop not dimmed behind it
xp-10 | MINOR | ux-deep-xp:50 | open | Date and Time: 12 px text, 72x20 buttons, single click on the clock does nothing, no taskbar button
xp-11 | MINOR | ux-deep-xp:47 | open | Help window: Esc does not close, close box only, empty right pane on first topic, key table ends at y 712 with no scroll to more rows
xp-12 | MINOR | ux-deep-xp:51 | verify | Chat reopened after the no-helper balloon: field detached from the balloon, over the clip (5a560e2b refit)
xp-13 | MINOR | ux-deep-xp:52,53 | open | Recycle Bin hidden under the open chat balloon, label 13 px from the edge; desktop labels and titles about 12 px
xp-14 | MINOR | ux-deep-xp:45 | verify | Question card: "Typing does not answer." outside the card, field shows two lines, card text not Tahoma; time left and answer line: sh-13
xp-15 | MINOR | pixel-xp-r4:32,33 | verify | Start menu: ImageMagick label truncated "(color depth=..."; header 65 px vs 54 (a836cbd6 touched the header gap)
xp-16 | MINOR | fb-erika6:47; swarm-6:30 | open | Paperclip helper is tiny and unlabelled in the corner (tray clip thin, own art); taste, but Erika did not find it
xp-17 | MINOR | fb-swarm2:148,170 | verify | Menu leftovers: keyboard focus hidden or on the wrong item, Sounds does nothing, Display has no taskbar button
xp-18 | MINOR | ux-deep-xp:62 | verify | Alt+Tab switcher not seen in a 600 ms capture; verify with a test that holds Alt
xp-19 | MINOR | pixel-xp-r4:56 | open | Workspace buttons 1-4 are own flat blue squares in the taskbar and read as foreign; dim to the tray tone
xp-20 | COSMETIC | ux-deep-xp:57,59,61 | open | Start: "Chat with a helper" icon is a green dot; Control Panel rows mix icon and none; both windows titled "foot"
xp-21 | COSMETIC | pixel-xp-r4:23,34-38 | open | Turn Off flat rows, submenu rows flat and without separators, pale fallback icons, Send vs input height, Help close box only
xp-22 | COSMETIC | pixel-xp-r4:58 | open | Turn Off uses own power art and a ring instead of the XP flag logo (not importable from the sparse set)
xp-23 | MAJOR | owner 11-16 09:01; media-r1 tour v3 | open | The paperclip (xp.md: the one helper door) is gone from the xp desktop; bring it back and gate it (a cell that finds it at its place)
xp-24 | MAJOR | owner 11-16 09:01 | open | Text fields poorly aligned (Run, search, dialogs): label baseline, field height and padding off the XP grid; a polish round with pixel asserts
xp-25 | MAJOR | journey-1 (tour v4, opus review) X1 01-03-letter.png | open | Editor for Erika is nano in a dark terminal (^X, Write Out); xp needs a GUI editor when one is installed (Writer, mousepad), nano only as fallback; j1-2
xp-26 | MAJOR | journey-1 (tour v4, opus review) X2 01-10-volume-75.png | open | Sound 75% from Control Panel: tray shows no level, menu no tick on the current level
xp-27 | MAJOR | journey-1 (tour v4, opus review) X3 01-12-question.png | open | Question card: other font, white box, covers the transcript; keyboard words (Alt+Y/N) for a mouse user; 'Typing does not answer' twice
xp-28 | MINOR | journey-1 (tour v4, opus review) X4, X6 | open | Chat header stays 'Working, step 1 of 1' and Stop active after the answer; editor opens over desktop icons, labels clipped
xp-29 | MINOR | owner 11-17 10:35 "Xp: why click in tray to open clippy chat?" | open | The helper chat opens from the tray icon; POLS for XP users: a click on the paperclip itself opens its balloon and chat.
xp-30 | MINOR | owner 11-17 10:35 "Why no wrlcom mgs from clippy?" | open | No welcome balloon: at the first session start the paperclip greets once ("Hi, I am your helper ...", names how to ask and how to close; honest "no helper" state when none is set up), then stays quiet.
xp-31 | MAJOR | P2 r1 pixel-watcher (feedback/2026-11-20-p2r1-pixel.md, T5) | open | Super+D hides the windows and nothing brings them back (no second Super+D, no taskbar button restores); POLS: Super+D toggles, as in XP
xp-32 | MAJOR | P2 r1 Murat (feedback/2026-11-20-p2r1-murat.md, T4 volume) | open | A click on the tray speaker shows no volume control (shared RC bar: volume reachable by mouse); XP shows the level slider with Mute
xp-33 | MAJOR | P2 r1 pixel-watcher (T2) | open | Super+F1 (the key list by keys) shows nothing on xp; the key list is reachable only through Help (shared RC bar: key list by keys)
