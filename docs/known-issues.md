# Known issues

This list holds the small faults that chora ships with at this release
candidate: wrong words, missing hints, misplaced pixels. The review rated them minor or cosmetic. Each line is a state of the
review of the last release candidate, not a promise of a fix.

"verify" means a later commit may have fixed it and nobody has checked
yet. Larger faults are tracked as issues on
[GitHub](https://github.com/MArpogaus/chora/issues).

## tiling

| Id | Kind | State | Issue |
|----|------|-------|-------|
| ti-5 | minor | open | A toast whose mako starts while the chat is open is fully hidden under the chat panel (x896-1255 under 868-1268); read chat state at start |
| ti-6 | minor | open | Tooltips Apps "Run a program" and Keys "Every key" stay after click or Esc (3 s late over fullscreen); set tooltip false on Apps, Keys, power |
| ti-7 | minor | open | Fullscreen: the bar floats over the window and covers the first terminal line; put the bar under a fullscreen window |
| ti-8 | minor | verify | New windows do not take the focus (Super+Return twice leaves typing in the first); focus new windows or say it in the key list |
| ti-9 | minor | verify | Key list 692 px high (runs out on 720); "Window that asks" unexplained, say "Window with a question" |
| ti-10 | minor | verify | Key list translucent over seams; layout chip sat on the seam between tiles |
| ti-11 | minor | open | After settings.set look.theme, running programs and assistant surfaces keep the old palette (Ink and Paper side by side) |
| ti-12 | minor | open | No focus-left/right key; Super+Right switches the tag and Super+l widens the master (awesome keymap, by design): list it or decide |
| ti-13 | minor | verify | Bar network shows the address, not "Connected"/"Not connected"; Tile and sound words unlabelled and inert |
| ti-14 | minor | open | Key list shows only keyboard chords, no mouse basics (low) |
| ti-15 | cosmetic | open | 17 open pixel items: toast LCD fringes, orb ring, toast type, chat header, launcher size, tooltip shadow, 1.25 hairlines, island width jumps |
| ti-16 | cosmetic | open | Question card one size (title bold not semibold, path not mono); empty chat panel while a question waits |
| ti-17 | cosmetic | open | Ink shadows invisible; unfocused border lighter and uneven; left island fixed 448 px with empty space |
| ti-18 | cosmetic | verify | Listening chip on the window border; three font families in one frame; key list says Super+Print while the note says Print |

## xp

| Id | Kind | State | Issue |
|----|------|-------|-------|
| xp-5 | minor | verify | Start lists LibreOffice though not installed: "Cannot open" after 30 s; list installed programs only |
| xp-6 | minor | verify | No sign that a program is starting (slow start looks like a dead click); a busy cursor or taskbar button |
| xp-7 | minor | verify | Notify balloon sat on the paperclip head, text 12 px under the 14 px floor |
| xp-8 | minor | verify | Caption buttons 22 px, 4 apart; spec 24 px, 2 apart, 4 from the edge |
| xp-9 | minor | open | Turn Off dialog: Cancel about 56x16 (under 24 px target), icon captions 13 px, desktop not dimmed behind it |
| xp-10 | minor | open | Date and Time: 12 px text, 72x20 buttons, single click on the clock does nothing, no taskbar button |
| xp-11 | minor | open | Help window: Esc does not close, close box only, empty right pane on first topic, key table ends at y 712 with no scroll to more rows |
| xp-12 | minor | verify | Chat reopened after the no-helper balloon: field detached from the balloon, over the clip |
| xp-13 | minor | open | Recycle Bin hidden under the open chat balloon, label 13 px from the edge; desktop labels and titles about 12 px |
| xp-14 | minor | verify | Question card: "Typing does not answer." outside the card, field shows two lines, card text not in the look font; no time left and no answer line |
| xp-15 | minor | verify | Start menu: ImageMagick label truncated "(color depth=..."; header 65 px vs 54 |
| xp-16 | minor | open | Paperclip helper is tiny and unlabelled in the corner (tray clip thin, own art); a tester did not find it |
| xp-17 | minor | verify | Menu leftovers: keyboard focus hidden or on the wrong item, Sounds does nothing, Display has no taskbar button |
| xp-18 | minor | verify | Alt+Tab switcher not seen in a 600 ms capture; verify with a test that holds Alt |
| xp-19 | minor | open | Workspace buttons 1-4 are own flat blue squares in the taskbar and read as foreign; dim to the tray tone |
| xp-20 | cosmetic | open | Start: "Chat with a helper" icon is a green dot; Control Panel rows mix icon and none; both windows titled "foot" |
| xp-21 | cosmetic | open | Turn Off flat rows, submenu rows flat and without separators, pale fallback icons, Send vs input height, Help close box only |
| xp-22 | cosmetic | open | Turn Off uses own power art and a ring instead of the XP flag logo (not importable from the sparse set) |

## mc

| Id | Kind | State | Issue |
|----|------|-------|-------|
| mc-9 | minor | verify | Chat field is a 1.7:1 box with a generic placeholder, 36 px off the 28 grid |
| mc-10 | minor | verify | Card text wraps badly (file name orphaned, "(412 / bytes)"), mono 14 px at the floor |
| mc-11 | minor | verify | Stop is a 30x14 muted word on its own row, no "Stopped." line |
| mc-12 | minor | verify | "NOTIFICATIONS 0" with chat open; the chat door jumps with each notice; title and body not told apart |
| mc-13 | minor | open | Scene copy: "collector" jargon on page 2 reads as a failure, mixed fonts, red heads over nothing, divider off centre |
| mc-14 | minor | open | Chat body 18 px/27 vs spec 19/28; "Super+ <- ->" key text has a stray space |
| mc-15 | cosmetic | open | Yes/No hover thickens the rim (other hovers use the bar); stale hover bar; footer link flush right vs mock flush left |
| mc-16 | cosmetic | open | Yes/No 32 px off grid; notification wraps with "a" at line end; duplicate head: "1 AGENDA" above the page title |
| mc-17 | cosmetic | open | Serif/sans baseline and numeral indent ±1 px, Listening dot 3 px short of the edge, caret, Stop/Esc/Close gaps, 15 vs 14 px mono, thin 7:1 margins |
| mc-18 | cosmetic | open | Light terminal is a black block on cream (foot rejects colors-dark here); the question is stated 3 times in the column |

## construct

| Id | Kind | State | Issue |
|----|------|-------|-------|
| co-5 | minor | open | Step-back capsule missing at 2x after step-in (0 dark px); a 2x user sees no way out hint; show it on a timer, not on first paint |
| co-6 | minor | open | Failed face stood back at 2x has no scene (flat bg, no horizon, floor or reflection); 1x has them |
| co-7 | minor | open | Failed-face card not fitted to the face (spills 12 px left, flat, face reads as ghost); size inside the face with a 16 px margin |
| co-8 | minor | verify | Old open items not evidenced: m1 failed face FIXED at 1x; m5 placeholder and m7 pull frame still lack captures |
| co-9 | minor | verify | S5 leftovers: OPEN CUBE black for 16 frames and SUNSET pill on the face; OPEN CUBE overlaps the chat and moves at 2x |
| co-10 | minor | verify | D in walk view drops into a fullscreen terminal; give-it-to keys answer with a toast that has no reason or nothing |
| co-11 | minor | verify | A closed window leaves its foot pill on the face; launcher rows cut labels, no icons, no empty-result text |
| co-12 | minor | open | Cube faces B, C, D show wallpaper only until each face runs its own manager; say so on the face |
| co-13 | cosmetic | open | Tray text differs 1x/2x, duplicate captures, "XP 1  · face" spacing, tiny terminal text in side panes |
| co-14 | cosmetic | verify | Pull frame: blank blue slab and a yellow 12 px square (a marker of the test harness); mid-turn sawtooth; blurred side reflections |
| co-15 | cosmetic | open | Empty floor gives no landmark that walking works (the start card helps; verify it shows with no program) |

## All looks

| Id | Kind | State | Issue |
|----|------|-------|-------|
| sh-12 | minor | open | Question card: no time left (all looks) and no transcript line after an answer (xp, mc) |
| sh-13 | minor | verify | State word spelled three ways, helper surface counted twice (tiling pill+card, mc row+list); one surface per conversation; helper list "1 atlas" |
| sh-14 | minor | open | Mute word missing in tiling and xp (dot or red sign only); mc: mc-4 |
| sh-15 | minor | open | Esc differs: xp closes the chat, tiling closes list and keys, mc first Esc closes the card and keeps the chat; keys sheet blocked while chat is open |
| sh-16 | minor | open | Launcher words: xp "Open:", tiling "Apps", mc ">" prompt with no label; help says "Run a program" in xp and tiling, "Start a program" in mc |
| sh-17 | minor | verify | HUD colours per theme: chat field over the xp taskbar; assistant surfaces must follow a theme change |
| sh-18 | cosmetic | open | Door wording differs ("Chat ..." pill, "Chat with a helper ...", "Ask in your words"); notification shape differs per look (balloon, count, toast) |
